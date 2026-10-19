use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use routeros_core::build_command;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::{AppState, RouterTarget};

#[derive(Deserialize, Debug, Default)]
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TelegramSendReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub bot_token: String,
    pub chat_id: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct SetupNetwatchReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub host: String, // IP to monitor, e.g. "192.168.88.2" or "8.8.8.8"
    pub interval: Option<String>, // default "10s"
    pub timeout: Option<String>,  // default "1000ms"
    pub bot_token: String,
    pub chat_id: String,
    pub device_name: Option<String>, // e.g. "AP-Ruijie-Lantai2"
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

/// Helper function to URL-encode text for /tool fetch query parameters
fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => encoded.push_str(&format!("%{:02X}", b)),
        }
    }
    encoded
}

/// POST /api/v1/telegram/send-message - Dispatches an instant Telegram alert via RouterOS /tool/fetch
pub async fn send_message(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<TelegramSendReq>,
) -> Result<Json<Value>, ApiError> {
    if req.message.len() > 4096 {
        return Err(ApiError::BadRequest("message exceeds Telegram limit of 4096 characters".into()));
    }

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_telegram_bot_token(&req.bot_token)?;
    crate::routes::sanitize::validate_telegram_chat_id(&req.chat_id)?;

    let clean_msg = url_encode(&req.message);
    let fetch_url = format!(
        "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}",
        req.bot_token, req.chat_id, clean_msg
    );

    client.run(build_command(
        "/tool/fetch",
        [
            ("url", fetch_url.as_str()),
            ("keep-result", "no"),
            ("http-method", "get"),
        ]
    )).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Pesan Telegram berhasil dipicu via /tool/fetch MikroTik"
    })))
}

/// POST /api/v1/telegram/setup-netwatch - Installs an automated Netwatch monitor that dispatches
/// Telegram alert notifications instantly when a host goes DOWN or UP.
pub async fn setup_netwatch(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetupNetwatchReq>,
) -> Result<Json<Value>, ApiError> {
    if !st.allow_terminal_exec {
        return Err(ApiError::Forbidden(
            "Netwatch script setup is disabled (allow_terminal_exec is false)".to_string(),
        ));
    }

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_telegram_bot_token(&req.bot_token)?;
    crate::routes::sanitize::validate_telegram_chat_id(&req.chat_id)?;

    // Validate monitored host (must be valid IP/CIDR or safe hostname)
    if crate::routes::sanitize::validate_ip_or_cidr(&req.host).is_err()
        && !req.host.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_')) {
        return Err(ApiError::BadRequest("Monitored host must be a valid IP address or hostname".into()));
    }

    if let Some(ref dev) = req.device_name {
        if dev.contains(['\r', '\n', '"', ';']) {
            return Err(ApiError::BadRequest("device_name contains invalid characters".into()));
        }
    }

    let dev_label = req.device_name.as_deref().unwrap_or(req.host.as_str());
    let interval = req.interval.as_deref().unwrap_or("10s");
    let timeout = req.timeout.as_deref().unwrap_or("1000ms");
    crate::routes::sanitize::validate_validity(interval)?;
    crate::routes::sanitize::validate_validity(timeout)?;
    if let Some(ref c) = req.comment {
        if c.contains(['\r', '\n', '"', ';']) {
            return Err(ApiError::BadRequest("comment contains invalid characters".into()));
        }
    }
    let comment = req.comment.unwrap_or_else(|| format!("[Telegram-Netwatch] {}", dev_label));

    // Construct RouterOS Netwatch UP & DOWN scripts
    let down_text = url_encode(&format!("🚨 ALERT: Perangkat '{}' ({}) DOWN / Terputus!", dev_label, req.host));
    let up_text = url_encode(&format!("✅ RECOVERED: Perangkat '{}' ({}) UP / Normal Kembali!", dev_label, req.host));

    let down_script = format!(
        "/tool fetch url=\"https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}\" keep-result=no",
        req.bot_token, req.chat_id, down_text
    );
    let up_script = format!(
        "/tool fetch url=\"https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}\" keep-result=no",
        req.bot_token, req.chat_id, up_text
    );

    client.run(build_command(
        "/tool/netwatch/add",
        [
            ("host", req.host.as_str()),
            ("interval", interval),
            ("timeout", timeout),
            ("up-script", up_script.as_str()),
            ("down-script", down_script.as_str()),
            ("comment", comment.as_str()),
        ]
    )).await?;

    Ok(Json(json!({
        "success": true,
        "monitored_host": req.host,
        "device_name": dev_label,
        "interval": interval,
        "message": format!("Netwatch dengan notifikasi Telegram berhasil dikonfigurasi untuk host {}", req.host)
    })))
}

/// POST or GET /api/v1/telegram/list-monitors - Lists all configured Netwatch monitors on the router
pub async fn list_monitors(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/tool/netwatch/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "count": data.len(),
        "monitors": data
    })))
}

/// POST /api/v1/telegram/remove-monitor - Removes a Netwatch monitor by its .id
pub async fn remove_monitor(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    crate::routes::sanitize::validate_safe_identifier(&req.id, "id")?;

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    client.run(build_command("/tool/netwatch/remove", [(".id", req.id.as_str())])).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Netwatch monitor berhasil dihapus"
    })))
}

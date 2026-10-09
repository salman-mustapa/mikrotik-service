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
pub struct AppBlockReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub app_type: String, // "whatsapp", "tiktok", "youtube", "judi_online", "torrent_p2p", "custom"
    pub custom_domain: Option<String>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AppUnblockReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub app_type: String,
}

/// POST /api/v1/security/app-block - One-click blocking of apps/content (WhatsApp, TikTok, YouTube, Judi Online, Torrent)
pub async fn block_app(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AppBlockReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let app = req.app_type.to_lowercase();
    let mut rules_created = Vec::new();

    match app.as_str() {
        "whatsapp" => {
            // Drop WhatsApp TLS-Hosts and VoIP/Chat ports
            let tls_domains = ["*whatsapp.com*", "*whatsapp.net*"];
            for domain in tls_domains {
                let comment = format!("[App-Blocker] Block WhatsApp TLS ({})", domain);
                client.run(build_command(
                    "/ip/firewall/filter/add",
                    [
                        ("chain", "forward"),
                        ("protocol", "tcp"),
                        ("dst-port", "443"),
                        ("tls-host", domain),
                        ("action", "drop"),
                        ("comment", comment.as_str()),
                    ]
                )).await?;
                rules_created.push(comment);
            }
            // Drop WhatsApp direct messaging ports (5222, 5223, 5228)
            let comment_port = "[App-Blocker] Block WhatsApp Media/Chat Ports (5222,5223,5228)";
            client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "forward"),
                    ("protocol", "tcp"),
                    ("dst-port", "5222,5223,5228"),
                    ("action", "drop"),
                    ("comment", comment_port),
                ]
            )).await?;
            rules_created.push(comment_port.to_string());
        }
        "tiktok" => {
            let tls_domains = ["*tiktok.com*", "*tiktokv.com*", "*byteoversea.com*", "*musical.ly*"];
            for domain in tls_domains {
                let comment = format!("[App-Blocker] Block TikTok TLS ({})", domain);
                client.run(build_command(
                    "/ip/firewall/filter/add",
                    [
                        ("chain", "forward"),
                        ("protocol", "tcp"),
                        ("dst-port", "443"),
                        ("tls-host", domain),
                        ("action", "drop"),
                        ("comment", comment.as_str()),
                    ]
                )).await?;
                rules_created.push(comment);
            }
        }
        "youtube" => {
            let tls_domains = ["*youtube.com*", "*googlevideo.com*", "*ytimg.com*"];
            for domain in tls_domains {
                let comment = format!("[App-Blocker] Block YouTube Video Stream ({})", domain);
                client.run(build_command(
                    "/ip/firewall/filter/add",
                    [
                        ("chain", "forward"),
                        ("protocol", "tcp"),
                        ("dst-port", "443"),
                        ("tls-host", domain),
                        ("action", "drop"),
                        ("comment", comment.as_str()),
                    ]
                )).await?;
                rules_created.push(comment);
            }
        }
        "judi_online" => {
            let keywords = ["*slot*", "*gacor*", "*judi*", "*togel*", "*casino*", "*poker*", "*betting*"];
            for kw in keywords {
                let comment = format!("[App-Blocker] Block Judi Online Host ({})", kw);
                client.run(build_command(
                    "/ip/firewall/filter/add",
                    [
                        ("chain", "forward"),
                        ("protocol", "tcp"),
                        ("dst-port", "443"),
                        ("tls-host", kw),
                        ("action", "drop"),
                        ("comment", comment.as_str()),
                    ]
                )).await?;
                rules_created.push(comment);
            }
        }
        "torrent_p2p" => {
            // Drop BitTorrent P2P traffic
            let comment_p2p = "[App-Blocker] Drop BitTorrent / P2P Protocol Traffic";
            client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "forward"),
                    ("p2p", "all-p2p"),
                    ("action", "drop"),
                    ("comment", comment_p2p),
                ]
            )).await?;
            rules_created.push(comment_p2p.to_string());
        }
        "custom" => {
            let domain = req.custom_domain.as_deref().ok_or_else(|| {
                ApiError::BadRequest("custom_domain wajib diisi untuk app_type custom".into())
            })?;
            let wildcard = format!("*{}*", domain);
            let comment = req.comment.unwrap_or_else(|| format!("[App-Blocker] Custom Block ({})", domain));
            client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "forward"),
                    ("protocol", "tcp"),
                    ("dst-port", "443"),
                    ("tls-host", wildcard.as_str()),
                    ("action", "drop"),
                    ("comment", comment.as_str()),
                ]
            )).await?;
            rules_created.push(comment);
        }
        _ => {
            return Err(ApiError::BadRequest(format!("app_type '{}' tidak dikenali. Opsi: whatsapp, tiktok, youtube, judi_online, torrent_p2p, custom", app)));
        }
    }

    Ok(Json(json!({
        "success": true,
        "app_blocked": app,
        "total_rules_deployed": rules_created.len(),
        "rules": rules_created,
        "message": format!("Aplikasi / Konten '{}' berhasil diblokir di Firewall Filter", app)
    })))
}

/// POST /api/v1/security/app-unblock - Unblocks an application by removing matching firewall rules
pub async fn unblock_app(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AppUnblockReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let app = req.app_type.to_lowercase();
    let rows = client.run(build_command("/ip/firewall/filter/print", std::iter::empty::<(&str, &str)>())).await?;

    let search_tag = format!("[App-Blocker]");
    let mut removed_count = 0;

    for r in rows {
        if let Some(comment) = r.get("comment") {
            if comment.starts_pub_with(&search_tag) && comment.to_lowercase().contains(&app) {
                if let Some(id) = r.get(".id") {
                    let _ = client.run(build_command("/ip/firewall/filter/remove", [(".id", id)])).await;
                    removed_count += 1;
                }
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "app_unblocked": app,
        "rules_removed": removed_count,
        "message": format!("Blokir aplikasi '{}' berhasil dicabut ({} aturan dihapus)", app, removed_count)
    })))
}

/// POST /api/v1/security/blocked-apps - Lists all active app-blocking firewall rules
pub async fn list_blocked_apps(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/firewall/filter/print", std::iter::empty::<(&str, &str)>())).await?;
    let mut blocked: Vec<HashMap<String, String>> = Vec::new();

    for r in rows {
        if let Some(comment) = r.get("comment") {
            if comment.starts_with("[App-Blocker]") {
                blocked.push(r.attrs);
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "count": blocked.len(),
        "blocked_rules": blocked
    })))
}

trait StartsPubWith {
    fn starts_pub_with(&self, pat: &str) -> bool;
}

impl StartsPubWith for str {
    fn starts_pub_with(&self, pat: &str) -> bool {
        self.starts_with(pat)
    }
}

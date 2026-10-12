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
pub struct SetCloudReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub ddns_enabled: Option<bool>,
    pub update_time: Option<bool>,
    pub ddns_update_interval: Option<String>, // e.g. "1m", "5m", "10m"
}

/// GET or POST /api/v1/cloud/status - Status IP Cloud DDNS MikroTik (*.sn.mynetname.net)
pub async fn cloud_status(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client
        .run(build_command("/ip/cloud/print", std::iter::empty::<(&str, &str)>()))
        .await?;

    let data: HashMap<String, String> = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    let dns_name = data.get("dns-name").cloned();
    let public_ip = data.get("public-address").cloned();
    let status = data.get("status").cloned().unwrap_or_else(|| "unknown".into());
    let enabled = data.get("ddns-enabled").map(|v| v == "yes" || v == "true").unwrap_or(false);

    Ok(Json(json!({
        "success": true,
        "ddns_enabled": enabled,
        "dns_name": dns_name,
        "public_ip": public_ip,
        "status": status,
        "full_data": data
    })))
}

/// POST /api/v1/cloud/set - Mengatur konfigurasi IP Cloud DDNS (Aktifkan / Nonaktifkan)
pub async fn cloud_set(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetCloudReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut args = Vec::new();
    let ddns_str = req.ddns_enabled.map(|b| if b { "yes" } else { "no" });
    if let Some(s) = ddns_str.as_deref() {
        args.push(("ddns-enabled", s));
    }

    let ut_str = req.update_time.map(|b| if b { "yes" } else { "no" });
    if let Some(s) = ut_str.as_deref() {
        args.push(("update-time", s));
    }

    if let Some(iv) = req.ddns_update_interval.as_deref() {
        args.push(("ddns-update-interval", iv));
    }

    if !args.is_empty() {
        client.run(build_command("/ip/cloud/set", args)).await?;
    }

    // Ambil status terbaru setelah perubahan
    let rows = client
        .run(build_command("/ip/cloud/print", std::iter::empty::<(&str, &str)>()))
        .await?;
    let updated: HashMap<String, String> = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();

    Ok(Json(json!({
        "success": true,
        "message": "Konfigurasi IP Cloud DDNS MikroTik berhasil diperbarui",
        "dns_name": updated.get("dns-name"),
        "public_ip": updated.get("public-address"),
        "status": updated.get("status"),
        "current_settings": updated
    })))
}

/// POST /api/v1/cloud/force-update - Paksa pembaruan rekaman DDNS publik ke server MikroTik
pub async fn cloud_force_update(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let _ = client
        .run(build_command("/ip/cloud/force-update", std::iter::empty::<(&str, &str)>()))
        .await;

    let rows = client
        .run(build_command("/ip/cloud/print", std::iter::empty::<(&str, &str)>()))
        .await?;
    let data: HashMap<String, String> = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();

    Ok(Json(json!({
        "success": true,
        "message": "Permintaan force-update IP Cloud berhasil dikirim ke server mynetname.net",
        "dns_name": data.get("dns-name"),
        "public_ip": data.get("public-address"),
        "status": data.get("status")
    })))
}

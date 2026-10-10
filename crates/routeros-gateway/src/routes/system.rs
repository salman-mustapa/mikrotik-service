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
pub struct IdentityReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
}

pub async fn resource(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/resource/print", std::iter::empty::<(&str, &str)>())).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

pub async fn routerboard(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/routerboard/print", std::iter::empty::<(&str, &str)>())).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

pub async fn identity(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/identity/print", std::iter::empty::<(&str, &str)>())).await?;
    let name = rows.first().and_then(|r| r.get("name")).unwrap_or("MikroTik");
    Ok(Json(json!({ "success": true, "name": name })))
}

pub async fn set_identity(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdentityReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/identity/set", [("name", req.name.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Identity updated" })))
}

pub async fn check_update(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/package/update/check-for-updates", std::iter::empty::<(&str, &str)>())).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

pub async fn install_update(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/package/update/install", std::iter::empty::<(&str, &str)>())).await?;
    Ok(Json(json!({ "success": true, "message": "Download and install triggered" })))
}

pub async fn reboot(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/reboot", std::iter::empty::<(&str, &str)>())).await?;
    Ok(Json(json!({ "success": true, "message": "Reboot command dispatched" })))
}

#[derive(Deserialize, Debug)]
pub struct ChannelReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub channel: String, // "stable", "long-term", "testing", "development"
}

pub async fn download_update(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/package/update/download", std::iter::empty::<(&str, &str)>())).await?;
    Ok(Json(json!({ "success": true, "message": "Package download initiated in background" })))
}

pub async fn set_channel(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ChannelReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/package/update/set", [("channel", req.channel.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": format!("Update channel set to '{}'", req.channel) })))
}

pub async fn packages(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/package/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<_> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/system/package/downgrade - Downgrade RouterOS ke versi sebelumnya
pub async fn package_downgrade(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/package/downgrade", std::iter::empty::<(&str, &str)>())).await?;
    Ok(Json(json!({
        "success": true,
        "message": "Downgrade command dispatched. Router will reboot into the previous version."
    })))
}

/// POST /api/v1/system/package/cancel - Batalkan unduhan paket pembaruan yang sedang berjalan
pub async fn package_cancel(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/package/update/cancel", std::iter::empty::<(&str, &str)>())).await?;
    Ok(Json(json!({
        "success": true,
        "message": "Package update/download canceled."
    })))
}


pub async fn clock(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/clock/print", std::iter::empty::<(&str, &str)>())).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

#[derive(Deserialize, Debug)]
pub struct ToggleServiceReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub service_name: String, // "telnet", "ftp", "www", "ssh", "api", "winbox"
    pub disabled: bool,
    pub port: Option<u16>,
    pub address: Option<String>,
}

pub async fn services(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/service/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<_> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn toggle_service(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ToggleServiceReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/service/print", [("?name", req.service_name.as_str())])).await?;
    let id = rows.first().and_then(|r| r.get(".id")).ok_or_else(|| {
        ApiError::BadRequest(format!("Service '{}' not found", req.service_name))
    })?;

    let dis_str = if req.disabled { "yes" } else { "no" };
    let mut args = vec![(".id", id), ("disabled", dis_str)];
    let port_str = req.port.map(|p| p.to_string());
    if let Some(p) = port_str.as_deref() {
        args.push(("port", p));
    }
    if let Some(a) = req.address.as_deref() {
        args.push(("address", a));
    }

    client.run(build_command("/ip/service/set", args)).await?;
    Ok(Json(json!({
        "success": true,
        "message": format!("Service '{}' updated (disabled: {})", req.service_name, req.disabled)
    })))
}

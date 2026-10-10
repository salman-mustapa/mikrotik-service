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
pub struct FilterReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    #[serde(default)]
    pub filter: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

pub async fn servers(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/dhcp-server/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn leases(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/dhcp-server/lease/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn make_static(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dhcp-server/lease/make-static", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Lease made static" })))
}

pub async fn remove_lease(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dhcp-server/lease/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Lease removed" })))
}

#[derive(Deserialize, Debug)]
pub struct AddLeaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    pub mac_address: String,
    pub server: Option<String>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SetLeaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
    pub address: Option<String>,
    pub mac_address: Option<String>,
    pub comment: Option<String>,
    pub disabled: Option<bool>,
}

/// POST /api/v1/dhcp/lease/add - Add static DHCP lease
pub async fn add_lease(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddLeaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("address", req.address.as_str()),
        ("mac-address", req.mac_address.as_str()),
    ];
    if let Some(ref s) = req.server {
        args.push(("server", s.as_str()));
    }
    if let Some(ref c) = req.comment {
        args.push(("comment", c.as_str()));
    }
    client.run(build_command("/ip/dhcp-server/lease/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP static lease added" })))
}

/// POST /api/v1/dhcp/lease/set - Update DHCP lease IP, MAC, or comment
pub async fn set_lease(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetLeaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![(".id", req.id.as_str())];
    if let Some(ref a) = req.address {
        args.push(("address", a.as_str()));
    }
    if let Some(ref m) = req.mac_address {
        args.push(("mac-address", m.as_str()));
    }
    if let Some(ref c) = req.comment {
        args.push(("comment", c.as_str()));
    }
    let dis_str;
    if let Some(d) = req.disabled {
        dis_str = if d { "yes" } else { "no" };
        args.push(("disabled", dis_str));
    }
    client.run(build_command("/ip/dhcp-server/lease/set", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP lease updated" })))
}

#[derive(Deserialize, Debug)]
pub struct AddDhcpNetworkReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    pub gateway: Option<String>,
    pub dns_server: Option<String>,
    pub domain: Option<String>,
    pub comment: Option<String>,
}

pub async fn networks(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/dhcp-server/network/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn add_network(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddDhcpNetworkReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("address", req.address.as_str())];
    if let Some(gw) = req.gateway.as_deref() {
        args.push(("gateway", gw));
    }
    if let Some(dns) = req.dns_server.as_deref() {
        args.push(("dns-server", dns));
    }
    if let Some(dom) = req.domain.as_deref() {
        args.push(("domain", dom));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ip/dhcp-server/network/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP network added" })))
}

pub async fn remove_network(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dhcp-server/network/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP network removed" })))
}

#[derive(Deserialize, Debug)]
pub struct AddDhcpAlertReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub interface: String,
    pub valid_server: Option<String>,
    pub alert_timeout: Option<String>,
}

/// GET or POST /api/v1/dhcp/alerts - Daftar pemantauan Rogue DHCP Server Alert
pub async fn alerts(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/dhcp-server/alert/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/dhcp/alert/add - Pasang alarm deteksi Rogue DHCP Server pada interface
pub async fn add_alert(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddDhcpAlertReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("interface", req.interface.as_str())];
    if let Some(vs) = req.valid_server.as_deref() {
        args.push(("valid-server", vs));
    }
    if let Some(at) = req.alert_timeout.as_deref() {
        args.push(("alert-timeout", at));
    }
    client.run(build_command("/ip/dhcp-server/alert/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP alert watcher added for interface" })))
}

/// POST /api/v1/dhcp/alert/remove - Hapus alarm deteksi DHCP
pub async fn remove_alert(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dhcp-server/alert/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP alert watcher removed" })))
}


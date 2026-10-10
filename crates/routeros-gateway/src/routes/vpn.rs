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
pub struct SstpServerReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub enabled: bool,
    pub port: Option<u16>,
    pub default_profile: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct L2tpServerReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub enabled: bool,
    pub use_ipsec: Option<String>, // "yes", "no", "required"
    pub ipsec_secret: Option<String>,
    pub default_profile: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OvpnServerReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub enabled: bool,
    pub port: Option<u16>,
    pub mode: Option<String>, // "ip", "ethernet"
    pub default_profile: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AddEoipReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub remote_address: String,
    pub tunnel_id: u16,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

pub async fn sstp_server(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/sstp-server/server/print", std::iter::empty::<(&str, &str)>())).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

pub async fn set_sstp_server(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SstpServerReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("enabled", if req.enabled { "yes" } else { "no" })];
    let port_str = req.port.map(|p| p.to_string());
    if let Some(p) = port_str.as_deref() {
        args.push(("port", p));
    }
    if let Some(prof) = req.default_profile.as_deref() {
        args.push(("default-profile", prof));
    }
    client.run(build_command("/interface/sstp-server/server/set", args)).await?;
    Ok(Json(json!({ "success": true, "message": "SSTP VPN server updated" })))
}

pub async fn l2tp_server(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/l2tp-server/server/print", std::iter::empty::<(&str, &str)>())).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

pub async fn set_l2tp_server(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<L2tpServerReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("enabled", if req.enabled { "yes" } else { "no" })];
    if let Some(ipsec) = req.use_ipsec.as_deref() {
        args.push(("use-ipsec", ipsec));
    }
    if let Some(sec) = req.ipsec_secret.as_deref() {
        args.push(("ipsec-secret", sec));
    }
    if let Some(prof) = req.default_profile.as_deref() {
        args.push(("default-profile", prof));
    }
    client.run(build_command("/interface/l2tp-server/server/set", args)).await?;
    Ok(Json(json!({ "success": true, "message": "L2TP/IPsec VPN server updated" })))
}

pub async fn ovpn_server(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/ovpn-server/server/print", std::iter::empty::<(&str, &str)>())).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

pub async fn set_ovpn_server(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<OvpnServerReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("enabled", if req.enabled { "yes" } else { "no" })];
    let port_str = req.port.map(|p| p.to_string());
    if let Some(p) = port_str.as_deref() {
        args.push(("port", p));
    }
    if let Some(m) = req.mode.as_deref() {
        args.push(("mode", m));
    }
    if let Some(prof) = req.default_profile.as_deref() {
        args.push(("default-profile", prof));
    }
    client.run(build_command("/interface/ovpn-server/server/set", args)).await?;
    Ok(Json(json!({ "success": true, "message": "OpenVPN server updated" })))
}

pub async fn eoip_tunnels(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/eoip/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn add_eoip(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddEoipReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let tid_str = req.tunnel_id.to_string();
    let mut args = vec![
        ("name", req.name.as_str()),
        ("remote-address", req.remote_address.as_str()),
        ("tunnel-id", tid_str.as_str()),
    ];
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/interface/eoip/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "EoIP tunnel created" })))
}

pub async fn remove_eoip(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/interface/eoip/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "EoIP tunnel removed" })))
}

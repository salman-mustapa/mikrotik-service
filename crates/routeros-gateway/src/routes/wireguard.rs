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
pub struct AddPeerReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub interface: String,
    pub public_key: String,
    pub allowed_address: String, // e.g. "10.10.10.2/32"
    pub endpoint_address: Option<String>,
    pub endpoint_port: Option<String>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

/// POST /api/v1/wireguard/interfaces - Daftar interface WireGuard (RouterOS v7)
pub async fn interfaces(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/wireguard/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/wireguard/peers - Daftar peer WireGuard
pub async fn peers(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/wireguard/peers/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/wireguard/peer/add - Tambah peer WireGuard baru
pub async fn add_peer(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddPeerReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("interface", req.interface.as_str()),
        ("public-key", req.public_key.as_str()),
        ("allowed-address", req.allowed_address.as_str()),
    ];
    if let Some(ea) = req.endpoint_address.as_deref() {
        args.push(("endpoint-address", ea));
    }
    if let Some(ep) = req.endpoint_port.as_deref() {
        args.push(("endpoint-port", ep));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/interface/wireguard/peers/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "WireGuard peer added" })))
}

/// POST /api/v1/wireguard/peer/remove - Hapus peer WireGuard
pub async fn remove_peer(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/interface/wireguard/peers/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Peer removed" })))
}

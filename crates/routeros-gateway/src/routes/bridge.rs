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
pub struct AddBridgeReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub vlan_filtering: Option<bool>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AddBridgePortReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub bridge: String,
    pub interface: String,
    pub pvid: Option<u16>,
    pub hw: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct AddBridgeVlanReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub bridge: String,
    pub vlan_ids: String, // e.g. "10,20" or "10-50"
    pub tagged: Option<String>,
    pub untagged: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

pub async fn bridges(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/bridge/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn add_bridge(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddBridgeReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("name", req.name.as_str())];
    let vlan_str = req.vlan_filtering.map(|v| if v { "yes" } else { "no" });
    if let Some(v) = vlan_str.as_deref() {
        args.push(("vlan-filtering", v));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/interface/bridge/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Bridge created" })))
}

pub async fn remove_bridge(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/interface/bridge/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Bridge removed" })))
}

pub async fn ports(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/bridge/port/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn add_port(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddBridgePortReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("bridge", req.bridge.as_str()),
        ("interface", req.interface.as_str()),
    ];
    let pvid_str = req.pvid.map(|p| p.to_string());
    if let Some(p) = pvid_str.as_deref() {
        args.push(("pvid", p));
    }
    let hw_str = req.hw.map(|h| if h { "yes" } else { "no" });
    if let Some(h) = hw_str.as_deref() {
        args.push(("hw", h));
    }
    client.run(build_command("/interface/bridge/port/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Port added to bridge" })))
}

pub async fn remove_port(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/interface/bridge/port/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Bridge port removed" })))
}

pub async fn vlans(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/bridge/vlan/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn add_vlan(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddBridgeVlanReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("bridge", req.bridge.as_str()),
        ("vlan-ids", req.vlan_ids.as_str()),
    ];
    if let Some(t) = req.tagged.as_deref() {
        args.push(("tagged", t));
    }
    if let Some(u) = req.untagged.as_deref() {
        args.push(("untagged", u));
    }
    client.run(build_command("/interface/bridge/vlan/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Bridge VLAN table configured" })))
}

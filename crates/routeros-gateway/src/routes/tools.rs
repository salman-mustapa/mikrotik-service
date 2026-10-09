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

#[derive(Deserialize, Debug)]
pub struct PingReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    pub count: Option<u32>,
    pub size: Option<u32>,
    pub interface: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TracerouteReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    pub count: Option<u32>,
}

#[derive(Deserialize, Debug, Default)]
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BandwidthTestReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    pub user: Option<String>,
    pub password: Option<String>,
    pub direction: Option<String>, // "receive", "transmit", "both"
}

/// POST /api/v1/tools/ping - Ping diagnostik dari router
pub async fn ping(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<PingReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("address", req.address.as_str())];
    let count_str = req.count.unwrap_or(4).to_string();
    args.push(("count", count_str.as_str()));
    let size_str;
    if let Some(s) = req.size {
        size_str = s.to_string();
        args.push(("size", size_str.as_str()));
    }
    if let Some(i) = req.interface.as_deref() {
        args.push(("interface", i));
    }
    let rows = client.run(build_command("/ping", args)).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/tools/traceroute - Traceroute diagnostik dari router
pub async fn traceroute(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<TracerouteReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let count_str = req.count.unwrap_or(1).to_string();
    let args = [("address", req.address.as_str()), ("count", count_str.as_str())];
    let rows = client.run(build_command("/tool/traceroute", args)).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/tools/profile - CPU Profiler (melihat proses apa yang memakai CPU)
pub async fn profile(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/tool/profile", [("once", "")])).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/tools/netwatch - Daftar pemantauan host Netwatch
pub async fn netwatch(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/tool/netwatch/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/tools/bandwidth-test - Tes bandwidth MikroTik ke MikroTik
pub async fn bandwidth_test(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BandwidthTestReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("address", req.address.as_str()), ("duration", "5s")];
    if let Some(u) = req.user.as_deref() {
        args.push(("user", u));
    }
    if let Some(p) = req.password.as_deref() {
        args.push(("password", p));
    }
    if let Some(d) = req.direction.as_deref() {
        args.push(("direction", d));
    }
    let rows = client.run(build_command("/tool/bandwidth-test", args)).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

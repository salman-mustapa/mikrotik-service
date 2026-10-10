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
pub struct BlockIpReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    #[serde(default = "default_list")]
    pub list: String,
    pub timeout: Option<String>,
    pub comment: Option<String>,
}

fn default_list() -> String {
    "BLACKLIST".into()
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

pub async fn filters(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/firewall/filter/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn nat(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/firewall/nat/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn address_lists(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/firewall/address-list/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn block_ip(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BlockIpReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("address", req.address.as_str()), ("list", req.list.as_str())];
    if let Some(t) = req.timeout.as_deref() {
        args.push(("timeout", t));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ip/firewall/address-list/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "IP added to address list" })))
}

pub async fn unblock_ip(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/firewall/address-list/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Address list item removed" })))
}

#[derive(Deserialize, Debug, Default)]
pub struct ConnectionsReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub limit: Option<usize>,
    pub protocol: Option<String>,
    pub src_address: Option<String>,
    pub dst_address: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct FlushConnectionsReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub src_address: Option<String>,
    pub dst_address: Option<String>,
    pub protocol: Option<String>,
}

/// GET or POST /api/v1/firewall/connections - Inspeksi tabel Connection Tracking (Conntrack) aktif
pub async fn connections(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<ConnectionsReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut query = Vec::new();
    if let Some(p) = req.protocol.as_deref() {
        query.push(("?protocol", p));
    }
    if let Some(s) = req.src_address.as_deref() {
        query.push(("?src-address", s));
    }
    if let Some(d) = req.dst_address.as_deref() {
        query.push(("?dst-address", d));
    }

    let rows = client.run(build_command("/ip/firewall/connection/print", query)).await?;
    let limit = req.limit.unwrap_or(200);
    let total = rows.len();
    let data: Vec<HashMap<String, String>> = rows.into_iter().take(limit).map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "total_connections": total,
        "limit_applied": limit,
        "count": data.len(),
        "connections": data
    })))
}

/// GET or POST /api/v1/firewall/connections/top-talkers - Deteksi host penghabis sesi koneksi (Top Talkers / BitTorrent / DDoS Hogs)
pub async fn top_talkers(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/firewall/connection/print", std::iter::empty::<(&str, &str)>())).await?;
    let total = rows.len();

    let mut src_counts: HashMap<String, usize> = HashMap::new();
    let mut proto_counts: HashMap<String, usize> = HashMap::new();

    for r in &rows {
        if let Some(src) = r.get("src-address") {
            // Strip port if present (e.g. 192.168.1.50:54321 -> 192.168.1.50)
            let ip = src.split(':').next().unwrap_or(src).trim();
            if !ip.is_empty() {
                *src_counts.entry(ip.to_string()).or_insert(0) += 1;
            }
        }
        if let Some(proto) = r.get("protocol") {
            *proto_counts.entry(proto.to_string()).or_insert(0) += 1;
        }
    }

    let mut sorted_src: Vec<_> = src_counts.into_iter().collect();
    sorted_src.sort_by(|a, b| b.1.cmp(&a.1));

    let top_talkers: Vec<_> = sorted_src
        .into_iter()
        .take(15)
        .map(|(ip, count)| {
            let pct = if total > 0 { (count as f64 / total as f64) * 100.0 } else { 0.0 };
            json!({
                "ip_address": ip,
                "session_count": count,
                "session_percentage": (pct * 10.0).round() / 10.0
            })
        })
        .collect();

    Ok(Json(json!({
        "success": true,
        "total_active_sessions": total,
        "protocols": proto_counts,
        "top_connection_hogs": top_talkers
    })))
}

/// POST /api/v1/firewall/connections/flush - Flush / hapus sesi koneksi tertentu
pub async fn flush_connections(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<FlushConnectionsReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut query = Vec::new();
    if let Some(s) = req.src_address.as_deref() {
        query.push(("?src-address", s));
    }
    if let Some(d) = req.dst_address.as_deref() {
        query.push(("?dst-address", d));
    }
    if let Some(p) = req.protocol.as_deref() {
        query.push(("?protocol", p));
    }

    let rows = client.run(build_command("/ip/firewall/connection/print", query)).await?;
    let count = rows.len();

    for r in &rows {
        if let Some(id) = r.get(".id") {
            let _ = client.run(build_command("/ip/firewall/connection/remove", [(".id", id)])).await;
        }
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("Flushed {} connection sessions", count),
        "removed_count": count
    })))
}

#[derive(Deserialize, Debug, Default)]
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}


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
pub struct AddAddressReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    pub interface: String,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

pub async fn addresses(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/address/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn add_address(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddAddressReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("address", req.address.as_str()), ("interface", req.interface.as_str())];
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ip/address/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Address added" })))
}

pub async fn remove_address(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/address/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Address removed" })))
}

pub async fn routes(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/route/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

#[derive(Deserialize, Debug)]
pub struct AddRouteReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub dst_address: String,
    pub gateway: String,
    pub distance: Option<String>,
    pub check_gateway: Option<String>, // "ping", "arp"
    pub comment: Option<String>,
}

pub async fn add_route(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddRouteReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("dst-address", req.dst_address.as_str()),
        ("gateway", req.gateway.as_str()),
    ];
    if let Some(d) = req.distance.as_deref() {
        args.push(("distance", d));
    }
    if let Some(cg) = req.check_gateway.as_deref() {
        args.push(("check-gateway", cg));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ip/route/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "IP route added" })))
}

pub async fn remove_route(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/route/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "IP route removed" })))
}

pub async fn dns(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/dns/print", std::iter::empty::<(&str, &str)>())).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

#[derive(Deserialize, Debug)]
pub struct AddPoolReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub ranges: String,
    pub next_pool: Option<String>,
    pub comment: Option<String>,
}

pub async fn pools(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/pool/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn add_pool(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddPoolReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("name", req.name.as_str()), ("ranges", req.ranges.as_str())];
    if let Some(np) = req.next_pool.as_deref() {
        args.push(("next-pool", np));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ip/pool/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "IP pool added" })))
}

pub async fn remove_pool(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/pool/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "IP pool removed" })))
}

#[derive(Deserialize, Debug)]
pub struct AddArpReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    pub mac_address: String,
    pub interface: String,
    pub comment: Option<String>,
}

pub async fn arp(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/arp/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn add_arp(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddArpReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("address", req.address.as_str()),
        ("mac-address", req.mac_address.as_str()),
        ("interface", req.interface.as_str()),
    ];
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ip/arp/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "ARP entry added" })))
}

pub async fn remove_arp(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/arp/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "ARP entry removed" })))
}

/// GET /api/v1/ip/pools/used - List active allocated IPs from pools
pub async fn pools_used(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client
        .run(build_command(
            "/ip/pool/used/print",
            req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        ))
        .await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "used_ips": data })))
}

/// GET /api/v1/ip/pools/utilization - Telemetry analisis utilisasi kapasitas IP Pool & Deteksi Exhaustion
pub async fn pools_utilization(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let pool_rows = client
        .run(build_command(
            "/ip/pool/print",
            std::iter::empty::<(&str, &str)>(),
        ))
        .await?;
    let used_rows = client
        .run(build_command(
            "/ip/pool/used/print",
            std::iter::empty::<(&str, &str)>(),
        ))
        .await
        .unwrap_or_default();

    let mut used_counts: HashMap<String, usize> = HashMap::new();
    for u in &used_rows {
        if let Some(p_name) = u.get("pool") {
            *used_counts.entry(p_name.to_string()).or_insert(0) += 1;
        }
    }

    let mut pool_stats = Vec::new();
    for p in pool_rows {
        let name = p.get("name").unwrap_or("unknown").to_string();
        let ranges = p.get("ranges").unwrap_or("").to_string();
        let used = *used_counts.get(&name).unwrap_or(&0);

        let total_est = estimate_pool_capacity(&ranges);
        let usage_percent = if total_est > 0 {
            (used as f64 / total_est as f64) * 100.0
        } else {
            0.0
        };

        let status = if usage_percent >= 95.0 {
            "CRITICAL_EXHAUSTED"
        } else if usage_percent >= 80.0 {
            "WARNING_HIGH"
        } else {
            "HEALTHY"
        };

        pool_stats.push(json!({
            "name": name,
            "ranges": ranges,
            "used_ips": used,
            "estimated_capacity": total_est,
            "usage_percent": format!("{:.1}%", usage_percent),
            "status": status
        }));
    }

    Ok(Json(json!({
        "success": true,
        "total_pools": pool_stats.len(),
        "total_used_ips": used_rows.len(),
        "pools": pool_stats
    })))
}

fn estimate_pool_capacity(ranges: &str) -> usize {
    let mut total = 0;
    for part in ranges.split(',') {
        let part = part.trim();
        if let Some((start_ip, end_ip)) = part.split_once('-') {
            if let (Some(s_last), Some(e_last)) = (start_ip.split('.').last(), end_ip.split('.').last()) {
                if let (Ok(s), Ok(e)) = (s_last.parse::<usize>(), e_last.parse::<usize>()) {
                    if e >= s {
                        total += (e - s) + 1;
                    }
                }
            }
        } else {
            total += 1;
        }
    }
    if total == 0 {
        254
    } else {
        total
    }
}



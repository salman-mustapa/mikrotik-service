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

/// GET & POST /api/v1/routing/bgp/sessions - Carrier BGP Peering Session Status
/// Compatible with RouterOS v7 (/routing/bgp/session/print) and v6 (/routing/bgp/peer/print)
pub async fn bgp_sessions(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req_inner = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req_inner.router, req_inner.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Try RouterOS v7 syntax first
    let res = match client.run(build_command("/routing/bgp/session/print", std::iter::empty::<(&str, &str)>())).await {
        Ok(rows) => rows,
        Err(_) => {
            // Fallback to RouterOS v6 syntax
            client.run(build_command("/routing/bgp/peer/print", std::iter::empty::<(&str, &str)>())).await?
        }
    };

    let mut sessions = Vec::new();
    for r in res {
        let name = r.get("name").unwrap_or_else(|| r.get("remote.as").unwrap_or("bgp-peer"));
        let state = r.get("state").unwrap_or_else(|| r.get("status").unwrap_or("unknown"));
        let remote_addr = r.get("remote.address").or_else(|| r.get("remote-address")).unwrap_or("");
        let remote_as = r.get("remote.as").or_else(|| r.get("remote-as")).unwrap_or("");
        let uptime = r.get("uptime").unwrap_or("");
        let prefix_count = r.get("prefix-count").or_else(|| r.get("prefixes")).unwrap_or("0");

        sessions.push(json!({
            "name": name,
            "remote_address": remote_addr,
            "remote_as": remote_as,
            "state": state,
            "is_established": state.to_lowercase() == "established",
            "uptime": uptime,
            "prefix_count": prefix_count,
            "raw": r.attrs,
        }));
    }

    Ok(Json(json!({
        "success": true,
        "total_sessions": sessions.len(),
        "sessions": sessions,
    })))
}

/// GET & POST /api/v1/routing/ospf/neighbors - OSPF Dynamic Routing Neighbor Adjacencies
pub async fn ospf_neighbors(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req_inner = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req_inner.router, req_inner.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let res = client.run(build_command("/routing/ospf/neighbor/print", std::iter::empty::<(&str, &str)>())).await?;

    let mut neighbors = Vec::new();
    for r in res {
        let router_id_val = r.get("router-id").unwrap_or("");
        let address = r.get("address").unwrap_or("");
        let interface = r.get("interface").unwrap_or("");
        let state = r.get("state").unwrap_or("");
        let priority = r.get("priority").unwrap_or("");

        neighbors.push(json!({
            "router_id": router_id_val,
            "address": address,
            "interface": interface,
            "state": state,
            "is_full": state.to_lowercase().contains("full"),
            "priority": priority,
            "raw": r.attrs,
        }));
    }

    Ok(Json(json!({
        "success": true,
        "total_neighbors": neighbors.len(),
        "neighbors": neighbors,
    })))
}

/// GET & POST /api/v1/routing/routes - Active Route Table Summary & Protocol Breakdown
pub async fn routing_table(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req_inner = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req_inner.router, req_inner.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let res = client.run(build_command("/ip/route/print", std::iter::empty::<(&str, &str)>())).await?;

    let mut static_count = 0;
    let mut connect_count = 0;
    let mut bgp_count = 0;
    let mut ospf_count = 0;
    let mut active_count = 0;
    let mut sample_routes = Vec::new();

    for r in res {
        let dst = r.get("dst-address").unwrap_or("");
        let gw = r.get("gateway").unwrap_or("");
        let active = r.get("active").map(|v| v == "true").unwrap_or(false);
        if active { active_count += 1; }

        let distance = r.get("distance").unwrap_or("1");
        let routing_mark = r.get("routing-mark").or_else(|| r.get("routing-table")).unwrap_or("main");

        let is_bgp = r.get("bgp").map(|v| v == "true").unwrap_or(false);
        let is_ospf = r.get("ospf").map(|v| v == "true").unwrap_or(false);
        let is_connect = r.get("connect").map(|v| v == "true").unwrap_or(false);
        let is_static = r.get("static").map(|v| v == "true").unwrap_or(false);

        if is_bgp { bgp_count += 1; }
        if is_ospf { ospf_count += 1; }
        if is_connect { connect_count += 1; }
        if is_static { static_count += 1; }

        if sample_routes.len() < 50 {
            sample_routes.push(json!({
                "destination": dst,
                "gateway": gw,
                "active": active,
                "distance": distance,
                "routing_table": routing_mark,
                "protocol": if is_bgp { "BGP" } else if is_ospf { "OSPF" } else if is_connect { "Connected" } else { "Static" },
            }));
        }
    }

    Ok(Json(json!({
        "success": true,
        "total_active_routes": active_count,
        "protocol_breakdown": {
            "connected": connect_count,
            "static": static_count,
            "bgp": bgp_count,
            "ospf": ospf_count,
        },
        "sample_routes": sample_routes,
    })))
}

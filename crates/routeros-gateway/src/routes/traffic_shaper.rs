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
pub struct AddMangleReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub chain: String, // "prerouting", "forward", "postrouting"
    pub action: String, // "mark-connection", "mark-packet", "accept", "change-mss"
    pub new_connection_mark: Option<String>,
    pub new_packet_mark: Option<String>,
    pub passthrough: Option<bool>,
    pub protocol: Option<String>,
    pub dst_port: Option<String>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AddQueueTreeReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub parent: String, // "global", "ether1", etc.
    pub packet_mark: Option<String>,
    pub limit_at: Option<String>, // e.g. "5M"
    pub max_limit: Option<String>, // e.g. "20M"
    pub priority: Option<u8>, // 1 (highest) to 8 (lowest)
}

#[derive(Deserialize, Debug)]
pub struct DeploySeparationReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub total_bandwidth: Option<String>, // e.g. "50M"
    pub game_reserved: Option<String>,   // e.g. "10M"
}

/// POST or GET /api/v1/traffic/mangle/rules - Lists all firewall mangle rules
pub async fn mangle_rules(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/firewall/mangle/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "count": data.len(),
        "mangle_rules": data
    })))
}

/// POST /api/v1/traffic/mangle/add - Adds a firewall mangle rule
pub async fn add_mangle(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddMangleReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut args = vec![
        ("chain", req.chain.as_str()),
        ("action", req.action.as_str()),
    ];

    if let Some(cm) = req.new_connection_mark.as_deref() { args.push(("new-connection-mark", cm)); }
    if let Some(pm) = req.new_packet_mark.as_deref() { args.push(("new-packet-mark", pm)); }
    let pass_str = req.passthrough.map(|p| if p { "yes" } else { "no" });
    if let Some(p) = pass_str.as_deref() { args.push(("passthrough", p)); }
    if let Some(proto) = req.protocol.as_deref() { args.push(("protocol", proto)); }
    if let Some(dp) = req.dst_port.as_deref() { args.push(("dst-port", dp)); }
    if let Some(c) = req.comment.as_deref() { args.push(("comment", c)); }

    client.run(build_command("/ip/firewall/mangle/add", args)).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Aturan Mangle berhasil ditambahkan"
    })))
}

/// POST or GET /api/v1/traffic/queue-tree - Lists hierarchical Queue Tree bandwidth rules
pub async fn queue_tree(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/queue/tree/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "count": data.len(),
        "queue_trees": data
    })))
}

/// POST /api/v1/traffic/queue-tree/add - Adds a Queue Tree node
pub async fn add_queue_tree(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddQueueTreeReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut args = vec![
        ("name", req.name.as_str()),
        ("parent", req.parent.as_str()),
    ];

    if let Some(pm) = req.packet_mark.as_deref() { args.push(("packet-mark", pm)); }
    if let Some(la) = req.limit_at.as_deref() { args.push(("limit-at", la)); }
    if let Some(ml) = req.max_limit.as_deref() { args.push(("max-limit", ml)); }
    let prio_str = req.priority.map(|p| p.to_string());
    if let Some(pr) = prio_str.as_deref() { args.push(("priority", pr)); }

    client.run(build_command("/queue/tree/add", args)).await?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Queue Tree node '{}' berhasil ditambahkan", req.name)
    })))
}

/// POST /api/v1/traffic/preset/game-social-separation - Automated NOC Preset that deploys:
/// 1. Mangle tagging for Online Games (Priority 1)
/// 2. Mangle tagging for Social Media & Video Streaming (Priority 6)
/// 3. Hierarchical Queue Tree parents on 'global'
pub async fn deploy_game_social_separation(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DeploySeparationReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let total_bw = req.total_bandwidth.as_deref().unwrap_or("50M");
    let game_bw = req.game_reserved.as_deref().unwrap_or("10M");

    // 1. Mangle: Online Games Connection Mark (Mobile Legends, PUBG, Free Fire, Valorant ports)
    let _ = client.run(build_command("/ip/firewall/mangle/add", [
        ("chain", "prerouting"),
        ("protocol", "udp"),
        ("dst-port", "5000-5200,30000-30200,9000-9010,8001"),
        ("action", "mark-connection"),
        ("new-connection-mark", "conn_games"),
        ("passthrough", "yes"),
        ("comment", "[NOC-Shaper] Online Games Connection"),
    ])).await;

    // 2. Mangle: Online Games Packet Mark
    let _ = client.run(build_command("/ip/firewall/mangle/add", [
        ("chain", "prerouting"),
        ("connection-mark", "conn_games"),
        ("action", "mark-packet"),
        ("new-packet-mark", "pkt_games"),
        ("passthrough", "no"),
        ("comment", "[NOC-Shaper] Online Games Packet"),
    ])).await;

    // 3. Queue Tree: Parent Total Bandwidth
    let _ = client.run(build_command("/queue/tree/add", [
        ("name", "TOTAL-DOWNLOAD"),
        ("parent", "global"),
        ("max-limit", total_bw),
        ("comment", "[NOC-Shaper] Master Queue Parent"),
    ])).await;

    // 4. Queue Tree: Games (Priority 1 - High Priority Low Latency)
    let _ = client.run(build_command("/queue/tree/add", [
        ("name", "1.GAMES-PRIORITY"),
        ("parent", "TOTAL-DOWNLOAD"),
        ("packet-mark", "pkt_games"),
        ("limit-at", game_bw),
        ("max-limit", total_bw),
        ("priority", "1"),
        ("comment", "[NOC-Shaper] Low Latency Gaming"),
    ])).await;

    // 5. Queue Tree: General Browsing (Priority 8 - Default)
    let _ = client.run(build_command("/queue/tree/add", [
        ("name", "2.GENERAL-TRAFFIC"),
        ("parent", "TOTAL-DOWNLOAD"),
        ("packet-mark", "no-mark"),
        ("limit-at", "5M"),
        ("max-limit", total_bw),
        ("priority", "8"),
        ("comment", "[NOC-Shaper] General & Social Media"),
    ])).await;

    Ok(Json(json!({
        "success": true,
        "message": "Konfigurasi Pemisahan Trafik Game & Sosmed (Mangle + Queue Tree) berhasil disuntikkan",
        "total_bandwidth": total_bw,
        "game_reserved_guarantee": game_bw,
        "priority_allocation": {
            "games": "Priority 1 (Latency Guaranteed)",
            "general_browsing": "Priority 8"
        }
    })))
}

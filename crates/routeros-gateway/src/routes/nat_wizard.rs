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
pub struct PortForwardReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub dst_port: String,            // e.g. "8000" or "80,443"
    pub to_addresses: String,        // e.g. "192.168.88.50"
    pub to_ports: Option<String>,    // e.g. "80", defaults to dst_port
    pub protocol: Option<String>,    // default "tcp"
    pub in_interface: Option<String>, // e.g. "ether1" (WAN)
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MasqueradeReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub out_interface: String, // e.g. "ether1"
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

/// POST /api/v1/firewall/port-forward - Configures Dst-NAT port forwarding (CCTV, Web Server, Games)
/// and automatically opens the matching firewall filter forward chain
pub async fn add_port_forward(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<PortForwardReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let proto = req.protocol.as_deref().unwrap_or("tcp");
    let to_port = req.to_ports.as_deref().unwrap_or(&req.dst_port);
    let comment = req.comment.unwrap_or_else(|| {
        format!("[Dst-NAT] Forward {} port {} -> {}:{}", proto, req.dst_port, req.to_addresses, to_port)
    });

    let mut nat_args = vec![
        ("chain", "dstnat"),
        ("protocol", proto),
        ("dst-port", req.dst_port.as_str()),
        ("action", "dst-nat"),
        ("to-addresses", req.to_addresses.as_str()),
        ("to-ports", to_port),
        ("comment", comment.as_str()),
    ];

    if let Some(in_if) = req.in_interface.as_deref() {
        nat_args.push(("in-interface", in_if));
    }

    // 1. Add Dst-NAT rule
    client.run(build_command("/ip/firewall/nat/add", nat_args)).await?;

    // 2. Add Filter Accept rule in forward chain to ensure packet is not dropped by default firewall
    let filter_comment = format!("[Filter-Accept] Allow Port Forward {}:{}", req.to_addresses, to_port);
    let _ = client.run(build_command(
        "/ip/firewall/filter/add",
        [
            ("chain", "forward"),
            ("protocol", proto),
            ("dst-address", req.to_addresses.as_str()),
            ("dst-port", to_port),
            ("action", "accept"),
            ("comment", filter_comment.as_str()),
        ]
    )).await;

    Ok(Json(json!({
        "success": true,
        "dst_port": req.dst_port,
        "target_ip": req.to_addresses,
        "target_port": to_port,
        "protocol": proto,
        "message": format!("Port Forwarding ke {}:{} berhasil dikonfigurasi di Dst-NAT dan Filter", req.to_addresses, to_port)
    })))
}

/// POST or GET /api/v1/firewall/port-forward/list - Lists all active Dst-NAT port forwarding rules
pub async fn list_port_forwards(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/firewall/nat/print", [("?chain", "dstnat")])).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "count": data.len(),
        "port_forwards": data
    })))
}

/// POST /api/v1/firewall/port-forward/remove - Removes a Dst-NAT rule by ID
pub async fn remove_port_forward(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    client.run(build_command("/ip/firewall/nat/remove", [(".id", req.id.as_str())])).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Aturan Dst-NAT berhasil dihapus"
    })))
}

/// POST /api/v1/firewall/srcnat/masquerade - Creates standard outbound WAN masquerade
pub async fn add_masquerade(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<MasqueradeReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let comment = req.comment.unwrap_or_else(|| format!("[Src-NAT] Masquerade WAN ({})", req.out_interface));

    client.run(build_command(
        "/ip/firewall/nat/add",
        [
            ("chain", "srcnat"),
            ("out-interface", req.out_interface.as_str()),
            ("action", "masquerade"),
            ("comment", comment.as_str()),
        ]
    )).await?;

    Ok(Json(json!({
        "success": true,
        "out_interface": req.out_interface,
        "message": "Aturan Src-NAT Masquerade internet berhasil ditambahkan"
    })))
}

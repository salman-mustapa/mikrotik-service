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
    pub enable_hairpin: Option<bool>, // enable Hairpin NAT for LAN loopback access
    pub lan_subnet: Option<String>,   // e.g. "192.168.88.0/24"
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct HairpinNatReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub lan_subnet: String,          // e.g. "192.168.88.0/24"
    pub server_address: String,      // e.g. "192.168.88.50"
    pub dst_port: Option<String>,    // e.g. "80"
    pub protocol: Option<String>,    // default "tcp"
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
/// and automatically opens the matching firewall filter forward chain, with optional Hairpin NAT
pub async fn add_port_forward(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<PortForwardReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_ports_list(&req.dst_port)?;
    crate::routes::sanitize::validate_ip_or_cidr(&req.to_addresses)?;
    if let Some(tp) = req.to_ports.as_deref() {
        crate::routes::sanitize::validate_ports_list(tp)?;
    }
    if let Some(in_if) = req.in_interface.as_deref() {
        crate::routes::sanitize::validate_safe_identifier(in_if, "in_interface")?;
    }
    if let Some(ref lan) = req.lan_subnet {
        crate::routes::sanitize::validate_ip_or_cidr(lan)?;
    }

    let proto = req.protocol.as_deref().unwrap_or("tcp");
    let to_port = req.to_ports.as_deref().unwrap_or(&req.dst_port);
    let comment = req.comment.unwrap_or_else(|| {
        format!("[Dst-NAT] Forward {} port {} -> {}:{}", proto, req.dst_port, req.to_addresses, to_port)
    });

    let is_hairpin = req.enable_hairpin.unwrap_or(false);

    let mut nat_args = vec![
        ("chain", "dstnat"),
        ("protocol", proto),
        ("dst-port", req.dst_port.as_str()),
        ("action", "dst-nat"),
        ("to-addresses", req.to_addresses.as_str()),
        ("to-ports", to_port),
        ("comment", comment.as_str()),
    ];

    if is_hairpin {
        // When hairpin is enabled, match local destination type instead of strictly in-interface
        // so packets originating from LAN can also trigger dstnat
        nat_args.push(("dst-address-type", "local"));
    } else if let Some(in_if) = req.in_interface.as_deref() {
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

    // 3. If Hairpin NAT is requested, add Src-NAT masquerade rule for LAN subnet to server
    if is_hairpin {
        let lan_sub = req.lan_subnet.unwrap_or_else(|| {
            // Deduce subnet from to_addresses, e.g. 192.168.88.50 -> 192.168.88.0/24
            if let Some((prefix, _)) = req.to_addresses.rsplit_once('.') {
                format!("{}.0/24", prefix)
            } else {
                "192.168.88.0/24".to_string()
            }
        });

        let hairpin_comment = format!("[Hairpin-NAT] Loopback masquerade {} to {}:{}", lan_sub, req.to_addresses, to_port);
        let _ = client.run(build_command(
            "/ip/firewall/nat/add",
            [
                ("chain", "srcnat"),
                ("src-address", lan_sub.as_str()),
                ("dst-address", req.to_addresses.as_str()),
                ("protocol", proto),
                ("dst-port", to_port),
                ("action", "masquerade"),
                ("comment", hairpin_comment.as_str()),
            ]
        )).await;
    }

    Ok(Json(json!({
        "success": true,
        "dst_port": req.dst_port,
        "target_ip": req.to_addresses,
        "target_port": to_port,
        "protocol": proto,
        "hairpin_enabled": is_hairpin,
        "message": format!(
            "Port Forwarding ke {}:{} berhasil dikonfigurasi di Dst-NAT dan Filter{}",
            req.to_addresses,
            to_port,
            if is_hairpin { " (dengan Hairpin NAT Loopback)" } else { "" }
        )
    })))
}

/// POST /api/v1/firewall/hairpin-nat - Konfigurasi aturan mandiri Hairpin NAT Loopback
pub async fn add_hairpin_nat(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<HairpinNatReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_ip_or_cidr(&req.lan_subnet)?;
    crate::routes::sanitize::validate_ip_or_cidr(&req.server_address)?;
    if let Some(ref dp) = req.dst_port {
        crate::routes::sanitize::validate_ports_list(dp)?;
    }

    let proto = req.protocol.as_deref().unwrap_or("tcp");
    let comment = req.comment.unwrap_or_else(|| {
        format!("[Hairpin-NAT] Loopback NAT {} -> {}", req.lan_subnet, req.server_address)
    });

    let mut args = vec![
        ("chain", "srcnat"),
        ("src-address", req.lan_subnet.as_str()),
        ("dst-address", req.server_address.as_str()),
        ("protocol", proto),
        ("action", "masquerade"),
        ("comment", comment.as_str()),
    ];

    if let Some(ref dp) = req.dst_port {
        args.push(("dst-port", dp.as_str()));
    }

    client.run(build_command("/ip/firewall/nat/add", args)).await?;

    Ok(Json(json!({
        "success": true,
        "lan_subnet": req.lan_subnet,
        "server_address": req.server_address,
        "protocol": proto,
        "message": "Aturan Hairpin NAT Loopback berhasil ditambahkan ke RouterOS"
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
    crate::routes::sanitize::validate_safe_identifier(&req.id, "id")?;

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

    crate::routes::sanitize::validate_safe_identifier(&req.out_interface, "out_interface")?;

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

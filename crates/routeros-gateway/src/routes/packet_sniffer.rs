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
pub struct StartSnifferReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// Interface yang ingin disniff (default: "all")
    pub interface: Option<String>,
    /// Filter IP spesifik (misal: "192.168.88.50")
    pub filter_ip_address: Option<String>,
    /// Filter Port spesifik (misal: "80,443")
    pub filter_port: Option<String>,
    /// Filter Protokol (misal: "tcp", "udp", "icmp")
    pub filter_protocol: Option<String>,
    /// Simpan ke nama file di router (misal: "capture.pcap")
    pub file_name: Option<String>,
    /// Batas ukuran file (default: "1000KiB")
    pub file_limit: Option<String>,
    /// Streaming langsung via TZSP ke IP PC Wireshark (misal: "192.168.88.10")
    pub streaming_server: Option<String>,
}

/// GET or POST /api/v1/tools/sniffer/status - Menampilkan status Packet Sniffer saat ini
pub async fn sniffer_status(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client
        .run(build_command("/tool/sniffer/print", std::iter::empty::<(&str, &str)>()))
        .await?;

    let data: HashMap<String, String> = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    let is_running = data.get("running").map(|v| v == "yes" || v == "true").unwrap_or(false);

    Ok(Json(json!({
        "success": true,
        "running": is_running,
        "sniffer_status": data
    })))
}

/// POST /api/v1/tools/sniffer/start - Memulai perekaman atau streaming paket data (Wireshark / .pcap)
pub async fn sniffer_start(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<StartSnifferReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // 1. Konfigurasi parameter sniffer
    let mut set_args = Vec::new();
    let iface = req.interface.as_deref().unwrap_or("all");
    set_args.push(("interface", iface));

    if let Some(ip) = req.filter_ip_address.as_deref() {
        set_args.push(("filter-ip-address", ip));
    }
    if let Some(p) = req.filter_port.as_deref() {
        crate::routes::sanitize::validate_ports_list(p)?;
        set_args.push(("filter-port", p));
    }
    if let Some(proto) = req.filter_protocol.as_deref() {
        set_args.push(("filter-protocol", proto));
    }

    if let Some(fn_str) = req.file_name.as_deref() {
        crate::routes::sanitize::validate_safe_filename(fn_str)?;
        set_args.push(("file-name", fn_str));
        let fl = req.file_limit.as_deref().unwrap_or("2000KiB");
        set_args.push(("file-limit", fl));
    } else {
        set_args.push(("file-name", ""));
    }

    if let Some(srv) = req.streaming_server.as_deref() {
        if crate::state::is_blocked_host(srv) {
            return Err(ApiError::Forbidden(
                "Streaming server target is blocked by security policy (loopback, link-local, or cloud metadata)".into()
            ));
        }
        set_args.push(("streaming-enabled", "yes"));
        set_args.push(("streaming-server", srv));
    } else {
        set_args.push(("streaming-enabled", "no"));
    }

    client.run(build_command("/tool/sniffer/set", set_args)).await?;

    // 2. Jalankan sniffer
    client.run(build_command("/tool/sniffer/start", std::iter::empty::<(&str, &str)>())).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Packet Sniffer berhasil dijalankan",
        "interface": iface,
        "streaming_to": req.streaming_server,
        "file_name": req.file_name,
        "filter": {
            "ip": req.filter_ip_address,
            "port": req.filter_port,
            "protocol": req.filter_protocol
        }
    })))
}

/// POST /api/v1/tools/sniffer/stop - Menghentikan perekaman Packet Sniffer
pub async fn sniffer_stop(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    client.run(build_command("/tool/sniffer/stop", std::iter::empty::<(&str, &str)>())).await?;

    let rows = client
        .run(build_command("/tool/sniffer/print", std::iter::empty::<(&str, &str)>()))
        .await?;
    let data: HashMap<String, String> = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();

    Ok(Json(json!({
        "success": true,
        "message": "Packet Sniffer berhasil dihentikan",
        "final_status": data
    })))
}

/// POST /api/v1/tools/sniffer/reset - Mereset buffer paket internal sniffer
pub async fn sniffer_reset(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let _ = client.run(build_command("/tool/sniffer/stop", std::iter::empty::<(&str, &str)>())).await;
    client.run(build_command("/tool/sniffer/packet/clear", std::iter::empty::<(&str, &str)>())).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Buffer packet sniffer berhasil dibersihkan"
    })))
}

#[derive(Deserialize, Debug, Default)]
pub struct PacketQueryReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub limit: Option<usize>,
}

/// GET or POST /api/v1/tools/sniffer/packets - Melihat isi buffer paket yang tertangkap (/tool/sniffer/packet/print)
pub async fn sniffer_packets(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<PacketQueryReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client
        .run(build_command("/tool/sniffer/packet/print", std::iter::empty::<(&str, &str)>()))
        .await?;

    let limit = req.limit.unwrap_or(100);
    let total = rows.len();
    let data: Vec<HashMap<String, String>> = rows.into_iter().take(limit).map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "total_packets_captured": total,
        "limit_applied": limit,
        "packets": data
    })))
}

/// GET or POST /api/v1/tools/sniffer/hosts - Melihat daftar host dan statistik trafik yang ditangkap sniffer
pub async fn sniffer_hosts(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client
        .run(build_command("/tool/sniffer/host/print", std::iter::empty::<(&str, &str)>()))
        .await?;

    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "total_hosts": data.len(),
        "hosts": data
    })))
}

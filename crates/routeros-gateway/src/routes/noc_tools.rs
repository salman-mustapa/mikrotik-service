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
pub struct TorchReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub interface: String,
    pub src_address: Option<String>,
    pub dst_address: Option<String>,
    pub port: Option<String>,
    pub protocol: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IpScanReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub interface: String,
    pub address_range: Option<String>, // e.g. "192.168.88.0/24"
    pub duration: Option<String>,      // default "3s"
}

#[derive(Deserialize, Debug)]
pub struct SetRomonReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub enabled: bool,
    pub id: Option<String>,
    pub secrets: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct SetWatchdogReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub watchdog_timer: Option<bool>,
    pub ping_address: Option<String>, // e.g. "8.8.8.8"
    pub ping_timeout: Option<String>, // e.g. "1m"
    pub ping_start_after_boot: Option<String>, // e.g. "5m"
    pub send_email_from: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct SetNtpReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub enabled: Option<bool>,
    pub servers: Option<String>, // e.g. "0.pool.ntp.org,1.pool.ntp.org"
    pub time_zone_name: Option<String>, // e.g. "Asia/Jakarta"
}

#[derive(Deserialize, Debug)]
pub struct AddDhcpClientReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub interface: String,
    pub add_default_route: Option<bool>,
    pub use_peer_dns: Option<bool>,
    pub use_peer_ntp: Option<bool>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TerminalExecReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub script: String, // Multi-line script content pasted from terminal
}

/// POST /api/v1/tools/torch - Real-time traffic sniffer on an interface
pub async fn torch(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<TorchReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut args = vec![
        ("interface", req.interface.as_str()),
        ("duration", "2s"),
    ];

    if let Some(src) = req.src_address.as_deref() { args.push(("src-address", src)); }
    if let Some(dst) = req.dst_address.as_deref() { args.push(("dst-address", dst)); }
    if let Some(p) = req.port.as_deref() { args.push(("port", p)); }
    if let Some(proto) = req.protocol.as_deref() { args.push(("ip-protocol", proto)); }

    let rows = client.run(build_command("/tool/torch", args)).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "interface": req.interface,
        "flows_count": data.len(),
        "traffic_flows": data
    })))
}

/// POST /api/v1/tools/ip-scan - Discovers active IP/MAC devices on an interface or subnet range
pub async fn ip_scan(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IpScanReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let dur = req.duration.as_deref().unwrap_or("3s");
    let mut args = vec![
        ("interface", req.interface.as_str()),
        ("duration", dur),
    ];

    if let Some(rng) = req.address_range.as_deref() {
        args.push(("address-range", rng));
    }

    let rows = client.run(build_command("/tool/ip-scan", args)).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "interface": req.interface,
        "discovered_count": data.len(),
        "devices": data
    })))
}

/// POST or GET /api/v1/tools/romon/status - Checks RoMON (Router Management Overlay Network) status
pub async fn romon_status(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/tool/romon/print", std::iter::empty::<(&str, &str)>())).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();

    Ok(Json(json!({ "success": true, "romon": first })))
}

/// POST /api/v1/tools/romon/set - Configures RoMON
pub async fn romon_set(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetRomonReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut args = vec![("enabled", if req.enabled { "yes" } else { "no" })];
    if let Some(id) = req.id.as_deref() { args.push(("id", id)); }
    if let Some(sec) = req.secrets.as_deref() { args.push(("secrets", sec)); }

    client.run(build_command("/tool/romon/set", args)).await?;

    Ok(Json(json!({
        "success": true,
        "message": format!("RoMON status diubah menjadi enabled={}", req.enabled)
    })))
}

/// POST or GET /api/v1/tools/romon/discover - Discovers peer routers via RoMON L2 overlay
pub async fn romon_discover(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/tool/romon/discovery/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "count": data.len(),
        "romon_peers": data
    })))
}

/// POST or GET /api/v1/system/watchdog - Configures hardware watchdog auto-reboot on ISP gateway failure
pub async fn watchdog(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<SetWatchdogReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut args = Vec::new();
    let wt_str = req.watchdog_timer.map(|w| if w { "yes" } else { "no" });
    if let Some(wt) = wt_str.as_deref() { args.push(("watchdog-timer", wt)); }
    if let Some(ping) = req.ping_address.as_deref() { args.push(("ping-address", ping)); }
    if let Some(to) = req.ping_timeout.as_deref() { args.push(("ping-timeout", to)); }
    if let Some(boot) = req.ping_start_after_boot.as_deref() { args.push(("ping-start-after-boot", boot)); }

    let is_updating = !args.is_empty();
    if is_updating {
        client.run(build_command("/system/watchdog/set", args)).await?;
    }

    let rows = client.run(build_command("/system/watchdog/print", std::iter::empty::<(&str, &str)>())).await?;
    let current = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();

    Ok(Json(json!({
        "success": true,
        "message": if is_updating { "Watchdog konfigurasi diperbarui" } else { "Watchdog konfigurasi saat ini" },
        "current_settings": current
    })))
}

/// POST or GET /api/v1/system/ntp - Configures NTP Client time synchronization
pub async fn ntp_config(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<SetNtpReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut configured = false;
    if let Some(en) = req.enabled {
        let en_str = if en { "yes" } else { "no" };
        let mut args = vec![("enabled", en_str)];
        if let Some(srv) = req.servers.as_deref() { args.push(("servers", srv)); }

        // Try RouterOS v7 /system/ntp/client/set, fallback to v6 /system/ntp/client/set
        let res = client.run(build_command("/system/ntp/client/set", args.clone())).await;
        if res.is_err() {
            // v6 format e.g. primary-ntp
            let _ = client.run(build_command("/system/ntp/client/set", [("enabled", en_str)])).await;
        }
        configured = true;
    }

    if let Some(tz) = req.time_zone_name.as_deref() {
        let _ = client.run(build_command("/system/clock/set", [("time-zone-name", tz)])).await;
        configured = true;
    }

    let rows = client.run(build_command("/system/ntp/client/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let current = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();

    Ok(Json(json!({
        "success": true,
        "message": if configured { "NTP Client & Clock Timezone berhasil disetel" } else { "NTP Client konfigurasi saat ini" },
        "settings": current
    })))
}

/// POST or GET /api/v1/ip/dhcp-client/all - Lists all DHCP Clients on the router
pub async fn dhcp_clients(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/dhcp-client/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "count": data.len(),
        "dhcp_clients": data
    })))
}

/// POST /api/v1/ip/dhcp-client/add - Adds a DHCP Client on a specified WAN interface
pub async fn add_dhcp_client(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddDhcpClientReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let def_route = if req.add_default_route.unwrap_or(true) { "yes" } else { "no" };
    let peer_dns = if req.use_peer_dns.unwrap_or(true) { "yes" } else { "no" };
    let peer_ntp = if req.use_peer_ntp.unwrap_or(true) { "yes" } else { "no" };

    let mut args = vec![
        ("interface", req.interface.as_str()),
        ("disabled", "no"),
        ("add-default-route", def_route),
        ("use-peer-dns", peer_dns),
        ("use-peer-ntp", peer_ntp),
    ];
    if let Some(c) = req.comment.as_deref() { args.push(("comment", c)); }

    client.run(build_command("/ip/dhcp-client/add", args)).await?;

    Ok(Json(json!({
        "success": true,
        "message": format!("DHCP Client aktif pada interface '{}'", req.interface)
    })))
}

/// POST /api/v1/scripts/terminal-exec - Executes multi-line RouterOS CLI script batch
/// (Supports copy-pasting raw terminal configuration commands directly)
pub async fn terminal_exec(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<TerminalExecReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Create a temporary script in /system/script and run it to support complex syntax
    let temp_name = format!("tmp_exec_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0));

    // 1. Add script
    client.run(build_command(
        "/system/script/add",
        [
            ("name", temp_name.as_str()),
            ("source", req.script.as_str()),
            ("policy", "ftp,reboot,read,write,policy,test,password,sniff,sensitive,romon"),
        ]
    )).await?;

    // 2. Run script
    let run_res = client.run(build_command("/system/script/run", [("number", temp_name.as_str())])).await;

    // 3. Clean up script
    let _ = client.run(build_command("/system/script/remove", [(".id", temp_name.as_str())])).await;

    match run_res {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "message": "Script CLI berhasil dieksekusi secara atomik di router"
        }))),
        Err(e) => Err(ApiError::Internal(format!("Gagal mengeksekusi script: {}", e)))
    }
}

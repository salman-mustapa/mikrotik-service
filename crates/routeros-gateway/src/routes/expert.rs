use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
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

#[derive(Deserialize, Debug, Default)]
pub struct QuickDiagnoseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub ping_target: Option<String>,
}

/// POST /api/v1/expert/quick-diagnose - Dispatches ICMP ping, DNS test, default gateway check,
/// and CPU/memory health in parallel, returning an instant Network Quality Score.
pub async fn quick_diagnose(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<QuickDiagnoseReq>,
) -> Result<Json<Value>, ApiError> {
    let t0 = Instant::now();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let ping_host = req.ping_target.as_deref().unwrap_or("8.8.8.8");

    // 1. Concurrently run ping, default route, dns, and resources
    let ping_fut = client.run(build_command("/ping", [("address", ping_host), ("count", "2")]));
    let route_fut = client.run(build_command("/ip/route/print", [("?dst-address", "0.0.0.0/0")]));
    let dns_fut = client.run(build_command("/ip/dns/print", std::iter::empty::<(&str, &str)>()));
    let res_fut = client.run(build_command("/system/resource/print", std::iter::empty::<(&str, &str)>()));

    let (ping_res, route_res, dns_res, res_res) = tokio::join!(ping_fut, route_fut, dns_fut, res_fut);

    // Evaluate Ping
    let mut ping_ok = false;
    let mut avg_rtt = "timeout".to_string();
    let mut packet_loss = 100;
    if let Ok(rows) = ping_res {
        let sent = rows.len();
        let received = rows.iter().filter(|r| r.get("received").is_some_and(|v| v != "0")).count();
        if sent > 0 {
            packet_loss = ((sent.saturating_sub(received)) * 100) / sent;
            ping_ok = received > 0;
        }
        if let Some(last) = rows.last() {
            if let Some(rtt) = last.get("avg-rtt").or_else(|| last.get("time")) {
                avg_rtt = rtt.to_string();
            }
        }
    }

    // Evaluate Default Gateway
    let mut default_gw = "none".to_string();
    let mut gw_reachable = false;
    if let Ok(routes) = route_res {
        if let Some(first) = routes.first() {
            if let Some(gw) = first.get("gateway") {
                default_gw = gw.to_string();
            }
            gw_reachable = first.get("active").map(|v| v == "true").unwrap_or(false);
        }
    }

    // Evaluate DNS
    let mut dns_servers = "none".to_string();
    if let Ok(dns_rows) = dns_res {
        if let Some(first) = dns_rows.first() {
            if let Some(srv) = first.get("servers").or_else(|| first.get("dynamic-servers")) {
                dns_servers = srv.to_string();
            }
        }
    }

    // Evaluate Resource
    let mut cpu_load = 0;
    let mut free_mem_mb = 0;
    let mut uptime = "".to_string();
    if let Ok(res_rows) = res_res {
        if let Some(first) = res_rows.first() {
            cpu_load = first.get("cpu-load").and_then(|v| v.parse().ok()).unwrap_or(0);
            let free_bytes: u64 = first.get("free-memory").and_then(|v| v.parse().ok()).unwrap_or(0);
            free_mem_mb = free_bytes / (1024 * 1024);
            uptime = first.get("uptime").unwrap_or("").to_string();
        }
    }

    // Compute Health Score (0 - 100)
    let mut score: u32 = 100;
    if !gw_reachable { score = score.saturating_sub(40); }
    if !ping_ok { score = score.saturating_sub(30); } else if packet_loss > 0 { score = score.saturating_sub(15); }
    if cpu_load > 85 { score = score.saturating_sub(20); } else if cpu_load > 60 { score = score.saturating_sub(10); }

    let elapsed = t0.elapsed();

    Ok(Json(json!({
        "success": true,
        "diagnose_time_ms": elapsed.as_millis() as u64,
        "network_score": score,
        "status": if score >= 80 { "EXCELLENT" } else if score >= 50 { "DEGRADED" } else { "CRITICAL" },
        "diagnostics": {
            "internet_connectivity": {
                "ping_target": ping_host,
                "reachable": ping_ok,
                "packet_loss_percent": packet_loss,
                "average_rtt": avg_rtt
            },
            "routing": {
                "default_gateway": default_gw,
                "gateway_active": gw_reachable
            },
            "dns": {
                "upstream_servers": dns_servers
            },
            "hardware_health": {
                "cpu_load_percent": cpu_load,
                "free_memory_mb": free_mem_mb,
                "uptime": uptime
            }
        }
    })))
}

/// POST /api/v1/expert/traffic-matrix - Real-time traffic snapshot across all physical ports
pub async fn traffic_matrix(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let t0 = Instant::now();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let ifaces = client.run(build_command("/interface/print", std::iter::empty::<(&str, &str)>())).await?;

    let mut running_ifaces = Vec::new();
    for row in ifaces {
        let is_running = row.get("running").map(|v| v == "true").unwrap_or(false);
        let name = row.get("name").unwrap_or("").to_string();
        let if_type = row.get("type").unwrap_or("").to_string();
        let rx_b = row.get("rx-byte").unwrap_or("0").to_string();
        let tx_b = row.get("tx-byte").unwrap_or("0").to_string();
        let rx_p = row.get("rx-packet").unwrap_or("0").to_string();
        let tx_p = row.get("tx-packet").unwrap_or("0").to_string();

        running_ifaces.push(json!({
            "name": name,
            "type": if_type,
            "running": is_running,
            "rx_bytes": rx_b,
            "tx_bytes": tx_b,
            "rx_packets": rx_p,
            "tx_packets": tx_p,
        }));
    }

    let elapsed = t0.elapsed();

    Ok(Json(json!({
        "success": true,
        "execution_time_ms": elapsed.as_millis() as u64,
        "total_interfaces": running_ifaces.len(),
        "matrix": running_ifaces
    })))
}

/// POST /api/v1/expert/security-audit - Audits router services and configurations for security risks
pub async fn security_audit(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let services = client.run(build_command("/ip/service/print", std::iter::empty::<(&str, &str)>())).await?;
    let users = client.run(build_command("/user/print", std::iter::empty::<(&str, &str)>())).await?;
    let dns = client.run(build_command("/ip/dns/print", std::iter::empty::<(&str, &str)>())).await?;

    let mut warnings = Vec::new();
    let mut score: u32 = 100;

    for s in services {
        let name = s.get("name").unwrap_or("");
        let disabled = s.get("disabled").map(|v| v == "true").unwrap_or(false);
        if !disabled {
            if name == "telnet" {
                warnings.push("Port Telnet aktif (Plaintext unencrypted protocol. Disarankan dinonaktifkan)".to_string());
                score = score.saturating_sub(20);
            }
            if name == "ftp" {
                warnings.push("Port FTP aktif (Rentan sniffing kredensial. Disarankan dinonaktifkan)".to_string());
                score = score.saturating_sub(15);
            }
        }
    }

    if let Some(admin) = users.iter().find(|u| u.get("name").is_some_and(|n| n == "admin")) {
        let has_no_comment = admin.get("comment").is_none();
        if has_no_comment {
            warnings.push("User default 'admin' masih ada. Disarankan buat user nama lain dan nonaktifkan 'admin'".to_string());
            score = score.saturating_sub(15);
        }
    }

    if let Some(first_dns) = dns.first() {
        if first_dns.get("allow-remote-requests").is_some_and(|v| v == "true") {
            warnings.push("DNS Allow-Remote-Requests aktif. Pastikan firewall memblokir port 53 UDP/TCP dari WAN agar tidak menjadi open resolver".to_string());
            score = score.saturating_sub(25);
        }
    }

    Ok(Json(json!({
        "success": true,
        "security_score": score,
        "rating": if score >= 85 { "SECURE" } else if score >= 60 { "MODERATE_RISK" } else { "CRITICAL_RISK" },
        "total_warnings": warnings.len(),
        "warnings": warnings,
        "recommendation": if score < 100 { "Gunakan endpoint /api/v1/system/service/toggle untuk mematikan servis tidak terenkripsi" } else { "Konfigurasi keamanan sangat baik" }
    })))
}

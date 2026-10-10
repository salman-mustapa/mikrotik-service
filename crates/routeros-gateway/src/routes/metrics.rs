use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use routeros_core::build_command;
use serde::Deserialize;

use crate::state::{AppState, RouterTarget};

#[derive(Deserialize, Debug, Default)]
pub struct MetricsQuery {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub pass: Option<String>,
    pub token: Option<String>,
    pub router_id: Option<String>,
}

fn parse_uptime_to_seconds(uptime: &str) -> u64 {
    // MikroTik uptime format examples: "2w3d14h20m10s", "1d04:22:15", "03:15:20"
    let mut total_secs: u64 = 0;
    let mut current_num: u64 = 0;

    for ch in uptime.chars() {
        if ch.is_ascii_digit() {
            current_num = current_num * 10 + (ch as u64 - '0' as u64);
        } else {
            match ch {
                'w' => { total_secs += current_num * 7 * 86400; current_num = 0; }
                'd' => { total_secs += current_num * 86400; current_num = 0; }
                'h' => { total_secs += current_num * 3600; current_num = 0; }
                'm' => { total_secs += current_num * 60; current_num = 0; }
                's' => { total_secs += current_num; current_num = 0; }
                _ => {}
            }
        }
    }
    total_secs + current_num
}

/// GET /metrics - Native Prometheus Exporter for MikroTik RouterOS Fleet
/// Scrapes system resources, interface counters, PPPoE sessions, and Hotspot vouchers
/// Returns standard Prometheus text/plain exposition format (version 0.0.4)
pub async fn prometheus_metrics(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<MetricsQuery>,
) -> impl IntoResponse {
    let target = if let Some(h) = query.host {
        Some(RouterTarget {
            host: Some(h),
            port: Some(query.port.unwrap_or(8728)),
            user: Some(query.user.unwrap_or_else(|| "admin".into())),
            password: Some(query.pass.unwrap_or_default()),
        })
    } else {
        None
    };

    let (resolved_target, router_id) = AppState::parse_target(&headers, target, query.router_id);

    let client_res = st.resolve_client(resolved_target.as_ref(), router_id.as_deref()).await;

    let target_label = resolved_target
        .as_ref()
        .and_then(|t| t.host.clone())
        .or_else(|| router_id.clone())
        .unwrap_or_else(|| "default_router".into());

    let mut out = String::with_capacity(4096);

    out.push_str("# HELP mikrotik_up Scrape status of target MikroTik router (1=up, 0=down)\n");
    out.push_str("# TYPE mikrotik_up gauge\n");

    let client = match client_res {
        Ok(c) => {
            out.push_str(&format!("mikrotik_up{{host=\"{}\"}} 1\n\n", target_label));
            c
        }
        Err(_) => {
            out.push_str(&format!("mikrotik_up{{host=\"{}\"}} 0\n", target_label));
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
                out,
            );
        }
    };

    // Parallel fetch: resource, interface, ppp active, hotspot active
    let res_fut = client.run(build_command("/system/resource/print", std::iter::empty::<(&str, &str)>()));
    let if_fut = client.run(build_command("/interface/print", std::iter::empty::<(&str, &str)>()));
    let ppp_fut = client.run(build_command("/ppp/active/print", std::iter::empty::<(&str, &str)>()));
    let hs_fut = client.run(build_command("/ip/hotspot/active/print", std::iter::empty::<(&str, &str)>()));

    let (res_res, if_res, ppp_res, hs_res) = tokio::join!(res_fut, if_fut, ppp_fut, hs_fut);

    // 1. System Resource Metrics
    if let Ok(rows) = res_res {
        if let Some(r) = rows.first() {
            let cpu_load: u32 = r.get("cpu-load").and_then(|v| v.parse().ok()).unwrap_or(0);
            let free_mem: u64 = r.get("free-memory").and_then(|v| v.parse().ok()).unwrap_or(0);
            let total_mem: u64 = r.get("total-memory").and_then(|v| v.parse().ok()).unwrap_or(0);
            let free_hdd: u64 = r.get("free-hdd-space").and_then(|v| v.parse().ok()).unwrap_or(0);
            let total_hdd: u64 = r.get("total-hdd-space").and_then(|v| v.parse().ok()).unwrap_or(0);
            let uptime_str = r.get("uptime").unwrap_or("");
            let uptime_secs = parse_uptime_to_seconds(uptime_str);

            out.push_str("# HELP mikrotik_cpu_load_percent Current CPU load percentage\n# TYPE mikrotik_cpu_load_percent gauge\n");
            out.push_str(&format!("mikrotik_cpu_load_percent{{host=\"{}\"}} {}\n\n", target_label, cpu_load));

            out.push_str("# HELP mikrotik_memory_free_bytes Free memory in bytes\n# TYPE mikrotik_memory_free_bytes gauge\n");
            out.push_str(&format!("mikrotik_memory_free_bytes{{host=\"{}\"}} {}\n\n", target_label, free_mem));

            out.push_str("# HELP mikrotik_memory_total_bytes Total installed memory in bytes\n# TYPE mikrotik_memory_total_bytes gauge\n");
            out.push_str(&format!("mikrotik_memory_total_bytes{{host=\"{}\"}} {}\n\n", target_label, total_mem));

            out.push_str("# HELP mikrotik_hdd_free_bytes Free storage space in bytes\n# TYPE mikrotik_hdd_free_bytes gauge\n");
            out.push_str(&format!("mikrotik_hdd_free_bytes{{host=\"{}\"}} {}\n\n", target_label, free_hdd));

            out.push_str("# HELP mikrotik_hdd_total_bytes Total storage space in bytes\n# TYPE mikrotik_hdd_total_bytes gauge\n");
            out.push_str(&format!("mikrotik_hdd_total_bytes{{host=\"{}\"}} {}\n\n", target_label, total_hdd));

            out.push_str("# HELP mikrotik_uptime_seconds Router uptime in seconds\n# TYPE mikrotik_uptime_seconds counter\n");
            out.push_str(&format!("mikrotik_uptime_seconds{{host=\"{}\"}} {}\n\n", target_label, uptime_secs));
        }
    }

    // 2. Subscriber Metrics
    let ppp_count = ppp_res.ok().map(|r| r.len()).unwrap_or(0);
    out.push_str("# HELP mikrotik_pppoe_active_subscribers Number of online PPPoE sessions\n# TYPE mikrotik_pppoe_active_subscribers gauge\n");
    out.push_str(&format!("mikrotik_pppoe_active_subscribers{{host=\"{}\"}} {}\n\n", target_label, ppp_count));

    let hs_count = hs_res.ok().map(|r| r.len()).unwrap_or(0);
    out.push_str("# HELP mikrotik_hotspot_active_vouchers Number of authenticated Hotspot users\n# TYPE mikrotik_hotspot_active_vouchers gauge\n");
    out.push_str(&format!("mikrotik_hotspot_active_vouchers{{host=\"{}\"}} {}\n\n", target_label, hs_count));

    // 3. Interface Traffic & Link Metrics
    if let Ok(interfaces) = if_res {
        out.push_str("# HELP mikrotik_interface_rx_bytes_total Received bytes on interface\n# TYPE mikrotik_interface_rx_bytes_total counter\n");
        for iface in &interfaces {
            let name = iface.get("name").unwrap_or("unknown");
            let rx_bytes: u64 = iface.get("rx-byte").and_then(|v| v.parse().ok()).unwrap_or(0);
            out.push_str(&format!("mikrotik_interface_rx_bytes_total{{host=\"{}\",interface=\"{}\"}} {}\n", target_label, name, rx_bytes));
        }
        out.push('\n');

        out.push_str("# HELP mikrotik_interface_tx_bytes_total Transmitted bytes on interface\n# TYPE mikrotik_interface_tx_bytes_total counter\n");
        for iface in &interfaces {
            let name = iface.get("name").unwrap_or("unknown");
            let tx_bytes: u64 = iface.get("tx-byte").and_then(|v| v.parse().ok()).unwrap_or(0);
            out.push_str(&format!("mikrotik_interface_tx_bytes_total{{host=\"{}\",interface=\"{}\"}} {}\n", target_label, name, tx_bytes));
        }
        out.push('\n');

        out.push_str("# HELP mikrotik_interface_rx_packets_total Received packets on interface\n# TYPE mikrotik_interface_rx_packets_total counter\n");
        for iface in &interfaces {
            let name = iface.get("name").unwrap_or("unknown");
            let rx_pkts: u64 = iface.get("rx-packet").and_then(|v| v.parse().ok()).unwrap_or(0);
            out.push_str(&format!("mikrotik_interface_rx_packets_total{{host=\"{}\",interface=\"{}\"}} {}\n", target_label, name, rx_pkts));
        }
        out.push('\n');

        out.push_str("# HELP mikrotik_interface_tx_packets_total Transmitted packets on interface\n# TYPE mikrotik_interface_tx_packets_total counter\n");
        for iface in &interfaces {
            let name = iface.get("name").unwrap_or("unknown");
            let tx_pkts: u64 = iface.get("tx-packet").and_then(|v| v.parse().ok()).unwrap_or(0);
            out.push_str(&format!("mikrotik_interface_tx_packets_total{{host=\"{}\",interface=\"{}\"}} {}\n", target_label, name, tx_pkts));
        }
        out.push('\n');

        out.push_str("# HELP mikrotik_interface_up Operational link running status (1=up, 0=down)\n# TYPE mikrotik_interface_up gauge\n");
        for iface in &interfaces {
            let name = iface.get("name").unwrap_or("unknown");
            let running = iface.get("running").map(|v| v == "true").unwrap_or(false);
            let val = if running { 1 } else { 0 };
            out.push_str(&format!("mikrotik_interface_up{{host=\"{}\",interface=\"{}\"}} {}\n", target_label, name, val));
        }
        out.push('\n');
    }

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        out,
    )
}

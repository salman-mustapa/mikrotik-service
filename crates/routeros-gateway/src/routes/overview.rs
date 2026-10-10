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
pub struct OverviewReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

/// Fast-Path Aggregated Snapshot:
/// Dispatches 6 concurrent queries over the multiplexed TCP socket using `tokio::join!`.
/// Returns router identity, system resources, routerboard info, active hotspots,
/// active PPPoE sessions, and interface link states in a single sub-millisecond response.
pub async fn get_overview(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<OverviewReq>>,
) -> Result<Json<Value>, ApiError> {
    let t0 = Instant::now();
    let req_inner = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req_inner.router, req_inner.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Prepare 6 concurrent futures across the single persistent socket
    let res_fut = client.run(build_command("/system/resource/print", std::iter::empty::<(&str, &str)>()));
    let id_fut = client.run(build_command("/system/identity/print", std::iter::empty::<(&str, &str)>()));
    let rb_fut = client.run(build_command("/system/routerboard/print", std::iter::empty::<(&str, &str)>()));
    let hs_fut = client.run(build_command("/ip/hotspot/active/print", std::iter::empty::<(&str, &str)>()));
    let ppp_fut = client.run(build_command("/ppp/active/print", std::iter::empty::<(&str, &str)>()));
    let if_fut = client.run(build_command("/interface/print", std::iter::empty::<(&str, &str)>()));

    // Execute concurrently with .tag multiplexing
    let (res_res, id_res, rb_res, hs_res, ppp_res, if_res) =
        tokio::join!(res_fut, id_fut, rb_fut, hs_fut, ppp_fut, if_fut);

    let res_attrs = res_res
        .ok()
        .and_then(|rows| rows.into_iter().next())
        .map(|r| r.attrs)
        .unwrap_or_default();

    let identity_name = id_res
        .ok()
        .and_then(|rows| rows.into_iter().next())
        .and_then(|r| r.get("name").map(str::to_owned))
        .unwrap_or_else(|| "MikroTik".into());

    let rb_attrs = rb_res
        .ok()
        .and_then(|rows| rows.into_iter().next())
        .map(|r| r.attrs)
        .unwrap_or_default();

    let hs_rows = hs_res.ok().unwrap_or_default();
    let hs_count = hs_rows.len();
    let hs_users: Vec<_> = hs_rows.into_iter().take(10).map(|r| r.attrs).collect();

    let ppp_rows = ppp_res.ok().unwrap_or_default();
    let ppp_count = ppp_rows.len();
    let ppp_sessions: Vec<_> = ppp_rows.into_iter().take(10).map(|r| r.attrs).collect();

    let if_rows = if_res.ok().unwrap_or_default();
    let if_total = if_rows.len();
    let mut if_running_count = 0;
    let mut interfaces_summary = Vec::new();

    for row in if_rows {
        let is_running = row.get("running").map(|v| v == "true").unwrap_or(false);
        if is_running {
            if_running_count += 1;
        }
        interfaces_summary.push(json!({
            "name": row.get("name").unwrap_or(""),
            "type": row.get("type").unwrap_or(""),
            "running": is_running,
            "disabled": row.get("disabled").map(|v| v == "true").unwrap_or(false),
            "rx_byte": row.get("rx-byte").unwrap_or("0"),
            "tx_byte": row.get("tx-byte").unwrap_or("0"),
        }));
    }

    // Health evaluation
    let cpu_load: u32 = res_attrs
        .get("cpu-load")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let free_mem: u64 = res_attrs
        .get("free-memory")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let total_mem: u64 = res_attrs
        .get("total-memory")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);

    let memory_used_percent = ((total_mem.saturating_sub(free_mem)) as f64 / total_mem as f64) * 100.0;

    let cpu_health = if cpu_load < 60 {
        "healthy"
    } else if cpu_load < 85 {
        "warning"
    } else {
        "critical"
    };

    let elapsed = t0.elapsed();

    Ok(Json(json!({
        "success": true,
        "execution_time_ms": elapsed.as_millis() as u64,
        "data": {
            "identity": identity_name,
            "system": {
                "cpu_load_percent": cpu_load,
                "free_memory_mb": free_mem / (1024 * 1024),
                "total_memory_mb": total_mem / (1024 * 1024),
                "memory_used_percent": format!("{:.1}%", memory_used_percent),
                "uptime": res_attrs.get("uptime").unwrap_or(&"".into()),
                "version": res_attrs.get("version").unwrap_or(&"".into()),
                "board_name": res_attrs.get("board-name").unwrap_or(&"".into()),
                "architecture": res_attrs.get("architecture-name").unwrap_or(&"".into()),
                "cpu_count": res_attrs.get("cpu-count").unwrap_or(&"1".into()),
                "cpu_frequency_mhz": res_attrs.get("cpu-frequency").unwrap_or(&"0".into()),
                "attributes": res_attrs,
            },
            "routerboard": {
                "model": rb_attrs.get("model").unwrap_or(&"".into()),
                "serial_number": rb_attrs.get("serial-number").unwrap_or(&"".into()),
                "current_firmware": rb_attrs.get("current-firmware").unwrap_or(&"".into()),
                "upgrade_firmware": rb_attrs.get("upgrade-firmware").unwrap_or(&"".into()),
            },
            "hotspot": {
                "active_total": hs_count,
                "active_recent": hs_users,
            },
            "ppp": {
                "active_total": ppp_count,
                "active_recent": ppp_sessions,
            },
            "interfaces": {
                "total": if_total,
                "running": if_running_count,
                "list": interfaces_summary,
            },
            "health_status": {
                "cpu": cpu_health,
                "memory_status": if memory_used_percent > 90.0 { "warning" } else { "healthy" },
            }
        }
    })))
}

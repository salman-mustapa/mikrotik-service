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

#[derive(Deserialize, Debug)]
pub struct PingReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    pub count: Option<u32>,
    pub size: Option<u32>,
    pub interface: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TracerouteReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    pub count: Option<u32>,
}

#[derive(Deserialize, Debug, Default)]
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BandwidthTestReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    pub user: Option<String>,
    pub password: Option<String>,
    pub direction: Option<String>, // "receive", "transmit", "both"
}

/// POST /api/v1/tools/ping - Ping diagnostik dari router
pub async fn ping(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<PingReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("address", req.address.as_str())];
    let count_str = req.count.unwrap_or(4).to_string();
    args.push(("count", count_str.as_str()));
    let size_str;
    if let Some(s) = req.size {
        size_str = s.to_string();
        args.push(("size", size_str.as_str()));
    }
    if let Some(i) = req.interface.as_deref() {
        args.push(("interface", i));
    }
    let rows = client.run(build_command("/ping", args)).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/tools/traceroute - Traceroute diagnostik dari router
pub async fn traceroute(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<TracerouteReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let count_str = req.count.unwrap_or(1).to_string();
    let args = [("address", req.address.as_str()), ("count", count_str.as_str())];
    let rows = client.run(build_command("/tool/traceroute", args)).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/tools/profile - CPU Profiler (melihat proses apa yang memakai CPU)
pub async fn profile(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/tool/profile", [("once", "")])).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/tools/netwatch - Daftar pemantauan host Netwatch
pub async fn netwatch(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/tool/netwatch/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/tools/bandwidth-test - Tes bandwidth MikroTik ke MikroTik
pub async fn bandwidth_test(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BandwidthTestReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("address", req.address.as_str()), ("duration", "5s")];
    if let Some(u) = req.user.as_deref() {
        args.push(("user", u));
    }
    if let Some(p) = req.password.as_deref() {
        args.push(("password", p));
    }
    if let Some(d) = req.direction.as_deref() {
        args.push(("direction", d));
    }
    let rows = client.run(build_command("/tool/bandwidth-test", args)).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

#[derive(Deserialize, Debug, Default)]
pub struct MultiPingReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub targets: Option<Vec<String>>,
    pub count: Option<u32>,
    pub size: Option<u32>,
}

/// POST or GET /api/v1/tools/multi-ping - Ping matriks multi-target sekaligus (Gateway, Cloudflare 1.1.1.1, Google 8.8.8.8, OpenDNS)
pub async fn multi_ping(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<MultiPingReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut target_list = req.targets.unwrap_or_default();
    if target_list.is_empty() {
        if let Ok(routes) = client.run(build_command("/ip/route/print", [("?dst-address", "0.0.0.0/0")])).await {
            if let Some(gw) = routes.first().and_then(|r| r.get("gateway")) {
                let clean_gw = gw.split('%').next().unwrap_or("").trim();
                if !clean_gw.is_empty() {
                    target_list.push(clean_gw.to_string());
                }
            }
        }
        target_list.push("1.1.1.1".to_string());
        target_list.push("8.8.8.8".to_string());
        target_list.push("208.67.222.222".to_string());
    }

    let count = req.count.unwrap_or(3);
    let count_str = count.to_string();
    let size_str = req.size.map(|s| s.to_string());

    let mut results = Vec::new();
    let mut total_latency = 0.0f64;
    let mut latency_count = 0usize;
    let mut online_count = 0usize;

    for host in &target_list {
        let mut args = vec![("address", host.as_str()), ("count", count_str.as_str())];
        if let Some(s) = size_str.as_deref() {
            args.push(("size", s));
        }

        let ping_res = client.run(build_command("/ping", args)).await;
        match ping_res {
            Ok(rows) => {
                let mut received = 0;
                let mut sent = 0;
                let mut sum_rtt_ms = 0.0f64;
                let mut rtt_samples = 0;
                let mut min_ms: Option<f64> = None;
                let mut max_ms: Option<f64> = None;

                for r in &rows {
                    if let Some(s) = r.get("sent").and_then(|v| v.parse::<u32>().ok()) {
                        sent = sent.max(s);
                    }
                    if let Some(rcv) = r.get("received").and_then(|v| v.parse::<u32>().ok()) {
                        received = received.max(rcv);
                    }
                    if let Some(t_str) = r.get("time").or_else(|| r.get("rtt")).or_else(|| r.get("avg-rtt")) {
                        let ms = parse_rtt_ms(t_str);
                        if ms > 0.0 {
                            sum_rtt_ms += ms;
                            rtt_samples += 1;
                            min_ms = Some(min_ms.map_or(ms, |m| m.min(ms)));
                            max_ms = Some(max_ms.map_or(ms, |m| m.max(ms)));
                        }
                    }
                }

                if sent == 0 {
                    sent = count;
                }
                if received == 0 && rtt_samples > 0 {
                    received = rtt_samples as u32;
                }

                let loss_pct = if sent > 0 {
                    ((sent.saturating_sub(received)) as f64 / sent as f64) * 100.0
                } else {
                    100.0
                };

                let avg_ms = if rtt_samples > 0 {
                    sum_rtt_ms / (rtt_samples as f64)
                } else {
                    0.0
                };

                let status = if received == 0 || loss_pct >= 100.0 {
                    "DOWN"
                } else if loss_pct > 20.0 || avg_ms > 150.0 {
                    "DEGRADED"
                } else if avg_ms <= 40.0 && loss_pct == 0.0 {
                    "EXCELLENT"
                } else {
                    "HEALTHY"
                };

                if status != "DOWN" {
                    online_count += 1;
                    if avg_ms > 0.0 {
                        total_latency += avg_ms;
                        latency_count += 1;
                    }
                }

                results.push(json!({
                    "target": host,
                    "sent": sent,
                    "received": received,
                    "packet_loss_pct": (loss_pct * 10.0).round() / 10.0,
                    "min_rtt_ms": min_ms.map(|v| (v * 10.0).round() / 10.0),
                    "avg_rtt_ms": if avg_ms > 0.0 { Some((avg_ms * 10.0).round() / 10.0) } else { None },
                    "max_rtt_ms": max_ms.map(|v| (v * 10.0).round() / 10.0),
                    "status": status
                }));
            }
            Err(e) => {
                results.push(json!({
                    "target": host,
                    "sent": count,
                    "received": 0,
                    "packet_loss_pct": 100.0,
                    "status": "ERROR",
                    "error": e.to_string()
                }));
            }
        }
    }

    let global_avg_latency = if latency_count > 0 {
        Some(((total_latency / latency_count as f64) * 10.0).round() / 10.0)
    } else {
        None
    };

    let overall_sla = if online_count == target_list.len() {
        "ALL_ONLINE_OPTIMAL"
    } else if online_count > 0 {
        "PARTIAL_OUTAGE_DEGRADED"
    } else {
        "CRITICAL_LINK_DOWN"
    };

    Ok(Json(json!({
        "success": true,
        "sla_status": overall_sla,
        "total_targets": target_list.len(),
        "online_targets": online_count,
        "avg_latency_ms": global_avg_latency,
        "matrix": results
    })))
}

fn parse_rtt_ms(s: &str) -> f64 {
    let clean = s.trim();
    if clean.ends_with("ms") {
        clean.trim_end_matches("ms").trim().parse::<f64>().unwrap_or(0.0)
    } else if clean.ends_with("s") {
        clean.trim_end_matches('s').trim().parse::<f64>().map(|v| v * 1000.0).unwrap_or(0.0)
    } else if clean.ends_with("us") {
        clean.trim_end_matches("us").trim().parse::<f64>().map(|v| v / 1000.0).unwrap_or(0.0)
    } else {
        clean.parse::<f64>().unwrap_or(0.0)
    }
}


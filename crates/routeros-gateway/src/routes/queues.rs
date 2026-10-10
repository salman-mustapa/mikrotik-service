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
pub struct FilterReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    #[serde(default)]
    pub filter: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct AddQueueReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub target: String,
    pub max_limit: String, // e.g. "2M/10M"
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SetLimitReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
    pub max_limit: String,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

pub async fn simple(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/queue/simple/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn add_simple(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddQueueReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("name", req.name.as_str()),
        ("target", req.target.as_str()),
        ("max-limit", req.max_limit.as_str()),
    ];
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/queue/simple/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Simple queue added" })))
}

pub async fn set_limit(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetLimitReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/queue/simple/set", [
        (".id", req.id.as_str()),
        ("max-limit", req.max_limit.as_str()),
    ])).await?;
    Ok(Json(json!({ "success": true, "message": "Queue limit updated" })))
}

pub async fn remove_simple(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/queue/simple/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Queue removed" })))
}

fn parse_bps(s: &str) -> u64 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }
    let (num_part, multiplier) = if s.ends_with('k') || s.ends_with('K') {
        (&s[..s.len() - 1], 1_000u64)
    } else if s.ends_with('m') || s.ends_with('M') {
        (&s[..s.len() - 1], 1_000_000u64)
    } else if s.ends_with('g') || s.ends_with('G') {
        (&s[..s.len() - 1], 1_000_000_000u64)
    } else {
        (s, 1u64)
    };
    num_part.parse::<f64>().map(|v| (v * multiplier as f64) as u64).unwrap_or(0)
}

fn parse_pair(s: &str) -> (u64, u64) {
    if let Some((up, down)) = s.split_once('/') {
        (parse_bps(up), parse_bps(down))
    } else {
        (0, 0)
    }
}

fn format_bps(bps: u64) -> String {
    if bps >= 1_000_000_000 {
        format!("{:.2} Gbps", bps as f64 / 1_000_000_000.0)
    } else if bps >= 1_000_000 {
        format!("{:.2} Mbps", bps as f64 / 1_000_000.0)
    } else if bps >= 1_000 {
        format!("{:.1} Kbps", bps as f64 / 1_000.0)
    } else {
        format!("{} bps", bps)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

#[derive(Deserialize, Debug)]
pub struct InspectUserReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub query: String, // IP target (e.g. 192.168.88.50), nama antrean, atau nama user
}

/// POST /api/v1/queues/inspect-user
/// Mendeteksi limit kecepatan yang dikonfigurasi, konsumsi real-time, kuota transfer, dan status overload/throttled.
pub async fn inspect_user(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<InspectUserReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let q_lower = req.query.trim().to_lowercase();

    // 1. Ambil data Simple Queues
    let queue_rows = client.run(build_command("/queue/simple/print", std::iter::empty::<(&str, &str)>())).await?;

    let matched_q = queue_rows.into_iter().find(|r| {
        let name = r.get("name").unwrap_or_default().to_lowercase();
        let tgt = r.get("target").unwrap_or_default();
        let comment = r.get("comment").unwrap_or_default().to_lowercase();
        name.contains(&q_lower) || tgt.contains(&req.query) || comment.contains(&q_lower)
    });

    // 2. Ambil data sesi Hotspot & PPPoE jika relevan
    let hotspot_rows = client.run(build_command("/ip/hotspot/active/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let matched_hs = hotspot_rows.into_iter().find(|r| {
        let u = r.get("user").unwrap_or_default().to_lowercase();
        let addr = r.get("address").unwrap_or_default();
        let mac = r.get("mac-address").unwrap_or_default().to_lowercase();
        u.contains(&q_lower) || addr == req.query || mac.contains(&q_lower)
    });

    let ppp_rows = client.run(build_command("/ppp/active/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let matched_ppp = ppp_rows.into_iter().find(|r| {
        let u = r.get("name").unwrap_or_default().to_lowercase();
        let addr = r.get("address").unwrap_or_default();
        let caller = r.get("caller-id").unwrap_or_default().to_lowercase();
        u.contains(&q_lower) || addr == req.query || caller.contains(&q_lower)
    });

    if matched_q.is_none() && matched_hs.is_none() && matched_ppp.is_none() {
        return Ok(Json(json!({
            "success": true,
            "found": false,
            "query": req.query,
            "message": format!("Tidak ditemukan antrean atau sesi aktif untuk query '{}'", req.query)
        })));
    }

    // Ekstraksi data antrean
    let mut configured_limit = json!({ "upload_bps": 0, "download_bps": 0, "upload_formatted": "Unlimited", "download_formatted": "Unlimited", "raw": "none" });
    let mut current_speed = json!({ "upload_bps": 0, "download_bps": 0, "upload_formatted": "0 bps", "download_formatted": "0 bps", "raw": "0/0" });
    let mut total_transferred = json!({ "upload_bytes": 0, "download_bytes": 0, "upload_formatted": "0 B", "download_formatted": "0 B" });
    let mut utilization_up = 0.0;
    let mut utilization_down = 0.0;
    let mut is_throttled = false;
    let mut dropped_packets = json!({ "upload": 0, "download": 0, "raw": "0/0" });
    let mut queue_id = None;
    let mut queue_name = None;
    let mut queue_target = None;

    if let Some(q) = matched_q {
        queue_id = q.get(".id").map(|s| s.to_string());
        queue_name = q.get("name").map(|s| s.to_string());
        queue_target = q.get("target").map(|s| s.to_string());

        let max_lim_raw = q.get("max-limit").unwrap_or("0/0");
        let (max_up, max_down) = parse_pair(max_lim_raw);
        configured_limit = json!({
            "upload_bps": max_up,
            "download_bps": max_down,
            "upload_formatted": if max_up > 0 { format_bps(max_up) } else { "Unlimited".into() },
            "download_formatted": if max_down > 0 { format_bps(max_down) } else { "Unlimited".into() },
            "raw": max_lim_raw
        });

        let rate_raw = q.get("rate").unwrap_or("0/0");
        let (cur_up, cur_down) = parse_pair(rate_raw);
        current_speed = json!({
            "upload_bps": cur_up,
            "download_bps": cur_down,
            "upload_formatted": format_bps(cur_up),
            "download_formatted": format_bps(cur_down),
            "raw": rate_raw
        });

        let bytes_raw = q.get("bytes").unwrap_or("0/0");
        let (b_up, b_down) = parse_pair(bytes_raw);
        total_transferred = json!({
            "upload_bytes": b_up,
            "download_bytes": b_down,
            "upload_formatted": format_bytes(b_up),
            "download_formatted": format_bytes(b_down)
        });

        let drop_raw = q.get("dropped").unwrap_or("0/0");
        let (dr_up, dr_down) = parse_pair(drop_raw);
        dropped_packets = json!({ "upload": dr_up, "download": dr_down, "raw": drop_raw });

        if max_up > 0 {
            utilization_up = ((cur_up as f64 / max_up as f64) * 100.0).min(100.0);
        }
        if max_down > 0 {
            utilization_down = ((cur_down as f64 / max_down as f64) * 100.0).min(100.0);
        }

        if utilization_down >= 90.0 || utilization_up >= 90.0 || dr_down > 0 {
            is_throttled = true;
        }
    }

    let status = if is_throttled {
        "THROTTLED (Mencapai Batas Kecepatan / Merah di Winbox)"
    } else if utilization_down > 5.0 || utilization_up > 5.0 {
        "ACTIVE (Sedang Menggunakan Internet)"
    } else {
        "IDLE (Tidak Ada Trafik Signifikan)"
    };

    let session_type = if matched_hs.is_some() {
        "Hotspot"
    } else if matched_ppp.is_some() {
        "PPPoE"
    } else {
        "Static / DHCP Client"
    };

    Ok(Json(json!({
        "success": true,
        "found": true,
        "query": req.query,
        "session_type": session_type,
        "queue_info": {
            "id": queue_id,
            "name": queue_name,
            "target": queue_target
        },
        "bandwidth_limit": configured_limit,
        "current_speed": current_speed,
        "total_quota_consumed": total_transferred,
        "utilization": {
            "upload_percent": (utilization_up * 10.0).round() / 10.0,
            "download_percent": (utilization_down * 10.0).round() / 10.0,
            "is_throttled": is_throttled,
            "status": status
        },
        "dropped_packets": dropped_packets,
        "session_details": {
            "hotspot": matched_hs.map(|h| h.attrs),
            "pppoe": matched_ppp.map(|p| p.attrs)
        }
    })))
}

#[derive(Deserialize, Debug, Default)]
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

/// POST /api/v1/queues/overview-summary
/// Rekapitulasi eksekutif NOC untuk semua antrean Simple Queues: total bandwidth dialokasikan,
/// total pemakaian saat ini, top downloader, dan antrean yang sedang bottleneck.
pub async fn overview_summary(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/queue/simple/print", std::iter::empty::<(&str, &str)>())).await?;

    let mut total_queues = 0usize;
    let mut total_alloc_up = 0u64;
    let mut total_alloc_down = 0u64;
    let mut total_cur_up = 0u64;
    let mut total_cur_down = 0u64;
    let mut total_bytes_up = 0u64;
    let mut total_bytes_down = 0u64;

    struct QueueItem {
        name: String,
        target: String,
        cur_down: u64,
        cur_up: u64,
        bytes_down: u64,
        max_down: u64,
        max_up: u64,
        is_throttled: bool,
    }

    let mut items = Vec::new();

    for r in rows {
        total_queues += 1;
        let name = r.get("name").unwrap_or("unknown").to_string();
        let tgt = r.get("target").unwrap_or("-").to_string();

        let (max_up, max_down) = parse_pair(r.get("max-limit").unwrap_or("0/0"));
        let (c_up, c_down) = parse_pair(r.get("rate").unwrap_or("0/0"));
        let (b_up, b_down) = parse_pair(r.get("bytes").unwrap_or("0/0"));
        let (dr_up, dr_down) = parse_pair(r.get("dropped").unwrap_or("0/0"));

        total_alloc_up += max_up;
        total_alloc_down += max_down;
        total_cur_up += c_up;
        total_cur_down += c_down;
        total_bytes_up += b_up;
        total_bytes_down += b_down;

        let down_pct = if max_down > 0 { (c_down as f64 / max_down as f64) * 100.0 } else { 0.0 };
        let up_pct = if max_up > 0 { (c_up as f64 / max_up as f64) * 100.0 } else { 0.0 };
        let throttled = down_pct >= 90.0 || up_pct >= 90.0 || dr_down > 0 || dr_up > 0;

        items.push(QueueItem {
            name,
            target: tgt,
            cur_down: c_down,
            cur_up: c_up,
            bytes_down: b_down,
            max_down,
            max_up,
            is_throttled: throttled,
        });
    }

    // Sort top 5 downloaders berdasarkan pemakaian real-time saat ini
    items.sort_by(|a, b| b.cur_down.cmp(&a.cur_down));
    let top_active_users: Vec<Value> = items.iter().take(5).map(|it| {
        json!({
            "name": it.name,
            "target": it.target,
            "current_download": format_bps(it.cur_down),
            "current_upload": format_bps(it.cur_up),
            "max_limit_download": if it.max_down > 0 { format_bps(it.max_down) } else { "Unlimited".into() },
            "max_limit_upload": if it.max_up > 0 { format_bps(it.max_up) } else { "Unlimited".into() },
            "total_bytes": format_bytes(it.bytes_down),
            "is_throttled": it.is_throttled
        })
    }).collect();

    let throttled_count = items.iter().filter(|it| it.is_throttled).count();

    Ok(Json(json!({
        "success": true,
        "total_active_queues": total_queues,
        "bandwidth_allocated": {
            "download_bps": total_alloc_down,
            "upload_bps": total_alloc_up,
            "download_formatted": format_bps(total_alloc_down),
            "upload_formatted": format_bps(total_alloc_up)
        },
        "realtime_consumption": {
            "download_bps": total_cur_down,
            "upload_bps": total_cur_up,
            "download_formatted": format_bps(total_cur_down),
            "upload_formatted": format_bps(total_cur_up)
        },
        "total_traffic_cumulative": {
            "download_formatted": format_bytes(total_bytes_down),
            "upload_formatted": format_bytes(total_bytes_up)
        },
        "congestion_indicator": {
            "throttled_queues_count": throttled_count,
            "health_status": if throttled_count > 5 { "WARNING: Beberapa antrean mengalami bottleneck" } else { "OPTIMAL: Trafik antrean berjalan lancar" }
        },
        "top_active_users": top_active_users
    })))
}

#[derive(Deserialize, Debug)]
pub struct TestQueueLimitReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub target_ip: String,
}

/// POST /api/v1/queues/test-limit
/// Memvalidasi efektivitas limit antrean pada IP target dengan membandingkan konfigurasi max-limit terhadap live rate.
pub async fn test_limit(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<TestQueueLimitReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/queue/simple/print", std::iter::empty::<(&str, &str)>())).await?;
    let found = rows.into_iter().find(|r| {
        let tgt = r.get("target").unwrap_or_default();
        tgt.contains(&req.target_ip)
    });

    if let Some(q) = found {
        let max_lim_raw = q.get("max-limit").unwrap_or("0/0");
        let (max_up, max_down) = parse_pair(max_lim_raw);
        let rate_raw = q.get("rate").unwrap_or("0/0");
        let (cur_up, cur_down) = parse_pair(rate_raw);

        let limit_enforced = cur_down <= (max_down + (max_down / 10)); // toleransi 10%

        Ok(Json(json!({
            "success": true,
            "target_ip": req.target_ip,
            "queue_name": q.get("name").unwrap_or("-"),
            "max_limit": {
                "download": format_bps(max_down),
                "upload": format_bps(max_up)
            },
            "live_rate": {
                "download": format_bps(cur_down),
                "upload": format_bps(cur_up)
            },
            "limit_enforced": limit_enforced,
            "verdict": if max_down == 0 {
                "Antrean tidak memiliki batasan (Unlimited)"
            } else if cur_down > max_down {
                "Trafik saat ini melampaui limit (burst aktif atau packet drop sedang berlangsung)"
            } else {
                "Batasan bandwidth aktif dan membatasi trafik secara efektif"
            }
        })))
    } else {
        Ok(Json(json!({
            "success": false,
            "message": format!("Tidak ditemukan Simple Queue untuk target IP '{}'. Pastikan IP terdaftar di antrean.", req.target_ip)
        })))
    }
}


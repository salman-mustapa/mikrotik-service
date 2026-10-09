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

/// POST /api/v1/system/cpu-profiler - In-depth Profiling of CPU & RAM hogs,
/// identifying which processes (firewall, networking, queues, dns) are choking the CPU
pub async fn cpu_profiler(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // 1. Run profile for 2 seconds
    let profile_rows = client.run(build_command(
        "/tool/profile",
        [
            ("duration", "2s"),
            ("cpu", "all"),
        ]
    )).await?;

    // 2. Fetch filters to check FastTrack, Queues count, and DNS
    let filter_rows = client.run(build_command("/ip/firewall/filter/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let queue_rows = client.run(build_command("/queue/simple/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let res_rows = client.run(build_command("/system/resource/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();

    let mut process_usage: HashMap<String, f64> = HashMap::new();
    for row in profile_rows {
        let name = row.get("name").unwrap_or("idle").to_string();
        let usage_str = row.get("usage").unwrap_or("0").trim_end_matches('%');
        let usage: f64 = usage_str.parse().unwrap_or(0.0);
        *process_usage.entry(name).or_insert(0.0) += usage;
    }

    // Sort processes by highest usage
    let mut sorted_procs: Vec<_> = process_usage.into_iter().collect();
    sorted_procs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Check FastTrack status
    let has_fasttrack = filter_rows.iter().any(|r| {
        r.get("action").is_some_and(|a| a == "fasttrack-connection")
    });

    let simple_queue_count = queue_rows.len();

    // Resource info
    let first_res = res_rows.first();
    let cpu_load = first_res.and_then(|r| r.get("cpu-load")).unwrap_or("0");
    let free_ram_bytes: u64 = first_res.and_then(|r| r.get("free-memory")).and_then(|v| v.parse().ok()).unwrap_or(0);
    let total_ram_bytes: u64 = first_res.and_then(|r| r.get("total-memory")).and_then(|v| v.parse().ok()).unwrap_or(1);
    let free_ram_percent = (free_ram_bytes * 100) / total_ram_bytes;

    let mut findings: Vec<String> = Vec::new();
    let mut recommendations: Vec<String> = Vec::new();

    // Analyze Bottlenecks
    let net_firewall_usage = sorted_procs.iter()
        .filter(|(name, _)| name == "networking" || name == "firewall")
        .map(|(_, u)| u)
        .sum::<f64>();

    if net_firewall_usage > 40.0 && !has_fasttrack {
        findings.push("Proses 'networking/firewall' memakan CPU tinggi dan FastTrack TIDAK AKTIF.".into());
        recommendations.push("Pasang FastTrack (/api/v1/system/fasttrack/deploy) untuk membypass paket yang sudah established/related, menurunkan beban CPU dari 90% menjadi ~15% pada router kecil (hAP lite / RB750).".into());
    }

    if simple_queue_count > 150 {
        findings.push(format!("Terdeteksi {} aturan Simple Queue. Jumlah antrean sederhana > 100 dapat memicu lonjakan CPU core tunggal.", simple_queue_count));
        recommendations.push("Pertimbangkan migrasi ke Queue Tree bertingkat (/api/v1/traffic/queue-tree) untuk efisiensi multithread.".into());
    }

    if free_ram_percent < 15 {
        findings.push(format!("Sisa RAM sangat tipis: {}% ({} MB tersisa).", free_ram_percent, free_ram_bytes / (1024 * 1024)));
        recommendations.push("Bersihkan DNS Cache (/api/v1/dns/cache/flush) dan kurangi ukuran log buffer (/system/logging).".into());
    }

    Ok(Json(json!({
        "success": true,
        "overall_cpu_load": format!("{}%", cpu_load),
        "free_ram": format!("{}% ({} MB)", free_ram_percent, free_ram_bytes / (1024 * 1024)),
        "fasttrack_enabled": has_fasttrack,
        "simple_queues_count": simple_queue_count,
        "cpu_breakdown": sorted_procs.into_iter().map(|(k, v)| json!({ "process": k, "usage_percent": v })).collect::<Vec<_>>(),
        "bottleneck_findings": findings,
        "performance_recommendations": recommendations
    })))
}

/// POST /api/v1/system/fasttrack/deploy - Automatically injects FastTrack connection acceleration
/// rule into firewall filter, slashing CPU usage on budget routers by up to 80%!
pub async fn deploy_fasttrack(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let comment_ft = "[FastTrack] CPU Hardware Accelerator (established, related)";
    let comment_accept = "[FastTrack] Accept established, related";

    // 1. FastTrack Connection
    client.run(build_command(
        "/ip/firewall/filter/add",
        [
            ("chain", "forward"),
            ("action", "fasttrack-connection"),
            ("connection-state", "established,related"),
            ("comment", comment_ft),
        ]
    )).await?;

    // 2. Accept established/related
    client.run(build_command(
        "/ip/firewall/filter/add",
        [
            ("chain", "forward"),
            ("action", "accept"),
            ("connection-state", "established,related"),
            ("comment", comment_accept),
        ]
    )).await?;

    Ok(Json(json!({
        "success": true,
        "message": "FastTrack Connection berhasil diaktifkan. Beban CPU router pada throughput tinggi akan turun drastis!"
    })))
}

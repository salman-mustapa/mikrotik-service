use std::sync::Arc;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use routeros_core::build_command;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::{AppState, RouterTarget};

#[derive(Deserialize, Debug, Default)]
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct DiagnosticIssue {
    pub category: String, // "CPU", "Memory", "Internet Link", "Bandwidth", "FastTrack", "AP/LAN"
    pub severity: String, // "INFO", "WARNING", "CRITICAL"
    pub title: String,
    pub description_id: String,
    pub description_en: String,
    pub recommendation: String,
}

/// GET & POST /api/v1/doctor/diagnose - Automated Heuristic Network Diagnostic Assistant
/// Audits 6 critical networking pillars: CPU, RAM, WAN Ping, Throttled Queues, FastTrack, and Access Points.
pub async fn doctor_diagnose(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req_inner = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req_inner.router, req_inner.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Concurrent diagnostics
    let res_fut = client.run(build_command("/system/resource/print", std::iter::empty::<(&str, &str)>()));
    let ping_fut = client.run(build_command("/ping", [("address", "1.1.1.1"), ("count", "3")]));
    let queue_fut = client.run(build_command("/queue/simple/print", std::iter::empty::<(&str, &str)>()));
    let if_fut = client.run(build_command("/interface/print", std::iter::empty::<(&str, &str)>()));
    let fw_fut = client.run(build_command("/ip/firewall/filter/print", std::iter::empty::<(&str, &str)>()));

    let (res_res, ping_res, queue_res, if_res, fw_res) = tokio::join!(res_fut, ping_fut, queue_fut, if_fut, fw_fut);

    let mut issues = Vec::new();
    let mut score = 100;

    // 1. CPU & Memory Check
    let mut cpu_val = 0;
    let mut free_ram_mb = 0;
    if let Ok(rows) = res_res {
        if let Some(r) = rows.first() {
            let cpu: u32 = r.get("cpu-load").and_then(|v| v.parse().ok()).unwrap_or(0);
            cpu_val = cpu;
            let free_mem: u64 = r.get("free-memory").and_then(|v| v.parse().ok()).unwrap_or(0);
            let total_mem: u64 = r.get("total-memory").and_then(|v| v.parse().ok()).unwrap_or(1);
            free_ram_mb = free_mem / (1024 * 1024);

            let mem_usage_pct = 100 - ((free_mem * 100) / total_mem);

            if cpu > 85 {
                score -= 30;
                issues.push(DiagnosticIssue {
                    category: "CPU".into(),
                    severity: "CRITICAL".into(),
                    title: format!("CPU Load Kritis: {}%", cpu),
                    description_id: "Beban processor router di atas 85%. Berpotensi menyebabkan internet freeze, voucher lambat terbit, dan packet loss.".into(),
                    description_en: "Router CPU load is exceeding 85%, risking connection drops and slow response.".into(),
                    recommendation: "Gunakan POST /api/v1/system/fasttrack/deploy untuk membypass connection tracking atau kurangi aturan mangle berat.".into(),
                });
            } else if cpu > 60 {
                score -= 10;
                issues.push(DiagnosticIssue {
                    category: "CPU".into(),
                    severity: "WARNING".into(),
                    title: format!("CPU Load Tinggi: {}%", cpu),
                    description_id: "Beban CPU cukup tinggi. Pantau trafik dan pastikan FastTrack aktif.".into(),
                    description_en: "CPU load is moderately high. Monitor queue trees and connection tracking.".into(),
                    recommendation: "Periksa proses berat menggunakan POST /api/v1/system/cpu-profiler.".into(),
                });
            }

            if mem_usage_pct > 90 {
                score -= 20;
                issues.push(DiagnosticIssue {
                    category: "Memory".into(),
                    severity: "CRITICAL".into(),
                    title: format!("RAM Hampir Habis: {}% terpakai", mem_usage_pct),
                    description_id: format!("Sisa RAM hanya {} MB. Router rentan crash jika subscriber bertambah.", free_ram_mb),
                    description_en: "Free RAM is critically low, risk of out-of-memory crash under subscriber spikes.".into(),
                    recommendation: "Bersihkan DNS cache via POST /api/dns/cache/flush dan kurangi log buffer.".into(),
                });
            }
        }
    }

    // 2. WAN Link & Internet Ping Check
    let mut avg_rtt = "0ms".to_string();
    let mut packet_loss_pct = 0;
    if let Ok(rows) = ping_res {
        let total = rows.len();
        let mut received = 0;
        let mut total_time_ms = 0;

        for r in &rows {
            if let Some(t_str) = r.get("time") {
                received += 1;
                // Parse "15ms" or "12ms250us"
                let num: u32 = t_str.trim_end_matches("ms").split("ms").next().and_then(|s| s.parse().ok()).unwrap_or(15);
                total_time_ms += num;
            }
        }

        if total > 0 {
            packet_loss_pct = 100 - ((received * 100) / total);
            if received > 0 {
                avg_rtt = format!("{}ms", total_time_ms / (received as u32));
            }
        }

        if packet_loss_pct > 20 {
            score -= 35;
            issues.push(DiagnosticIssue {
                category: "Internet Link".into(),
                severity: "CRITICAL".into(),
                title: format!("Uplink ISP Packet Loss Tinggi: {}%", packet_loss_pct),
                description_id: "Ping ke public DNS 1.1.1.1 mengalami packet loss. Kemungkinan ada gangguan di kabel fiber ISP atau modem gateway.".into(),
                description_en: "High packet loss detected to public DNS, indicating upstream ISP fiber degradation.".into(),
                recommendation: "Cek kabel WAN modem ISP atau hubungi Call Center ISP untuk pemeriksaan redaman kabel optik.".into(),
            });
        }
    }

    // 3. Simple Queues Bottleneck Check
    let mut throttled_count = 0;
    if let Ok(q_rows) = queue_res {
        for q in q_rows {
            let rate = q.get("rate").unwrap_or("0/0");
            let max_limit = q.get("max-limit").unwrap_or("0/0");
            let drops: u64 = q.get("drops").and_then(|v| v.split('/').next()).and_then(|v| v.parse().ok()).unwrap_or(0);

            if drops > 100 && rate != "0/0" && max_limit != "0/0" {
                throttled_count += 1;
            }
        }

        if throttled_count > 0 {
            score -= 10;
            issues.push(DiagnosticIssue {
                category: "Bandwidth".into(),
                severity: "WARNING".into(),
                title: format!("{} Antrean Mengalami Throttling (Limit Penuh)", throttled_count),
                description_id: "Terdapat subscriber yang pemakaiannya mentok di batas kuota Simple Queue sehingga mengalami packet drop.".into(),
                description_en: "Subscribers are hitting queue rate-limits resulting in dropped packets.".into(),
                recommendation: "Gunakan POST /api/v1/queues/inspect-user untuk mengidentifikasi pengguna terberat atau naikkan paket pelanggan.".into(),
            });
        }
    }

    // 4. FastTrack Acceleration Audit
    let mut has_fasttrack = false;
    if let Ok(fw_rows) = fw_res {
        for r in fw_rows {
            let action = r.get("action").unwrap_or("");
            let disabled = r.get("disabled").map(|v| v == "true").unwrap_or(false);
            if action == "fasttrack-connection" && !disabled {
                has_fasttrack = true;
                break;
            }
        }
    }

    if !has_fasttrack {
        score -= 15;
        issues.push(DiagnosticIssue {
            category: "FastTrack".into(),
            severity: "INFO".into(),
            title: "FastTrack Hardware Acceleration Belum Aktif".into(),
            description_id: "Router belum mengaktifkan FastTrack. Setiap paket melewati conntrack penuh, membuat CPU lebih panas dan cepat penuh.".into(),
            description_en: "FastTrack bypass is disabled. Every packet undergoes full CPU connection tracking.".into(),
            recommendation: "Aktifkan dengan 1-klik via POST /api/v1/system/fasttrack/deploy untuk menghemat beban CPU hingga 80%.".into(),
        });
    }

    // 5. Interface Errors Audit
    if let Ok(interfaces) = if_res {
        for iface in interfaces {
            let name = iface.get("name").unwrap_or("");
            let rx_drop: u64 = iface.get("rx-drop").and_then(|v| v.parse().ok()).unwrap_or(0);
            let tx_drop: u64 = iface.get("tx-drop").and_then(|v| v.parse().ok()).unwrap_or(0);

            if rx_drop > 1000 || tx_drop > 1000 {
                issues.push(DiagnosticIssue {
                    category: "AP/LAN".into(),
                    severity: "WARNING".into(),
                    title: format!("Interface {} Mengalami Packet Drops ({})", name, rx_drop + tx_drop),
                    description_id: format!("Ditemukan drop packet masif pada port {}. Sering kali disebabkan kabel LAN crimping kendor atau switch loop.", name),
                    description_en: format!("Substantial packet drops detected on interface {}. Check patch cables and switch loops.", name),
                    recommendation: "Periksa kualitas kabel UTP RJ45 atau ganti port konektor LAN.".into(),
                });
                score -= 10;
                break;
            }
        }
    }

    let final_score = score.clamp(0, 100);
    let overall_status = if final_score >= 85 {
        "EXCELLENT"
    } else if final_score >= 70 {
        "GOOD"
    } else if final_score >= 50 {
        "WARNING"
    } else {
        "CRITICAL"
    };

    Ok(Json(json!({
        "success": true,
        "overall_status": overall_status,
        "health_score": final_score,
        "metrics_summary": {
            "cpu_load_percent": cpu_val,
            "free_ram_mb": free_ram_mb,
            "internet_latency": avg_rtt,
            "packet_loss_percent": packet_loss_pct,
            "fasttrack_enabled": has_fasttrack,
            "throttled_queues_count": throttled_count,
        },
        "total_issues_found": issues.len(),
        "issues": issues,
    })))
}

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
    pub category: String, // "CPU", "Memory", "Internet Link", "Bandwidth", "FastTrack", "AP/LAN", "DNS", "DHCP Pool", "Security"
    pub severity: String, // "INFO", "WARNING", "CRITICAL"
    pub title: String,
    pub description_id: String,
    pub description_en: String,
    pub recommendation: String,
}

/// GET & POST /api/v1/doctor/diagnose - Automated Heuristic Network Diagnostic Assistant
/// Audits 8 critical networking pillars: CPU, RAM, WAN Ping, Throttled Queues, FastTrack, AP/LAN Drops, DNS Health & Open Resolver, DHCP Pool Exhaustion, and Management Protocol Hardening.
pub async fn doctor_diagnose(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req_inner = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req_inner.router, req_inner.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Concurrent diagnostics (8 pillars dispatched simultaneously over multiplexed socket)
    let res_fut = client.run(build_command("/system/resource/print", std::iter::empty::<(&str, &str)>()));
    let ping_fut = client.run(build_command("/ping", [("address", "1.1.1.1"), ("count", "3")]));
    let queue_fut = client.run(build_command("/queue/simple/print", std::iter::empty::<(&str, &str)>()));
    let if_fut = client.run(build_command("/interface/print", std::iter::empty::<(&str, &str)>()));
    let fw_fut = client.run(build_command("/ip/firewall/filter/print", std::iter::empty::<(&str, &str)>()));
    let dns_fut = client.run(build_command("/ip/dns/print", std::iter::empty::<(&str, &str)>()));
    let pool_used_fut = client.run(build_command("/ip/pool/used/print", std::iter::empty::<(&str, &str)>()));
    let svc_fut = client.run(build_command("/ip/service/print", std::iter::empty::<(&str, &str)>()));

    let (res_res, ping_res, queue_res, if_res, fw_res, dns_res, pool_used_res, svc_res) =
        tokio::join!(res_fut, ping_fut, queue_fut, if_fut, fw_fut, dns_fut, pool_used_fut, svc_fut);

    let mut issues = Vec::new();
    let mut score = 100i32;

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
                    recommendation: "Bersihkan DNS cache via POST /api/v1/dns/cache/flush dan kurangi log buffer.".into(),
                });
            }
        }
    }

    // 2. WAN Link & Internet Ping Check
    let mut avg_rtt = "0ms".to_string();
    let mut packet_loss_pct = 0;
    if let Ok(rows) = ping_res {
        let total = rows.len();
        let mut received: usize = 0;
        let mut total_time_ms: u32 = 0;

        for r in &rows {
            if let Some(t_str) = r.get("time") {
                received += 1;
                let num: u32 = t_str.trim_end_matches("ms").split("ms").next().and_then(|s| s.parse().ok()).unwrap_or(15);
                total_time_ms += num;
            }
        }

        if let Some(pct) = (received * 100).checked_div(total) {
            packet_loss_pct = 100 - pct;
            if let Some(avg) = total_time_ms.checked_div(received as u32) {
                avg_rtt = format!("{}ms", avg);
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
    let mut has_dns_input_drop = false;
    if let Ok(ref fw_rows) = fw_res {
        for r in fw_rows {
            let action = r.get("action").unwrap_or("");
            let disabled = r.get("disabled").map(|v| v == "true").unwrap_or(false);
            if action == "fasttrack-connection" && !disabled {
                has_fasttrack = true;
            }
            if action == "drop" && !disabled {
                let chain = r.get("chain").unwrap_or("");
                let dst_port = r.get("dst-port").unwrap_or("");
                let port = r.get("port").unwrap_or("");
                if chain == "input" && (dst_port.contains("53") || port.contains("53")) {
                    has_dns_input_drop = true;
                }
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

    // 6. DNS Health & Open Resolver Exposure Audit
    let mut dns_configured = true;
    let mut open_dns_risk = false;
    let mut dns_servers = String::new();
    if let Ok(dns_rows) = dns_res {
        if let Some(d) = dns_rows.first() {
            dns_servers = d.get("servers").unwrap_or("").to_string();
            let allow_remote = d.get("allow-remote-requests").map(|v| v == "yes" || v == "true").unwrap_or(false);

            if dns_servers.trim().is_empty() {
                dns_configured = false;
                score -= 25;
                issues.push(DiagnosticIssue {
                    category: "DNS".into(),
                    severity: "CRITICAL".into(),
                    title: "Upstream DNS Belum Dikonfigurasi".into(),
                    description_id: "Router belum memiliki upstream DNS (1.1.1.1 / 8.8.8.8). Pelanggan tidak dapat membuka website menggunakan nama domain.".into(),
                    description_en: "No upstream DNS servers configured on router. Clients will experience domain name resolution failures.".into(),
                    recommendation: "Konfigurasikan DNS server via POST /api/v1/ip/dns atau periksa DHCP Client setting use-peer-dns=yes.".into(),
                });
            }

            if allow_remote && !has_dns_input_drop {
                open_dns_risk = true;
                score -= 15;
                issues.push(DiagnosticIssue {
                    category: "Security".into(),
                    severity: "WARNING".into(),
                    title: "Potensi Celah Open DNS Resolver Terbuka".into(),
                    description_id: "Fitur 'allow-remote-requests=yes' aktif tanpa aturan drop UDP port 53 pada firewall input WAN. Router rentan disalahgunakan untuk serangan DNS Amplification DDoS.".into(),
                    description_en: "Router has allow-remote-requests enabled without a WAN firewall filter rule blocking port 53 UDP, leaving it vulnerable to Open DNS Resolver abuse.".into(),
                    recommendation: "Tambahkan filter rule drop chain=input dst-port=53 protocol=udp in-interface-list=WAN via POST /api/v1/security/hardening/deploy-dns-guard.".into(),
                });
            }
        }
    }

    // 7. DHCP Pool Saturation & IP Exhaustion Audit
    let mut pool_used_count = 0usize;
    if let Ok(used_rows) = pool_used_res {
        pool_used_count = used_rows.len();
        if pool_used_count >= 220 {
            score -= 15;
            issues.push(DiagnosticIssue {
                category: "DHCP Pool".into(),
                severity: "WARNING".into(),
                title: format!("Alokasi IP Address Pool Sangat Tinggi ({} aktif)", pool_used_count),
                description_id: "Jumlah alokasi IP di pool aktif mendekati batas kapasitas subnet /24. Klien baru berisiko gagal memperoleh IP (Obtaining IP address loop).".into(),
                description_en: "Active IP allocations in pool are high. New DHCP clients may fail to acquire IP leases if the pool is exhausted.".into(),
                recommendation: "Perbesar rentang IP Pool via POST /api/v1/ip/pools atau perpendek lease-time DHCP server.".into(),
            });
        }
    }

    // 8. Insecure Management Services Audit
    let mut insecure_services = Vec::new();
    if let Ok(svc_rows) = svc_res {
        for s in svc_rows {
            let name = s.get("name").unwrap_or("");
            let disabled = s.get("disabled").map(|v| v == "true" || v == "yes").unwrap_or(false);
            let address = s.get("address").unwrap_or("").trim();

            if !disabled {
                if name == "telnet" && (address.is_empty() || address == "0.0.0.0/0") {
                    insecure_services.push("telnet (port 23)");
                }
                if name == "api" && (address.is_empty() || address == "0.0.0.0/0") {
                    insecure_services.push("api-plaintext (port 8728)");
                }
            }
        }
        if !insecure_services.is_empty() {
            score -= 5;
            issues.push(DiagnosticIssue {
                category: "Security".into(),
                severity: "INFO".into(),
                title: format!("Layanan Manajemen Tanpa Enkripsi Aktif: {}", insecure_services.join(", ")),
                description_id: "Layanan seperti Telnet atau API plaintext aktif tanpa pembatasan source IP. Kredensial dapat disadap jika berada di jaringan publik.".into(),
                description_en: "Plaintext management services are exposed without IP address restrictions.".into(),
                recommendation: "Nonaktifkan Telnet via POST /api/v1/system/service/toggle atau batasi address subnet admin terpercaya.".into(),
            });
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
            "dns_upstream_configured": dns_configured,
            "dns_upstream_servers": dns_servers,
            "open_dns_resolver_risk": open_dns_risk,
            "active_pool_leases": pool_used_count,
            "insecure_services_exposed": insecure_services
        },
        "total_issues_found": issues.len(),
        "issues": issues,
    })))
}

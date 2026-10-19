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
pub struct WanInterfaceReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// Nama interface WAN internet publik (misal: "ether1" atau "pppoe-out1")
    pub wan_interface: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct AutoRebootReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// Waktu reboot harian (default: "04:00:00")
    pub reboot_time: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct FullHardeningReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub wan_interface: String,
    pub enable_bogon_filter: Option<bool>,
    pub enable_dns_guard: Option<bool>,
    pub enable_raw_antiddos: Option<bool>,
    pub enable_auto_reboot: Option<bool>,
    pub reboot_time: Option<String>,
}

const BOGON_LIST_NAME: &str = "BOGON_IPS";
const HARDENING_COMMENT_TAG: &str = "[Security-Hardening]";

/// POST /api/v1/security/hardening/deploy-bogon-filter - Mencegah IP Bogon/Martian masuk dari interface WAN (RFC 1918 & Spoofing)
pub async fn deploy_bogon_filter(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<WanInterfaceReq>,
) -> Result<Json<Value>, ApiError> {
    crate::routes::sanitize::validate_safe_identifier(&req.wan_interface, "wan_interface")?;

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let bogon_subnets = [
        ("0.0.0.0/8", "RFC 1122 This Network"),
        ("10.0.0.0/8", "RFC 1918 Private-Use"),
        ("100.64.0.0/10", "RFC 6598 Shared Address Space / CGNAT"),
        ("127.0.0.0/8", "RFC 1122 Loopback"),
        ("169.254.0.0/16", "RFC 3927 Link Local / APIPA"),
        ("172.16.0.0/12", "RFC 1918 Private-Use"),
        ("192.0.0.0/24", "RFC 6890 IETF Protocol Assignments"),
        ("192.0.2.0/24", "RFC 5737 Documentation TEST-NET-1"),
        ("192.168.0.0/16", "RFC 1918 Private-Use"),
        ("198.18.0.0/15", "RFC 2544 Benchmarking"),
        ("198.51.100.0/24", "RFC 5737 Documentation TEST-NET-2"),
        ("203.0.113.0/24", "RFC 5737 Documentation TEST-NET-3"),
        ("224.0.0.0/4", "RFC 5771 Multicast"),
        ("240.0.0.0/4", "RFC 1112 Reserved"),
    ];

    let mut added_subnets = 0;
    for (subnet, comment) in bogon_subnets {
        let full_comment = format!("{} {}", HARDENING_COMMENT_TAG, comment);
        let cmd = build_command(
            "/ip/firewall/address-list/add",
            [
                ("list", BOGON_LIST_NAME),
                ("address", subnet),
                ("comment", full_comment.as_str()),
            ],
        );
        if client.run(cmd).await.is_ok() {
            added_subnets += 1;
        }
    }

    // Pasang filter drop paket masuk dari WAN jika sumbernya adalah IP bogon
    let input_comment = format!("{} Drop Bogon IPs on Input WAN", HARDENING_COMMENT_TAG);
    let _ = client.run(build_command(
        "/ip/firewall/filter/add",
        [
            ("chain", "input"),
            ("in-interface", req.wan_interface.as_str()),
            ("src-address-list", BOGON_LIST_NAME),
            ("action", "drop"),
            ("comment", input_comment.as_str()),
        ],
    )).await;

    let forward_comment = format!("{} Drop Bogon IPs on Forward WAN", HARDENING_COMMENT_TAG);
    let _ = client.run(build_command(
        "/ip/firewall/filter/add",
        [
            ("chain", "forward"),
            ("in-interface", req.wan_interface.as_str()),
            ("src-address-list", BOGON_LIST_NAME),
            ("action", "drop"),
            ("comment", forward_comment.as_str()),
        ],
    )).await;

    Ok(Json(json!({
        "success": true,
        "message": format!("Bogon & Martian IP Filter berhasil diterapkan pada WAN interface '{}'!", req.wan_interface),
        "wan_interface": req.wan_interface,
        "subnets_registered": added_subnets,
        "address_list": BOGON_LIST_NAME
    })))
}

/// POST /api/v1/security/hardening/deploy-dns-guard - Memblokir DNS Open-Resolver abuse di interface WAN
pub async fn deploy_dns_guard(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<WanInterfaceReq>,
) -> Result<Json<Value>, ApiError> {
    crate::routes::sanitize::validate_safe_identifier(&req.wan_interface, "wan_interface")?;

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let udp_comment = format!("{} Drop WAN DNS UDP/53 (Anti Open Resolver DDoS)", HARDENING_COMMENT_TAG);
    let _ = client.run(build_command(
        "/ip/firewall/filter/add",
        [
            ("chain", "input"),
            ("in-interface", req.wan_interface.as_str()),
            ("protocol", "udp"),
            ("dst-port", "53"),
            ("action", "drop"),
            ("comment", udp_comment.as_str()),
        ],
    )).await;

    let tcp_comment = format!("{} Drop WAN DNS TCP/53 (Anti Open Resolver DDoS)", HARDENING_COMMENT_TAG);
    let _ = client.run(build_command(
        "/ip/firewall/filter/add",
        [
            ("chain", "input"),
            ("in-interface", req.wan_interface.as_str()),
            ("protocol", "tcp"),
            ("dst-port", "53"),
            ("action", "drop"),
            ("comment", tcp_comment.as_str()),
        ],
    )).await;

    Ok(Json(json!({
        "success": true,
        "wan_interface": req.wan_interface,
        "message": "Proteksi DNS Open-Resolver berhasil aktif! Permintaan DNS port 53 dari luar internet diblokir."
    })))
}

/// POST /api/v1/security/hardening/deploy-raw-antiddos - Mengaktifkan Raw Firewall Anti-DDoS sebelum Conntrack (MTCTCE)
pub async fn deploy_raw_antiddos(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<WanInterfaceReq>,
) -> Result<Json<Value>, ApiError> {
    crate::routes::sanitize::validate_safe_identifier(&req.wan_interface, "wan_interface")?;

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // 1. Drop paket TCP tanpa flag (Null scan) di RAW table
    let null_comment = format!("{} Raw Drop TCP Null Packets", HARDENING_COMMENT_TAG);
    let _ = client.run(build_command(
        "/ip/firewall/raw/add",
        [
            ("chain", "prerouting"),
            ("in-interface", req.wan_interface.as_str()),
            ("protocol", "tcp"),
            ("tcp-flags", "!fin,!syn,!rst,!psh,!ack,!urg"),
            ("action", "drop"),
            ("comment", null_comment.as_str()),
        ],
    )).await;

    // 2. Drop paket SYN & FIN scan di RAW table
    let synfin_comment = format!("{} Raw Drop TCP SYN/FIN Packets", HARDENING_COMMENT_TAG);
    let _ = client.run(build_command(
        "/ip/firewall/raw/add",
        [
            ("chain", "prerouting"),
            ("in-interface", req.wan_interface.as_str()),
            ("protocol", "tcp"),
            ("tcp-flags", "fin,syn"),
            ("action", "drop"),
            ("comment", synfin_comment.as_str()),
        ],
    )).await;

    // 3. Drop paket invalid sebelum masuk Connection Tracking
    let invalid_comment = format!("{} Raw Drop Invalid Connection State", HARDENING_COMMENT_TAG);
    let _ = client.run(build_command(
        "/ip/firewall/raw/add",
        [
            ("chain", "prerouting"),
            ("in-interface", req.wan_interface.as_str()),
            ("connection-state", "invalid"),
            ("action", "drop"),
            ("comment", invalid_comment.as_str()),
        ],
    )).await;

    Ok(Json(json!({
        "success": true,
        "wan_interface": req.wan_interface,
        "table": "/ip firewall raw",
        "message": "Aturan Raw Firewall Anti-DDoS berhasil dipasang! Paket anomali dibuang sebelum membebani tabel Connection Tracking."
    })))
}

/// POST /api/v1/security/hardening/setup-auto-reboot - Menjadwalkan Auto-Reboot Subuh (Praktik Wajib RT-RW Net)
pub async fn setup_auto_reboot(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AutoRebootReq>,
) -> Result<Json<Value>, ApiError> {
    if !st.allow_system_reboot {
        return Err(ApiError::Forbidden(
            "Auto reboot setup is disabled (allow_system_reboot is false)".to_string(),
        ));
    }
    if !st.allow_terminal_exec {
        return Err(ApiError::Forbidden(
            "Scheduler configuration requires terminal execution permission (allow_terminal_exec is false)".to_string(),
        ));
    }

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let reboot_time = req.reboot_time.as_deref().unwrap_or("04:00:00");
    crate::routes::sanitize::validate_time_format(reboot_time)?;
    let sched_name = "AutoReboot_Subuh_0400";
    let sched_comment = format!("{} Pembersihan RAM & Reset Session ISP Harian", HARDENING_COMMENT_TAG);

    // Hapus scheduler lama jika ada
    let schedulers = client.run(build_command(
        "/system/scheduler/print",
        [("?name", sched_name)],
    )).await.unwrap_or_default();

    for s in schedulers {
        if let Some(id) = s.get(".id") {
            let _ = client.run(build_command("/system/scheduler/remove", [(".id", id)])).await;
        }
    }

    client.run(build_command(
        "/system/scheduler/add",
        [
            ("name", sched_name),
            ("start-time", reboot_time),
            ("interval", "1d"),
            ("on-event", "/system reboot"),
            ("comment", sched_comment.as_str()),
        ],
    )).await?;

    Ok(Json(json!({
        "success": true,
        "scheduler_name": sched_name,
        "reboot_time": reboot_time,
        "interval": "1d (Setiap hari)",
        "message": format!("Scheduler Auto-Reboot berhasil diaktifkan setiap pukul {} WIB!", reboot_time)
    })))
}

/// POST /api/v1/security/hardening/deploy-full-hardening - 1-Click Deploy SEMUA proteksi Hardening & Auto-Reboot
pub async fn deploy_full_hardening(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<FullHardeningReq>,
) -> Result<Json<Value>, ApiError> {
    crate::routes::sanitize::validate_safe_identifier(&req.wan_interface, "wan_interface")?;

    let (target, router_id) = AppState::parse_target(&headers, req.router.clone(), req.router_id.clone());
    let _client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut actions_taken = Vec::new();

    // 1. Bogon Filter
    if req.enable_bogon_filter.unwrap_or(true) {
        let bogon_req = WanInterfaceReq {
            router: req.router.clone(),
            router_id: req.router_id.clone(),
            wan_interface: req.wan_interface.clone(),
        };
        if deploy_bogon_filter(State(st.clone()), headers.clone(), Json(bogon_req)).await.is_ok() {
            actions_taken.push("WAN Bogon & Martian IP Filtering deployed");
        }
    }

    // 2. DNS Guard
    if req.enable_dns_guard.unwrap_or(true) {
        let dns_req = WanInterfaceReq {
            router: req.router.clone(),
            router_id: req.router_id.clone(),
            wan_interface: req.wan_interface.clone(),
        };
        if deploy_dns_guard(State(st.clone()), headers.clone(), Json(dns_req)).await.is_ok() {
            actions_taken.push("WAN DNS Open-Resolver port 53 drop deployed");
        }
    }

    // 3. Raw Anti-DDoS
    if req.enable_raw_antiddos.unwrap_or(true) {
        let raw_req = WanInterfaceReq {
            router: req.router.clone(),
            router_id: req.router_id.clone(),
            wan_interface: req.wan_interface.clone(),
        };
        if deploy_raw_antiddos(State(st.clone()), headers.clone(), Json(raw_req)).await.is_ok() {
            actions_taken.push("Raw Firewall Pre-Conntrack Anti-DDoS deployed");
        }
    }

    // 4. Auto Reboot
    if req.enable_auto_reboot.unwrap_or(true) {
        let reboot_req = AutoRebootReq {
            router: req.router.clone(),
            router_id: req.router_id.clone(),
            reboot_time: req.reboot_time.clone(),
        };
        if setup_auto_reboot(State(st.clone()), headers.clone(), Json(reboot_req)).await.is_ok() {
            actions_taken.push("Daily 04:00 AM Auto-Reboot Scheduler deployed");
        }
    }

    Ok(Json(json!({
        "success": true,
        "wan_interface": req.wan_interface,
        "message": "Full Enterprise Hardening Suite berhasil diterapkan pada router!",
        "deployed_protections": actions_taken
    })))
}

/// GET or POST /api/v1/security/hardening/audit-status - Menampilkan ringkasan status hardening router saat ini
pub async fn audit_status(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let filter_rows = client
        .run(build_command("/ip/firewall/filter/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();

    let raw_rows = client
        .run(build_command("/ip/firewall/raw/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();

    let sched_rows = client
        .run(build_command("/system/scheduler/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();

    let mut hardening_rules: Vec<HashMap<String, String>> = Vec::new();

    for r in filter_rows {
        if let Some(c) = r.get("comment") {
            if c.contains(HARDENING_COMMENT_TAG) {
                hardening_rules.push(r.attrs);
            }
        }
    }

    for r in raw_rows {
        if let Some(c) = r.get("comment") {
            if c.contains(HARDENING_COMMENT_TAG) {
                hardening_rules.push(r.attrs);
            }
        }
    }

    let has_auto_reboot = sched_rows.iter().any(|s| {
        s.get("name").map(|n| n.contains("AutoReboot")).unwrap_or(false)
            || s.get("comment").map(|c| c.contains(HARDENING_COMMENT_TAG)).unwrap_or(false)
    });

    Ok(Json(json!({
        "success": true,
        "active_hardening_rules_count": hardening_rules.len(),
        "has_auto_reboot_scheduler": has_auto_reboot,
        "rules": hardening_rules
    })))
}

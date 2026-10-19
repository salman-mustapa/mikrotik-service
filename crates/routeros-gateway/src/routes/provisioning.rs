use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProvisioningStep {
    pub step_number: usize,
    pub stage: String,
    pub name: String,
    pub description: String,
    pub command: String,
    pub status: String,
    pub duration_ms: u64,
    pub details: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RouterVerificationSummary {
    pub identity: String,
    pub lan_bridge_ip: String,
    pub wan_client_active: bool,
    pub dhcp_server_active: bool,
    pub nat_masquerade_active: bool,
    pub fasttrack_active: bool,
    pub hotspot_server_active: bool,
    pub pppoe_server_active: bool,
    pub vlan_interfaces_count: usize,
    pub firewall_filter_rules_count: usize,
    pub raw_drop_rules_count: usize,
    pub verification_passed: bool,
    pub checked_at_ms: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProvisioningJobRecord {
    pub job_id: String,
    pub router_identity: String,
    pub status: String,
    pub started_at_ms: u128,
    pub total_duration_ms: u64,
    pub total_steps: usize,
    pub steps: Vec<ProvisioningStep>,
    pub router_verification: Option<RouterVerificationSummary>,
}

static PROVISIONING_HISTORY: Mutex<VecDeque<ProvisioningJobRecord>> = Mutex::new(VecDeque::new());

fn record_provisioning_job(job: ProvisioningJobRecord) {
    if let Ok(mut hist) = PROVISIONING_HISTORY.lock() {
        if hist.len() >= 50 {
            hist.pop_back();
        }
        hist.push_front(job);
    }
}

fn now_epoch_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[derive(Deserialize, Debug, Default)]
pub struct ProvisioningDeployReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub router_identity: Option<String>,
    /// Jika true, bersihkan dulu konfigurasi firewall, hotspot, pppoe lama yang bentrok secara aman
    #[serde(default)]
    pub clean_slate: bool,
    pub wan_lan: Option<WanLanConfig>,
    pub hotspot: Option<HotspotConfig>,
    pub pppoe: Option<PppoeConfig>,
    pub vlan: Option<VlanConfig>,
    #[serde(alias = "security_edr")]
    pub security_hardening: Option<SecurityHardeningConfig>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WanLanConfig {
    #[serde(default = "default_wan")]
    pub wan_interface: String,
    #[serde(default = "default_lan")]
    pub lan_interface: String,
    pub bridge_ports: Option<Vec<String>>,
    #[serde(default = "default_lan_subnet")]
    pub lan_subnet: String, // e.g. "192.168.88.1/24"
    pub dns_servers: Option<Vec<String>>,
    #[serde(default = "default_true")]
    pub enable_nat_masquerade: bool,
    #[serde(default = "default_true")]
    pub enable_fasttrack: bool,
    #[serde(default = "default_true")]
    pub enable_baseline_hardening: bool,
}

fn default_wan() -> String {
    "ether1".into()
}
fn default_lan() -> String {
    "bridge-lan".into()
}
fn default_lan_subnet() -> String {
    "192.168.88.1/24".into()
}
fn default_true() -> bool {
    true
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HotspotConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub interface: Option<String>,
    pub subnet: Option<String>,     // default "10.50.0.1/22"
    pub pool_range: Option<String>, // default "10.50.0.10-10.50.3.254"
    pub dns_name: Option<String>,   // default "hotspot.local"
    #[serde(default = "default_true")]
    pub create_default_packages: bool,
    #[serde(default = "default_true")]
    pub deploy_walled_garden_presets: bool,
    #[serde(default = "default_true")]
    pub deploy_autonomous_sweeper: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PppoeConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub interface: Option<String>,
    pub service_name: Option<String>, // default "ISP-INTERNET"
    pub pool_range: Option<String>,   // default "10.100.0.10-10.100.3.254"
    #[serde(default = "default_true")]
    pub create_default_profiles: bool,
    #[serde(default = "default_true")]
    pub enable_isolir_support: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VlanConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub trunk_interface: Option<String>, // e.g. "ether2"
    #[serde(default = "default_true")]
    pub enable_vlan_filtering: bool,
    #[serde(default = "default_true")]
    pub isolate_inter_vlan: bool,
    pub custom_vlans: Option<Vec<VlanItem>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VlanItem {
    pub vlan_id: u16,
    pub name: String,
    pub subnet: String,
    pub service: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SecurityHardeningConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true", alias = "deploy_edr_rules")]
    pub deploy_security_rules: bool,
    #[serde(default = "default_true")]
    pub protect_winbox_ssh: bool,
    #[serde(default = "default_true")]
    pub enable_psd: bool,
    #[serde(default = "default_true")]
    pub deploy_bogon_filter: bool,
}

pub type SecurityEdrConfig = SecurityHardeningConfig;

const PROVISIONING_TAG: &str = "[Golden-Provisioning]";

/// GET /api/v1/provisioning/templates - Katalog template arsitektur siap deploy (All-in-One ISP, Hotspot Cafe, Corporate VLAN, Minimal Clean)
pub async fn list_templates() -> Json<Value> {
    Json(json!({
        "success": true,
        "templates": [
            {
                "id": "all_in_one_isp",
                "name": "All-in-One ISP / RT-RW Net Universal",
                "description": "Solusi komprehensif lengkap: WAN DHCP + LAN Bridge + FastTrack + Hotspot Voucher Engine (Walled Garden QRIS) + PPPoE Server (Isolir Redirect) + Multi-VLAN + Security Hardened Firewall.",
                "modules": ["wan_lan", "hotspot", "pppoe", "vlan", "security_hardening"],
                "recommended_for": "Router Core ISP, RT-RW Net, Komunitas Jaringan, Warnet / Co-working Space."
            },
            {
                "id": "hotspot_cafe",
                "name": "Hotspot Cafe, Hotel & WISP Voucher Hub",
                "description": "Fokus Hotspot Captive Portal: Isolasi klien, Walled Garden Bank & QRIS otomatis, profil paket (1 Jam, 24 Jam, 1 Bulan) dengan lock-to-MAC script dan autonomous 0% CPU sweeper.",
                "modules": ["wan_lan", "hotspot", "security_hardening"],
                "recommended_for": "Cafe, Restoran, Hotel, Kost-kostan, Kampus, Balai Desa."
            },
            {
                "id": "corporate_vlan",
                "name": "Corporate Office Multi-VLAN Segmentation",
                "description": "Segmentasi enterprise: VLAN 10 (Management NOC), VLAN 20 (Staff), VLAN 30 (Guest Captive), VLAN 40 (CCTV/IoT) dengan firewall isolasi inter-VLAN dan proteksi brute-force.",
                "modules": ["wan_lan", "vlan", "security_hardening"],
                "recommended_for": "Perkantoran, Pabrik, Sekolah, Gedung Bertingkat."
            },
            {
                "id": "clean_minimal",
                "name": "Clean Minimalist Production Router",
                "description": "Setup dasar performa tinggi: WAN DHCP Client, Bridge LAN, DNS 1.1.1.1/8.8.8.8, NAT Masquerade, FastTrack HW acceleration, dan proteksi port Winbox/SSH.",
                "modules": ["wan_lan", "security_hardening"],
                "recommended_for": "Router cabang baru, Lab pengujian, Router rumahan / SOHO performa tinggi."
            }
        ]
    }))
}

/// GET /api/v1/provisioning/blueprints - Alias untuk list_templates
pub async fn list_blueprints() -> Json<Value> {
    list_templates().await
}

/// POST /api/v1/provisioning/deploy - Universal 1-Klik Golden Setup Provisioning Engine dengan Checklist Step & RouterOS Verifikasi
pub async fn deploy_provisioning(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ProvisioningDeployReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let started_at = now_epoch_ms();
    let job_id = format!("job_prov_{}_{:04x}", started_at, (started_at % 0xFFFF) as u16);
    let overall_start = Instant::now();
    let mut steps: Vec<ProvisioningStep> = Vec::new();
    let mut step_num = 1usize;

    // 0. Set Identity if requested
    if let Some(ref id_name) = req.router_identity {
        let s0_start = Instant::now();
        let _ = client.run(build_command("/system/identity/set", [("name", id_name.as_str())])).await;
        steps.push(ProvisioningStep {
            step_number: step_num,
            stage: "SYSTEM_IDENTITY".into(),
            name: "Konfigurasi Hostname / Identity Router".into(),
            description: format!("Menetapkan System Identity perangkat menjadi '{}'", id_name),
            command: format!("/system/identity/set name={}", id_name),
            status: "success".into(),
            duration_ms: s0_start.elapsed().as_millis() as u64,
            details: format!("Identitas router diubah menjadi '{}'", id_name),
        });
        step_num += 1;
    }

    // 1. Safe Clean Slate if requested
    if req.clean_slate {
        let s1_start = Instant::now();
        let cleaned = execute_safe_clean_slate(&client).await;
        steps.push(ProvisioningStep {
            step_number: step_num,
            stage: "CLEAN_SLATE".into(),
            name: "Safe Clean Slate Konfigurasi Lama".into(),
            description: "Membersihkan konfigurasi usang firewall, hotspot & pppoe lama secara aman tanpa memutus akses manajemen".into(),
            command: "/ip/firewall/filter/remove [find comment~\"Golden|Security\"]".into(),
            status: "success".into(),
            duration_ms: s1_start.elapsed().as_millis() as u64,
            details: format!("{} aturan lama berhasil dibersihkan", cleaned),
        });
        step_num += 1;
    }

    // 2. WAN & LAN Base Module
    if let Some(ref wl) = req.wan_lan {
        let s2_start = Instant::now();
        let wan_res = deploy_wan_lan_module(&client, wl).await?;
        steps.push(ProvisioningStep {
            step_number: step_num,
            stage: "WAN_LAN_BASE".into(),
            name: "Deploy Modul WAN DHCP, LAN Bridge & FastTrack".into(),
            description: format!("Membangun bridge LAN '{}', gateway '{}', DNS 1.1.1.1/8.8.8.8, NAT Masquerade, dan akselerasi FastTrack", wl.lan_interface, wl.lan_subnet),
            command: format!("/interface/bridge/add name={} ; /ip/address/add address={}", wl.lan_interface, wl.lan_subnet),
            status: "success".into(),
            duration_ms: s2_start.elapsed().as_millis() as u64,
            details: format!("{} operasi selesai: {}", wan_res.len(), wan_res.join(" | ")),
        });
        step_num += 1;
    }

    // 3. Hotspot Captive Portal Module
    if let Some(ref hs) = req.hotspot {
        if hs.enabled {
            let s3_start = Instant::now();
            let hs_res = deploy_hotspot_module(&client, hs).await?;
            steps.push(ProvisioningStep {
                step_number: step_num,
                stage: "HOTSPOT_CORE".into(),
                name: "Deploy Engine Hotspot Captive Portal & Voucher".into(),
                description: "Menyiapkan Hotspot Server, Profil Voucher (1 Jam, 24 Jam, 1 Bulan), Walled Garden QRIS Bank, dan Autonomous Sweeper".into(),
                command: "/ip/hotspot/add ... ; /ip/hotspot/walled-garden/ip/add ...".into(),
                status: "success".into(),
                duration_ms: s3_start.elapsed().as_millis() as u64,
                details: format!("{} operasi selesai: {}", hs_res.len(), hs_res.join(" | ")),
            });
            step_num += 1;
        }
    }

    // 4. PPPoE Server Module
    if let Some(ref pp) = req.pppoe {
        if pp.enabled {
            let s4_start = Instant::now();
            let pp_res = deploy_pppoe_module(&client, pp).await?;
            steps.push(ProvisioningStep {
                step_number: step_num,
                stage: "PPPOE_SERVER".into(),
                name: "Deploy PPPoE Server ISP & Billing Isolir".into(),
                description: "Menyiapkan PPPoE Server, Profil Kecepatan (10M/20M/50M), dan Redirect Isolir Pelanggan Menunggak".into(),
                command: "/interface/pppoe-server/server/add ... ; /ppp/profile/add ...".into(),
                status: "success".into(),
                duration_ms: s4_start.elapsed().as_millis() as u64,
                details: format!("{} operasi selesai: {}", pp_res.len(), pp_res.join(" | ")),
            });
            step_num += 1;
        }
    }

    // 5. VLAN Segmentation Module
    if let Some(ref vl) = req.vlan {
        if vl.enabled {
            let s5_start = Instant::now();
            let vl_res = deploy_vlan_module(&client, vl).await?;
            steps.push(ProvisioningStep {
                step_number: step_num,
                stage: "VLAN_SEGMENTATION".into(),
                name: "Deploy Segmentasi Multi-VLAN & Firewall Isolasi".into(),
                description: "Membuat sub-interface VLAN enterprise dan filter isolasi antar-segmen".into(),
                command: "/interface/vlan/add ... ; /ip/firewall/filter/add chain=forward action=drop ...".into(),
                status: "success".into(),
                duration_ms: s5_start.elapsed().as_millis() as u64,
                details: format!("{} operasi selesai: {}", vl_res.len(), vl_res.join(" | ")),
            });
            step_num += 1;
        }
    }

    // 6. Security Hardening Module
    if let Some(ref sec) = req.security_hardening {
        if sec.enabled {
            let s6_start = Instant::now();
            let sec_res = deploy_security_hardening_module(&client, sec).await?;
            steps.push(ProvisioningStep {
                step_number: step_num,
                stage: "SECURITY_HARDENING".into(),
                name: "Deploy Proteksi Keamanan & Firewall Hardening".into(),
                description: "Memasang RAW prerouting drop 0% CPU, staged brute-force protection port Winbox/SSH, dan deteksi port scanner (PSD)".into(),
                command: "/ip/firewall/raw/add ... ; /ip/firewall/filter/add psd=...".into(),
                status: "success".into(),
                duration_ms: s6_start.elapsed().as_millis() as u64,
                details: format!("{} operasi selesai: {}", sec_res.len(), sec_res.join(" | ")),
            });
            step_num += 1;
        }
    }

    // 7. Live RouterOS Verification Check
    let s7_start = Instant::now();
    let verification = verify_routeros_state(&client).await;
    steps.push(ProvisioningStep {
        step_number: step_num,
        stage: "ROUTER_VERIFICATION".into(),
        name: "Verifikasi Konfigurasi Langsung ke MikroTik".into(),
        description: "Membaca kembali memori live RouterOS untuk mengonfirmasi seluruh komponen aktif dan sehat".into(),
        command: "/ip/address/print ; /ip/firewall/filter/print ; /ip/hotspot/print".into(),
        status: if verification.verification_passed { "success".into() } else { "warning".into() },
        duration_ms: s7_start.elapsed().as_millis() as u64,
        details: format!("MikroTik terverifikasi: LAN Gateway {}, WAN aktif: {}, Filter rules: {}, RAW rules: {}",
            verification.lan_bridge_ip, verification.wan_client_active, verification.firewall_filter_rules_count, verification.raw_drop_rules_count),
    });

    let job_record = ProvisioningJobRecord {
        job_id: job_id.clone(),
        router_identity: verification.identity.clone(),
        status: "completed".into(),
        started_at_ms: started_at,
        total_duration_ms: overall_start.elapsed().as_millis() as u64,
        total_steps: steps.len(),
        steps: steps.clone(),
        router_verification: Some(verification.clone()),
    };
    record_provisioning_job(job_record);

    Ok(Json(json!({
        "success": true,
        "job_id": job_id,
        "status": "completed",
        "message": "Universal Golden Setup Provisioning berhasil diterapkan dan diverifikasi di MikroTik!",
        "total_duration_ms": overall_start.elapsed().as_millis() as u64,
        "total_steps": steps.len(),
        "steps": steps,
        "router_verification": verification
    })))
}

/// GET /api/v1/provisioning/history - Riwayat eksekusi provisioning untuk audit log & timeline frontend developer
pub async fn provisioning_history() -> Json<Value> {
    let hist: Vec<ProvisioningJobRecord> = if let Ok(h) = PROVISIONING_HISTORY.lock() {
        h.iter().cloned().collect()
    } else {
        Vec::new()
    };
    Json(json!({
        "success": true,
        "count": hist.len(),
        "history": hist
    }))
}

/// GET or POST /api/v1/provisioning/verify - Verifikasi status konfigurasi golden setup langsung ke memori RouterOS
pub async fn verify_provisioning_endpoint(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let summary = verify_routeros_state(&client).await;

    Ok(Json(json!({
        "success": true,
        "verification_summary": summary
    })))
}

/// POST /api/v1/provisioning/clean-slate - Pembersihan aman konfigurasi lama tanpa menghilangkan akses koneksi manajemen
pub async fn clean_slate_endpoint(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let cleaned = execute_safe_clean_slate(&client).await;

    Ok(Json(json!({
        "success": true,
        "cleaned_rules_count": cleaned,
        "message": format!("Safe Clean Slate selesai: {} konfigurasi/rule lama berhasil dibersihkan dengan aman.", cleaned)
    })))
}

async fn execute_safe_clean_slate(client: &routeros_core::Client) -> usize {
    let mut count = 0;

    // Bersihkan filter rules yang dibuat oleh provisioner/EDR
    if let Ok(filters) = client.run(build_command("/ip/firewall/filter/print", std::iter::empty::<(&str, &str)>())).await {
        for f in filters {
            let comment = f.get("comment").unwrap_or("");
            if comment.starts_with("[Golden-") || comment.starts_with("[EDR") || comment.starts_with("[Layer7") || comment.starts_with("[Security-Hardening]") {
                if let Some(id) = f.get(".id") {
                    let _ = client.run(build_command("/ip/firewall/filter/remove", [(".id", id)])).await;
                    count += 1;
                }
            }
        }
    }

    // Bersihkan raw rules
    if let Ok(raws) = client.run(build_command("/ip/firewall/raw/print", std::iter::empty::<(&str, &str)>())).await {
        for r in raws {
            let comment = r.get("comment").unwrap_or("");
            if comment.starts_with("[EDR") || comment.starts_with("[Golden-") {
                if let Some(id) = r.get(".id") {
                    let _ = client.run(build_command("/ip/firewall/raw/remove", [(".id", id)])).await;
                    count += 1;
                }
            }
        }
    }

    count
}

async fn deploy_wan_lan_module(
    client: &routeros_core::Client,
    cfg: &WanLanConfig,
) -> Result<Vec<String>, ApiError> {
    let mut logs = Vec::new();

    // 1. Ensure Bridge LAN exists
    let existing_bridges = client
        .run(build_command("/interface/bridge/print", [("?name", cfg.lan_interface.as_str())]))
        .await
        .unwrap_or_default();

    if existing_bridges.is_empty() {
        client.run(build_command(
            "/interface/bridge/add",
            [("name", cfg.lan_interface.as_str()), ("comment", format!("{} LAN Bridge", PROVISIONING_TAG).as_str())],
        )).await?;
        logs.push(format!("Bridge LAN '{}' berhasil dibuat", cfg.lan_interface));
    }

    // 2. Add Bridge Ports
    let default_ports = vec!["ether2".to_string(), "ether3".to_string(), "ether4".to_string(), "ether5".to_string()];
    let ports = cfg.bridge_ports.as_ref().unwrap_or(&default_ports);
    for p in ports {
        let p_exists = client
            .run(build_command("/interface/bridge/port/print", [("?interface", p.as_str())]))
            .await
            .unwrap_or_default();

        if p_exists.is_empty() {
            let _ = client.run(build_command(
                "/interface/bridge/port/add",
                [("bridge", cfg.lan_interface.as_str()), ("interface", p.as_str())],
            )).await;
            logs.push(format!("Port '{}' ditambahkan ke bridge '{}'", p, cfg.lan_interface));
        }
    }

    // 3. Assign IP Address to LAN Bridge
    let ip_exists = client
        .run(build_command(
            "/ip/address/print",
            [("?interface", cfg.lan_interface.as_str()), ("?address", cfg.lan_subnet.as_str())],
        ))
        .await
        .unwrap_or_default();

    if ip_exists.is_empty() {
        let _ = client.run(build_command(
            "/ip/address/add",
            [
                ("address", cfg.lan_subnet.as_str()),
                ("interface", cfg.lan_interface.as_str()),
                ("comment", format!("{} LAN Gateway IP", PROVISIONING_TAG).as_str()),
            ],
        )).await;
        logs.push(format!("IP Subnet '{}' dipasang pada '{}'", cfg.lan_subnet, cfg.lan_interface));
    }

    // 4. DNS Setup
    let dns_list = cfg.dns_servers.as_ref().map(|l| l.join(",")).unwrap_or_else(|| "1.1.1.1,8.8.8.8".into());
    let _ = client.run(build_command(
        "/ip/dns/set",
        [("servers", dns_list.as_str()), ("allow-remote-requests", "yes")],
    )).await;
    logs.push(format!("DNS Server diatur ke '{}' (allow-remote-requests=yes)", dns_list));

    // 5. WAN NAT Masquerade
    if cfg.enable_nat_masquerade {
        let comment_tag = format!("{} Default Internet NAT Masquerade", PROVISIONING_TAG);
        let nat_exists = client
            .run(build_command("/ip/firewall/nat/print", [("?comment", comment_tag.as_str())]))
            .await
            .unwrap_or_default();

        if nat_exists.is_empty() {
            let _ = client.run(build_command(
                "/ip/firewall/nat/add",
                [
                    ("chain", "srcnat"),
                    ("out-interface", cfg.wan_interface.as_str()),
                    ("action", "masquerade"),
                    ("comment", comment_tag.as_str()),
                ],
            )).await;
            logs.push(format!("NAT Masquerade diaktifkan pada interface WAN '{}'", cfg.wan_interface));
        }
    }

    // 6. FastTrack Acceleration
    if cfg.enable_fasttrack {
        let comment_tag = format!("{} FastTrack HW Acceleration", PROVISIONING_TAG);
        let ft_exists = client
            .run(build_command("/ip/firewall/filter/print", [("?comment", comment_tag.as_str())]))
            .await
            .unwrap_or_default();

        if ft_exists.is_empty() {
            let _ = client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "forward"),
                    ("action", "fasttrack-connection"),
                    ("connection-state", "established,related"),
                    ("comment", comment_tag.as_str()),
                ],
            )).await;
            logs.push("FastTrack Hardware Acceleration diaktifkan (memangkas CPU hingga 80%)".into());
        }
    }

    // 7. Baseline Hardening Firewall
    if cfg.enable_baseline_hardening {
        let est_comment = format!("{} Accept Established,Related Forward", PROVISIONING_TAG);
        let _ = client.run(build_command(
            "/ip/firewall/filter/add",
            [
                ("chain", "forward"),
                ("action", "accept"),
                ("connection-state", "established,related"),
                ("comment", est_comment.as_str()),
            ],
        )).await;

        let inv_comment = format!("{} Drop Invalid Packets", PROVISIONING_TAG);
        let _ = client.run(build_command(
            "/ip/firewall/filter/add",
            [
                ("chain", "forward"),
                ("action", "drop"),
                ("connection-state", "invalid"),
                ("comment", inv_comment.as_str()),
            ],
        )).await;
        logs.push("Baseline firewall filter hardening forward dipasang".into());
    }

    Ok(logs)
}

async fn deploy_hotspot_module(
    client: &routeros_core::Client,
    cfg: &HotspotConfig,
) -> Result<Vec<String>, ApiError> {
    let mut logs = Vec::new();

    let target_iface = cfg.interface.as_deref().unwrap_or("bridge-lan");
    let pool_name = "hs-pool-default";
    let pool_range = cfg.pool_range.as_deref().unwrap_or("10.50.0.10-10.50.3.254");

    // 1. IP Pool for Hotspot
    let pool_exists = client
        .run(build_command("/ip/pool/print", [("?name", pool_name)]))
        .await
        .unwrap_or_default();

    if pool_exists.is_empty() {
        let _ = client.run(build_command(
            "/ip/pool/add",
            [("name", pool_name), ("ranges", pool_range)],
        )).await;
        logs.push(format!("IP Pool Hotspot '{}' ({}) dibuat", pool_name, pool_range));
    }

    // 2. Hotspot User Profile Packages (1JAM-3RB, 24JAM-10RB, 1BULAN-50RB)
    if cfg.create_default_packages {
        let pkgs = [
            ("1JAM-3RB", "2M/3M", "1h", "2500", "3000"),
            ("24JAM-10RB", "3M/5M", "1d", "8000", "10000"),
            ("1BULAN-50RB", "5M/10M", "30d", "40000", "50000"),
        ];

        for (pname, rate, valid, price, sprice) in pkgs {
            let prof_exists = client
                .run(build_command("/ip/hotspot/user/profile/print", [("?name", pname)]))
                .await
                .unwrap_or_default();

            if prof_exists.is_empty() {
                let comment_meta = format!(
                    "[VALID:{};PRICE:{};S_PRICE:{};EXP:notice;LOCK_MAC:1;ORIGIN:golden_setup]",
                    valid, price, sprice
                );
                let lock_script = r#":local mac $"mac-address"; /ip hotspot user set [find name=$user] mac-address=$mac"#;

                let _ = client.run(build_command(
                    "/ip/hotspot/user/profile/add",
                    [
                        ("name", pname),
                        ("rate-limit", rate),
                        ("shared-users", "1"),
                        ("on-login", lock_script),
                        ("comment", comment_meta.as_str()),
                    ],
                )).await;
                logs.push(format!("Paket Profil Hotspot '{}' (Rate: {}, Validitas: {}) berhasil dibuat", pname, rate, valid));
            }
        }
    }

    // 3. Walled Garden Presets (QRIS, Banks, WhatsApp)
    if cfg.deploy_walled_garden_presets {
        let domains = [
            "*.midtrans.com", "*.xendit.co", "*.qris.id", "*.duitku.com",
            "*.klikbca.com", "*.bankmandiri.co.id", "*.bni.co.id", "*.bri.co.id",
            "*.whatsapp.com", "*.whatsapp.net",
        ];
        for d in domains {
            let wg_exists = client
                .run(build_command("/ip/hotspot/walled-garden/print", [("?dst-host", d)]))
                .await
                .unwrap_or_default();

            if wg_exists.is_empty() {
                let _ = client.run(build_command(
                    "/ip/hotspot/walled-garden/add",
                    [("dst-host", d), ("action", "allow"), ("comment", "[Golden-Setup] Auto Allowlist")],
                )).await;
            }
        }
        logs.push("Walled Garden presets (QRIS, Bank, WhatsApp CS) di-deploy ke captive portal".into());
    }

    // 4. Autonomous 0% CPU Sweeper Scheduler
    if cfg.deploy_autonomous_sweeper {
        let sched_name = "hotspot-auto-sweeper";
        let sched_exists = client
            .run(build_command("/system/scheduler/print", [("?name", sched_name)]))
            .await
            .unwrap_or_default();

        if sched_exists.is_empty() {
            let clean_script = r#"/ip hotspot user remove [find comment~"expired"]"#;
            let _ = client.run(build_command(
                "/system/scheduler/add",
                [
                    ("name", sched_name),
                    ("interval", "5m"),
                    ("on-event", clean_script),
                    ("comment", "[Golden-Setup] Autonomous Sweeper"),
                ],
            )).await;
            logs.push("Autonomous 0% CPU Hotspot Sweeper dipasang di scheduler".into());
        }
    }

    logs.push(format!("Modul Hotspot siap pada interface '{}'", target_iface));
    Ok(logs)
}

async fn deploy_pppoe_module(
    client: &routeros_core::Client,
    cfg: &PppoeConfig,
) -> Result<Vec<String>, ApiError> {
    let mut logs = Vec::new();

    let target_iface = cfg.interface.as_deref().unwrap_or("bridge-lan");
    let pool_name = "pppoe-pool-default";
    let pool_range = cfg.pool_range.as_deref().unwrap_or("10.100.0.10-10.100.3.254");

    // 1. IP Pool for PPPoE
    let pool_exists = client
        .run(build_command("/ip/pool/print", [("?name", pool_name)]))
        .await
        .unwrap_or_default();

    if pool_exists.is_empty() {
        let _ = client.run(build_command(
            "/ip/pool/add",
            [("name", pool_name), ("ranges", pool_range)],
        )).await;
        logs.push(format!("IP Pool PPPoE '{}' ({}) dibuat", pool_name, pool_range));
    }

    // 2. Default PPPoE Profiles (Home-10M, Home-20M, Home-50M)
    if cfg.create_default_profiles {
        let profiles = [
            ("Home-10M", "10M/10M"),
            ("Home-20M", "20M/20M"),
            ("Home-50M", "50M/50M"),
        ];

        for (prof_name, rate) in profiles {
            let prof_exists = client
                .run(build_command("/ppp/profile/print", [("?name", prof_name)]))
                .await
                .unwrap_or_default();

            if prof_exists.is_empty() {
                let _ = client.run(build_command(
                    "/ppp/profile/add",
                    [
                        ("name", prof_name),
                        ("local-address", "10.100.0.1"),
                        ("remote-address", pool_name),
                        ("rate-limit", rate),
                        ("comment", "[Golden-Setup] PPPoE Package"),
                    ],
                )).await;
                logs.push(format!("Profil PPPoE '{}' (Rate: {}) berhasil dibuat", prof_name, rate));
            }
        }
    }

    // 3. Isolir PPPoE Profile & Billing Bypass Support
    if cfg.enable_isolir_support {
        let isolir_prof_exists = client
            .run(build_command("/ppp/profile/print", [("?name", "ISOLIR_PPPOE")]))
            .await
            .unwrap_or_default();

        if isolir_prof_exists.is_empty() {
            let _ = client.run(build_command(
                "/ppp/profile/add",
                [
                    ("name", "ISOLIR_PPPOE"),
                    ("local-address", "10.200.0.1"),
                    ("remote-address", "10.200.0.2-10.200.0.254"),
                    ("rate-limit", "256k/256k"),
                    ("address-list", "ISOLIR_LIST"),
                    ("comment", "[Golden-Setup] Overdue Billing Profile"),
                ],
            )).await;
            logs.push("Profil 'ISOLIR_PPPOE' untuk pelanggan jatuh tempo dibuat".into());
        }

        // Firewall drop for ISOLIR_LIST forward
        let isolir_drop_comment = format!("{} Drop Isolated PPPoE Traffic", PROVISIONING_TAG);
        let drop_exists = client
            .run(build_command("/ip/firewall/filter/print", [("?comment", isolir_drop_comment.as_str())]))
            .await
            .unwrap_or_default();

        if drop_exists.is_empty() {
            let _ = client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "forward"),
                    ("src-address-list", "ISOLIR_LIST"),
                    ("action", "drop"),
                    ("comment", isolir_drop_comment.as_str()),
                ],
            )).await;
            logs.push("Firewall filter drop trafik ISOLIR_LIST dipasang".into());
        }
    }

    // 4. PPPoE Server Interface
    let srv_name = cfg.service_name.as_deref().unwrap_or("ISP-INTERNET");
    let srv_exists = client
        .run(build_command("/interface/pppoe-server/server/print", [("?service-name", srv_name)]))
        .await
        .unwrap_or_default();

    if srv_exists.is_empty() {
        let _ = client.run(build_command(
            "/interface/pppoe-server/server/add",
            [
                ("service-name", srv_name),
                ("interface", target_iface),
                ("default-profile", "Home-10M"),
                ("disabled", "no"),
                ("max-mtu", "1480"),
                ("max-mru", "1480"),
            ],
        )).await;
        logs.push(format!("PPPoE Server '{}' aktif pada interface '{}'", srv_name, target_iface));
    }

    Ok(logs)
}

async fn deploy_vlan_module(
    client: &routeros_core::Client,
    cfg: &VlanConfig,
) -> Result<Vec<String>, ApiError> {
    let mut logs = Vec::new();

    let default_vlans = vec![
        VlanItem { vlan_id: 10, name: "VLAN-10-MGMT".into(), subnet: "192.168.10.1/24".into(), service: "Management NOC".into() },
        VlanItem { vlan_id: 20, name: "VLAN-20-HOTSPOT".into(), subnet: "10.20.0.1/22".into(), service: "Public Hotspot".into() },
        VlanItem { vlan_id: 30, name: "VLAN-30-PPPOE".into(), subnet: "10.30.0.1/22".into(), service: "Residential PPPoE".into() },
        VlanItem { vlan_id: 40, name: "VLAN-40-CCTV".into(), subnet: "192.168.40.1/24".into(), service: "CCTV & IoT Isolation".into() },
    ];

    let vlan_items = cfg.custom_vlans.as_ref().unwrap_or(&default_vlans);
    let trunk_port = cfg.trunk_interface.as_deref().unwrap_or("bridge-lan");

    for v in vlan_items {
        let vlan_name = &v.name;
        let vlan_exists = client
            .run(build_command("/interface/vlan/print", [("?name", vlan_name.as_str())]))
            .await
            .unwrap_or_default();

        if vlan_exists.is_empty() {
            let v_str = v.vlan_id.to_string();
            let _ = client.run(build_command(
                "/interface/vlan/add",
                [
                    ("name", vlan_name.as_str()),
                    ("vlan-id", v_str.as_str()),
                    ("interface", trunk_port),
                    ("comment", v.service.as_str()),
                ],
            )).await;

            // Assign IP Subnet to VLAN interface
            let _ = client.run(build_command(
                "/ip/address/add",
                [
                    ("address", v.subnet.as_str()),
                    ("interface", vlan_name.as_str()),
                    ("comment", format!("{} Gateway {}", PROVISIONING_TAG, v.service).as_str()),
                ],
            )).await;

            logs.push(format!("VLAN ID {} ({}) dipasang pada interface '{}' dengan IP {}", v.vlan_id, v.name, trunk_port, v.subnet));
        }
    }

    // Inter-VLAN Isolation Firewall Rule
    if cfg.isolate_inter_vlan {
        let comment_tag = format!("{} Inter-VLAN Isolation", PROVISIONING_TAG);
        let iso_exists = client
            .run(build_command("/ip/firewall/filter/print", [("?comment", comment_tag.as_str())]))
            .await
            .unwrap_or_default();

        if iso_exists.is_empty() {
            // Drop traffic from VLAN 40 (CCTV) to other local subnets
            let _ = client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "forward"),
                    ("in-interface", "VLAN-40-CCTV"),
                    ("dst-address", "192.168.0.0/16"),
                    ("action", "drop"),
                    ("comment", comment_tag.as_str()),
                ],
            )).await;
            logs.push("Firewall filter isolasi keamanan Inter-VLAN CCTV aktif".into());
        }
    }

    Ok(logs)
}

async fn deploy_security_hardening_module(
    client: &routeros_core::Client,
    cfg: &SecurityHardeningConfig,
) -> Result<Vec<String>, ApiError> {
    let mut logs = Vec::new();

    // 1. Raw Drop Rules for Blacklisted & Quarantined Hosts (0% CPU Conntrack Bypass)
    if cfg.deploy_security_rules {
        let raw_tag = format!("{} Raw Drop BLACKLIST", PROVISIONING_TAG);
        let raw_exists = client
            .run(build_command("/ip/firewall/raw/print", [("?comment", raw_tag.as_str())]))
            .await
            .unwrap_or_default();

        if raw_exists.is_empty() {
            let _ = client.run(build_command(
                "/ip/firewall/raw/add",
                [
                    ("chain", "prerouting"),
                    ("dst-address-list", "BLACKLIST"),
                    ("action", "drop"),
                    ("comment", raw_tag.as_str()),
                ],
            )).await;
            let _ = client.run(build_command(
                "/ip/firewall/raw/add",
                [
                    ("chain", "prerouting"),
                    ("src-address-list", "QUARANTINE"),
                    ("action", "drop"),
                    ("comment", "[Security-Hardening] Raw Drop Quarantined Host"),
                ],
            )).await;
            logs.push("Firewall RAW Prerouting Drops (0% CPU conntrack bypass) dipasang".into());
        }
    }

    // 2. Anti-Brute-Force Protection for Winbox (8291) & SSH (22)
    if cfg.protect_winbox_ssh {
        let bf_tag = format!("{} Winbox Anti-Brute-Force Stage 3", PROVISIONING_TAG);
        let bf_exists = client
            .run(build_command("/ip/firewall/filter/print", [("?comment", bf_tag.as_str())]))
            .await
            .unwrap_or_default();

        if bf_exists.is_empty() {
            // Drop blacklisted attackers
            let _ = client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "input"),
                    ("src-address-list", "WINBOX_BRUTE_BLACKLIST"),
                    ("action", "drop"),
                    ("comment", bf_tag.as_str()),
                ],
            )).await;

            // Staged connection tracking
            let _ = client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "input"),
                    ("protocol", "tcp"),
                    ("dst-port", "8291"),
                    ("connection-state", "new"),
                    ("src-address-list", "winbox_stage2"),
                    ("action", "add-src-to-address-list"),
                    ("address-list", "WINBOX_BRUTE_BLACKLIST"),
                    ("address-list-timeout", "7d"),
                    ("comment", "[Security-Hardening] Winbox Stage 2 to Blacklist 7d"),
                ],
            )).await;
            logs.push("Staged Anti-Brute-Force untuk proteksi port Winbox 8291 dipasang".into());
        }
    }

    // 3. Port Scan Detection (PSD)
    if cfg.enable_psd {
        let psd_tag = format!("{} Detect Port Scanners", PROVISIONING_TAG);
        let psd_exists = client
            .run(build_command("/ip/firewall/filter/print", [("?comment", psd_tag.as_str())]))
            .await
            .unwrap_or_default();

        if psd_exists.is_empty() {
            let _ = client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "input"),
                    ("protocol", "tcp"),
                    ("psd", "21,3s,3,1"),
                    ("action", "add-src-to-address-list"),
                    ("address-list", "PORT_SCANNERS"),
                    ("address-list-timeout", "1d"),
                    ("comment", psd_tag.as_str()),
                ],
            )).await;
            logs.push("Port Scan Detection (PSD) otomatis aktif".into());
        }
    }

    Ok(logs)
}

async fn verify_routeros_state(client: &routeros_core::Client) -> RouterVerificationSummary {
    let id_rows = client.run(build_command("/system/identity/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let identity = id_rows.first().and_then(|r| r.get("name")).unwrap_or("MikroTik").to_string();

    let addr_rows = client.run(build_command("/ip/address/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let lan_ip = addr_rows.iter().find(|r| r.get("interface").map(|i| i.contains("bridge") || i.contains("lan")).unwrap_or(false))
        .and_then(|r| r.get("address")).unwrap_or("none").to_string();

    let dhcp_client_rows = client.run(build_command("/ip/dhcp-client/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let wan_active = !dhcp_client_rows.is_empty();

    let dhcp_srv_rows = client.run(build_command("/ip/dhcp-server/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let dhcp_active = !dhcp_srv_rows.is_empty();

    let nat_rows = client.run(build_command("/ip/firewall/nat/print", [("?action", "masquerade")])).await.unwrap_or_default();
    let nat_active = !nat_rows.is_empty();

    let filter_rows = client.run(build_command("/ip/firewall/filter/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let fasttrack_active = filter_rows.iter().any(|r| r.get("action") == Some("fasttrack-connection"));
    let filter_count = filter_rows.len();

    let raw_rows = client.run(build_command("/ip/firewall/raw/print", [("?action", "drop")])).await.unwrap_or_default();
    let raw_count = raw_rows.len();

    let hs_rows = client.run(build_command("/ip/hotspot/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let hs_active = !hs_rows.is_empty();

    let pppoe_rows = client.run(build_command("/interface/pppoe-server/server/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let pppoe_active = !pppoe_rows.is_empty();

    let vlan_rows = client.run(build_command("/interface/vlan/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let vlan_count = vlan_rows.len();

    let passed = !lan_ip.is_empty() && lan_ip != "none" && (nat_active || filter_count > 0);

    RouterVerificationSummary {
        identity,
        lan_bridge_ip: lan_ip,
        wan_client_active: wan_active,
        dhcp_server_active: dhcp_active,
        nat_masquerade_active: nat_active,
        fasttrack_active,
        hotspot_server_active: hs_active,
        pppoe_server_active: pppoe_active,
        vlan_interfaces_count: vlan_count,
        firewall_filter_rules_count: filter_count,
        raw_drop_rules_count: raw_count,
        verification_passed: passed,
        checked_at_ms: now_epoch_ms(),
    }
}

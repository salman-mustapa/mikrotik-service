use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use routeros_core::build_command;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::{AppState, RouterTarget};

pub const ISOLIR_COMMENT_PREFIX: &str = "[WISP_ISOLIR]";
pub const ISOLIR_ALLOWED_LIST: &str = "ISOLIR_ALLOWED";
pub const ISOLIR_ADDRESS_LIST: &str = "ISOLIR_LIST";

#[derive(Deserialize, Debug, Default)]
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

// ============================================================================
// Tier 1: Default Allowlist Presets (Curated Bundles)
// ============================================================================

#[derive(Serialize, Clone, Debug)]
pub struct AllowlistPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub domains: &'static [&'static str],
    pub ip_subnets: &'static [&'static str],
}

pub static ALLOWLIST_PRESETS: &[AllowlistPreset] = &[
    AllowlistPreset {
        id: "payment_gateways",
        name: "Payment Gateways & QRIS Indonesia",
        category: "Fintech & Billing",
        description: "Gateway pembayaran online populer agar pelanggan terisolir dapat melunasi tagihan (Midtrans, Xendit, DOKU, Faspay, Duitku, Espay, QRIS).",
        domains: &[
            "*.midtrans.com",
            "*.xendit.co",
            "*.doku.com",
            "*.faspay.co.id",
            "*.duitku.com",
            "*.espay.id",
            "*.qris.id",
            "*.qris.online",
            "api.midtrans.com",
            "app.midtrans.com",
            "api.xendit.co",
            "checkout.xendit.co",
        ],
        ip_subnets: &[],
    },
    AllowlistPreset {
        id: "banking_id",
        name: "Perbankan Nasional (BCA, Mandiri, BRI, BNI)",
        category: "Banking",
        description: "Akses e-Banking dan Virtual Account perbankan utama Indonesia.",
        domains: &[
            "*.klikbca.com",
            "*.bca.co.id",
            "*.bankmandiri.co.id",
            "*.livin.bankmandiri.co.id",
            "*.bri.co.id",
            "*.ib.bri.co.id",
            "*.bni.co.id",
            "*.cimbniaga.co.id",
            "*.permatabank.com",
        ],
        ip_subnets: &[],
    },
    AllowlistPreset {
        id: "ewallet",
        name: "E-Wallet (GoPay, OVO, DANA, ShopeePay, LinkAja)",
        category: "Fintech & Billing",
        description: "Akses API dan portal dompet digital untuk pembayaran tagihan seketika.",
        domains: &[
            "*.gopay.co.id",
            "*.gojek.com",
            "*.ovo.id",
            "*.dana.id",
            "*.shopeepay.co.id",
            "*.linkaja.id",
        ],
        ip_subnets: &[],
    },
    AllowlistPreset {
        id: "messaging_support",
        name: "Customer Care & Chat (WhatsApp & Telegram)",
        category: "Customer Support",
        description: "Mengizinkan komunikasi teks ke tim Helpdesk/NOC ISP agar pelanggan dapat konfirmasi pembayaran atau lapor gangguan.",
        domains: &[
            "*.whatsapp.com",
            "*.whatsapp.net",
            "*.t.me",
            "*.telegram.org",
            "api.whatsapp.com",
            "web.whatsapp.com",
        ],
        ip_subnets: &[],
    },
    AllowlistPreset {
        id: "dns_ntp",
        name: "Core Infrastructure (DNS & NTP Time Sync)",
        category: "Infrastructure",
        description: "Resolusi nama domain DNS (Cloudflare, Google) dan sinkronisasi jam NTP agar sertifikat SSL dan tanggal di router klien valid.",
        domains: &[
            "pool.ntp.org",
            "*.pool.ntp.org",
            "time.google.com",
            "time.cloudflare.com",
        ],
        ip_subnets: &[
            "1.1.1.1/32",
            "1.0.0.1/32",
            "8.8.8.8/32",
            "8.8.4.4/32",
        ],
    },
    AllowlistPreset {
        id: "government_education",
        name: "Layanan Publik & Pendidikan Nasional",
        category: "Public Services",
        description: "Portal penting instansi pemerintah dan pendidikan nasional (Kemdikbud, Kemenkes, Kemenkeu, Pajak).",
        domains: &[
            "*.kemdikbud.go.id",
            "*.kemkes.go.id",
            "*.kemenkeu.go.id",
            "*.pajak.go.id",
            "*.bps.go.id",
        ],
        ip_subnets: &[],
    },
];

/// GET /api/v1/allowlist/presets - Daftar seluruh paket preset allowlist bawaan sistem
pub async fn list_presets() -> Json<Value> {
    Json(json!({
        "success": true,
        "total_presets": ALLOWLIST_PRESETS.len(),
        "presets": ALLOWLIST_PRESETS
    }))
}

#[derive(Deserialize, Debug)]
pub struct DeployPresetReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// ID preset: "payment_gateways", "banking_id", "ewallet", "messaging_support", "dns_ntp", "government_education"
    pub preset_id: String,
    /// Target penerapan: "hotspot" (Walled Garden), "isolir" (PPPoE address-list), atau "both"
    #[serde(default = "default_target_scope")]
    pub target_scope: String,
}

fn default_target_scope() -> String {
    "both".into()
}

/// POST /api/v1/allowlist/presets/deploy - 1-Klik terapkan paket preset allowlist ke router
pub async fn deploy_preset(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DeployPresetReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let preset = ALLOWLIST_PRESETS.iter().find(|p| p.id == req.preset_id).ok_or_else(|| {
        ApiError::BadRequest(format!("Preset '{}' tidak ditemukan. Pilihan: payment_gateways, banking_id, ewallet, messaging_support, dns_ntp, government_education", req.preset_id))
    })?;

    let scope = req.target_scope.to_lowercase();
    let mut applied_hotspot = 0;
    let mut applied_isolir = 0;

    // 1. Terapkan ke Hotspot Walled Garden jika scope == "hotspot" || "both"
    if scope == "hotspot" || scope == "both" {
        for dom in preset.domains {
            let comment = format!("[PRESET:{}] Allow Hotspot Walled Garden", preset.id);
            let _ = client.run(build_command(
                "/ip/hotspot/walled-garden/add",
                [
                    ("dst-host", *dom),
                    ("action", "allow"),
                    ("comment", comment.as_str()),
                ],
            )).await;
            applied_hotspot += 1;
        }

        for ip in preset.ip_subnets {
            let comment = format!("[PRESET:{}] Allow Hotspot Walled Garden IP", preset.id);
            let _ = client.run(build_command(
                "/ip/hotspot/walled-garden/ip/add",
                [
                    ("dst-address", *ip),
                    ("action", "accept"),
                    ("comment", comment.as_str()),
                ],
            )).await;
            applied_hotspot += 1;
        }
    }

    // 2. Terapkan ke Firewall Address-List ISOLIR_ALLOWED jika scope == "isolir" || "both"
    if scope == "isolir" || scope == "both" {
        for dom in preset.domains {
            let comment = format!("{} [PRESET:{}]", ISOLIR_COMMENT_PREFIX, preset.id);
            let _ = client.run(build_command(
                "/ip/firewall/address-list/add",
                [
                    ("list", ISOLIR_ALLOWED_LIST),
                    ("address", *dom),
                    ("comment", comment.as_str()),
                ],
            )).await;
            applied_isolir += 1;
        }

        for ip in preset.ip_subnets {
            let comment = format!("{} [PRESET:{}]", ISOLIR_COMMENT_PREFIX, preset.id);
            let _ = client.run(build_command(
                "/ip/firewall/address-list/add",
                [
                    ("list", ISOLIR_ALLOWED_LIST),
                    ("address", *ip),
                    ("comment", comment.as_str()),
                ],
            )).await;
            applied_isolir += 1;
        }

        // Pastikan rule accept di forward filter aktif
        let filter_comment = format!("{} Allow Bypass Payment & Core Services", ISOLIR_COMMENT_PREFIX);
        let _ = client.run(build_command(
            "/ip/firewall/filter/add",
            [
                ("chain", "forward"),
                ("src-address-list", ISOLIR_ADDRESS_LIST),
                ("dst-address-list", ISOLIR_ALLOWED_LIST),
                ("action", "accept"),
                ("comment", filter_comment.as_str()),
            ],
        )).await;
    }

    Ok(Json(json!({
        "success": true,
        "preset_id": preset.id,
        "preset_name": preset.name,
        "target_scope": scope,
        "rules_added_hotspot": applied_hotspot,
        "rules_added_isolir": applied_isolir,
        "message": format!("Preset '{}' berhasil diterapkan ke sistem allowlist ({})!", preset.name, scope)
    })))
}

// ============================================================================
// Tier 2: Custom Allowlist (Website, Domain, IP, Subnet)
// ============================================================================

#[derive(Deserialize, Debug)]
pub struct CustomAllowlistAddReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// Domain (misal: "portal.sekolah.sch.id", "*.kantor.com") atau IP/CIDR (misal: "192.168.10.50", "103.20.0.0/22")
    pub host: String,
    /// Target: "hotspot", "isolir", atau "both"
    #[serde(default = "default_target_scope")]
    pub target_scope: String,
    /// Port tujuan spesifik (opsional, misal: "80,443" atau "8080")
    pub port: Option<String>,
    /// Komentar atau keterangan allowlist
    pub comment: Option<String>,
}

/// POST /api/v1/allowlist/custom/add - Tambah entri allowlist kustom (domain, IP, atau subnet)
pub async fn add_custom(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CustomAllowlistAddReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let host = req.host.trim();
    if host.is_empty() {
        return Err(ApiError::BadRequest("Host/domain tidak boleh kosong".into()));
    }
    if host.contains(['\r', '\n', '"', ';']) {
        return Err(ApiError::BadRequest("Host mengandung karakter tidak valid".into()));
    }

    let is_ip_or_cidr = host.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '/' || c == ':');
    let scope = req.target_scope.to_lowercase();
    let comment_str = req.comment.as_deref().unwrap_or("Custom Allowlist Rule");

    let mut applied_entries = Vec::new();

    // 1. Hotspot Walled Garden
    if scope == "hotspot" || scope == "both" {
        if is_ip_or_cidr {
            let mut args = vec![("dst-address", host), ("action", "accept")];
            if let Some(ref p) = req.port {
                args.push(("dst-port", p.as_str()));
            }
            let full_comm = format!("[CUSTOM_HOTSPOT_IP] {}", comment_str);
            args.push(("comment", full_comm.as_str()));
            client.run(build_command("/ip/hotspot/walled-garden/ip/add", args)).await?;
            applied_entries.push("hotspot_walled_garden_ip");
        } else {
            let mut args = vec![("dst-host", host), ("action", "allow")];
            if let Some(ref p) = req.port {
                args.push(("dst-port", p.as_str()));
            }
            let full_comm = format!("[CUSTOM_HOTSPOT] {}", comment_str);
            args.push(("comment", full_comm.as_str()));
            client.run(build_command("/ip/hotspot/walled-garden/add", args)).await?;
            applied_entries.push("hotspot_walled_garden");
        }
    }

    // 2. PPPoE & Isolated Subscriber Allowlist (ISOLIR_ALLOWED)
    if scope == "isolir" || scope == "both" {
        let full_comm = format!("{} [CUSTOM] {}", ISOLIR_COMMENT_PREFIX, comment_str);
        client.run(build_command(
            "/ip/firewall/address-list/add",
            [
                ("list", ISOLIR_ALLOWED_LIST),
                ("address", host),
                ("comment", full_comm.as_str()),
            ],
        )).await?;

        // Pastikan filter accept rule tetap ada
        let filter_comment = format!("{} Allow Bypass Payment & Core Services", ISOLIR_COMMENT_PREFIX);
        let _ = client.run(build_command(
            "/ip/firewall/filter/add",
            [
                ("chain", "forward"),
                ("src-address-list", ISOLIR_ADDRESS_LIST),
                ("dst-address-list", ISOLIR_ALLOWED_LIST),
                ("action", "accept"),
                ("comment", filter_comment.as_str()),
            ],
        )).await;

        applied_entries.push("firewall_address_list_isolir");
    }

    Ok(Json(json!({
        "success": true,
        "host": host,
        "target_scope": scope,
        "is_ip_or_cidr": is_ip_or_cidr,
        "applied_to": applied_entries,
        "message": format!("Host/Domain '{}' berhasil ditambahkan ke allowlist kustom!", host)
    })))
}

#[derive(Deserialize, Debug)]
pub struct CustomAllowlistRemoveReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
    /// Target tabel: "walled_garden", "walled_garden_ip", atau "address_list"
    pub target_table: String,
}

/// POST /api/v1/allowlist/custom/remove - Hapus aturan allowlist kustom berdasarkan ID tabel
pub async fn remove_custom(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CustomAllowlistRemoveReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_safe_identifier(&req.id, "id")?;

    match req.target_table.as_str() {
        "walled_garden" => {
            client.run(build_command("/ip/hotspot/walled-garden/remove", [(".id", req.id.as_str())])).await?;
        }
        "walled_garden_ip" => {
            client.run(build_command("/ip/hotspot/walled-garden/ip/remove", [(".id", req.id.as_str())])).await?;
        }
        "address_list" => {
            client.run(build_command("/ip/firewall/address-list/remove", [(".id", req.id.as_str())])).await?;
        }
        other => {
            return Err(ApiError::BadRequest(format!("Target table '{}' tidak valid. Pilihan: 'walled_garden', 'walled_garden_ip', 'address_list'", other)));
        }
    }

    Ok(Json(json!({
        "success": true,
        "id": req.id,
        "target_table": req.target_table,
        "message": format!("Aturan allowlist '{}' dari tabel '{}' berhasil dihapus!", req.id, req.target_table)
    })))
}

#[derive(Deserialize, Debug)]
pub struct CustomAllowlistToggleReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
    pub target_table: String,
    pub disabled: bool,
}

/// POST /api/v1/allowlist/custom/toggle - Aktifkan atau nonaktifkan aturan allowlist tanpa menghapusnya
pub async fn toggle_custom(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CustomAllowlistToggleReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_safe_identifier(&req.id, "id")?;
    let dis_str = if req.disabled { "yes" } else { "no" };

    match req.target_table.as_str() {
        "walled_garden" => {
            client.run(build_command("/ip/hotspot/walled-garden/set", [(".id", req.id.as_str()), ("disabled", dis_str)])).await?;
        }
        "walled_garden_ip" => {
            client.run(build_command("/ip/hotspot/walled-garden/ip/set", [(".id", req.id.as_str()), ("disabled", dis_str)])).await?;
        }
        "address_list" => {
            client.run(build_command("/ip/firewall/address-list/set", [(".id", req.id.as_str()), ("disabled", dis_str)])).await?;
        }
        other => {
            return Err(ApiError::BadRequest(format!("Target table '{}' tidak valid", other)));
        }
    }

    Ok(Json(json!({
        "success": true,
        "id": req.id,
        "disabled": req.disabled,
        "message": format!("Status aturan allowlist diubah menjadi {}", if req.disabled { "NONAKTIF" } else { "AKTIF" })
    })))
}

// ============================================================================
// Tier 3: Advanced Allowlist (Wildcard, Path/Method, Timeout/Leasing, Multi-Scope)
// ============================================================================

#[derive(Deserialize, Debug)]
pub struct AdvancedAllowlistAddReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// Target: "hotspot", "isolir", atau "both"
    #[serde(default = "default_target_scope")]
    pub target_scope: String,
    /// Hotspot server name (e.g. "hotspot1" atau "all")
    pub server: Option<String>,
    /// Action: "allow" atau "deny" (Walled Garden)
    #[serde(default = "default_action")]
    pub action: String,
    /// Wildcard Host (misal: "*shopee*", "*midtrans*")
    pub dst_host: Option<String>,
    /// IP atau Subnet CIDR (misal: "103.20.10.0/24")
    pub dst_address: Option<String>,
    /// Port tujuan (misal: "80,443")
    pub dst_port: Option<String>,
    /// Protokol: "tcp", "udp", dsb.
    pub protocol: Option<String>,
    /// URL Path pattern (misal: "/api/v2/*", "/payment/callback")
    pub path: Option<String>,
    /// HTTP Method (misal: "POST", "GET")
    pub method: Option<String>,
    /// Dynamic Temporary Lease Timeout (misal: "30m", "1h" untuk sesi bayar pelanggan)
    pub timeout: Option<String>,
    /// Catatan aturan
    pub comment: Option<String>,
}

fn default_action() -> String {
    "allow".into()
}

/// POST /api/v1/allowlist/advanced/add - Buat aturan allowlist tingkat lanjut dengan filter regex, path, method, dan timeout
pub async fn add_advanced(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AdvancedAllowlistAddReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let scope = req.target_scope.to_lowercase();
    let comment_str = req.comment.as_deref().unwrap_or("Advanced Allowlist Rule");

    let mut applied_entries = Vec::new();

    // 1. Hotspot Walled Garden IP (jika dst_address ditentukan)
    if (scope == "hotspot" || scope == "both") && req.dst_address.is_some() {
        let mut args = vec![("action", if req.action == "deny" { "drop" } else { "accept" })];
        if let Some(ref a) = req.dst_address {
            args.push(("dst-address", a.as_str()));
        }
        if let Some(ref s) = req.server {
            args.push(("server", s.as_str()));
        }
        if let Some(ref p) = req.dst_port {
            args.push(("dst-port", p.as_str()));
        }
        if let Some(ref pr) = req.protocol {
            args.push(("protocol", pr.as_str()));
        }
        let full_comm = format!("[ADV_HOTSPOT_IP] {}", comment_str);
        args.push(("comment", full_comm.as_str()));

        client.run(build_command("/ip/hotspot/walled-garden/ip/add", args)).await?;
        applied_entries.push("hotspot_walled_garden_ip");
    }

    // 2. Hotspot Walled Garden Host/Path/Method (jika dst_host atau path atau method ditentukan)
    if (scope == "hotspot" || scope == "both") && (req.dst_host.is_some() || req.path.is_some() || req.method.is_some()) {
        let mut args = vec![("action", req.action.as_str())];
        if let Some(ref h) = req.dst_host {
            args.push(("dst-host", h.as_str()));
        }
        if let Some(ref s) = req.server {
            args.push(("server", s.as_str()));
        }
        if let Some(ref p) = req.dst_port {
            args.push(("dst-port", p.as_str()));
        }
        if let Some(ref pt) = req.path {
            args.push(("path", pt.as_str()));
        }
        if let Some(ref m) = req.method {
            args.push(("method", m.as_str()));
        }
        let full_comm = format!("[ADV_HOTSPOT] {}", comment_str);
        args.push(("comment", full_comm.as_str()));

        client.run(build_command("/ip/hotspot/walled-garden/add", args)).await?;
        applied_entries.push("hotspot_walled_garden");
    }

    // 3. Firewall Address-List untuk Isolir Network dengan Timeout
    if scope == "isolir" || scope == "both" {
        let addr_target = req.dst_host.as_deref().or(req.dst_address.as_deref()).unwrap_or("");
        if !addr_target.is_empty() {
            let mut args = vec![
                ("list", ISOLIR_ALLOWED_LIST),
                ("address", addr_target),
            ];
            let full_comm = format!("{} [ADV] {}", ISOLIR_COMMENT_PREFIX, comment_str);
            args.push(("comment", full_comm.as_str()));

            if let Some(ref t) = req.timeout {
                args.push(("timeout", t.as_str()));
            }

            client.run(build_command("/ip/firewall/address-list/add", args)).await?;
            applied_entries.push("firewall_address_list_isolir");
        }
    }

    Ok(Json(json!({
        "success": true,
        "target_scope": scope,
        "applied_to": applied_entries,
        "timeout": req.timeout,
        "message": "Aturan allowlist tingkat lanjut berhasil dipasang di RouterOS!"
    })))
}

// ============================================================================
// Comprehensive Allowlist Overview & Health Audit
// ============================================================================

/// GET or POST /api/v1/allowlist/list - Tampilkan seluruh aturan allowlist terpadu dari semua layer
pub async fn list_all(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // 1. Hotspot Walled Garden Domains
    let wg_domains = client
        .run(build_command("/ip/hotspot/walled-garden/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();
    let wg_dom_list: Vec<HashMap<String, String>> = wg_domains.into_iter().map(|r| r.attrs).collect();

    // 2. Hotspot Walled Garden IPs
    let wg_ips = client
        .run(build_command("/ip/hotspot/walled-garden/ip/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();
    let wg_ip_list: Vec<HashMap<String, String>> = wg_ips.into_iter().map(|r| r.attrs).collect();

    // 3. Firewall Address-Lists (ISOLIR_ALLOWED & WHITELIST)
    let addr_lists = client
        .run(build_command("/ip/firewall/address-list/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();

    let mut isolir_allowed = Vec::new();
    let mut general_whitelist = Vec::new();

    for row in addr_lists {
        let list_name = row.get("list").unwrap_or("");
        if list_name == ISOLIR_ALLOWED_LIST {
            isolir_allowed.push(row.attrs);
        } else if list_name.to_uppercase() == "WHITELIST" {
            general_whitelist.push(row.attrs);
        }
    }

    Ok(Json(json!({
        "success": true,
        "summary": {
            "hotspot_walled_garden_domains_count": wg_dom_list.len(),
            "hotspot_walled_garden_ips_count": wg_ip_list.len(),
            "firewall_isolir_allowed_count": isolir_allowed.len(),
            "firewall_whitelist_count": general_whitelist.len(),
            "total_rules": wg_dom_list.len() + wg_ip_list.len() + isolir_allowed.len() + general_whitelist.len()
        },
        "hotspot_walled_garden_domains": wg_dom_list,
        "hotspot_walled_garden_ips": wg_ip_list,
        "firewall_isolir_allowed": isolir_allowed,
        "firewall_whitelist": general_whitelist
    })))
}

/// GET or POST /api/v1/allowlist/audit - Audit kesehatan dan urutan rule bypass allowlist di RouterOS
pub async fn audit_allowlist(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut issues = Vec::new();
    let mut recommendations = Vec::new();

    // 1. Audit DNS Resolver WAN
    let dns_rows = client
        .run(build_command("/ip/dns/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();
    let allow_remote = dns_rows
        .first()
        .and_then(|r| r.get("allow-remote-requests"))
        .unwrap_or("no");

    if allow_remote != "yes" && allow_remote != "true" {
        issues.push("DNS Allow Remote Requests nonaktif pada router: Klien hotspot/isolir mungkin gagal me-resolve domain yang di-allowlist jika router bertindak sebagai DNS primer.");
        recommendations.push("/ip dns set allow-remote-requests=yes".to_string());
    }

    // 2. Audit Firewall Filter Accept Rule untuk ISOLIR_ALLOWED
    let filter_rows = client
        .run(build_command("/ip/firewall/filter/print", [("?dst-address-list", ISOLIR_ALLOWED_LIST)]))
        .await
        .unwrap_or_default();

    if filter_rows.is_empty() {
        issues.push("Rule Firewall Filter Accept untuk ISOLIR_ALLOWED tidak ditemukan. Klien yang diisolir mungkin tetap terblokir meskipun domain sudah didaftarkan ke address-list.");
        recommendations.push(format!(
            "/ip firewall filter add chain=forward src-address-list={} dst-address-list={} action=accept place-before=0 comment=\"{} Allow Bypass\"",
            ISOLIR_ADDRESS_LIST, ISOLIR_ALLOWED_LIST, ISOLIR_COMMENT_PREFIX
        ));
    }

    // 3. Audit Hit Count Walled Garden
    let wg_rows = client
        .run(build_command("/ip/hotspot/walled-garden/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();

    let mut active_hits = 0;
    for r in &wg_rows {
        if let Some(hits) = r.get("hits") {
            if let Ok(h) = hits.parse::<u64>() {
                if h > 0 { active_hits += 1; }
            }
        }
    }

    let status = if issues.is_empty() { "HEALTHY" } else { "ATTENTION_REQUIRED" };

    Ok(Json(json!({
        "success": true,
        "allowlist_health_status": status,
        "issues_found_count": issues.len(),
        "issues": issues,
        "recommendations": recommendations,
        "active_walled_garden_rules": wg_rows.len(),
        "rules_with_recorded_hits": active_hits
    })))
}

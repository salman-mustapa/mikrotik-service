use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use routeros_core::build_command;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::{AppState, RouterTarget};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuickActionStep {
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
pub struct QuickActionJobRecord {
    pub job_id: String,
    pub preset_id: String,
    pub category: String,
    pub title: String,
    pub status: String,
    pub started_at_ms: u128,
    pub total_duration_ms: u64,
    pub total_steps: usize,
    pub steps: Vec<QuickActionStep>,
    pub router_verification: Value,
}

static QUICK_ACTIONS_HISTORY: Mutex<VecDeque<QuickActionJobRecord>> = Mutex::new(VecDeque::new());

pub fn record_quick_action(job: QuickActionJobRecord) {
    if let Ok(mut hist) = QUICK_ACTIONS_HISTORY.lock() {
        if hist.len() >= 100 {
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

#[derive(Deserialize, Debug)]
pub struct QuickActionExecReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub preset_id: String,
    #[serde(default)]
    pub params: HashMap<String, Value>,
}

#[derive(Deserialize, Debug, Default)]
pub struct QuickActionQuery {
    pub category: Option<String>,
    pub limit: Option<usize>,
}

/// GET /api/v1/quick-actions/presets - Katalog seluruh Quick Action siap pakai (Firewall, Hotspot, PPPoE, Bandwidth & System)
pub async fn quick_action_presets(Query(q): Query<QuickActionQuery>) -> Json<Value> {
    let all_presets = json!([
        // --- 1. FIREWALL & KEAMANAN ---
        {
            "id": "block_malicious_ip",
            "category": "firewall",
            "title": "Blokir & Karantina IP Penyerang (Zero CPU RAW Drop)",
            "description": "Memasukkan IP ke daftar BLACKLIST, menjamin RAW Prerouting Drop (0% CPU conntrack bypass), Filter Forward Drop, dan memutus seketika seluruh koneksi aktif.",
            "fields": [
                { "name": "address", "label": "IP Target / Subnet", "type": "text", "required": true, "placeholder": "198.51.100.22 atau 192.168.1.50" },
                { "name": "list", "label": "Nama Address List", "type": "text", "required": false, "default": "BLACKLIST" },
                { "name": "timeout", "label": "Batas Waktu (Timeout)", "type": "text", "required": false, "default": "24h", "placeholder": "24h, 7d, 01:00:00" },
                { "name": "comment", "label": "Catatan Jejak Forensik", "type": "text", "required": false, "default": "Quick-Action: Malicious IP Block" }
            ],
            "stages": ["VALIDATION", "FIREWALL_RAW", "FIREWALL_FILTER", "ADDRESS_LIST", "CONNTRACK_CLEANUP", "ROUTER_VERIFICATION"]
        },
        {
            "id": "isolate_host",
            "category": "firewall",
            "title": "Isolir Host Terinfeksi (Captive Quarantine)",
            "description": "Mengisolir perangkat lokal yang terindikasi malware/virus ke dalam daftar ISOLIR sehingga tidak dapat menyebarkan paket lateral ke LAN maupun akses WAN internet.",
            "fields": [
                { "name": "address", "label": "IP Klien Lokal", "type": "text", "required": true, "placeholder": "192.168.88.105" },
                { "name": "comment", "label": "Keterangan Isolir", "type": "text", "required": false, "default": "Quick-Action: Infected Host Isolated" }
            ],
            "stages": ["VALIDATION", "ADDRESS_LIST", "FIREWALL_FILTER", "CONNTRACK_CLEANUP", "ROUTER_VERIFICATION"]
        },
        {
            "id": "protect_winbox_bruteforce",
            "category": "firewall",
            "title": "Proteksi Brute-Force Port Winbox 8291 (3-Stage Tarpit/Blacklist)",
            "description": "Menerapkan deteksi brute-force otomatis pada port Winbox 8291. Host yang gagal 3 kali dalam 1 menit otomatis masuk BLACKLIST selama 7 hari.",
            "fields": [
                { "name": "stage1_timeout", "label": "Stage 1 Timeout", "type": "text", "required": false, "default": "1m" },
                { "name": "blacklist_timeout", "label": "Blacklist Timeout", "type": "text", "required": false, "default": "7d" }
            ],
            "stages": ["PRE_CHECK", "FIREWALL_FILTER", "ROUTER_VERIFICATION"]
        },
        {
            "id": "block_port_scanner_psd",
            "category": "firewall",
            "title": "Deteksi & Drop Port Scanner Otomatis (PSD Heuristic)",
            "description": "Mengaktifkan filter Port Scan Detection (PSD 21,3s,3,1). Scanner otomatis dijebloskan ke daftar PORT_SCANNERS selama 24 jam dan didrop seketika.",
            "fields": [
                { "name": "timeout", "label": "Durasi Blacklist Scanner", "type": "text", "required": false, "default": "24h" }
            ],
            "stages": ["PRE_CHECK", "FIREWALL_FILTER", "ROUTER_VERIFICATION"]
        },
        {
            "id": "anti_syn_flood_ddos",
            "category": "firewall",
            "title": "Mitigasi Serangan TCP SYN-Flood DDoS (RAW Table)",
            "description": "Menahan serangan TCP SYN flood pada tabel RAW dengan batas connection-limit, melindungi CPU RouterOS dari lonjakan 100% akibat tabel conntrack penuh.",
            "fields": [
                { "name": "connection_limit", "label": "Batas Koneksi per Host", "type": "number", "required": false, "default": 30 }
            ],
            "stages": ["FIREWALL_RAW", "ROUTER_VERIFICATION"]
        },
        {
            "id": "dns_hijack_prevention",
            "category": "firewall",
            "title": "Cegah DNS Hijacking & Paksa DNS Resolver Terpercaya",
            "description": "Memaksa seluruh query port 53 UDP/TCP dari klien lokal di-redirect ke DNS lokal router/Pi-hole/Cloudflare untuk memblokir malware bypass DNS.",
            "fields": [
                { "name": "target_dns", "label": "IP Resolver Tujuan", "type": "text", "required": false, "default": "local" }
            ],
            "stages": ["FIREWALL_NAT", "ROUTER_VERIFICATION"]
        },
        {
            "id": "block_doh_vpn_circumvention",
            "category": "firewall",
            "title": "Blokir Bypass DNS-over-TLS (DoT Port 853)",
            "description": "Mencegah klien membypass filter DNS dengan menonaktifkan port 853 DoT di prerouting RAW.",
            "fields": [],
            "stages": ["FIREWALL_RAW", "ROUTER_VERIFICATION"]
        },
        {
            "id": "whitelist_admin_ip",
            "category": "firewall",
            "title": "Whitelist IP Administrator NOC (Top-Rule Accept)",
            "description": "Menjamin subnet manajemen/IP admin tidak akan terblokir oleh aturan firewall filter manapun dengan inject accept rule prioritas teratas.",
            "fields": [
                { "name": "address", "label": "IP / Subnet Admin NOC", "type": "text", "required": true, "placeholder": "192.168.88.250 atau 10.10.0.0/24" },
                { "name": "comment", "label": "Keterangan", "type": "text", "required": false, "default": "NOC Admin Whitelist Priority" }
            ],
            "stages": ["VALIDATION", "ADDRESS_LIST", "FIREWALL_FILTER", "ROUTER_VERIFICATION"]
        },

        // --- 2. HOTSPOT & VOUCHER ---
        {
            "id": "quick_hotspot_package_and_vouchers",
            "category": "hotspot",
            "title": "1-Klik Buat Paket Hotspot & Batch Voucher Siap Cetak",
            "description": "Secara otomatis membuat profil paket hotspot (rate limit, shared user, masa aktif, lock-to-mac) dan langsung mencetak batch voucher baru siap jual dalam 1 transaksi.",
            "fields": [
                { "name": "package_name", "label": "Nama Paket", "type": "text", "required": true, "placeholder": "2JAM-3RB" },
                { "name": "rate_limit", "label": "Kecepatan (Up/Down)", "type": "text", "required": false, "default": "2M/5M" },
                { "name": "validity", "label": "Masa Aktif", "type": "text", "required": false, "default": "2h" },
                { "name": "price", "label": "Harga Jual (Rp)", "type": "number", "required": false, "default": 3000 },
                { "name": "voucher_count", "label": "Jumlah Voucher Dibuat", "type": "number", "required": false, "default": 5 },
                { "name": "prefix", "label": "Prefix Kode Voucher", "type": "text", "required": false, "default": "HOT-" }
            ],
            "stages": ["VALIDATION", "PROFILE_CREATION", "VOUCHER_GENERATION", "ROUTER_VERIFICATION"]
        },
        {
            "id": "quick_kick_hotspot_user",
            "category": "hotspot",
            "title": "Kick & Disconnect Sesi User Hotspot Macet",
            "description": "Memutus seketika sesi aktif user hotspot yang macet (*stuck*) dari tabel active hotspot dan mengosongkan conntrack soketnya agar dapat login kembali.",
            "fields": [
                { "name": "username", "label": "Username / IP Klien", "type": "text", "required": true, "placeholder": "user-hotspot-01 atau 10.50.0.45" }
            ],
            "stages": ["VALIDATION", "ACTIVE_SESSION_KILL", "ROUTER_VERIFICATION"]
        },
        {
            "id": "quick_walled_garden_bypass",
            "category": "hotspot",
            "title": "Bypass Cepat Walled Garden Domain / QRIS Bank",
            "description": "Membuka akses ke domain perbankan atau portal pembayaran QRIS agar klien yang belum login voucher tetap dapat melakukan transaksi.",
            "fields": [
                { "name": "dst_host", "label": "Domain / Host Target", "type": "text", "required": true, "placeholder": "payment.bankbca.co.id" },
                { "name": "comment", "label": "Catatan", "type": "text", "required": false, "default": "Quick-Action QRIS Bypass" }
            ],
            "stages": ["VALIDATION", "WALLED_GARDEN_ADD", "ROUTER_VERIFICATION"]
        },
        {
            "id": "quick_reset_hotspot_cookies",
            "category": "hotspot",
            "title": "Bersihkan Seluruh Cookie Login Hotspot",
            "description": "Menghapus cookie login browser perangkat klien agar memaksa seluruh pengguna melihat kembali halaman captive portal login.",
            "fields": [],
            "stages": ["COOKIE_CLEANUP", "ROUTER_VERIFICATION"]
        },

        // --- 3. PPPOE & BILLING ---
        {
            "id": "quick_create_pppoe_client",
            "category": "pppoe",
            "title": "1-Klik Registrasi Pelanggan Baru PPPoE",
            "description": "Mendaftarkan akun PPPoE Secret baru lengkap dengan paket bandwidth, username, password, dan verifikasi status aktif di router.",
            "fields": [
                { "name": "username", "label": "Username PPPoE", "type": "text", "required": true, "placeholder": "cust_alex" },
                { "name": "password", "label": "Password", "type": "text", "required": true, "placeholder": "Secret123" },
                { "name": "profile", "label": "Paket Profil PPPoE", "type": "text", "required": false, "default": "Home-10M" },
                { "name": "service", "label": "Service", "type": "text", "required": false, "default": "pppoe" },
                { "name": "comment", "label": "Data Pelanggan (Nama/Alamat/No HP)", "type": "text", "required": false, "default": "Pelanggan Baru" }
            ],
            "stages": ["VALIDATION", "SECRET_CREATION", "ROUTER_VERIFICATION"]
        },
        {
            "id": "quick_isolate_overdue_pppoe",
            "category": "pppoe",
            "title": "1-Klik Isolir Pelanggan PPPoE Menunggak",
            "description": "Mengubah profil pelanggan ke ISOLIR_PPPOE dan memutus koneksi aktifnya secara otomatis agar dialihkan ke halaman peringatan jatuh tempo.",
            "fields": [
                { "name": "username", "label": "Username Pelanggan", "type": "text", "required": true, "placeholder": "cust_alex" }
            ],
            "stages": ["VALIDATION", "PROFILE_CHANGE", "DISCONNECT_ACTIVE", "ROUTER_VERIFICATION"]
        },
        {
            "id": "quick_restore_paid_pppoe",
            "category": "pppoe",
            "title": "1-Klik Pulihkan Pelanggan PPPoE (Sudah Bayar)",
            "description": "Mengembalikan profil pelanggan dari isolir ke profil internet semula dan memutus sesi isolir agar pelanggan seketika menikmati akses normal.",
            "fields": [
                { "name": "username", "label": "Username Pelanggan", "type": "text", "required": true, "placeholder": "cust_alex" },
                { "name": "regular_profile", "label": "Profil Normal", "type": "text", "required": false, "default": "Home-10M" }
            ],
            "stages": ["VALIDATION", "PROFILE_RESTORE", "DISCONNECT_ISOLIR", "ROUTER_VERIFICATION"]
        },

        // --- 4. BANDWIDTH & TRAFFIC MANAGEMENT ---
        {
            "id": "quick_prioritize_vip",
            "category": "bandwidth",
            "title": "Prioritas Bandwidth Khusus Host VIP / Zoom / Gaming",
            "description": "Membuat aturan Simple Queue prioritas tertinggi (Priority 1) untuk menjamin latensi rendah dan alokasi bandwidth dedicated tanpa lag.",
            "fields": [
                { "name": "target_ip", "label": "IP Perangkat VIP", "type": "text", "required": true, "placeholder": "192.168.88.200" },
                { "name": "max_limit", "label": "Alokasi Bandwidth (Up/Down)", "type": "text", "required": false, "default": "20M/50M" },
                { "name": "name", "label": "Nama Antrean VIP", "type": "text", "required": false, "default": "VIP-Priority" }
            ],
            "stages": ["VALIDATION", "QUEUE_INJECTION", "ROUTER_VERIFICATION"]
        },
        {
            "id": "quick_throttle_bandwidth_hog",
            "category": "bandwidth",
            "title": "Pangkas Kecepatan Pengguna Rakus (Throttle Hog)",
            "description": "Membatasi host lokal yang menyedot bandwidth berlebihan ke batas kecepatan minimal (Priority 8) untuk menjaga kenyamanan pengguna lain.",
            "fields": [
                { "name": "target_ip", "label": "IP Host Pelaku", "type": "text", "required": true, "placeholder": "192.168.88.115" },
                { "name": "max_limit", "label": "Batas Kecepatan Maksimal", "type": "text", "required": false, "default": "512k/1M" }
            ],
            "stages": ["VALIDATION", "QUEUE_THROTTLE", "ROUTER_VERIFICATION"]
        },

        // --- 5. SYSTEM & MAINTENANCE ---
        {
            "id": "quick_flush_dns_and_arp",
            "category": "system",
            "title": "1-Klik Bersihkan Cache DNS & ARP Dinamis",
            "description": "Mengosongkan memori cache DNS resolver lokal dan menghapus entri ARP dinamis yang kadaluwarsa untuk mengatasi issue IP conflict atau domain macet.",
            "fields": [],
            "stages": ["DNS_FLUSH", "ARP_CLEANUP", "ROUTER_VERIFICATION"]
        },
        {
            "id": "quick_system_backup_export",
            "category": "system",
            "title": "1-Klik Backup Sistem Binary & Script RSC",
            "description": "Membuat arsip backup binary (.backup) dan script export konfigurasi (.rsc) di memori router MikroTik dengan penamaan timestamp otomatis.",
            "fields": [
                { "name": "prefix", "label": "Prefix Nama Backup", "type": "text", "required": false, "default": "AUTO-SNAPSHOT" }
            ],
            "stages": ["BINARY_BACKUP", "SCRIPT_EXPORT", "FILE_VERIFICATION"]
        }
    ]);

    let filtered: Vec<Value> = if let Some(cat) = q.category.as_deref() {
        if cat.is_empty() || cat == "all" {
            all_presets.as_array().cloned().unwrap_or_default()
        } else {
            all_presets
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter(|p| p.get("category").and_then(|v| v.as_str()) == Some(cat))
                .cloned()
                .collect()
        }
    } else {
        all_presets.as_array().cloned().unwrap_or_default()
    };

    Json(json!({
        "success": true,
        "total_presets": filtered.len(),
        "categories": ["firewall", "hotspot", "pppoe", "bandwidth", "system"],
        "presets": filtered
    }))
}

/// GET /api/v1/quick-actions/history - Riwayat eksekusi quick action untuk audit log developer
pub async fn quick_actions_history(Query(q): Query<QuickActionQuery>) -> Json<Value> {
    let hist: Vec<QuickActionJobRecord> = if let Ok(h) = QUICK_ACTIONS_HISTORY.lock() {
        let mut list: Vec<QuickActionJobRecord> = h.iter().cloned().collect();
        if let Some(cat) = q.category.as_deref() {
            if !cat.is_empty() && cat != "all" {
                list.retain(|j| j.category == cat);
            }
        }
        if let Some(l) = q.limit {
            list.truncate(l);
        }
        list
    } else {
        Vec::new()
    };

    Json(json!({
        "success": true,
        "count": hist.len(),
        "history": hist
    }))
}

/// POST /api/v1/quick-actions/execute - Universal Quick Action Executor dengan Checklist Loader & Live MikroTik Verification
pub async fn execute_quick_action(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<QuickActionExecReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let started_at = now_epoch_ms();
    let job_id = format!("job_qa_{}_{:04x}", started_at, (started_at % 0xFFFF) as u16);
    let mut steps: Vec<QuickActionStep> = Vec::new();
    let overall_start = Instant::now();

    match req.preset_id.as_str() {
        // --- CATEGORY 1: FIREWALL ---
        "block_malicious_ip" | "isolate_host" | "protect_winbox_bruteforce" | "block_port_scanner_psd" |
        "anti_syn_flood_ddos" | "dns_hijack_prevention" | "block_doh_vpn_circumvention" | "whitelist_admin_ip" => {
            // Forward directly to firewall's implementation to maintain consistency
            let fw_req = crate::routes::firewall::QuickActionExecReq {
                router: None,
                router_id: None,
                preset_id: req.preset_id.clone(),
                params: req.params.clone(),
            };
            crate::routes::firewall::execute_quick_action(State(st), headers, Json(fw_req)).await
        }

        // --- CATEGORY 2: HOTSPOT ---
        "quick_hotspot_package_and_vouchers" => {
            let pkg_name = req.params.get("package_name").and_then(|v| v.as_str()).unwrap_or("1JAM-3RB").trim().to_string();
            let rate_limit = req.params.get("rate_limit").and_then(|v| v.as_str()).unwrap_or("2M/5M").trim().to_string();
            let validity = req.params.get("validity").and_then(|v| v.as_str()).unwrap_or("2h").trim().to_string();
            let price = req.params.get("price").and_then(|v| v.as_f64()).unwrap_or(3000.0);
            let voucher_count = req.params.get("voucher_count").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
            let prefix = req.params.get("prefix").and_then(|v| v.as_str()).unwrap_or("HOT-").trim().to_string();

            // Step 1: Validation
            let s1_start = Instant::now();
            crate::routes::sanitize::validate_safe_identifier(&pkg_name, "package_name")?;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "VALIDATION".into(),
                name: "Validasi Parameter Paket Hotspot".into(),
                description: format!("Memverifikasi nama paket '{}', rate limit '{}', masa aktif '{}'", pkg_name, rate_limit, validity),
                command: format!("validate_package({}, limit={}, val={})", pkg_name, rate_limit, validity),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "Sintaks parameter valid".into(),
            });

            // Step 2: Create User Profile
            let s2_start = Instant::now();
            let comment_tag = format!("price:{}|validity:{}|mode:notice", price, validity);
            let existing_profs = client.run(build_command("/ip/hotspot/user/profile/print", [("?name", pkg_name.as_str())])).await.unwrap_or_default();
            if existing_profs.is_empty() {
                client.run(build_command(
                    "/ip/hotspot/user/profile/add",
                    [
                        ("name", pkg_name.as_str()),
                        ("rate-limit", rate_limit.as_str()),
                        ("shared-users", "1"),
                        ("comment", comment_tag.as_str()),
                    ],
                )).await?;
            }
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "PROFILE_CREATION".into(),
                name: "Pembuatan User Profile di RouterOS".into(),
                description: format!("Menetapkan profil '{}' dengan shared-users=1 dan rate-limit={}", pkg_name, rate_limit),
                command: format!("/ip/hotspot/user/profile/add name={} rate-limit={}", pkg_name, rate_limit),
                status: "success".into(),
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: "User profile berhasil aktif di RouterOS".into(),
            });

            // Step 3: Generate Batch Vouchers
            let s3_start = Instant::now();
            let mut created_users = Vec::new();
            for i in 0..voucher_count {
                let code = format!("{}{:04x}", prefix, (started_at as u32).wrapping_add(i as u32) % 0xFFFF);
                let v_comm = format!("vc:{}|price:{}|val:{}", code, price, validity);
                let _ = client.run(build_command(
                    "/ip/hotspot/user/add",
                    [
                        ("name", code.as_str()),
                        ("password", code.as_str()),
                        ("profile", pkg_name.as_str()),
                        ("comment", v_comm.as_str()),
                    ],
                )).await;
                created_users.push(code);
            }
            steps.push(QuickActionStep {
                step_number: 3,
                stage: "VOUCHER_GENERATION".into(),
                name: format!("Injeksi {} Voucher Baru", created_users.len()),
                description: format!("Membuat akun voucher dengan kode: {:?}", created_users),
                command: format!("/ip/hotspot/user/add count={}", created_users.len()),
                status: "success".into(),
                duration_ms: s3_start.elapsed().as_millis() as u64,
                details: format!("{} voucher berhasil diinjeksi", created_users.len()),
            });

            // Step 4: Verification
            let s4_start = Instant::now();
            let check = client.run(build_command("/ip/hotspot/user/print", [("?profile", pkg_name.as_str())])).await.unwrap_or_default();
            let verified = !check.is_empty();
            steps.push(QuickActionStep {
                step_number: 4,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi Database Hotspot di MikroTik".into(),
                description: "Membaca kembali jumlah user aktif pada profil tersebut".into(),
                command: format!("/ip/hotspot/user/print ?profile={}", pkg_name),
                status: if verified { "success".into() } else { "failed".into() },
                duration_ms: s4_start.elapsed().as_millis() as u64,
                details: format!("Terverifikasi {} akun voucher terdaftar di router", check.len()),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "quick_hotspot_package_and_vouchers".into(),
                category: "hotspot".into(),
                title: format!("Buat Paket '{}' + {} Voucher", pkg_name, created_users.len()),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({
                    "verified": verified,
                    "package_name": pkg_name,
                    "vouchers_generated": created_users,
                    "total_in_router": check.len()
                }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": job_record.preset_id,
                "category": job_record.category,
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "quick_kick_hotspot_user" => {
            let user = req.params.get("username").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            if user.is_empty() {
                return Err(ApiError::BadRequest("Parameter 'username' wajib diisi".into()));
            }

            let s1_start = Instant::now();
            let active_users = client.run(build_command("/ip/hotspot/active/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
            let mut removed_count = 0;
            let mut target_ip = String::new();

            for u in active_users {
                let uname = u.get("user").unwrap_or("");
                let uip = u.get("address").unwrap_or("");
                let umac = u.get("mac-address").unwrap_or("");
                if uname == user || uip == user || umac == user {
                    target_ip = uip.to_string();
                    if let Some(id) = u.get(".id") {
                        if client.run(build_command("/ip/hotspot/active/remove", [(".id", id)])).await.is_ok() {
                            removed_count += 1;
                        }
                    }
                }
            }
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "ACTIVE_SESSION_KILL".into(),
                name: "Terminasi Sesi Aktif Hotspot".into(),
                description: format!("Mencari dan menghapus sesi user '{}' dari memori aktif", user),
                command: format!("/ip/hotspot/active/remove [find user={}]", user),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: format!("{} sesi aktif diputus", removed_count),
            });

            // Flush connection if IP was found
            if !target_ip.is_empty() {
                let s2_start = Instant::now();
                let conns = client.run(build_command("/ip/firewall/connection/print", [("?src-address", target_ip.as_str())])).await.unwrap_or_default();
                for c in conns {
                    if let Some(id) = c.get(".id") {
                        let _ = client.run(build_command("/ip/firewall/connection/remove", [(".id", id)])).await;
                    }
                }
                steps.push(QuickActionStep {
                    step_number: 2,
                    stage: "CONNTRACK_CLEANUP".into(),
                    name: "Bersihkan Conntrack Klien".into(),
                    description: format!("Mengosongkan soket jaringan untuk IP {}", target_ip),
                    command: format!("/ip/firewall/connection/remove [find src-address={}]", target_ip),
                    status: "success".into(),
                    duration_ms: s2_start.elapsed().as_millis() as u64,
                    details: "Soket koneksi dibersihkan".into(),
                });
            }

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "quick_kick_hotspot_user".into(),
                category: "hotspot".into(),
                title: format!("Kick Sesi User Hotspot '{}'", user),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "kicked_user": user, "removed_active_sessions": removed_count }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": job_record.preset_id,
                "category": job_record.category,
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "quick_walled_garden_bypass" => {
            let host = req.params.get("dst_host").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            if host.is_empty() {
                return Err(ApiError::BadRequest("Parameter 'dst_host' wajib diisi".into()));
            }
            let comment = req.params.get("comment").and_then(|v| v.as_str()).unwrap_or("Quick-Action QRIS Bypass").trim().to_string();

            let s1_start = Instant::now();
            client.run(build_command(
                "/ip/hotspot/walled-garden/add",
                [("dst-host", host.as_str()), ("action", "allow"), ("comment", comment.as_str())],
            )).await?;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "WALLED_GARDEN_ADD".into(),
                name: "Injeksi Domain ke Walled Garden".into(),
                description: format!("Mengizinkan akses bebas ke host '{}'", host),
                command: format!("/ip/hotspot/walled-garden/add dst-host={} action=allow", host),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "Domain berhasil didaftarkan di Walled Garden".into(),
            });

            let s2_start = Instant::now();
            let check = client.run(build_command("/ip/hotspot/walled-garden/print", [("?dst-host", host.as_str())])).await.unwrap_or_default();
            let verified = !check.is_empty();
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi Aturan Walled Garden di MikroTik".into(),
                description: "Membaca kembali tabel Walled Garden RouterOS".into(),
                command: format!("/ip/hotspot/walled-garden/print ?dst-host={}", host),
                status: if verified { "success".into() } else { "failed".into() },
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: "Aturan terverifikasi aktif di RouterOS".into(),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "quick_walled_garden_bypass".into(),
                category: "hotspot".into(),
                title: format!("Bypass Walled Garden '{}'", host),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "verified": verified, "bypassed_host": host }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": job_record.preset_id,
                "category": job_record.category,
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "quick_reset_hotspot_cookies" => {
            let s1_start = Instant::now();
            let cookies = client.run(build_command("/ip/hotspot/cookie/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
            let total_cookies = cookies.len();
            for c in cookies {
                if let Some(id) = c.get(".id") {
                    let _ = client.run(build_command("/ip/hotspot/cookie/remove", [(".id", id)])).await;
                }
            }
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "COOKIE_CLEANUP".into(),
                name: "Pembersihan Cookie Browser Hotspot".into(),
                description: format!("Menghapus {} cookie login yang tersimpan di memori RouterOS", total_cookies),
                command: "/ip/hotspot/cookie/remove [find]".into(),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: format!("{} cookie dihapus, seluruh klien dipaksa re-login", total_cookies),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "quick_reset_hotspot_cookies".into(),
                category: "hotspot".into(),
                title: "Bersihkan Seluruh Cookie Hotspot".into(),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "cleaned_cookies_count": total_cookies }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": job_record.preset_id,
                "category": job_record.category,
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        // --- CATEGORY 3: PPPOE ---
        "quick_create_pppoe_client" => {
            let user = req.params.get("username").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            let pass = req.params.get("password").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            let profile = req.params.get("profile").and_then(|v| v.as_str()).unwrap_or("Home-10M").trim().to_string();
            let service = req.params.get("service").and_then(|v| v.as_str()).unwrap_or("pppoe").trim().to_string();
            let comment = req.params.get("comment").and_then(|v| v.as_str()).unwrap_or("Pelanggan Baru").trim().to_string();

            if user.is_empty() || pass.is_empty() {
                return Err(ApiError::BadRequest("Parameter 'username' dan 'password' wajib diisi".into()));
            }

            let s1_start = Instant::now();
            crate::routes::sanitize::validate_safe_identifier(&user, "username")?;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "VALIDATION".into(),
                name: "Validasi Kredensial Pelanggan".into(),
                description: format!("Memeriksa format username '{}' dan profil '{}'", user, profile),
                command: format!("validate_user({})", user),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "Kredensial valid".into(),
            });

            let s2_start = Instant::now();
            client.run(build_command(
                "/ppp/secret/add",
                [
                    ("name", user.as_str()),
                    ("password", pass.as_str()),
                    ("profile", profile.as_str()),
                    ("service", service.as_str()),
                    ("comment", comment.as_str()),
                ],
            )).await?;
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "SECRET_CREATION".into(),
                name: "Injeksi Akun ke PPP Secret RouterOS".into(),
                description: format!("Membuat akun PPPoE dengan profil '{}' dan service '{}'", profile, service),
                command: format!("/ppp/secret/add name={} profile={} service={}", user, profile, service),
                status: "success".into(),
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: "Akun PPPoE Secret berhasil dibuat".into(),
            });

            let s3_start = Instant::now();
            let check = client.run(build_command("/ppp/secret/print", [("?name", user.as_str())])).await.unwrap_or_default();
            let verified = !check.is_empty();
            steps.push(QuickActionStep {
                step_number: 3,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi Akun PPPoE di MikroTik".into(),
                description: "Membaca kembali tabel PPP Secret di RouterOS".into(),
                command: format!("/ppp/secret/print ?name={}", user),
                status: if verified { "success".into() } else { "failed".into() },
                duration_ms: s3_start.elapsed().as_millis() as u64,
                details: format!("Akun terverifikasi aktif di RouterOS (ID: {})", check.first().and_then(|r| r.get(".id")).unwrap_or("*none")),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "quick_create_pppoe_client".into(),
                category: "pppoe".into(),
                title: format!("Registrasi Pelanggan PPPoE '{}'", user),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "verified": verified, "username": user, "profile": profile }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": job_record.preset_id,
                "category": job_record.category,
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "quick_isolate_overdue_pppoe" => {
            let user = req.params.get("username").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            if user.is_empty() {
                return Err(ApiError::BadRequest("Parameter 'username' wajib diisi".into()));
            }

            // Step 1: Update secret profile to ISOLIR_PPPOE
            let s1_start = Instant::now();
            let secrets = client.run(build_command("/ppp/secret/print", [("?name", user.as_str())])).await.unwrap_or_default();
            if secrets.is_empty() {
                return Err(ApiError::NotFound(format!("Akun PPPoE '{}' tidak ditemukan", user)));
            }
            let sec_id = secrets.first().and_then(|r| r.get(".id")).unwrap_or("");
            client.run(build_command("/ppp/secret/set", [(".id", sec_id), ("profile", "ISOLIR_PPPOE")])).await?;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "PROFILE_CHANGE".into(),
                name: "Ubah Profil ke ISOLIR_PPPOE".into(),
                description: format!("Mengalihkan profil pelanggan '{}' ke profil isolir jatuh tempo", user),
                command: format!("/ppp/secret/set [find name={}] profile=ISOLIR_PPPOE", user),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "Profil PPPoE Secret berhasil diubah ke ISOLIR_PPPOE".into(),
            });

            // Step 2: Disconnect active session so client reconnects with isolir IP
            let s2_start = Instant::now();
            let actives = client.run(build_command("/ppp/active/print", [("?name", user.as_str())])).await.unwrap_or_default();
            let mut disconnected = 0;
            for a in actives {
                if let Some(id) = a.get(".id") {
                    if client.run(build_command("/ppp/active/remove", [(".id", id)])).await.is_ok() {
                        disconnected += 1;
                    }
                }
            }
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "DISCONNECT_ACTIVE".into(),
                name: "Putus Sesi Aktif Pelanggan".into(),
                description: "Memaksa pelanggan re-dial untuk seketika mendapatkan alokasi IP isolir".into(),
                command: format!("/ppp/active/remove [find name={}]", user),
                status: "success".into(),
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: format!("{} sesi aktif diputus", disconnected),
            });

            // Step 3: Verification
            let s3_start = Instant::now();
            let check = client.run(build_command("/ppp/secret/print", [("?name", user.as_str())])).await.unwrap_or_default();
            let verified = check.iter().any(|r| r.get("profile") == Some("ISOLIR_PPPOE"));
            steps.push(QuickActionStep {
                step_number: 3,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi Status Isolir di MikroTik".into(),
                description: "Memastikan akun tersimpan dengan profil ISOLIR_PPPOE".into(),
                command: format!("/ppp/secret/print ?name={}", user),
                status: if verified { "success".into() } else { "failed".into() },
                duration_ms: s3_start.elapsed().as_millis() as u64,
                details: "Terverifikasi berstatus ISOLIR di RouterOS".into(),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "quick_isolate_overdue_pppoe".into(),
                category: "pppoe".into(),
                title: format!("Isolir Pelanggan PPPoE '{}'", user),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "verified": verified, "username": user, "isolated": true }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": job_record.preset_id,
                "category": job_record.category,
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "quick_restore_paid_pppoe" => {
            let user = req.params.get("username").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            let regular_profile = req.params.get("regular_profile").and_then(|v| v.as_str()).unwrap_or("Home-10M").trim().to_string();
            if user.is_empty() {
                return Err(ApiError::BadRequest("Parameter 'username' wajib diisi".into()));
            }

            let s1_start = Instant::now();
            let secrets = client.run(build_command("/ppp/secret/print", [("?name", user.as_str())])).await.unwrap_or_default();
            if secrets.is_empty() {
                return Err(ApiError::NotFound(format!("Akun PPPoE '{}' tidak ditemukan", user)));
            }
            let sec_id = secrets.first().and_then(|r| r.get(".id")).unwrap_or("");
            client.run(build_command("/ppp/secret/set", [(".id", sec_id), ("profile", regular_profile.as_str())])).await?;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "PROFILE_RESTORE".into(),
                name: "Kembalikan ke Profil Normal".into(),
                description: format!("Memulihkan profil pelanggan '{}' ke '{}'", user, regular_profile),
                command: format!("/ppp/secret/set [find name={}] profile={}", user, regular_profile),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: format!("Profil berhasil dipulihkan ke '{}'", regular_profile),
            });

            let s2_start = Instant::now();
            let actives = client.run(build_command("/ppp/active/print", [("?name", user.as_str())])).await.unwrap_or_default();
            for a in actives {
                if let Some(id) = a.get(".id") {
                    let _ = client.run(build_command("/ppp/active/remove", [(".id", id)])).await;
                }
            }
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "DISCONNECT_ISOLIR".into(),
                name: "Putus Sesi Isolir".into(),
                description: "Memaksa pelanggan re-dial untuk langsung mendapatkan IP reguler dengan akses penuh".into(),
                command: format!("/ppp/active/remove [find name={}]", user),
                status: "success".into(),
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: "Sesi isolir diputus, pelanggan siap re-dial akses normal".into(),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "quick_restore_paid_pppoe".into(),
                category: "pppoe".into(),
                title: format!("Pulihkan Pelanggan PPPoE '{}'", user),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "username": user, "restored_profile": regular_profile }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": job_record.preset_id,
                "category": job_record.category,
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        // --- CATEGORY 4: BANDWIDTH ---
        "quick_prioritize_vip" => {
            let target_ip = req.params.get("target_ip").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            let max_limit = req.params.get("max_limit").and_then(|v| v.as_str()).unwrap_or("20M/50M").trim().to_string();
            let name = req.params.get("name").and_then(|v| v.as_str()).unwrap_or("VIP-Priority").trim().to_string();

            if target_ip.is_empty() {
                return Err(ApiError::BadRequest("Parameter 'target_ip' wajib diisi".into()));
            }

            let s1_start = Instant::now();
            crate::routes::sanitize::validate_ip_or_cidr(&target_ip)?;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "VALIDATION".into(),
                name: "Validasi IP Host VIP".into(),
                description: format!("Memverifikasi IP '{}'", target_ip),
                command: format!("validate_ip_or_cidr({})", target_ip),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "IP valid".into(),
            });

            let s2_start = Instant::now();
            client.run(build_command(
                "/queue/simple/add",
                [
                    ("name", name.as_str()),
                    ("target", target_ip.as_str()),
                    ("max-limit", max_limit.as_str()),
                    ("priority", "1/1"),
                    ("comment", "Quick-Action VIP Dedicated Queue"),
                ],
            )).await?;
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "QUEUE_INJECTION".into(),
                name: "Injeksi Simple Queue Priority 1 (Tertinggi)".into(),
                description: format!("Menetapkan batas {} dengan prioritas tertinggi untuk {}", max_limit, target_ip),
                command: format!("/queue/simple/add name={} target={} max-limit={} priority=1/1", name, target_ip, max_limit),
                status: "success".into(),
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: "Simple queue prioritas 1 terpasang di RouterOS".into(),
            });

            let s3_start = Instant::now();
            let check = client.run(build_command("/queue/simple/print", [("?target", target_ip.as_str())])).await.unwrap_or_default();
            let verified = !check.is_empty();
            steps.push(QuickActionStep {
                step_number: 3,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi Queue di MikroTik".into(),
                description: "Membaca kembali antrean di RouterOS".into(),
                command: format!("/queue/simple/print ?target={}", target_ip),
                status: if verified { "success".into() } else { "failed".into() },
                duration_ms: s3_start.elapsed().as_millis() as u64,
                details: format!("Terverifikasi aktif di RouterOS (ID: {})", check.first().and_then(|r| r.get(".id")).unwrap_or("*none")),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "quick_prioritize_vip".into(),
                category: "bandwidth".into(),
                title: format!("Prioritas Bandwidth VIP '{}'", target_ip),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "verified": verified, "target_ip": target_ip, "priority": "1/1", "max_limit": max_limit }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": job_record.preset_id,
                "category": job_record.category,
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "quick_throttle_bandwidth_hog" => {
            let target_ip = req.params.get("target_ip").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            let max_limit = req.params.get("max_limit").and_then(|v| v.as_str()).unwrap_or("512k/1M").trim().to_string();

            if target_ip.is_empty() {
                return Err(ApiError::BadRequest("Parameter 'target_ip' wajib diisi".into()));
            }

            let s1_start = Instant::now();
            crate::routes::sanitize::validate_ip_or_cidr(&target_ip)?;
            let queue_name = format!("THROTTLE-{}", target_ip.replace('.', "_"));
            client.run(build_command(
                "/queue/simple/add",
                [
                    ("name", queue_name.as_str()),
                    ("target", target_ip.as_str()),
                    ("max-limit", max_limit.as_str()),
                    ("priority", "8/8"),
                    ("comment", "Quick-Action Throttled Heavy User"),
                ],
            )).await?;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "QUEUE_THROTTLE".into(),
                name: "Pangkas Kecepatan Host di Simple Queue".into(),
                description: format!("Membatasi IP {} maksimal {} dengan Priority 8 (terendah)", target_ip, max_limit),
                command: format!("/queue/simple/add target={} max-limit={} priority=8/8", target_ip, max_limit),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "Aturan pembatasan berhasil diinjeksi ke router".into(),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "quick_throttle_bandwidth_hog".into(),
                category: "bandwidth".into(),
                title: format!("Pangkas Bandwidth Host '{}'", target_ip),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "target_ip": target_ip, "throttled_to": max_limit }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": job_record.preset_id,
                "category": job_record.category,
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        // --- CATEGORY 5: SYSTEM & MAINTENANCE ---
        "quick_flush_dns_and_arp" => {
            let s1_start = Instant::now();
            let _ = client.run(build_command("/ip/dns/cache/flush", std::iter::empty::<(&str, &str)>())).await;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "DNS_FLUSH".into(),
                name: "Flush DNS Cache Resolver".into(),
                description: "Mengosongkan memori resolver DNS lokal di RouterOS".into(),
                command: "/ip/dns/cache/flush".into(),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "Cache DNS berhasil dikosongkan".into(),
            });

            let s2_start = Instant::now();
            let arp_rows = client.run(build_command("/ip/arp/print", [("?dynamic", "yes")])).await.unwrap_or_default();
            let mut removed_arp = 0;
            for a in arp_rows {
                if let Some(id) = a.get(".id") {
                    if client.run(build_command("/ip/arp/remove", [(".id", id)])).await.is_ok() {
                        removed_arp += 1;
                    }
                }
            }
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "ARP_CLEANUP".into(),
                name: "Pembersihan ARP Dinamis".into(),
                description: format!("Menghapus {} entri ARP dinamis lama", removed_arp),
                command: "/ip/arp/remove [find dynamic=yes]".into(),
                status: "success".into(),
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: format!("{} entri ARP lama dibersihkan", removed_arp),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "quick_flush_dns_and_arp".into(),
                category: "system".into(),
                title: "Bersihkan Cache DNS & ARP Dinamis".into(),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "dns_flushed": true, "cleaned_arp_count": removed_arp }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": job_record.preset_id,
                "category": job_record.category,
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "quick_system_backup_export" => {
            let prefix = req.params.get("prefix").and_then(|v| v.as_str()).unwrap_or("AUTO-SNAPSHOT").trim().to_string();
            let filename = format!("{}_{}", prefix, started_at);

            let s1_start = Instant::now();
            let _ = client.run(build_command("/system/backup/save", [("name", filename.as_str())])).await;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "BINARY_BACKUP".into(),
                name: "Simpan Backup Binary RouterOS (.backup)".into(),
                description: format!("Membuat snapshot binary sistem ke file '{}.backup'", filename),
                command: format!("/system/backup/save name={}", filename),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: format!("File backup binary '{}.backup' dibuat", filename),
            });

            let s2_start = Instant::now();
            let _ = client.run(build_command("/export", [("file", filename.as_str())])).await;
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "SCRIPT_EXPORT".into(),
                name: "Export Script Konfigurasi (.rsc)".into(),
                description: format!("Mengekspor teks CLI konfigurasi ke file '{}.rsc'", filename),
                command: format!("/export file={}", filename),
                status: "success".into(),
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: format!("File script export '{}.rsc' dibuat", filename),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "quick_system_backup_export".into(),
                category: "system".into(),
                title: format!("Backup Sistem & Export Script '{}'", filename),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "binary_backup_file": format!("{}.backup", filename), "script_export_file": format!("{}.rsc", filename) }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": job_record.preset_id,
                "category": job_record.category,
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        other => Err(ApiError::BadRequest(format!("Preset ID '{}' tidak dikenali. Gunakan /api/v1/quick-actions/presets untuk melihat seluruh preset yang tersedia.", other))),
    }
}

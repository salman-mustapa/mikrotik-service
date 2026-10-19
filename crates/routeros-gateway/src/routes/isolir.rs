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

pub const ISOLIR_ADDRESS_LIST: &str = "ISOLIR_LIST";
pub const ISOLIR_COMMENT_PREFIX: &str = "[WISP-ISOLIR]";

#[derive(Deserialize, Debug)]
pub struct SetupIsolirInfraReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// Alamat IP Server Halaman Peringatan / Portal Tagihan Isolir (misal: "192.168.88.254" atau "10.10.10.2")
    pub isolir_server_ip: String,
    /// Port web server isolir (default: 80)
    #[serde(default = "default_isolir_port")]
    pub isolir_server_port: u16,
}

fn default_isolir_port() -> u16 {
    80
}

#[derive(Deserialize, Debug)]
pub struct IsolateUserReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// IP address pelanggan statik atau PPPoE yang menunggak (misal: "192.168.100.50")
    pub client_ip: String,
    /// Nama pelanggan atau ID tagihan
    pub customer_name: Option<String>,
    /// Catatan jatuh tempo (misal: "Jatuh Tempo: 10 Oktober 2026")
    pub due_date: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct RestoreUserReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub client_ip: String,
}

/// POST /api/v1/wisp/isolir/setup - Mengonfigurasi infrastruktur NAT Redirect dan Firewall Filter Isolir
pub async fn setup_isolir_infra(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetupIsolirInfraReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_ip_or_cidr(&req.isolir_server_ip)?;
    if req.isolir_server_port == 0 {
        return Err(ApiError::BadRequest("isolir_server_port must be between 1 and 65535".into()));
    }

    let to_port_str = req.isolir_server_port.to_string();

    // 1. Tambahkan / Update NAT Redirect: Port 80 diarahkan ke Web Portal Isolir
    let nat_comment = format!("{} Redirect HTTP Port 80 to Billing Portal", ISOLIR_COMMENT_PREFIX);
    let _ = client.run(build_command(
        "/ip/firewall/nat/add",
        [
            ("chain", "dstnat"),
            ("protocol", "tcp"),
            ("dst-port", "80"),
            ("src-address-list", ISOLIR_ADDRESS_LIST),
            ("action", "dst-nat"),
            ("to-addresses", req.isolir_server_ip.as_str()),
            ("to-ports", to_port_str.as_str()),
            ("comment", nat_comment.as_str()),
        ],
    )).await;

    // 2. Filter 1: Allow akses ke Server Web Portal Isolir itu sendiri
    let filter_portal_comment = format!("{} Allow Access to Billing Server", ISOLIR_COMMENT_PREFIX);
    let _ = client.run(build_command(
        "/ip/firewall/filter/add",
        [
            ("chain", "forward"),
            ("src-address-list", ISOLIR_ADDRESS_LIST),
            ("dst-address", req.isolir_server_ip.as_str()),
            ("action", "accept"),
            ("comment", filter_portal_comment.as_str()),
        ],
    )).await;

    // 3. Filter 2: Allow DNS UDP & TCP (port 53) agar domain portal web tetap bisa di-resolve
    let filter_dns_udp = format!("{} Allow DNS UDP for Isolated Clients", ISOLIR_COMMENT_PREFIX);
    let _ = client.run(build_command(
        "/ip/firewall/filter/add",
        [
            ("chain", "forward"),
            ("src-address-list", ISOLIR_ADDRESS_LIST),
            ("protocol", "udp"),
            ("dst-port", "53"),
            ("action", "accept"),
            ("comment", filter_dns_udp.as_str()),
        ],
    )).await;

    let filter_dns_tcp = format!("{} Allow DNS TCP for Isolated Clients", ISOLIR_COMMENT_PREFIX);
    let _ = client.run(build_command(
        "/ip/firewall/filter/add",
        [
            ("chain", "forward"),
            ("src-address-list", ISOLIR_ADDRESS_LIST),
            ("protocol", "tcp"),
            ("dst-port", "53"),
            ("action", "accept"),
            ("comment", filter_dns_tcp.as_str()),
        ],
    )).await;

    // 4. Filter 3: Drop semua lalu lintas internet lainnya (termasuk HTTPS 443 & Game) untuk pelanggan isolir
    let filter_drop_comment = format!("{} Drop All Other Outgoing Traffic", ISOLIR_COMMENT_PREFIX);
    let _ = client.run(build_command(
        "/ip/firewall/filter/add",
        [
            ("chain", "forward"),
            ("src-address-list", ISOLIR_ADDRESS_LIST),
            ("action", "drop"),
            ("comment", filter_drop_comment.as_str()),
        ],
    )).await;

    Ok(Json(json!({
        "success": true,
        "message": "Infrastruktur Firewall Isolir Web Redirect berhasil dikonfigurasi!",
        "isolir_server": {
            "ip": req.isolir_server_ip,
            "port": req.isolir_server_port,
        },
        "address_list": ISOLIR_ADDRESS_LIST,
        "mechanism": "Klien di address-list ISOLIR_LIST akan dialihkan ke server web portal tagihan saat membuka browser HTTP, dan koneksi lainnya diblokir."
    })))
}

/// POST /api/v1/wisp/isolir/isolate - Mengisolir pelanggan yang menunggak tagihan
pub async fn isolate_user(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IsolateUserReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_ip_or_cidr(&req.client_ip)?;
    if let Some(ref name) = req.customer_name {
        if name.contains(['\r', '\n', '"', ';']) {
            return Err(ApiError::BadRequest("customer_name contains invalid characters".into()));
        }
    }
    if let Some(ref due) = req.due_date {
        if due.contains(['\r', '\n', '"', ';']) {
            return Err(ApiError::BadRequest("due_date contains invalid characters".into()));
        }
    }

    let name = req.customer_name.as_deref().unwrap_or("Pelanggan Menunggak");
    let due = req.due_date.as_deref().unwrap_or("-");
    let comment = format!("{} Customer: {} | Jatuh Tempo: {}", ISOLIR_COMMENT_PREFIX, name, due);

    // Periksa apakah IP sudah terdaftar di list
    let existing = client.run(build_command(
        "/ip/firewall/address-list/print",
        [
            ("?list", ISOLIR_ADDRESS_LIST),
            ("?address", req.client_ip.as_str()),
        ],
    )).await.unwrap_or_default();

    if let Some(row) = existing.first() {
        if let Some(id) = row.get(".id") {
            let _ = client.run(build_command(
                "/ip/firewall/address-list/set",
                [
                    (".id", id),
                    ("comment", comment.as_str()),
                    ("disabled", "no"),
                ],
            )).await;
        }
    } else {
        client.run(build_command(
            "/ip/firewall/address-list/add",
            [
                ("list", ISOLIR_ADDRESS_LIST),
                ("address", req.client_ip.as_str()),
                ("comment", comment.as_str()),
            ],
        )).await?;
    }

    Ok(Json(json!({
        "success": true,
        "client_ip": req.client_ip,
        "status": "ISOLATED",
        "customer": name,
        "due_date": due,
        "message": format!("Pelanggan '{}' ({}) berhasil dimasukkan ke daftar isolir!", name, req.client_ip)
    })))
}

/// POST /api/v1/wisp/isolir/restore - Memulihkan akses internet pelanggan setelah melunasi tagihan
pub async fn restore_user(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RestoreUserReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_ip_or_cidr(&req.client_ip)?;

    let rows = client.run(build_command(
        "/ip/firewall/address-list/print",
        [
            ("?list", ISOLIR_ADDRESS_LIST),
            ("?address", req.client_ip.as_str()),
        ],
    )).await.unwrap_or_default();

    let mut removed = false;
    for r in rows {
        if let Some(id) = r.get(".id") {
            let _ = client.run(build_command("/ip/firewall/address-list/remove", [(".id", id)])).await;
            removed = true;
        }
    }

    Ok(Json(json!({
        "success": true,
        "client_ip": req.client_ip,
        "status": "RESTORED",
        "removed_from_list": removed,
        "message": format!("Akses internet untuk klien {} berhasil dipulihkan sepenuhnya!", req.client_ip)
    })))
}

/// GET or POST /api/v1/wisp/isolir/list - Menampilkan daftar seluruh pelanggan yang sedang diisolir
pub async fn list_isolated(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command(
        "/ip/firewall/address-list/print",
        [("?list", ISOLIR_ADDRESS_LIST)],
    )).await?;

    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "total_isolated": data.len(),
        "address_list": ISOLIR_ADDRESS_LIST,
        "clients": data
    })))
}

/// POST /api/v1/wisp/isolir/teardown - Membersihkan seluruh aturan firewall isolir
pub async fn teardown_isolir(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut removed_count = 0;

    // 1. Bersihkan NAT
    if let Ok(nats) = client.run(build_command("/ip/firewall/nat/print", std::iter::empty::<(&str, &str)>())).await {
        for n in nats {
            if n.get("comment").map(|c| c.contains(ISOLIR_COMMENT_PREFIX)).unwrap_or(false) {
                if let Some(id) = n.get(".id") {
                    let _ = client.run(build_command("/ip/firewall/nat/remove", [(".id", id)])).await;
                    removed_count += 1;
                }
            }
        }
    }

    // 2. Bersihkan Filter
    if let Ok(filters) = client.run(build_command("/ip/firewall/filter/print", std::iter::empty::<(&str, &str)>())).await {
        for f in filters {
            if f.get("comment").map(|c| c.contains(ISOLIR_COMMENT_PREFIX)).unwrap_or(false) {
                if let Some(id) = f.get(".id") {
                    let _ = client.run(build_command("/ip/firewall/filter/remove", [(".id", id)])).await;
                    removed_count += 1;
                }
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("{} aturan firewall isolir berhasil dibersihkan", removed_count),
        "rules_removed": removed_count
    })))
}

// ============================================================================
// PPPoE Subscriber Isolation & Billing Suspension
// ============================================================================

#[derive(Deserialize, Debug)]
pub struct IsolatePppoeReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// Username akun PPPoE (/ppp/secret)
    pub username: String,
    /// Profile isolir tujuan (default: "ISOLIR_PPPOE")
    pub isolir_profile: Option<String>,
    /// Nama pelanggan
    pub customer_name: Option<String>,
    /// Catatan jatuh tempo tagihan (misal: "Jatuh Tempo: 15 Oktober 2026")
    pub due_date: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct RestorePppoeReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub username: String,
    /// Profile target pemulihan (jika None, sistem akan membaca profil awal dari tag [PREV_PROF:...])
    pub target_profile: Option<String>,
}

/// POST /api/v1/wisp/isolir/pppoe/isolate - Mengisolir pelanggan PPPoE yang belum membayar tagihan
pub async fn isolate_pppoe_user(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IsolatePppoeReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_safe_identifier(&req.username, "username")?;

    // 1. Cari data user di /ppp/secret
    let secrets = client
        .run(build_command("/ppp/secret/print", [("?name", req.username.as_str())]))
        .await?;

    let secret_row = secrets.into_iter().next().ok_or_else(|| {
        ApiError::BadRequest(format!("User PPPoE '{}' tidak ditemukan di /ppp/secret", req.username))
    })?;

    let id = secret_row.get(".id").ok_or_else(|| ApiError::BadRequest("ID secret tidak valid".into()))?;
    let current_profile = secret_row.get("profile").unwrap_or("default");
    let current_comment = secret_row.get("comment").unwrap_or("");

    let isolir_prof = req.isolir_profile.as_deref().unwrap_or("ISOLIR_PPPOE");
    let cust_name = req.customer_name.as_deref().unwrap_or("Pelanggan PPPoE");
    let due = req.due_date.as_deref().unwrap_or("-");

    // Simpan profil lama ke dalam comment jika belum pernah diisolir
    let prev_prof_tag = if !current_comment.contains("PREV_PROF:") {
        format!("[PREV_PROF:{}]", current_profile)
    } else {
        // Ambil tag yang sudah ada
        current_comment.split_whitespace()
            .find(|part| part.starts_with("[PREV_PROF:"))
            .unwrap_or("[PREV_PROF:default]")
            .to_string()
    };

    let new_comment = format!("{} {} Customer: {} | Jatuh Tempo: {}", prev_prof_tag, ISOLIR_COMMENT_PREFIX, cust_name, due);

    // 2. Update /ppp/secret ke profil isolir
    client.run(build_command(
        "/ppp/secret/set",
        [
            (".id", id),
            ("profile", isolir_prof),
            ("comment", new_comment.as_str()),
        ],
    )).await?;

    // 3. Kick sesi aktif pelanggan di /ppp/active agar reconnect masuk ke profil isolir
    let active_rows = client
        .run(build_command("/ppp/active/print", [("?name", req.username.as_str())]))
        .await
        .unwrap_or_default();

    let mut kicked_ip = None;
    for act in active_rows {
        if let Some(act_id) = act.get(".id") {
            let _ = client.run(build_command("/ppp/active/remove", [(".id", act_id)])).await;
        }
        if let Some(ip) = act.get("address") {
            kicked_ip = Some(ip.to_string());
            // Tambahkan IP ke ISOLIR_ADDRESS_LIST untuk redirect instan
            let _ = client.run(build_command(
                "/ip/firewall/address-list/add",
                [
                    ("list", ISOLIR_ADDRESS_LIST),
                    ("address", ip),
                    ("comment", new_comment.as_str()),
                ],
            )).await;
        }
    }

    Ok(Json(json!({
        "success": true,
        "username": req.username,
        "status": "ISOLATED",
        "previous_profile": current_profile,
        "assigned_profile": isolir_prof,
        "kicked_active_session": kicked_ip.is_some(),
        "client_ip": kicked_ip,
        "message": format!("Pelanggan PPPoE '{}' berhasil diisolir ke profil '{}'!", req.username, isolir_prof)
    })))
}

/// POST /api/v1/wisp/isolir/pppoe/restore - Memulihkan akses internet pelanggan PPPoE setelah bayar tagihan
pub async fn restore_pppoe_user(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RestorePppoeReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_safe_identifier(&req.username, "username")?;

    let secrets = client
        .run(build_command("/ppp/secret/print", [("?name", req.username.as_str())]))
        .await?;

    let secret_row = secrets.into_iter().next().ok_or_else(|| {
        ApiError::BadRequest(format!("User PPPoE '{}' tidak ditemukan di /ppp/secret", req.username))
    })?;

    let id = secret_row.get(".id").ok_or_else(|| ApiError::BadRequest("ID secret tidak valid".into()))?;
    let comment = secret_row.get("comment").unwrap_or("");

    // Ambil profil asli dari tag [PREV_PROF:nama_profile] jika tidak ditentukan secara eksplisit
    let target_prof = if let Some(ref p) = req.target_profile {
        p.clone()
    } else if let Some(start) = comment.find("[PREV_PROF:") {
        let remainder = &comment[start + 11..];
        if let Some(end) = remainder.find(']') {
            remainder[..end].to_string()
        } else {
            "default".into()
        }
    } else {
        "default".into()
    };

    crate::routes::sanitize::validate_safe_identifier(&target_prof, "target_profile")?;

    // Bersihkan comment dari tag isolir
    let cleaned_comment = comment
        .split_whitespace()
        .filter(|part| !part.starts_with("[PREV_PROF:") && !part.starts_with(ISOLIR_COMMENT_PREFIX))
        .collect::<Vec<_>>()
        .join(" ");

    // 1. Revert profil di /ppp/secret
    client.run(build_command(
        "/ppp/secret/set",
        [
            (".id", id),
            ("profile", target_prof.as_str()),
            ("comment", cleaned_comment.as_str()),
        ],
    )).await?;

    // 2. Kick sesi aktif agar reconnect langsung menikmati profil langganan semula
    let active_rows = client
        .run(build_command("/ppp/active/print", [("?name", req.username.as_str())]))
        .await
        .unwrap_or_default();

    for act in active_rows {
        if let Some(act_id) = act.get(".id") {
            let _ = client.run(build_command("/ppp/active/remove", [(".id", act_id)])).await;
        }
        if let Some(ip) = act.get("address") {
            // Hapus dari ISOLIR_ADDRESS_LIST
            let list_rows = client
                .run(build_command(
                    "/ip/firewall/address-list/print",
                    [("?list", ISOLIR_ADDRESS_LIST), ("?address", ip)],
                ))
                .await
                .unwrap_or_default();
            for lr in list_rows {
                if let Some(lid) = lr.get(".id") {
                    let _ = client.run(build_command("/ip/firewall/address-list/remove", [(".id", lid)])).await;
                }
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "username": req.username,
        "status": "RESTORED",
        "restored_profile": target_prof,
        "message": format!("Akses pelanggan PPPoE '{}' berhasil dipulihkan ke paket '{}'!", req.username, target_prof)
    })))
}

/// GET or POST /api/v1/wisp/isolir/pppoe/list - Daftar pelanggan PPPoE yang sedang dalam masa isolir tagihan
pub async fn list_pppoe_isolated(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let secrets = client
        .run(build_command("/ppp/secret/print", std::iter::empty::<(&str, &str)>()))
        .await?;

    let mut isolated_list = Vec::new();
    for s in secrets {
        let comment = s.get("comment").unwrap_or("");
        let profile = s.get("profile").unwrap_or("");

        if comment.contains(ISOLIR_COMMENT_PREFIX) || profile.to_lowercase().contains("isolir") {
            isolated_list.push(s.attrs);
        }
    }

    Ok(Json(json!({
        "success": true,
        "total_isolated_pppoe": isolated_list.len(),
        "subscribers": isolated_list
    })))
}

// ============================================================================
// Payment & Support Allowlist (Walled Garden Bypass untuk Pelanggan Isolir)
// ============================================================================

#[derive(Deserialize, Debug)]
pub struct AllowlistHostReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// Domain atau Subnet yang diizinkan (misal: "api.midtrans.com", "qris.id", "whatsapp.com")
    pub host: String,
    pub comment: Option<String>,
}

/// POST /api/v1/wisp/isolir/allowlist/add - Tambah host/domain pembayaran ke daftar bypass isolir
pub async fn add_isolir_allowlist(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AllowlistHostReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    if req.host.contains(['\r', '\n', '"', ';']) {
        return Err(ApiError::BadRequest("Host contains invalid characters".into()));
    }

    let comment_str = req.comment.unwrap_or_else(|| "Bypass Portal Pembayaran Isolir".into());
    let full_comment = format!("{} {}", ISOLIR_COMMENT_PREFIX, comment_str);

    // Tambahkan domain ke address-list ISOLIR_ALLOWED
    client.run(build_command(
        "/ip/firewall/address-list/add",
        [
            ("list", "ISOLIR_ALLOWED"),
            ("address", req.host.as_str()),
            ("comment", full_comment.as_str()),
        ],
    )).await?;

    // Pastikan firewall filter membolehkan akses ke ISOLIR_ALLOWED sebelum drop rule
    let filter_comment = format!("{} Allow Bypass Payment Destinations", ISOLIR_COMMENT_PREFIX);
    let _ = client.run(build_command(
        "/ip/firewall/filter/add",
        [
            ("chain", "forward"),
            ("src-address-list", ISOLIR_ADDRESS_LIST),
            ("dst-address-list", "ISOLIR_ALLOWED"),
            ("action", "accept"),
            ("comment", filter_comment.as_str()),
        ],
    )).await;

    Ok(Json(json!({
        "success": true,
        "host": req.host,
        "address_list": "ISOLIR_ALLOWED",
        "message": format!("Host/Domain '{}' berhasil dimasukkan ke allowlist bypass isolir!", req.host)
    })))
}

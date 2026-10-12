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

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
pub struct SetupPortKnockingReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// Port rahasia ketukan 1 (default: 7000)
    #[serde(default = "default_knock1")]
    pub knock_port_1: u16,
    /// Port rahasia ketukan 2 (default: 8000)
    #[serde(default = "default_knock2")]
    pub knock_port_2: u16,
    /// Port rahasia ketukan 3 (default: 9000)
    #[serde(default = "default_knock3")]
    pub knock_port_3: u16,
    /// Port layanan yang diamankan (default: "8291,22" untuk Winbox & SSH)
    #[serde(default = "default_target_ports")]
    pub target_ports: String,
    /// Durasi akses terbuka setelah knock berhasil (default: "1h", misal "30m", "2h", "1d")
    #[serde(default = "default_access_duration")]
    pub access_duration: String,
    /// Interface WAN masuk (opsional, jika kosong berlaku untuk semua interface)
    pub in_interface: Option<String>,
}

fn default_knock1() -> u16 { 7000 }
fn default_knock2() -> u16 { 8000 }
fn default_knock3() -> u16 { 9000 }
fn default_target_ports() -> String { "8291,22".into() }
fn default_access_duration() -> String { "1h".into() }

const KNOCK_COMMENT_TAG: &str = "[Port-Knocking-3Step]";
const LIST_STAGE1: &str = "KNOCK_STAGE_1";
const LIST_STAGE2: &str = "KNOCK_STAGE_2";
const LIST_SECURE: &str = "KNOCK_SECURE_ALLOWED";

/// POST /api/v1/security/port-knocking/setup - Mengonfigurasi 3-Step Port Knocking Wizard (MTCSE Security)
pub async fn setup_port_knocking(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetupPortKnockingReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let p1_str = req.knock_port_1.to_string();
    let p2_str = req.knock_port_2.to_string();
    let p3_str = req.knock_port_3.to_string();
    let iface = req.in_interface.as_deref();

    // 1. Bersihkan aturan lama
    let existing = client
        .run(build_command("/ip/firewall/filter/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();

    for r in existing {
        if let Some(c) = r.get("comment") {
            if c.contains(KNOCK_COMMENT_TAG) {
                if let Some(id) = r.get(".id") {
                    let _ = client.run(build_command("/ip/firewall/filter/remove", [(".id", id)])).await;
                }
            }
        }
    }

    // 2. Rule: ACCEPT target ports (Winbox/SSH) jika ada di list KNOCK_SECURE_ALLOWED
    let mut allow_args = vec![
        ("chain", "input"),
        ("protocol", "tcp"),
        ("dst-port", req.target_ports.as_str()),
        ("src-address-list", LIST_SECURE),
        ("action", "accept"),
        ("comment", "[Port-Knocking-3Step] Accept Allowed Admin Connections"),
    ];
    if let Some(i) = iface {
        allow_args.push(("in-interface", i));
    }
    client.run(build_command("/ip/firewall/filter/add", allow_args)).await?;

    // 3. Rule: Knock 3 -> Masukkan ke list KNOCK_SECURE_ALLOWED jika IP ada di KNOCK_STAGE_2
    let comment_k3 = format!("{} Knock 3 (Port {}) -> Grant Access {}", KNOCK_COMMENT_TAG, p3_str, req.access_duration);
    let mut k3_args = vec![
        ("chain", "input"),
        ("protocol", "tcp"),
        ("dst-port", p3_str.as_str()),
        ("src-address-list", LIST_STAGE2),
        ("action", "add-src-to-address-list"),
        ("address-list", LIST_SECURE),
        ("address-list-timeout", req.access_duration.as_str()),
        ("comment", comment_k3.as_str()),
    ];
    if let Some(i) = iface {
        k3_args.push(("in-interface", i));
    }
    client.run(build_command("/ip/firewall/filter/add", k3_args)).await?;

    // 4. Rule: Knock 2 -> Masukkan ke list KNOCK_STAGE_2 jika IP ada di KNOCK_STAGE_1 (timeout 1m)
    let comment_k2 = format!("{} Knock 2 (Port {}) -> Move to Stage 2", KNOCK_COMMENT_TAG, p2_str);
    let mut k2_args = vec![
        ("chain", "input"),
        ("protocol", "tcp"),
        ("dst-port", p2_str.as_str()),
        ("src-address-list", LIST_STAGE1),
        ("action", "add-src-to-address-list"),
        ("address-list", LIST_STAGE2),
        ("address-list-timeout", "1m"),
        ("comment", comment_k2.as_str()),
    ];
    if let Some(i) = iface {
        k2_args.push(("in-interface", i));
    }
    client.run(build_command("/ip/firewall/filter/add", k2_args)).await?;

    // 5. Rule: Knock 1 -> Masukkan ke list KNOCK_STAGE_1 (timeout 1m)
    let comment_k1 = format!("{} Knock 1 (Port {}) -> Start Stage 1", KNOCK_COMMENT_TAG, p1_str);
    let mut k1_args = vec![
        ("chain", "input"),
        ("protocol", "tcp"),
        ("dst-port", p1_str.as_str()),
        ("action", "add-src-to-address-list"),
        ("address-list", LIST_STAGE1),
        ("address-list-timeout", "1m"),
        ("comment", comment_k1.as_str()),
    ];
    if let Some(i) = iface {
        k1_args.push(("in-interface", i));
    }
    client.run(build_command("/ip/firewall/filter/add", k1_args)).await?;

    // 6. Rule: DROP semua koneksi baru lainnya ke target ports (Winbox/SSH)
    let mut drop_args = vec![
        ("chain", "input"),
        ("protocol", "tcp"),
        ("dst-port", req.target_ports.as_str()),
        ("connection-state", "new"),
        ("action", "drop"),
        ("comment", "[Port-Knocking-3Step] Drop Unauthorized Access to Target Ports"),
    ];
    if let Some(i) = iface {
        drop_args.push(("in-interface", i));
    }
    client.run(build_command("/ip/firewall/filter/add", drop_args)).await?;

    Ok(Json(json!({
        "success": true,
        "message": "3-Step Port Knocking Firewall Protection berhasil diaktifkan!",
        "knock_sequence": [
            { "step": 1, "port": req.knock_port_1, "proto": "TCP", "timeout": "1m" },
            { "step": 2, "port": req.knock_port_2, "proto": "TCP", "timeout": "1m" },
            { "step": 3, "port": req.knock_port_3, "proto": "TCP", "granted_duration": req.access_duration }
        ],
        "protected_ports": req.target_ports,
        "how_to_connect": format!(
            "Sebelum membuka Winbox/SSH, kirim paket TCP ke port {}, lalu port {}, lalu port {}. Port {} akan terbuka otomatis selama {}.",
            req.knock_port_1, req.knock_port_2, req.knock_port_3, req.target_ports, req.access_duration
        )
    })))
}

/// GET or POST /api/v1/security/port-knocking/status - Menampilkan status aturan dan IP admin yang sedang aktif
pub async fn port_knocking_status(
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

    let mut knock_rules = Vec::new();
    for r in filter_rows {
        if let Some(c) = r.get("comment") {
            if c.contains(KNOCK_COMMENT_TAG) {
                knock_rules.push(r.attrs);
            }
        }
    }

    let allowed_ips = client
        .run(build_command("/ip/firewall/address-list/print", [("?list", LIST_SECURE)]))
        .await
        .unwrap_or_default();

    let active_admins: Vec<HashMap<String, String>> = allowed_ips.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "enabled": !knock_rules.is_empty(),
        "total_rules": knock_rules.len(),
        "active_authorized_admins": active_admins,
        "rules": knock_rules
    })))
}

/// POST /api/v1/security/port-knocking/remove - Menghapus seluruh aturan port knocking
pub async fn remove_port_knocking(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let filter_rows = client
        .run(build_command("/ip/firewall/filter/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();

    let mut removed = 0;
    for r in filter_rows {
        if let Some(c) = r.get("comment") {
            if c.contains(KNOCK_COMMENT_TAG) {
                if let Some(id) = r.get(".id") {
                    let _ = client.run(build_command("/ip/firewall/filter/remove", [(".id", id)])).await;
                    removed += 1;
                }
            }
        }
    }

    // Bersihkan address lists
    let _ = client.run(build_command("/ip/firewall/address-list/remove", [("?list", LIST_STAGE1)])).await;
    let _ = client.run(build_command("/ip/firewall/address-list/remove", [("?list", LIST_STAGE2)])).await;
    let _ = client.run(build_command("/ip/firewall/address-list/remove", [("?list", LIST_SECURE)])).await;

    Ok(Json(json!({
        "success": true,
        "message": format!("{} aturan port knocking berhasil dihapus", removed),
        "removed_count": removed
    })))
}

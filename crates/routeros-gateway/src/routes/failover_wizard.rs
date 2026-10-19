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
pub struct SetupRecursiveFailoverReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// Gateway IP ISP 1 Utama (misal: "192.168.1.1" atau interface "ether1")
    pub primary_gateway: String,
    /// Gateway IP ISP 2 Cadangan (misal: "192.168.2.1" atau interface "ether2")
    pub backup_gateway: String,
    /// Host check-gateway untuk ISP 1 (default: "8.8.8.8" Google DNS)
    #[serde(default = "default_check_primary")]
    pub primary_check_host: String,
    /// Host check-gateway untuk ISP 2 (default: "1.1.1.1" Cloudflare DNS)
    #[serde(default = "default_check_backup")]
    pub backup_check_host: String,
    /// Jarak rute utama (default: 1)
    #[serde(default = "default_distance_primary")]
    pub primary_distance: u32,
    /// Jarak rute cadangan (default: 2)
    #[serde(default = "default_distance_backup")]
    pub backup_distance: u32,
    /// Routing mark atau routing table (RouterOS v7) opsional
    pub routing_table: Option<String>,
}

fn default_check_primary() -> String {
    "8.8.8.8".into()
}

fn default_check_backup() -> String {
    "1.1.1.1".into()
}

fn default_distance_primary() -> u32 {
    1
}

fn default_distance_backup() -> u32 {
    2
}

const FAILOVER_COMMENT_TAG: &str = "[Recursive-Failover]";

/// POST /api/v1/routing/failover/setup - Mengonfigurasi Recursive Gateway Multi-WAN Failover (MTCRE Standard)
pub async fn setup_recursive_failover(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetupRecursiveFailoverReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    if req.primary_distance == 0 || req.primary_distance > 255 || req.backup_distance == 0 || req.backup_distance > 255 {
        return Err(ApiError::BadRequest("Route distance must be between 1 and 255".into()));
    }
    if crate::routes::sanitize::validate_ip_or_cidr(&req.primary_gateway).is_err() {
        crate::routes::sanitize::validate_safe_identifier(&req.primary_gateway, "primary_gateway")?;
    }
    if crate::routes::sanitize::validate_ip_or_cidr(&req.backup_gateway).is_err() {
        crate::routes::sanitize::validate_safe_identifier(&req.backup_gateway, "backup_gateway")?;
    }
    crate::routes::sanitize::validate_ip_or_cidr(&req.primary_check_host)?;
    crate::routes::sanitize::validate_ip_or_cidr(&req.backup_check_host)?;
    if let Some(tbl) = req.routing_table.as_deref() {
        crate::routes::sanitize::validate_safe_identifier(tbl, "routing_table")?;
    }

    // 1. Bersihkan rute recursive failover lama dengan tag ini jika ada
    let existing_routes = client
        .run(build_command("/ip/route/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();

    for r in existing_routes {
        if let Some(c) = r.get("comment") {
            if c.contains(FAILOVER_COMMENT_TAG) {
                if let Some(id) = r.get(".id") {
                    let _ = client.run(build_command("/ip/route/remove", [(".id", id)])).await;
                }
            }
        }
    }

    // 2. Rute Virtual Host 1: IP Check Host 1 diarahkan ke Primary Gateway fisik dengan scope=10
    let comment_host1 = format!("{} Host Monitoring ISP1 via {}", FAILOVER_COMMENT_TAG, req.primary_gateway);
    let mut host1_args = vec![
        ("dst-address", req.primary_check_host.as_str()),
        ("gateway", req.primary_gateway.as_str()),
        ("scope", "10"),
        ("comment", comment_host1.as_str()),
    ];
    if let Some(tbl) = req.routing_table.as_deref() {
        host1_args.push(("routing-table", tbl));
    }
    client.run(build_command("/ip/route/add", host1_args)).await?;

    // 3. Rute Virtual Host 2: IP Check Host 2 diarahkan ke Backup Gateway fisik dengan scope=10
    let comment_host2 = format!("{} Host Monitoring ISP2 via {}", FAILOVER_COMMENT_TAG, req.backup_gateway);
    let mut host2_args = vec![
        ("dst-address", req.backup_check_host.as_str()),
        ("gateway", req.backup_gateway.as_str()),
        ("scope", "10"),
        ("comment", comment_host2.as_str()),
    ];
    if let Some(tbl) = req.routing_table.as_deref() {
        host2_args.push(("routing-table", tbl));
    }
    client.run(build_command("/ip/route/add", host2_args)).await?;

    // 4. Rute Default ISP 1: Rekursif ke Check Host 1 dengan target-scope=11, distance=1, check-gateway=ping
    let comment_def1 = format!("{} Primary Default Route (Active jika {} respon)", FAILOVER_COMMENT_TAG, req.primary_check_host);
    let dist1_str = req.primary_distance.to_string();
    let mut def1_args = vec![
        ("dst-address", "0.0.0.0/0"),
        ("gateway", req.primary_check_host.as_str()),
        ("check-gateway", "ping"),
        ("distance", dist1_str.as_str()),
        ("target-scope", "11"),
        ("comment", comment_def1.as_str()),
    ];
    if let Some(tbl) = req.routing_table.as_deref() {
        def1_args.push(("routing-table", tbl));
    }
    client.run(build_command("/ip/route/add", def1_args)).await?;

    // 5. Rute Default ISP 2: Rekursif ke Check Host 2 dengan target-scope=11, distance=2, check-gateway=ping
    let comment_def2 = format!("{} Backup Default Route via {} (Otomatis ambil alih jika ISP1 RTO)", FAILOVER_COMMENT_TAG, req.backup_check_host);
    let dist2_str = req.backup_distance.to_string();
    let mut def2_args = vec![
        ("dst-address", "0.0.0.0/0"),
        ("gateway", req.backup_check_host.as_str()),
        ("check-gateway", "ping"),
        ("distance", dist2_str.as_str()),
        ("target-scope", "11"),
        ("comment", comment_def2.as_str()),
    ];
    if let Some(tbl) = req.routing_table.as_deref() {
        def2_args.push(("routing-table", tbl));
    }
    client.run(build_command("/ip/route/add", def2_args)).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Konfigurasi Recursive Gateway Failover MTCRE berhasil diterapkan!",
        "architecture": {
            "primary_isp": {
                "physical_gateway": req.primary_gateway,
                "check_host": req.primary_check_host,
                "distance": req.primary_distance,
                "check_gateway": "ping (target-scope=11, scope=10)"
            },
            "backup_isp": {
                "physical_gateway": req.backup_gateway,
                "check_host": req.backup_check_host,
                "distance": req.backup_distance,
                "check_gateway": "ping (target-scope=11, scope=10)"
            }
        },
        "behavior": "Jika ping ke target internet (bukan hanya modem) RTO/putus, rute langsung beralih mulus ke ISP backup tanpa script eksternal."
    })))
}

/// GET or POST /api/v1/routing/failover/status - Menampilkan status aktif rute failover rekursif
pub async fn failover_status(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let routes = client
        .run(build_command("/ip/route/print", std::iter::empty::<(&str, &str)>()))
        .await?;

    let mut failover_routes: Vec<HashMap<String, String>> = Vec::new();
    for r in routes {
        if let Some(c) = r.get("comment") {
            if c.contains(FAILOVER_COMMENT_TAG) {
                failover_routes.push(r.attrs);
            }
        }
    }

    let is_configured = !failover_routes.is_empty();

    Ok(Json(json!({
        "success": true,
        "configured": is_configured,
        "total_rules": failover_routes.len(),
        "routes": failover_routes
    })))
}

/// POST /api/v1/routing/failover/remove - Menghapus semua aturan failover rekursif
pub async fn remove_recursive_failover(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let routes = client
        .run(build_command("/ip/route/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();

    let mut removed_count = 0;
    for r in routes {
        if let Some(c) = r.get("comment") {
            if c.contains(FAILOVER_COMMENT_TAG) {
                if let Some(id) = r.get(".id") {
                    let _ = client.run(build_command("/ip/route/remove", [(".id", id)])).await;
                    removed_count += 1;
                }
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("{} aturan Recursive Failover berhasil dihapus dari tabel routing", removed_count),
        "removed_count": removed_count
    })))
}

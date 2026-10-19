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
pub struct EnableGraphingReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// Aktifkan grafik interface traffic (default: true)
    pub enable_interfaces: Option<bool>,
    /// Interface target (default: "all")
    pub interface: Option<String>,
    /// Aktifkan grafik antrean queue bandwidth (default: true)
    pub enable_queues: Option<bool>,
    /// Aktifkan grafik resource CPU & Memory (default: true)
    pub enable_resource: Option<bool>,
    /// Batasi akses grafik hanya dari subnet IP tertentu (misal: "192.168.88.0/24", default: "0.0.0.0/0")
    pub allow_address: Option<String>,
}

/// GET or POST /api/v1/tools/graphing/status - Melihat status konfigurasi grafik bawaan RouterOS
pub async fn graphing_status(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let ifaces_fut = client.run(build_command("/tool/graphing/interface/print", std::iter::empty::<(&str, &str)>()));
    let queues_fut = client.run(build_command("/tool/graphing/queue/print", std::iter::empty::<(&str, &str)>()));
    let res_fut = client.run(build_command("/tool/graphing/resource/print", std::iter::empty::<(&str, &str)>()));

    let (ifaces_res, queues_res, res_res) = tokio::join!(ifaces_fut, queues_fut, res_fut);

    let ifaces: Vec<HashMap<String, String>> = ifaces_res.unwrap_or_default().into_iter().map(|r| r.attrs).collect();
    let queues: Vec<HashMap<String, String>> = queues_res.unwrap_or_default().into_iter().map(|r| r.attrs).collect();
    let res: Vec<HashMap<String, String>> = res_res.unwrap_or_default().into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "interfaces_graphing": ifaces,
        "queues_graphing": queues,
        "resource_graphing": res,
        "web_url_hint": "http://<IP-ROUTER>/graphs/"
    })))
}

/// POST /api/v1/tools/graphing/enable - Mengaktifkan grafik historis MRTG web bawaan RouterOS
pub async fn graphing_enable(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<EnableGraphingReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let allow = req.allow_address.as_deref().unwrap_or("0.0.0.0/0");
    crate::routes::sanitize::validate_ip_or_cidr(allow)?;
    if let Some(i) = req.interface.as_deref() {
        crate::routes::sanitize::validate_safe_identifier(i, "interface")?;
    }
    let mut configured = Vec::new();

    // 1. Graphing Interface
    if req.enable_interfaces.unwrap_or(true) {
        let iface = req.interface.as_deref().unwrap_or("all");
        let _ = client.run(build_command(
            "/tool/graphing/interface/add",
            [
                ("interface", iface),
                ("allow-address", allow),
                ("store-on-disk", "yes"),
            ],
        )).await;
        configured.push(format!("Interface Traffic ({})", iface));
    }

    // 2. Graphing Queues
    if req.enable_queues.unwrap_or(true) {
        let _ = client.run(build_command(
            "/tool/graphing/queue/add",
            [
                ("simple-queue", "all"),
                ("allow-address", allow),
                ("store-on-disk", "yes"),
            ],
        )).await;
        configured.push("Bandwidth Simple Queues".into());
    }

    // 3. Graphing Resource (CPU, RAM, HDD)
    if req.enable_resource.unwrap_or(true) {
        let _ = client.run(build_command(
            "/tool/graphing/resource/add",
            [
                ("allow-address", allow),
                ("store-on-disk", "yes"),
            ],
        )).await;
        configured.push("CPU & RAM System Resource".into());
    }

    Ok(Json(json!({
        "success": true,
        "message": "Grafik web historis bawaan RouterOS (/graphs) berhasil diaktifkan!",
        "configured_components": configured,
        "allow_address": allow,
        "info": "Anda dapat membuka browser ke http://<IP_ROUTER>/graphs/ untuk melihat grafik CPU, Interface, dan Queue."
    })))
}

/// POST /api/v1/tools/graphing/disable - Menonaktifkan seluruh konfigurasi grafik bawaan
pub async fn graphing_disable(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut removed = 0;

    // Bersihkan Interface Graphs
    if let Ok(rows) = client.run(build_command("/tool/graphing/interface/print", std::iter::empty::<(&str, &str)>())).await {
        for r in rows {
            if let Some(id) = r.get(".id") {
                let _ = client.run(build_command("/tool/graphing/interface/remove", [(".id", id)])).await;
                removed += 1;
            }
        }
    }

    // Bersihkan Queue Graphs
    if let Ok(rows) = client.run(build_command("/tool/graphing/queue/print", std::iter::empty::<(&str, &str)>())).await {
        for r in rows {
            if let Some(id) = r.get(".id") {
                let _ = client.run(build_command("/tool/graphing/queue/remove", [(".id", id)])).await;
                removed += 1;
            }
        }
    }

    // Bersihkan Resource Graphs
    if let Ok(rows) = client.run(build_command("/tool/graphing/resource/print", std::iter::empty::<(&str, &str)>())).await {
        for r in rows {
            if let Some(id) = r.get(".id") {
                let _ = client.run(build_command("/tool/graphing/resource/remove", [(".id", id)])).await;
                removed += 1;
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("{} entri graphing berhasil dihapus", removed),
        "removed_count": removed
    })))
}

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
pub struct ToggleDudeReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub enabled: bool,
    pub db_path: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ExportDbReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub backup_file: String,
}

#[derive(Deserialize, Debug)]
pub struct ImportDbReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub backup_file: String,
}

#[derive(Deserialize, Debug)]
pub struct VacuumDbReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub file: Option<String>, // e.g. "disk1/dude/dude.db" or "dude.db"
}

/// POST /api/v1/dude/status - Checks The Dude Server status, database location, and operational state
pub async fn status(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let res = client.run(build_command("/dude/print", std::iter::empty::<(&str, &str)>())).await;
    match res {
        Ok(rows) => {
            let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
            Ok(Json(json!({
                "success": true,
                "dude_installed": true,
                "data": first
            })))
        }
        Err(e) => {
            Ok(Json(json!({
                "success": false,
                "dude_installed": false,
                "message": format!("The Dude package is not installed or enabled on this router: {}", e)
            })))
        }
    }
}

/// POST /api/v1/dude/toggle - Enables or disables The Dude server (Recommended: disable before database maintenance)
pub async fn toggle(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ToggleDudeReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut args = vec![("enabled", if req.enabled { "yes" } else { "no" })];
    if let Some(p) = req.db_path.as_deref() {
        args.push(("data-directory", p));
    }

    client.run(build_command("/dude/set", args)).await?;
    Ok(Json(json!({
        "success": true,
        "message": format!("The Dude server status updated to enabled={}", req.enabled)
    })))
}

/// POST /api/v1/dude/export-db - Exports The Dude SQLite database to a backup file
pub async fn export_db(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ExportDbReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    client.run(build_command("/dude/export-db", [("backup-file", req.backup_file.as_str())])).await?;
    Ok(Json(json!({
        "success": true,
        "message": format!("Dude database exported to file '{}'", req.backup_file)
    })))
}

/// POST /api/v1/dude/import-db - Imports/Restores The Dude database from a backup file
pub async fn import_db(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ImportDbReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    client.run(build_command("/dude/import-db", [("backup-file", req.backup_file.as_str())])).await?;
    Ok(Json(json!({
        "success": true,
        "message": format!("Dude database imported from file '{}'", req.backup_file)
    })))
}

/// POST /api/v1/dude/vacuum - Vacuums/compacts The Dude database to reclaim fragmented disk space
pub async fn vacuum(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<VacuumDbReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let file_arg = req.file.as_deref().unwrap_or("dude.db");
    client.run(build_command("/dude/vacuum-db", [("file", file_arg)])).await?;
    Ok(Json(json!({
        "success": true,
        "message": format!("The Dude database '{}' vacuumed and compacted successfully", file_arg)
    })))
}

/// POST /api/v1/dude/devices - Lists all network devices/nodes monitored by The Dude
pub async fn devices(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/dude/device/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({
        "success": true,
        "count": data.len(),
        "devices": data
    })))
}

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
pub struct CreateBackupReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: Option<String>,
    pub password: Option<String>,
    pub encryption: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ExportReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub file: Option<String>,
    pub hide_sensitive: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct FileIdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

/// POST /api/v1/backup/create - Buat file backup (.backup) di storage MikroTik
pub async fn create_backup(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateBackupReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = Vec::new();
    if let Some(n) = req.name.as_deref() {
        args.push(("name", n));
    }
    if let Some(p) = req.password.as_deref() {
        args.push(("password", p));
    }
    if let Some(e) = req.encryption.as_deref() {
        args.push(("encryption", e));
    }
    client.run(build_command("/system/backup/save", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Backup created" })))
}

/// POST /api/v1/backup/export - Ekspor konfigurasi skrip RouterOS (.rsc)
pub async fn export_config(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ExportReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = Vec::new();
    if let Some(f) = req.file.as_deref() {
        args.push(("file", f));
    }
    let hide = if req.hide_sensitive.unwrap_or(true) { "yes" } else { "no" };
    args.push(("hide-sensitive", hide));
    client.run(build_command("/export", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Configuration export triggered" })))
}

/// POST or GET /api/v1/files/all - Daftar file di storage MikroTik
pub async fn files(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/file/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/files/remove - Hapus file dari storage MikroTik
pub async fn remove_file(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<FileIdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/file/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "File removed" })))
}

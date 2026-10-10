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
pub struct RunScriptReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub number_or_name: String,
}

#[derive(Deserialize, Debug)]
pub struct AddScriptReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub source: String,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AddSchedulerReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub on_event: String,
    pub interval: String, // e.g. "1d", "1h", "00:05:00"
    pub start_time: Option<String>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

/// POST or GET /api/v1/scripts/all - Daftar skrip RouterOS
pub async fn scripts(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/script/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/scripts/run - Eksekusi skrip RouterOS langsung
pub async fn run_script(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RunScriptReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/script/run", [(".id", req.number_or_name.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Script executed" })))
}

/// POST /api/v1/scripts/add - Tambah skrip baru
pub async fn add_script(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddScriptReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("name", req.name.as_str()), ("source", req.source.as_str())];
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/system/script/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Script added" })))
}

/// POST or GET /api/v1/schedulers/all - Daftar scheduler / cron otomatis
pub async fn schedulers(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/scheduler/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/schedulers/add - Tambah scheduler baru
pub async fn add_scheduler(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddSchedulerReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("name", req.name.as_str()),
        ("on-event", req.on_event.as_str()),
        ("interval", req.interval.as_str()),
    ];
    if let Some(s) = req.start_time.as_deref() {
        args.push(("start-time", s));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/system/scheduler/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Scheduler added" })))
}

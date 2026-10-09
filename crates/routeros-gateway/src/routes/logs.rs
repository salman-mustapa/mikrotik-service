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
pub struct LogReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub topic: Option<String>,
}

/// POST /api/v1/logs/all - Ambil baris log sistem router
pub async fn all(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<LogReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = Vec::new();
    if let Some(t) = req.topic.as_deref() {
        args.push(("?topics", t));
    }
    let rows = client.run(build_command("/log/print", args)).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

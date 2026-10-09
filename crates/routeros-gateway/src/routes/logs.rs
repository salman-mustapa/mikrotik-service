use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures::Stream;
use routeros_core::build_command;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::{AppState, RouterTarget};

#[derive(Deserialize, Debug, Default)]
pub struct LogReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub topic: Option<String>,
    pub search: Option<String>,
    pub severity: Option<String>, // "error", "warning", "info", "critical"
    pub limit: Option<usize>,
}

#[derive(Deserialize, Debug, Default)]
pub struct LogStreamQuery {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub router_id: Option<String>,
    pub topic: Option<String>,
}

/// POST /api/v1/logs/all - Filter and retrieve system logs with topic, severity, search, and limit
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
    let limit = req.limit.unwrap_or(100).clamp(1, 1000);
    let search_lower = req.search.as_deref().unwrap_or("").to_lowercase();
    let sev_filter = req.severity.as_deref().unwrap_or("").to_lowercase();

    let mut filtered: Vec<HashMap<String, String>> = rows
        .into_iter()
        .rev() // Start from the newest logs
        .filter_map(|r| {
            let topics = r.get("topics").unwrap_or("").to_lowercase();
            let msg = r.get("message").unwrap_or("").to_lowercase();

            if !sev_filter.is_empty() && !topics.contains(&sev_filter) {
                return None;
            }
            if !search_lower.is_empty() && !msg.contains(&search_lower) && !topics.contains(&search_lower) {
                return None;
            }
            Some(r.attrs)
        })
        .take(limit)
        .collect();

    // Reverse back to chronological order
    filtered.reverse();

    Ok(Json(json!({
        "success": true,
        "count": filtered.len(),
        "limit": limit,
        "data": filtered
    })))
}

/// GET /api/v1/logs/stream - Server-Sent Events (SSE) live system log stream
pub async fn stream_logs(
    State(st): State<Arc<AppState>>,
    axum::extract::Query(q): axum::extract::Query<LogStreamQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let target = RouterTarget {
        host: q.host,
        port: q.port,
        user: q.user,
        password: q.password,
    };
    let client = st.resolve_client(Some(&target), q.router_id.as_deref()).await?;
    
    let mut args = vec![("follow-only", "")];
    if let Some(t) = q.topic.as_deref() {
        args.push(("topics", t));
    }
    let sub = client.listen(build_command("/log/print", args)).await?;

    let sse_stream = sub.filter_map(|res| async move {
        match res {
            Ok(sentence) => {
                let json_str = serde_json::to_string(&sentence.attrs).unwrap_or_default();
                Some(Ok::<Event, Infallible>(Event::default().data(json_str)))
            }
            Err(_) => None,
        }
    });

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}

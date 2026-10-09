use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures::{Stream, StreamExt};
use routeros_core::build_command;
use serde::Deserialize;
use serde_json::{json, Map, Value};

use crate::error::ApiError;
use crate::state::{AppState, RouterTarget};

#[derive(Deserialize, Debug)]
pub struct RawCommandReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub command: String,
    #[serde(default)]
    pub args: Map<String, Value>,
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// POST /api/v1/command - Universal Arbitrary Command Runner
pub async fn run_raw_command(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RawCommandReq>,
) -> Result<Json<Value>, ApiError> {
    if !req.command.starts_with('/') {
        return Err(ApiError::BadRequest("command must start with '/'".into()));
    }

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let words = build_command(
        &req.command,
        req.args.iter().map(|(k, v)| (k.as_str(), value_to_string(v))),
    );

    let rows = client.run(words).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "count": data.len(),
        "data": data
    })))
}

/// GET /api/v1/listen - Universal Realtime Streaming SSE
pub async fn run_raw_listen(
    State(st): State<Arc<AppState>>,
    Query(mut q): Query<Vec<(String, String)>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let extract = |key: &str, q: &mut Vec<(String, String)>| {
        q.iter().position(|(k, _)| k == key).map(|pos| q.remove(pos).1)
    };

    let host = extract("host", &mut q);
    let port = extract("port", &mut q).and_then(|p| p.parse::<u16>().ok());
    let user = extract("user", &mut q);
    let pass = extract("password", &mut q);
    let router_id = extract("router_id", &mut q);

    let pos = q
        .iter()
        .position(|(k, _)| k == "command")
        .ok_or_else(|| ApiError::BadRequest("missing `command` query parameter".into()))?;
    let (_, command) = q.remove(pos);

    if !command.starts_with('/') {
        return Err(ApiError::BadRequest("command must start with '/'".into()));
    }

    let target = match (host, user) {
        (Some(h), Some(u)) => Some(RouterTarget {
            host: Some(h),
            port,
            user: Some(u),
            password: pass,
        }),
        _ => None,
    };

    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let sub = client.listen(build_command(&command, q)).await?;

    let stream = sub.map(|item| {
        Ok(match item {
            Ok(s) => Event::default().json_data(&s.attrs).unwrap_or_default(),
            Err(e) => Event::default().event("error").data(e.to_string()),
        })
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

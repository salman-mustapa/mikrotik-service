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
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::{AppState, RouterTarget};

#[derive(Deserialize, Debug, Default)]
pub struct FilterReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    #[serde(default)]
    pub filter: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct SampleTrafficReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub interface: String,
}

pub async fn all(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<FilterReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn sample_traffic(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SampleTrafficReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/monitor-traffic", [
        ("interface", req.interface.as_str()),
        ("once", ""),
    ])).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

/// GET /api/v1/interfaces/stream?interface=ether1&host=...&user=...
pub async fn stream_traffic(
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

    let iface = extract("interface", &mut q)
        .ok_or_else(|| ApiError::BadRequest("missing `interface` query parameter".into()))?;

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
    let sub = client.listen(build_command("/interface/monitor-traffic", [("interface", iface.as_str())])).await?;

    let stream = sub.map(|item| {
        Ok(match item {
            Ok(s) => Event::default().json_data(&s.attrs).unwrap_or_default(),
            Err(e) => Event::default().event("error").data(e.to_string()),
        })
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

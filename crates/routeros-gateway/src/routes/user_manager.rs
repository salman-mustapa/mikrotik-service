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
pub struct FilterReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    #[serde(default)]
    pub filter: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateUserReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub password: Option<String>,
    pub group: Option<String>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

pub async fn users(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    
    // Try RouterOS v7 User Manager first, fallback to RouterOS v6 /tool/user-manager
    let res = client.run(build_command("/user-manager/user/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await;
    let rows = match res {
        Ok(r) => r,
        Err(_) => client.run(build_command("/tool/user-manager/user/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await.unwrap_or_default(),
    };

    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn create_user(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateUserReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("name", req.name.as_str())];
    if let Some(p) = req.password.as_deref() {
        args.push(("password", p));
    }
    if let Some(g) = req.group.as_deref() {
        args.push(("group", g));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }

    let res = client.run(build_command("/user-manager/user/add", args.clone())).await;
    if res.is_err() {
        client.run(build_command("/tool/user-manager/user/add", args)).await?;
    }

    Ok(Json(json!({ "success": true, "message": "User Manager user created" })))
}

pub async fn remove_user(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let res = client.run(build_command("/user-manager/user/remove", [(".id", req.id.as_str())])).await;
    if res.is_err() {
        client.run(build_command("/tool/user-manager/user/remove", [(".id", req.id.as_str())])).await?;
    }
    Ok(Json(json!({ "success": true, "message": "User removed from User Manager" })))
}

pub async fn sessions(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let res = client.run(build_command("/user-manager/session/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await;
    let rows = match res {
        Ok(r) => r,
        Err(_) => client.run(build_command("/tool/user-manager/session/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await.unwrap_or_default(),
    };
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn profiles(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let res = client.run(build_command("/user-manager/profile/print", std::iter::empty::<(&str, &str)>())).await;
    let rows = match res {
        Ok(r) => r,
        Err(_) => client.run(build_command("/tool/user-manager/profile/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default(),
    };
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

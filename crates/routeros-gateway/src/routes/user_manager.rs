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

#[derive(Deserialize, Debug)]
pub struct AddUsermanProfileReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub name_for_users: Option<String>,
    pub price: Option<String>,
    pub validity: Option<String>, // e.g. "1d", "30d"
}

/// POST /api/v1/userman/profile/add - Add new User Manager billing profile
pub async fn add_profile(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddUsermanProfileReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut args = vec![("name", req.name.as_str())];
    if let Some(nfu) = req.name_for_users.as_deref() { args.push(("name-for-users", nfu)); }
    if let Some(p) = req.price.as_deref() { args.push(("price", p)); }
    if let Some(v) = req.validity.as_deref() { args.push(("validity", v)); }

    let res = client.run(build_command("/user-manager/profile/add", args.clone())).await;
    if res.is_err() {
        client.run(build_command("/tool/user-manager/profile/add", args)).await?;
    }

    Ok(Json(json!({ "success": true, "message": "User Manager profile created successfully" })))
}

#[derive(Deserialize, Debug)]
pub struct AddUsermanLimitationReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub rate_limit_rx: Option<String>, // Download limit e.g. "5M"
    pub rate_limit_tx: Option<String>, // Upload limit e.g. "2M"
    pub uptime_limit: Option<String>,  // e.g. "1h"
    pub download_limit: Option<String>,// e.g. "1G"
}

/// GET /api/v1/userman/limitations - List User Manager limitations
pub async fn limitations(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let res = client.run(build_command("/user-manager/limitation/print", std::iter::empty::<(&str, &str)>())).await;
    let rows = match res {
        Ok(r) => r,
        Err(_) => client.run(build_command("/tool/user-manager/profile-limitation/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default(),
    };
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "limitations": data })))
}

/// POST /api/v1/userman/limitation/add - Create a User Manager limitation
pub async fn add_limitation(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddUsermanLimitationReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut args = vec![("name", req.name.as_str())];
    if let (Some(rx), Some(tx)) = (req.rate_limit_rx.as_deref(), req.rate_limit_tx.as_deref()) {
        let combined = format!("{}/{}", tx, rx);
        args.push(("rate-limit-rx", rx));
        args.push(("rate-limit-tx", tx));
        args.push(("rate-limit", Box::leak(combined.into_boxed_str())));
    }
    if let Some(ut) = req.uptime_limit.as_deref() { args.push(("uptime-limit", ut)); }
    if let Some(dl) = req.download_limit.as_deref() { args.push(("download-limit", dl)); }

    let res = client.run(build_command("/user-manager/limitation/add", args.clone())).await;
    if res.is_err() {
        client.run(build_command("/tool/user-manager/profile-limitation/add", args)).await?;
    }

    Ok(Json(json!({ "success": true, "message": "User Manager limitation created" })))
}

#[derive(Deserialize, Debug)]
pub struct AssignProfileReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub user: String,
    pub profile: String,
}

/// POST /api/v1/userman/user/assign-profile - Assign User Manager profile to a user
pub async fn assign_profile(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AssignProfileReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // RouterOS v7 User Manager uses /user-manager/user-profile/add
    let res = client.run(build_command(
        "/user-manager/user-profile/add",
        [("user", req.user.as_str()), ("profile", req.profile.as_str())],
    )).await;

    if res.is_err() {
        // RouterOS v6 User Manager uses create-and-activate-profile
        client.run(build_command(
            "/tool/user-manager/user/create-and-activate-profile",
            [("numbers", req.user.as_str()), ("profile", req.profile.as_str())],
        )).await?;
    }

    Ok(Json(json!({
        "success": true,
        "user": req.user,
        "profile": req.profile,
        "message": format!("Profile '{}' berhasil di-assign ke user '{}'", req.profile, req.user)
    })))
}

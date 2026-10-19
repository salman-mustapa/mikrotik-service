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
    crate::routes::sanitize::validate_safe_identifier(&req.name, "user name")?;
    if let Some(g) = req.group.as_deref() {
        crate::routes::sanitize::validate_safe_identifier(g, "group")?;
    }
    if let Some(c) = req.comment.as_deref() {
        if c.len() > 128 || c.contains(['\r', '\n']) {
            return Err(ApiError::BadRequest("Comment exceeds 128 characters or contains newlines".into()));
        }
    }

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
    crate::routes::sanitize::validate_safe_identifier(&req.id, "user id or name")?;
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
    crate::routes::sanitize::validate_safe_identifier(&req.name, "profile name")?;
    if let Some(p) = req.price.as_deref() {
        crate::routes::sanitize::validate_price(p)?;
    }
    if let Some(v) = req.validity.as_deref() {
        crate::routes::sanitize::validate_validity(v)?;
    }

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
    crate::routes::sanitize::validate_safe_identifier(&req.name, "limitation name")?;
    if let Some(rx) = req.rate_limit_rx.as_deref() {
        crate::routes::sanitize::validate_bandwidth_rate(rx)?;
    }
    if let Some(tx) = req.rate_limit_tx.as_deref() {
        crate::routes::sanitize::validate_bandwidth_rate(tx)?;
    }
    if let Some(ut) = req.uptime_limit.as_deref() {
        crate::routes::sanitize::validate_validity(ut)?;
    }
    if let Some(dl) = req.download_limit.as_deref() {
        crate::routes::sanitize::validate_bandwidth_rate(dl)?;
    }

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let combined = if let (Some(rx), Some(tx)) = (req.rate_limit_rx.as_deref(), req.rate_limit_tx.as_deref()) {
        Some(format!("{}/{}", tx, rx))
    } else {
        None
    };

    let mut args = vec![("name", req.name.as_str())];
    if let Some(rx) = req.rate_limit_rx.as_deref() { args.push(("rate-limit-rx", rx)); }
    if let Some(tx) = req.rate_limit_tx.as_deref() { args.push(("rate-limit-tx", tx)); }
    if let Some(ref c) = combined { args.push(("rate-limit", c.as_str())); }
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
    crate::routes::sanitize::validate_safe_identifier(&req.user, "user")?;
    crate::routes::sanitize::validate_safe_identifier(&req.profile, "profile")?;

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

#[derive(Deserialize, Debug)]
pub struct AddUsermanRouterReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub ip_address: String,
    pub shared_secret: String,
}

pub async fn routers(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let res = client.run(build_command("/user-manager/router/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await;
    let rows = match res {
        Ok(r) => r,
        Err(_) => client.run(build_command("/tool/user-manager/router/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await.unwrap_or_default(),
    };
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "routers": data })))
}

pub async fn add_router(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddUsermanRouterReq>,
) -> Result<Json<Value>, ApiError> {
    crate::routes::sanitize::validate_safe_identifier(&req.name, "router name")?;
    crate::routes::sanitize::validate_ip_or_cidr(&req.ip_address)?;
    if req.shared_secret.is_empty() || req.shared_secret.len() > 64 {
        return Err(ApiError::BadRequest("shared_secret must be between 1 and 64 characters".into()));
    }

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let res = client.run(build_command(
        "/user-manager/router/add",
        [
            ("name", req.name.as_str()),
            ("address", req.ip_address.as_str()),
            ("shared-secret", req.shared_secret.as_str()),
        ],
    )).await;

    if res.is_err() {
        client.run(build_command(
            "/tool/user-manager/router/add",
            [
                ("customer", "admin"),
                ("name", req.name.as_str()),
                ("ip-address", req.ip_address.as_str()),
                ("shared-secret", req.shared_secret.as_str()),
            ],
        )).await?;
    }

    Ok(Json(json!({ "success": true, "message": format!("Router '{}' registered in User Manager", req.name) })))
}

pub async fn userman_status(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Check v7 first
    let v7_check = client.run(build_command("/user-manager/user/print", [("count-only", "")])).await;
    if let Ok(r) = v7_check {
        let count = r.first().and_then(|row| row.get("ret")).unwrap_or("0").to_string();
        return Ok(Json(json!({
            "success": true,
            "installed": true,
            "version": "v7",
            "command_base": "/user-manager",
            "user_count": count,
            "message": "User Manager RouterOS v7 active"
        })));
    }

    // Fallback check v6
    let v6_check = client.run(build_command("/tool/user-manager/user/print", [("count-only", "")])).await;
    if let Ok(r) = v6_check {
        let count = r.first().and_then(|row| row.get("ret")).unwrap_or("0").to_string();
        return Ok(Json(json!({
            "success": true,
            "installed": true,
            "version": "v6",
            "command_base": "/tool/user-manager",
            "user_count": count,
            "message": "User Manager RouterOS v6 active"
        })));
    }

    Ok(Json(json!({
        "success": true,
        "installed": false,
        "version": "unknown",
        "message": "Paket User Manager belum terinstal atau dinonaktifkan di router ini"
    })))
}


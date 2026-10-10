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
    pub password: String,
    #[serde(default = "default_profile")]
    pub profile: String,
    pub timelimit: Option<String>,
    pub datalimit: Option<String>,
    pub comment: Option<String>,
}

fn default_profile() -> String {
    "default".into()
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
    let rows = client.run(build_command("/ip/hotspot/user/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
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
    let mut args = vec![
        ("name", req.name.as_str()),
        ("password", req.password.as_str()),
        ("profile", req.profile.as_str()),
    ];
    if let Some(t) = req.timelimit.as_deref() {
        args.push(("limit-uptime", t));
    }
    if let Some(d) = req.datalimit.as_deref() {
        args.push(("limit-bytes-total", d));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ip/hotspot/user/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Hotspot user created" })))
}

pub async fn remove_user(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/hotspot/user/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "User removed" })))
}

pub async fn active(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/hotspot/active/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn kick(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/hotspot/active/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Active session terminated" })))
}

pub async fn profiles(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/hotspot/user/profile/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

#[derive(Deserialize, Debug)]
pub struct AddHotspotProfileReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub rate_limit: Option<String>,
    pub shared_users: Option<String>,
    pub session_timeout: Option<String>,
    pub keepalive_timeout: Option<String>,
    pub status_autorefresh: Option<String>,
    pub comment: Option<String>,
}

pub async fn add_profile(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddHotspotProfileReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("name", req.name.as_str())];
    if let Some(rl) = req.rate_limit.as_deref() {
        args.push(("rate-limit", rl));
    }
    if let Some(su) = req.shared_users.as_deref() {
        args.push(("shared-users", su));
    }
    if let Some(st) = req.session_timeout.as_deref() {
        args.push(("session-timeout", st));
    }
    if let Some(kt) = req.keepalive_timeout.as_deref() {
        args.push(("keepalive-timeout", kt));
    }
    if let Some(sa) = req.status_autorefresh.as_deref() {
        args.push(("status-autorefresh", sa));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ip/hotspot/user/profile/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Hotspot user profile created" })))
}

pub async fn remove_profile(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/hotspot/user/profile/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Hotspot user profile removed" })))
}

pub async fn ip_bindings(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/hotspot/ip-binding/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

#[derive(Deserialize, Debug)]
pub struct GenerateBatchReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub qty: usize,
    #[serde(default = "default_batch_prefix")]
    pub prefix: String,
    #[serde(default = "default_batch_len")]
    pub length: usize,
    #[serde(default = "default_profile")]
    pub profile: String,
    pub timelimit: Option<String>,
    pub datalimit: Option<String>,
    pub server: Option<String>,
    pub comment: Option<String>,
}

fn default_batch_prefix() -> String {
    "V-".into()
}

fn default_batch_len() -> usize {
    6
}

pub async fn generate_batch(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<GenerateBatchReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let qty = req.qty.clamp(1, 1000);
    let charset = b"abcdefghjkmnpqrstuvwxyz23456789";
    let len = req.length.clamp(4, 16);

    let mut vouchers = Vec::with_capacity(qty);
    let mut futures = Vec::with_capacity(qty);

    let seed_base = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(123456789);

    for i in 0..qty {
        let mut code = String::with_capacity(len);
        let mut val = seed_base.wrapping_add((i as u128).wrapping_mul(9876543211));
        for _ in 0..len {
            val = val.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let idx = (val as usize) % charset.len();
            code.push(charset[idx] as char);
        }
        let username = format!("{}{}", req.prefix, code);
        let password = username.clone();

        let mut args = vec![
            ("name".to_string(), username.clone()),
            ("password".to_string(), password.clone()),
            ("profile".to_string(), req.profile.clone()),
        ];
        if let Some(t) = req.timelimit.as_deref() {
            args.push(("limit-uptime".to_string(), t.to_string()));
        }
        if let Some(d) = req.datalimit.as_deref() {
            args.push(("limit-bytes-total".to_string(), d.to_string()));
        }
        if let Some(s) = req.server.as_deref() {
            args.push(("server".to_string(), s.to_string()));
        }
        let c_str = req.comment.as_deref().unwrap_or("Batch-Generated");
        args.push(("comment".to_string(), c_str.to_string()));

        let words: Vec<String> = build_command(
            "/ip/hotspot/user/add",
            args.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        );
        futures.push(client.run(words));

        vouchers.push(json!({
            "username": username,
            "password": password,
            "profile": req.profile,
            "timelimit": req.timelimit,
            "datalimit": req.datalimit,
        }));
    }

    let results = futures::future::join_all(futures).await;
    let success_count = results.into_iter().filter(|r| r.is_ok()).count();

    Ok(Json(json!({
        "success": true,
        "total_requested": qty,
        "total_created": success_count,
        "vouchers": vouchers,
    })))
}

#[derive(Deserialize, Debug)]
pub struct BindHostReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub mac_address: String,
    pub address: Option<String>,
    pub to_address: Option<String>,
    #[serde(default = "default_binding_type")]
    pub binding_type: String, // "bypassed", "regular", "blocked"
    pub comment: Option<String>,
}

fn default_binding_type() -> String {
    "bypassed".into()
}

pub async fn hosts(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/hotspot/host/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn remove_host(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/hotspot/host/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Host entry removed" })))
}

pub async fn bind_host(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BindHostReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("mac-address", req.mac_address.as_str()),
        ("type", req.binding_type.as_str()),
    ];
    if let Some(a) = req.address.as_deref() {
        args.push(("address", a));
    }
    if let Some(to) = req.to_address.as_deref() {
        args.push(("to-address", to));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ip/hotspot/ip-binding/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Host successfully added to IP binding (bypass/block/regular)" })))
}



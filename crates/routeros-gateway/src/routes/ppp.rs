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
pub struct CreateSecretReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub password: String,
    #[serde(default = "default_service")]
    pub service: String,
    #[serde(default = "default_profile")]
    pub profile: String,
    pub remote_address: Option<String>,
    pub comment: Option<String>,
}

fn default_service() -> String {
    "pppoe".into()
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

pub async fn secrets(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<FilterReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ppp/secret/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn create_secret(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateSecretReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("name", req.name.as_str()),
        ("password", req.password.as_str()),
        ("service", req.service.as_str()),
        ("profile", req.profile.as_str()),
    ];
    if let Some(r) = req.remote_address.as_deref() {
        args.push(("remote-address", r));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ppp/secret/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "PPP secret created" })))
}

pub async fn remove_secret(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ppp/secret/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Secret removed" })))
}

pub async fn active(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<FilterReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ppp/active/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn disconnect(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ppp/active/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Active session disconnected" })))
}

pub async fn profiles(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<FilterReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ppp/profile/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

#[derive(Deserialize, Debug)]
pub struct SetSecretReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
    pub password: Option<String>,
    pub profile: Option<String>,
    pub remote_address: Option<String>,
    pub comment: Option<String>,
    pub disabled: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct CustomerActionReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub username: String,
    #[serde(default = "default_isolate_profile")]
    pub isolate_profile: String,
    pub comment: Option<String>,
}

fn default_isolate_profile() -> String {
    "ISOLIR".into()
}

#[derive(Deserialize, Debug)]
pub struct RestoreCustomerReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub username: String,
    pub active_profile: String,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateProfileReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub local_address: Option<String>,
    pub remote_address: Option<String>,
    pub rate_limit: Option<String>,
    pub dns_server: Option<String>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreatePppoeServerReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub service_name: String,
    pub interface: String,
    #[serde(default = "default_profile")]
    pub default_profile: String,
    pub one_session_per_host: Option<bool>,
}

pub async fn set_secret(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetSecretReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![(".id", req.id.as_str())];
    if let Some(p) = req.password.as_deref() {
        args.push(("password", p));
    }
    if let Some(prof) = req.profile.as_deref() {
        args.push(("profile", prof));
    }
    if let Some(r) = req.remote_address.as_deref() {
        args.push(("remote-address", r));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    let dis_str = req.disabled.map(|d| if d { "yes" } else { "no" });
    if let Some(d) = dis_str.as_deref() {
        args.push(("disabled", d));
    }
    client.run(build_command("/ppp/secret/set", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Secret updated" })))
}

pub async fn isolate_customer(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CustomerActionReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    
    // 1. Find secret by username
    let secrets = client.run(build_command("/ppp/secret/print", [("?name", req.username.as_str())])).await?;
    let secret_id = secrets.first().and_then(|r| r.get(".id")).ok_or_else(|| {
        ApiError::BadRequest(format!("PPPoE secret user '{}' not found", req.username))
    })?;

    // 2. Set profile to ISOLIR
    let comment = req.comment.unwrap_or_else(|| "ISOLATED_OVERDUE".into());
    client.run(build_command("/ppp/secret/set", [
        (".id", secret_id),
        ("profile", req.isolate_profile.as_str()),
        ("comment", comment.as_str()),
    ])).await?;

    // 3. Find and kick active session
    let active_rows = client.run(build_command("/ppp/active/print", [("?name", req.username.as_str())])).await?;
    let mut kicked = false;
    for row in active_rows {
        if let Some(active_id) = row.get(".id") {
            let _ = client.run(build_command("/ppp/active/remove", [(".id", active_id)])).await;
            kicked = true;
        }
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("Customer '{}' isolated successfully. Active session kicked: {}", req.username, kicked),
        "secret_id": secret_id,
        "active_kicked": kicked,
    })))
}

pub async fn restore_customer(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RestoreCustomerReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // 1. Find secret by username
    let secrets = client.run(build_command("/ppp/secret/print", [("?name", req.username.as_str())])).await?;
    let secret_id = secrets.first().and_then(|r| r.get(".id")).ok_or_else(|| {
        ApiError::BadRequest(format!("PPPoE secret user '{}' not found", req.username))
    })?;

    // 2. Restore active profile and re-enable
    let comment = req.comment.unwrap_or_else(|| "ACTIVE_CUSTOMER".into());
    client.run(build_command("/ppp/secret/set", [
        (".id", secret_id),
        ("profile", req.active_profile.as_str()),
        ("disabled", "no"),
        ("comment", comment.as_str()),
    ])).await?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Customer '{}' restored to profile '{}'", req.username, req.active_profile),
        "secret_id": secret_id
    })))
}

pub async fn pppoe_servers(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<FilterReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/pppoe-server/server/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn create_pppoe_server(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreatePppoeServerReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("service-name", req.service_name.as_str()),
        ("interface", req.interface.as_str()),
        ("default-profile", req.default_profile.as_str()),
    ];
    let one_str = req.one_session_per_host.map(|b| if b { "yes" } else { "no" });
    if let Some(o) = one_str.as_deref() {
        args.push(("one-session-per-host", o));
    }
    client.run(build_command("/interface/pppoe-server/server/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "PPPoE Server instance created" })))
}

pub async fn create_profile(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateProfileReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("name", req.name.as_str())];
    if let Some(l) = req.local_address.as_deref() {
        args.push(("local-address", l));
    }
    if let Some(r) = req.remote_address.as_deref() {
        args.push(("remote-address", r));
    }
    if let Some(rt) = req.rate_limit.as_deref() {
        args.push(("rate-limit", rt));
    }
    if let Some(dns) = req.dns_server.as_deref() {
        args.push(("dns-server", dns));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ppp/profile/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "PPP profile created" })))
}


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

#[derive(Deserialize, Debug)]
pub struct AddWalledGardenReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub dst_host: Option<String>,      // e.g. "*.midtrans.com" or "api.whatsapp.com"
    pub dst_address: Option<String>,   // e.g. "103.10.10.0/24" (for walled-garden ip)
    pub protocol: Option<String>,
    pub dst_port: Option<String>,
    pub action: Option<String>,        // default "allow"
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct DeployWalledGardenPresetReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub preset: String, // "payment_gateways", "whatsapp_messaging", "banking_qris", "all_essential"
}

#[derive(Deserialize, Debug)]
pub struct ResetUserCountersReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String, // Hotspot username
}

#[derive(Deserialize, Debug)]
pub struct InjectExpiryScriptReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub profile_name: String,
    pub validity: String, // e.g. "1d", "3h", "30d"
}

/// GET /api/v1/hotspot/walled-garden - List allowed domain and IP bypasses before login
pub async fn walled_garden(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let host_rows = client.run(build_command("/ip/hotspot/walled-garden/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    let ip_rows = client.run(build_command("/ip/hotspot/walled-garden/ip/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();

    let hosts: Vec<HashMap<String, String>> = host_rows.into_iter().map(|r| r.attrs).collect();
    let ips: Vec<HashMap<String, String>> = ip_rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "total_rules": hosts.len() + ips.len(),
        "walled_garden_domains": hosts,
        "walled_garden_ips": ips
    })))
}

/// POST /api/v1/hotspot/walled-garden/add - Add custom domain or IP to Walled Garden
pub async fn add_walled_garden(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddWalledGardenReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let action = req.action.as_deref().unwrap_or("allow");
    let comment = req.comment.as_deref().unwrap_or("Walled Garden Bypass");

    if let Some(ref host) = req.dst_host {
        let args = vec![("dst-host", host.as_str()), ("action", action), ("comment", comment)];
        client.run(build_command("/ip/hotspot/walled-garden/add", args)).await?;
        return Ok(Json(json!({ "success": true, "type": "domain", "dst_host": host, "message": "Domain berhasil di-bypass pada Walled Garden" })));
    } else if let Some(ref ip) = req.dst_address {
        let mut args = vec![("dst-address", ip.as_str()), ("action", action), ("comment", comment)];
        if let Some(ref proto) = req.protocol { args.push(("protocol", proto.as_str())); }
        if let Some(ref port) = req.dst_port { args.push(("dst-port", port.as_str())); }
        client.run(build_command("/ip/hotspot/walled-garden/ip/add", args)).await?;
        return Ok(Json(json!({ "success": true, "type": "ip", "dst_address": ip, "message": "IP subnet berhasil di-bypass pada Walled Garden IP" })));
    }

    Err(ApiError::BadRequest("Harus menyertakan 'dst_host' (domain) atau 'dst_address' (IP)".into()))
}

/// POST /api/v1/hotspot/walled-garden/remove - Remove Walled Garden rule by ID
pub async fn remove_walled_garden(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let res = client.run(build_command("/ip/hotspot/walled-garden/remove", [(".id", req.id.as_str())])).await;
    if res.is_err() {
        client.run(build_command("/ip/hotspot/walled-garden/ip/remove", [(".id", req.id.as_str())])).await?;
    }

    Ok(Json(json!({ "success": true, "message": "Rule Walled Garden berhasil dihapus" })))
}

/// POST /api/v1/hotspot/walled-garden/deploy-preset - 1-Click bypass presets for Payment Gateways & Messaging
pub async fn deploy_walled_garden_preset(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DeployWalledGardenPresetReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let domains: Vec<&str> = match req.preset.as_str() {
        "payment_gateways" => vec![
            "*.midtrans.com", "*.xendit.co", "*.tripay.co.id", "*.faspay.co.id",
            "*.duitku.com", "*.espay.id", "app.sandbox.midtrans.com", "api.midtrans.com"
        ],
        "whatsapp_messaging" => vec![
            "*.whatsapp.com", "*.whatsapp.net", "api.whatsapp.com", "web.whatsapp.com",
            "*.fbcdn.net", "*.facebook.com"
        ],
        "banking_qris" => vec![
            "*.bca.co.id", "*.mandiri.co.id", "*.bri.co.id", "*.bni.co.id",
            "*.dana.id", "*.ovo.id", "*.gopay.co.id", "*.linkaja.id", "qris.id"
        ],
        "all_essential" => vec![
            "*.midtrans.com", "*.xendit.co", "*.tripay.co.id", "*.dana.id", "*.ovo.id",
            "*.whatsapp.com", "*.whatsapp.net", "fonts.googleapis.com", "fonts.gstatic.com",
            "cdn.jsdelivr.net", "cdnjs.cloudflare.com"
        ],
        _ => return Err(ApiError::BadRequest(format!("Preset '{}' tidak valid. Gunakan: 'payment_gateways', 'whatsapp_messaging', 'banking_qris', atau 'all_essential'", req.preset))),
    };

    let mut added_count = 0;
    for d in &domains {
        let comment = format!("[PRESET:{}]", req.preset);
        let res = client.run(build_command(
            "/ip/hotspot/walled-garden/add",
            [("dst-host", *d), ("action", "allow"), ("comment", comment.as_str())],
        )).await;
        if res.is_ok() { added_count += 1; }
    }

    Ok(Json(json!({
        "success": true,
        "preset": req.preset,
        "total_domains": domains.len(),
        "applied_rules": added_count,
        "message": format!("Preset '{}' berhasil diterapkan ke Walled Garden!", req.preset)
    })))
}

/// GET /api/v1/hotspot/cookies - List saved active Hotspot cookies
pub async fn cookies(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/hotspot/cookie/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({ "success": true, "count": data.len(), "cookies": data })))
}

/// POST /api/v1/hotspot/cookie/remove - Remove a Hotspot cookie
pub async fn remove_cookie(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/hotspot/cookie/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Cookie hotspot berhasil dihapus" })))
}

/// POST /api/v1/hotspot/user/reset-counters - Reset bytes and uptime counter of a hotspot user
pub async fn reset_counters(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ResetUserCountersReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Find user ID
    let rows = client.run(build_command(
        "/ip/hotspot/user/print",
        [("name", req.name.as_str())],
    )).await?;

    let u_row = rows.into_iter().next().ok_or_else(|| {
        ApiError::BadRequest(format!("User '{}' tidak ditemukan pada router", req.name))
    })?;

    if let Some(id) = u_row.get(".id") {
        client.run(build_command("/ip/hotspot/user/reset-counters", [(".id", id)])).await?;
    }

    Ok(Json(json!({
        "success": true,
        "username": req.name,
        "message": format!("Counter traffic dan uptime untuk user '{}' berhasil di-reset ke nol!", req.name)
    })))
}

/// POST /api/v1/hotspot/profile/inject-expiry-script - Mikhmon-style automatic On-Login first login validity injector
pub async fn inject_expiry_script(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<InjectExpiryScriptReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Find profile
    let rows = client.run(build_command(
        "/ip/hotspot/user/profile/print",
        [("name", req.profile_name.as_str())],
    )).await?;

    let p_row = rows.into_iter().next().ok_or_else(|| {
        ApiError::BadRequest(format!("Profile '{}' tidak ditemukan pada router", req.profile_name))
    })?;

    let id = p_row.get(".id").ok_or_else(|| ApiError::BadRequest("ID Profile tidak ditemukan".into()))?;

    // Mikhmon-style on-login script: checks if user has no expiration date, then tags comment with expiry timestamp
    let on_login_script = format!(
        ":local validity \"{}\";\n\
         :local uComment [/ip hotspot user get [find name=$user] comment];\n\
         :if ($uComment = \"\" or [:find $uComment \"EXP:\"] = nil) do={{\n\
           :local date [/system clock get date];\n\
           :local time [/system clock get time];\n\
           /ip hotspot user set [find name=$user] comment=([/ip hotspot user get [find name=$user] comment] . \" [FIRST-LOGIN:\" . $date . \" \" . $time . \"|VAL:\" . $validity . \"]\");\n\
         }}",
        req.validity
    );

    client.run(build_command(
        "/ip/hotspot/user/profile/set",
        [
            (".id", id),
            ("on-login", on_login_script.as_str()),
        ],
    )).await?;

    Ok(Json(json!({
        "success": true,
        "profile": req.profile_name,
        "validity": req.validity,
        "message": format!("Skrip On-Login First Login Expiry Tracker berhasil dipasang pada profil '{}'!", req.profile_name)
    })))
}

// =========================================================================
// HOTSPOT MEMBER MANAGEMENT & LOGIN VERIFIER (RECURRING / VIP / BULANAN)
// =========================================================================

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = (y - era * 400) as u64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 } as u64) + 2) / 5 + (d as u64) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + (doe as i64) - 719468
}

fn current_date_ymd() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = (secs / 86400) as i64;
    let (y, m, d) = civil_from_days(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn add_days_to_date(base_date: &str, add_days: i64) -> String {
    let parts: Vec<&str> = base_date.split('-').collect();
    if parts.len() == 3 {
        if let (Ok(y), Ok(m), Ok(d)) = (parts[0].parse::<i64>(), parts[1].parse::<u32>(), parts[2].parse::<u32>()) {
            let days = days_from_civil(y, m, d) + add_days;
            let (ny, nm, nd) = civil_from_days(days);
            return format!("{:04}-{:02}-{:02}", ny, nm, nd);
        }
    }
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = (secs / 86400) as i64 + add_days;
    let (y, m, d) = civil_from_days(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn days_between_dates(from_ymd: &str, to_ymd: &str) -> i64 {
    let p1: Vec<&str> = from_ymd.split('-').collect();
    let p2: Vec<&str> = to_ymd.split('-').collect();
    if p1.len() == 3 && p2.len() == 3 {
        if let (Ok(y1), Ok(m1), Ok(d1), Ok(y2), Ok(m2), Ok(d2)) = (
            p1[0].parse::<i64>(), p1[1].parse::<u32>(), p1[2].parse::<u32>(),
            p2[0].parse::<i64>(), p2[1].parse::<u32>(), p2[2].parse::<u32>(),
        ) {
            let days1 = days_from_civil(y1, m1, d1);
            let days2 = days_from_civil(y2, m2, d2);
            return days2 - days1;
        }
    }
    0
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MemberInfo {
    pub id: String,
    pub username: String,
    pub fullname: String,
    pub phone: String,
    pub profile: String,
    pub expires_at: String,
    pub days_left: i64,
    pub status: String,
    pub price: u64,
    pub notes: String,
    pub disabled: bool,
    pub uptime: String,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub bytes_total: u64,
}

fn parse_member_comment(
    id: &str,
    username: &str,
    profile: &str,
    comment: &str,
    disabled: bool,
    uptime: &str,
    bytes_in: u64,
    bytes_out: u64,
) -> Option<MemberInfo> {
    if !comment.contains("[MEMBER|") {
        return None;
    }
    let start = comment.find("[MEMBER|")? + 8;
    let end = comment[start..].find(']')? + start;
    let payload = &comment[start..end];
    let parts: Vec<&str> = payload.split('|').collect();

    let fullname = parts.first().copied().unwrap_or("").trim().to_string();
    let phone = parts.get(1).copied().unwrap_or("").trim().to_string();

    let exp_part = parts.get(2).copied().unwrap_or("").trim();
    let expires_at = if let Some(stripped) = exp_part.strip_prefix("EXP:") {
        stripped.trim().to_string()
    } else {
        "2099-12-31".to_string()
    };

    let raw_status = parts.get(3).copied().unwrap_or("ACTIVE").trim().to_uppercase();

    let price_part = parts.get(4).copied().unwrap_or("Rp0").trim();
    let price: u64 = price_part.trim_start_matches("Rp").parse().unwrap_or(0);

    let notes = parts.get(5).copied().unwrap_or("").trim().to_string();

    let today = current_date_ymd();
    let days_left = days_between_dates(&today, &expires_at);

    let effective_status = if disabled || raw_status == "SUSPENDED" {
        "SUSPENDED".to_string()
    } else if days_left < 0 {
        "EXPIRED".to_string()
    } else {
        "ACTIVE".to_string()
    };

    Some(MemberInfo {
        id: id.to_string(),
        username: username.to_string(),
        fullname,
        phone,
        profile: profile.to_string(),
        expires_at,
        days_left,
        status: effective_status,
        price,
        notes,
        disabled,
        uptime: uptime.to_string(),
        bytes_in,
        bytes_out,
        bytes_total: bytes_in + bytes_out,
    })
}

#[derive(Deserialize, Debug)]
pub struct RegisterMemberReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub username: String,
    pub password: String,
    pub fullname: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default = "default_member_days")]
    pub validity_days: u32,
    pub price: Option<u64>,
    pub notes: Option<String>,
}

fn default_member_days() -> u32 {
    30
}

#[derive(Deserialize, Debug, Default)]
pub struct ListMembersReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub status: Option<String>, // "all", "active", "expired", "suspended"
    pub search: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct RenewMemberReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub username: String,
    #[serde(default = "default_member_days")]
    pub add_days: u32,
    #[serde(default)]
    pub reset_traffic: bool,
}

#[derive(Deserialize, Debug)]
pub struct SuspendMemberReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub username: String,
    pub isolate_profile: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ActivateMemberReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub username: String,
    pub profile: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateMemberReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub username: String,
    pub new_password: Option<String>,
    pub fullname: Option<String>,
    pub phone: Option<String>,
    pub profile: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct DeleteMemberReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub username: String,
}

#[derive(Deserialize, Debug)]
pub struct VerifyLoginReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub username: String,
    pub password: Option<String>,
    pub mac_address: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AddIpBindingReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub mac_address: String,
    #[serde(default = "default_binding_type")]
    pub r#type: String, // "bypassed", "regular", "blocked"
    pub address: Option<String>,
    pub to_address: Option<String>,
    pub server: Option<String>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct KickActiveReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: Option<String>,
    pub username: Option<String>,
    pub mac_address: Option<String>,
}

/// POST /api/v1/hotspot/members/register - Register a monthly/recurring Hotspot member
pub async fn register_member(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RegisterMemberReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Check if user already exists
    let existing = client
        .run(build_command(
            "/ip/hotspot/user/print",
            [("name", req.username.as_str())],
        ))
        .await?;

    if !existing.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "Username '{}' sudah terdaftar di Hotspot",
            req.username
        )));
    }

    let today = current_date_ymd();
    let expires_at = add_days_to_date(&today, req.validity_days as i64);
    let price_val = req.price.unwrap_or(0);
    let notes_val = req.notes.unwrap_or_default();

    let comment = format!(
        "[MEMBER|{}|{}|EXP:{}|ACTIVE|Rp{}|{}]",
        req.fullname.trim(),
        req.phone.trim(),
        expires_at,
        price_val,
        notes_val.trim()
    );

    client
        .run(build_command(
            "/ip/hotspot/user/add",
            [
                ("name", req.username.as_str()),
                ("password", req.password.as_str()),
                ("profile", req.profile.as_str()),
                ("comment", comment.as_str()),
            ],
        ))
        .await?;

    Ok(Json(json!({
        "success": true,
        "member": {
            "username": req.username,
            "fullname": req.fullname,
            "phone": req.phone,
            "profile": req.profile,
            "validity_days": req.validity_days,
            "registered_at": today,
            "expires_at": expires_at,
            "price": price_val,
            "status": "ACTIVE"
        },
        "message": format!("Member Hotspot '{}' berhasil didaftarkan aktif hingga {}!", req.username, expires_at)
    })))
}

/// GET /api/v1/hotspot/members - List all registered Hotspot members with parsed metadata
pub async fn list_members(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<ListMembersReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client
        .run(build_command(
            "/ip/hotspot/user/print",
            std::iter::empty::<(&str, &str)>(),
        ))
        .await?;

    let mut members = Vec::new();

    for r in rows {
        let id = r.get(".id").unwrap_or_default();
        let username = r.get("name").unwrap_or_default();
        let profile = r.get("profile").unwrap_or("default");
        let comment = r.get("comment").unwrap_or_default();
        let disabled = r.get("disabled").map(|v| v == "yes" || v == "true").unwrap_or(false);
        let uptime = r.get("uptime").unwrap_or("0s");
        let bytes_in: u64 = r.get("bytes-in").and_then(|v| v.parse().ok()).unwrap_or(0);
        let bytes_out: u64 = r.get("bytes-out").and_then(|v| v.parse().ok()).unwrap_or(0);

        if let Some(m) = parse_member_comment(id, username, profile, comment, disabled, uptime, bytes_in, bytes_out) {
            let matches_status = match req.status.as_deref() {
                Some("active") => m.status == "ACTIVE",
                Some("expired") => m.status == "EXPIRED",
                Some("suspended") => m.status == "SUSPENDED",
                _ => true,
            };

            let matches_search = if let Some(ref q) = req.search {
                let q_lower = q.to_lowercase();
                m.username.to_lowercase().contains(&q_lower)
                    || m.fullname.to_lowercase().contains(&q_lower)
                    || m.phone.contains(&q_lower)
            } else {
                true
            };

            if matches_status && matches_search {
                members.push(m);
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "count": members.len(),
        "members": members
    })))
}

/// POST /api/v1/hotspot/members/renew - Renew member subscription duration
pub async fn renew_member(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RenewMemberReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client
        .run(build_command(
            "/ip/hotspot/user/print",
            [("name", req.username.as_str())],
        ))
        .await?;

    let u_row = rows.into_iter().next().ok_or_else(|| {
        ApiError::BadRequest(format!("Member '{}' tidak ditemukan pada router", req.username))
    })?;

    let id = u_row.get(".id").ok_or_else(|| ApiError::BadRequest("ID user tidak ditemukan".into()))?;
    let comment = u_row.get("comment").unwrap_or_default();

    let today = current_date_ymd();
    let old_exp = if let Some(idx) = comment.find("EXP:") {
        let after = &comment[idx + 4..];
        let end = after.find('|').or_else(|| after.find(']')).unwrap_or(after.len());
        after[..end].trim()
    } else {
        today.as_str()
    };

    // If existing expiry date is still in the future, extend from that date; otherwise from today
    let base_date = if days_between_dates(&today, old_exp) > 0 {
        old_exp
    } else {
        today.as_str()
    };

    let new_expires_at = add_days_to_date(base_date, req.add_days as i64);

    // Reconstruct comment with updated expiration and ACTIVE status
    let mut parts: Vec<String> = comment
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split('|')
        .map(|s| s.to_string())
        .collect();

    if parts.len() >= 4 && parts[0] == "MEMBER" {
        parts[3] = format!("EXP:{}", new_expires_at);
        if parts.len() > 4 {
            parts[4] = "ACTIVE".to_string();
        }
    }
    let new_comment = format!("[{}]", parts.join("|"));

    // Update in MikroTik: set active, new comment, enable
    client
        .run(build_command(
            "/ip/hotspot/user/set",
            [
                (".id", id),
                ("comment", new_comment.as_str()),
                ("disabled", "no"),
            ],
        ))
        .await?;

    if req.reset_traffic {
        let _ = client.run(build_command("/ip/hotspot/user/reset-counters", [(".id", id)])).await;
    }

    Ok(Json(json!({
        "success": true,
        "username": req.username,
        "previous_expiry": old_exp,
        "new_expiry": new_expires_at,
        "days_added": req.add_days,
        "status": "ACTIVE",
        "message": format!("Langganan member '{}' berhasil diperpanjang hingga {}!", req.username, new_expires_at)
    })))
}

/// POST /api/v1/hotspot/members/suspend - Suspend / isolate member
pub async fn suspend_member(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SuspendMemberReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client
        .run(build_command(
            "/ip/hotspot/user/print",
            [("name", req.username.as_str())],
        ))
        .await?;

    let u_row = rows.into_iter().next().ok_or_else(|| {
        ApiError::BadRequest(format!("Member '{}' tidak ditemukan", req.username))
    })?;

    let id = u_row.get(".id").ok_or_else(|| ApiError::BadRequest("ID user tidak ditemukan".into()))?;
    let comment = u_row.get("comment").unwrap_or_default();

    let mut parts: Vec<String> = comment
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split('|')
        .map(|s| s.to_string())
        .collect();

    if parts.len() > 4 && parts[0] == "MEMBER" {
        parts[4] = "SUSPENDED".to_string();
    }
    let new_comment = format!("[{}]", parts.join("|"));

    if let Some(ref iso_prof) = req.isolate_profile {
        client
            .run(build_command(
                "/ip/hotspot/user/set",
                [
                    (".id", id),
                    ("profile", iso_prof.as_str()),
                    ("comment", new_comment.as_str()),
                ],
            ))
            .await?;
    } else {
        client
            .run(build_command(
                "/ip/hotspot/user/set",
                [
                    (".id", id),
                    ("disabled", "yes"),
                    ("comment", new_comment.as_str()),
                ],
            ))
            .await?;
    }

    // Force disconnect active session if currently online
    let active_rows = client
        .run(build_command(
            "/ip/hotspot/active/print",
            [("user", req.username.as_str())],
        ))
        .await
        .unwrap_or_default();

    for a in active_rows {
        if let Some(aid) = a.get(".id") {
            let _ = client.run(build_command("/ip/hotspot/active/remove", [(".id", aid)])).await;
        }
    }

    Ok(Json(json!({
        "success": true,
        "username": req.username,
        "status": "SUSPENDED",
        "message": format!("Member '{}' berhasil di-suspend dan sesi aktif diputus", req.username)
    })))
}

/// POST /api/v1/hotspot/members/activate - Re-activate a suspended member
pub async fn activate_member(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ActivateMemberReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client
        .run(build_command(
            "/ip/hotspot/user/print",
            [("name", req.username.as_str())],
        ))
        .await?;

    let u_row = rows.into_iter().next().ok_or_else(|| {
        ApiError::BadRequest(format!("Member '{}' tidak ditemukan", req.username))
    })?;

    let id = u_row.get(".id").ok_or_else(|| ApiError::BadRequest("ID user tidak ditemukan".into()))?;
    let comment = u_row.get("comment").unwrap_or_default();

    let mut parts: Vec<String> = comment
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split('|')
        .map(|s| s.to_string())
        .collect();

    if parts.len() > 4 && parts[0] == "MEMBER" {
        parts[4] = "ACTIVE".to_string();
    }
    let new_comment = format!("[{}]", parts.join("|"));

    let mut args = vec![
        (".id", id),
        ("disabled", "no"),
        ("comment", new_comment.as_str()),
    ];

    if let Some(ref prof) = req.profile {
        args.push(("profile", prof.as_str()));
    }

    client.run(build_command("/ip/hotspot/user/set", args)).await?;

    Ok(Json(json!({
        "success": true,
        "username": req.username,
        "status": "ACTIVE",
        "message": format!("Member '{}' berhasil diaktifkan kembali!", req.username)
    })))
}

/// POST /api/v1/hotspot/members/update - Update member details
pub async fn update_member(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<UpdateMemberReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client
        .run(build_command(
            "/ip/hotspot/user/print",
            [("name", req.username.as_str())],
        ))
        .await?;

    let u_row = rows.into_iter().next().ok_or_else(|| {
        ApiError::BadRequest(format!("Member '{}' tidak ditemukan", req.username))
    })?;

    let id = u_row.get(".id").ok_or_else(|| ApiError::BadRequest("ID user tidak ditemukan".into()))?;
    let comment = u_row.get("comment").unwrap_or_default();

    let mut args = vec![(".id", id)];

    if let Some(ref np) = req.new_password {
        args.push(("password", np.as_str()));
    }
    if let Some(ref prof) = req.profile {
        args.push(("profile", prof.as_str()));
    }

    // Update comment fields if metadata changed
    let mut parts: Vec<String> = comment
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split('|')
        .map(|s| s.to_string())
        .collect();

    let mut comment_changed = false;
    if parts.len() >= 4 && parts[0] == "MEMBER" {
        if let Some(ref fn_val) = req.fullname {
            parts[1] = fn_val.trim().to_string();
            comment_changed = true;
        }
        if let Some(ref ph) = req.phone {
            parts[2] = ph.trim().to_string();
            comment_changed = true;
        }
        if let Some(ref nt) = req.notes {
            if parts.len() > 6 {
                parts[6] = nt.trim().to_string();
            } else {
                parts.push(nt.trim().to_string());
            }
            comment_changed = true;
        }
    }

    let updated_comment = if comment_changed {
        Some(format!("[{}]", parts.join("|")))
    } else {
        None
    };

    if let Some(ref uc) = updated_comment {
        args.push(("comment", uc.as_str()));
    }

    client.run(build_command("/ip/hotspot/user/set", args)).await?;

    Ok(Json(json!({
        "success": true,
        "username": req.username,
        "message": format!("Data member '{}' berhasil diperbarui!", req.username)
    })))
}

/// POST /api/v1/hotspot/members/delete - Delete member account permanently
pub async fn delete_member(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DeleteMemberReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client
        .run(build_command(
            "/ip/hotspot/user/print",
            [("name", req.username.as_str())],
        ))
        .await?;

    let u_row = rows.into_iter().next().ok_or_else(|| {
        ApiError::BadRequest(format!("Member '{}' tidak ditemukan", req.username))
    })?;

    let id = u_row.get(".id").ok_or_else(|| ApiError::BadRequest("ID user tidak ditemukan".into()))?;

    client.run(build_command("/ip/hotspot/user/remove", [(".id", id)])).await?;

    Ok(Json(json!({
        "success": true,
        "username": req.username,
        "message": format!("Akun member '{}' berhasil dihapus dari router!", req.username)
    })))
}

/// POST /api/v1/hotspot/verify-login - Unified pre-login verification engine for vouchers and members
pub async fn verify_login(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<VerifyLoginReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // 1. Fetch user from router
    let rows = client
        .run(build_command(
            "/ip/hotspot/user/print",
            [("name", req.username.as_str())],
        ))
        .await?;

    let u_row = match rows.into_iter().next() {
        Some(r) => r,
        None => {
            return Ok(Json(json!({
                "valid": false,
                "code": "USER_NOT_FOUND",
                "message": "Username atau kode voucher tidak terdaftar pada sistem hotspot"
            })));
        }
    };

    // 2. Check disabled status
    let disabled = u_row.get("disabled").map(|v| v == "yes" || v == "true").unwrap_or(false);
    if disabled {
        return Ok(Json(json!({
            "valid": false,
            "code": "ACCOUNT_DISABLED",
            "message": "Akun sedang dinonaktifkan atau dalam masa isolir"
        })));
    }

    let comment = u_row.get("comment").unwrap_or_default();
    let profile = u_row.get("profile").unwrap_or("default");
    let actual_pass = u_row.get("password").unwrap_or_default();
    let is_member = comment.contains("[MEMBER|");

    let account_type = if is_member { "member" } else { "voucher" };

    // 3. Password check
    if is_member {
        // Member requires password match
        let input_pass = req.password.as_deref().unwrap_or_default();
        if input_pass != actual_pass {
            return Ok(Json(json!({
                "valid": false,
                "code": "INVALID_PASSWORD",
                "account_type": account_type,
                "message": "Password salah untuk akun member ini"
            })));
        }
    } else {
        // Voucher mode: if password is non-empty on router, verify it
        if !actual_pass.is_empty() {
            let input_pass = req.password.as_deref().unwrap_or_default();
            if input_pass != actual_pass && input_pass != req.username {
                return Ok(Json(json!({
                    "valid": false,
                    "code": "INVALID_PASSWORD",
                    "account_type": account_type,
                    "message": "Password atau kode voucher tidak cocok"
                })));
            }
        }
    }

    // 4. Expiration check for members
    if is_member {
        let today = current_date_ymd();
        if let Some(idx) = comment.find("EXP:") {
            let after = &comment[idx + 4..];
            let end = after.find('|').or_else(|| after.find(']')).unwrap_or(after.len());
            let exp_date = after[..end].trim();
            let days_left = days_between_dates(&today, exp_date);
            if days_left < 0 {
                return Ok(Json(json!({
                    "valid": false,
                    "code": "SUBSCRIPTION_EXPIRED",
                    "account_type": "member",
                    "expires_at": exp_date,
                    "days_left": days_left,
                    "message": format!("Masa berlangganan bulanan telah berakhir pada {}", exp_date)
                })));
            }
        }
    }

    // 5. Quota / Uptime limit checks
    if let Some(uptime_limit) = u_row.get("limit-uptime") {
        let uptime_used = u_row.get("uptime").unwrap_or("0s");
        if uptime_used >= uptime_limit && uptime_limit != "0s" {
            return Ok(Json(json!({
                "valid": false,
                "code": "UPTIME_EXHAUSTED",
                "account_type": account_type,
                "message": "Batas waktu pemakaian (uptime) telah habis"
            })));
        }
    }

    if let Some(bytes_limit_str) = u_row.get("limit-bytes-total") {
        if let Ok(limit_bytes) = bytes_limit_str.parse::<u64>() {
            if limit_bytes > 0 {
                let bin: u64 = u_row.get("bytes-in").and_then(|v| v.parse().ok()).unwrap_or(0);
                let bout: u64 = u_row.get("bytes-out").and_then(|v| v.parse().ok()).unwrap_or(0);
                if (bin + bout) >= limit_bytes {
                    return Ok(Json(json!({
                        "valid": false,
                        "code": "QUOTA_EXHAUSTED",
                        "account_type": account_type,
                        "bytes_used": bin + bout,
                        "bytes_limit": limit_bytes,
                        "message": "Batas kuota data internet telah habis"
                    })));
                }
            }
        }
    }

    // 6. Active sessions concurrency check vs shared-users
    let active_sessions = client
        .run(build_command(
            "/ip/hotspot/active/print",
            [("user", req.username.as_str())],
        ))
        .await
        .unwrap_or_default();

    let prof_rows = client
        .run(build_command(
            "/ip/hotspot/user/profile/print",
            [("name", profile)],
        ))
        .await
        .unwrap_or_default();

    let shared_users: usize = prof_rows
        .first()
        .and_then(|r| r.get("shared-users"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);

    if active_sessions.len() >= shared_users {
        return Ok(Json(json!({
            "valid": false,
            "code": "MAX_CONCURRENT_USERS_REACHED",
            "account_type": account_type,
            "active_devices": active_sessions.len(),
            "max_shared_users": shared_users,
            "message": format!("Batas maksimum perangkat bersamaan ({}) telah tercapai", shared_users)
        })));
    }

    Ok(Json(json!({
        "valid": true,
        "code": "AUTHORIZED",
        "account_type": account_type,
        "username": req.username,
        "profile": profile,
        "active_devices": active_sessions.len(),
        "max_shared_users": shared_users,
        "message": "Autentikasi kredensial Hotspot valid dan siap login"
    })))
}

/// POST /api/v1/hotspot/ip-binding/add - Add MAC bypass / binding
pub async fn add_ip_binding(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddIpBindingReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut args = vec![
        ("mac-address", req.mac_address.as_str()),
        ("type", req.r#type.as_str()),
    ];

    if let Some(ref a) = req.address {
        args.push(("address", a.as_str()));
    }
    if let Some(ref ta) = req.to_address {
        args.push(("to-address", ta.as_str()));
    }
    if let Some(ref s) = req.server {
        args.push(("server", s.as_str()));
    }
    if let Some(ref c) = req.comment {
        args.push(("comment", c.as_str()));
    }

    client.run(build_command("/ip/hotspot/ip-binding/add", args)).await?;

    Ok(Json(json!({
        "success": true,
        "mac_address": req.mac_address,
        "type": req.r#type,
        "message": format!("IP Binding untuk MAC '{}' tipe '{}' berhasil ditambahkan!", req.mac_address, req.r#type)
    })))
}

/// POST /api/v1/hotspot/ip-binding/remove - Remove MAC binding
pub async fn remove_ip_binding(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    client
        .run(build_command(
            "/ip/hotspot/ip-binding/remove",
            [(".id", req.id.as_str())],
        ))
        .await?;

    Ok(Json(json!({ "success": true, "message": "IP Binding berhasil dihapus" })))
}

/// POST /api/v1/hotspot/active/kick - Force kick an active user/host session
pub async fn kick_active(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<KickActiveReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    if let Some(ref id) = req.id {
        client.run(build_command("/ip/hotspot/active/remove", [(".id", id.as_str())])).await?;
        return Ok(Json(json!({ "success": true, "message": "Sesi aktif berhasil di-kick!" })));
    }

    let rows = if let Some(ref u) = req.username {
        client.run(build_command("/ip/hotspot/active/print", [("user", u.as_str())])).await?
    } else if let Some(ref m) = req.mac_address {
        client.run(build_command("/ip/hotspot/active/print", [("mac-address", m.as_str())])).await?
    } else {
        return Err(ApiError::BadRequest("Wajib menyertakan id, username, atau mac_address untuk kick!".into()));
    };

    let mut kicked = 0;
    for r in rows {
        if let Some(id) = r.get(".id") {
            let _ = client.run(build_command("/ip/hotspot/active/remove", [(".id", id)])).await;
            kicked += 1;
        }
    }

    Ok(Json(json!({
        "success": true,
        "kicked_sessions": kicked,
        "message": format!("Berhasil memutuskan {} sesi aktif!", kicked)
    })))
}



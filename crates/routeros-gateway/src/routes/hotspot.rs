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



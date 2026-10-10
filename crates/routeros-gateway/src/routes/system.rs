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
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IdentityReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
}

pub async fn resource(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/resource/print", std::iter::empty::<(&str, &str)>())).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

pub async fn routerboard(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/routerboard/print", std::iter::empty::<(&str, &str)>())).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

pub async fn identity(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/identity/print", std::iter::empty::<(&str, &str)>())).await?;
    let name = rows.first().and_then(|r| r.get("name")).unwrap_or("MikroTik");
    Ok(Json(json!({ "success": true, "name": name })))
}

pub async fn set_identity(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdentityReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/identity/set", [("name", req.name.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Identity updated" })))
}

pub async fn check_update(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/package/update/check-for-updates", std::iter::empty::<(&str, &str)>())).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

pub async fn install_update(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/package/update/install", std::iter::empty::<(&str, &str)>())).await?;
    Ok(Json(json!({ "success": true, "message": "Download and install triggered" })))
}

pub async fn reboot(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/reboot", std::iter::empty::<(&str, &str)>())).await?;
    Ok(Json(json!({ "success": true, "message": "Reboot command dispatched" })))
}

#[derive(Deserialize, Debug)]
pub struct ChannelReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub channel: String, // "stable", "long-term", "testing", "development"
}

pub async fn download_update(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/package/update/download", std::iter::empty::<(&str, &str)>())).await?;
    Ok(Json(json!({ "success": true, "message": "Package download initiated in background" })))
}

pub async fn set_channel(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ChannelReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/package/update/set", [("channel", req.channel.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": format!("Update channel set to '{}'", req.channel) })))
}

pub async fn packages(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/package/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<_> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/system/package/downgrade - Downgrade RouterOS ke versi sebelumnya
pub async fn package_downgrade(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/package/downgrade", std::iter::empty::<(&str, &str)>())).await?;
    Ok(Json(json!({
        "success": true,
        "message": "Downgrade command dispatched. Router will reboot into the previous version."
    })))
}

/// POST /api/v1/system/package/cancel - Batalkan unduhan paket pembaruan yang sedang berjalan
pub async fn package_cancel(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/system/package/update/cancel", std::iter::empty::<(&str, &str)>())).await?;
    Ok(Json(json!({
        "success": true,
        "message": "Package update/download canceled."
    })))
}


pub async fn clock(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/system/clock/print", std::iter::empty::<(&str, &str)>())).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

#[derive(Deserialize, Debug)]
pub struct ToggleServiceReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub service_name: String, // "telnet", "ftp", "www", "ssh", "api", "winbox"
    pub disabled: bool,
    pub port: Option<u16>,
    pub address: Option<String>,
}

pub async fn services(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/service/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<_> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn toggle_service(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ToggleServiceReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/service/print", [("?name", req.service_name.as_str())])).await?;
    let id = rows.first().and_then(|r| r.get(".id")).ok_or_else(|| {
        ApiError::BadRequest(format!("Service '{}' not found", req.service_name))
    })?;

    let dis_str = if req.disabled { "yes" } else { "no" };
    let mut args = vec![(".id", id), ("disabled", dis_str)];
    let port_str = req.port.map(|p| p.to_string());
    if let Some(p) = port_str.as_deref() {
        args.push(("port", p));
    }
    if let Some(a) = req.address.as_deref() {
        args.push(("address", a));
    }

    client.run(build_command("/ip/service/set", args)).await?;
    Ok(Json(json!({
        "success": true,
        "message": format!("Service '{}' updated (disabled: {})", req.service_name, req.disabled)
    })))
}

/// GET /api/v1/system/health - Get Router hardware telemetry (CPU temp, voltage, wattage, fans)
pub async fn system_health(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows_res = client
        .run(build_command(
            "/system/health/print",
            std::iter::empty::<(&str, &str)>(),
        ))
        .await;

    match rows_res {
        Ok(rows) => {
            if rows.is_empty() {
                Ok(Json(json!({
                    "success": true,
                    "has_sensors": false,
                    "message": "Hardware tidak memiliki sensor temperatur/voltase aktif (CHR/Virtual)",
                    "sensors": {}
                })))
            } else {
                // Compatible with both ROS v7 (name/value list) and ROS v6 (single row key-values)
                let mut sensors = serde_json::Map::new();
                for r in &rows {
                    if let (Some(name), Some(val)) = (r.get("name"), r.get("value")) {
                        sensors.insert(name.to_string(), json!(val));
                    }
                }
                if sensors.is_empty() {
                    if let Some(first) = rows.first() {
                        for (k, v) in &first.attrs {
                            sensors.insert(k.clone(), json!(v));
                        }
                    }
                }

                Ok(Json(json!({
                    "success": true,
                    "has_sensors": true,
                    "sensors": sensors,
                    "raw_count": rows.len()
                })))
            }
        }
        Err(_) => {
            Ok(Json(json!({
                "success": true,
                "has_sensors": false,
                "message": "Platform virtual atau router ini tidak mendukung /system/health",
                "sensors": {}
            })))
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct SetupNtpReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    pub servers: Option<Vec<String>>,
}

fn default_timezone() -> String {
    "Asia/Jakarta".into()
}

/// POST /api/v1/system/ntp/setup - 1-Click NTP Client and Timezone Sync Engine
pub async fn setup_ntp(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetupNtpReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // 1. Set System Clock Timezone
    let _ = client
        .run(build_command(
            "/system/clock/set",
            [
                ("time-zone-name", req.timezone.as_str()),
                ("time-zone-autodetect", "no"),
            ],
        ))
        .await;

    // 2. Configure NTP Client (compatible with ROS v7 and v6)
    let ntp_servers = req.servers.unwrap_or_else(|| {
        vec![
            "id.pool.ntp.org".into(),
            "time.google.com".into(),
            "0.pool.ntp.org".into(),
        ]
    });
    let servers_joined = ntp_servers.join(",");

    let v7_res = client
        .run(build_command(
            "/system/ntp/client/set",
            [
                ("enabled", "yes"),
                ("servers", servers_joined.as_str()),
            ],
        ))
        .await;

    let v7_ok = v7_res.is_ok();
    if !v7_ok {
        let p_srv = ntp_servers.first().map(|s| s.as_str()).unwrap_or("id.pool.ntp.org");
        let s_srv = ntp_servers.get(1).map(|s| s.as_str()).unwrap_or("time.google.com");
        let _ = client
            .run(build_command(
                "/system/ntp/client/set",
                [
                    ("enabled", "yes"),
                    ("primary-ntp", p_srv),
                    ("secondary-ntp", s_srv),
                ],
            ))
            .await;
    }

    // Read back clock
    let clock_rows = client
        .run(build_command(
            "/system/clock/print",
            std::iter::empty::<(&str, &str)>(),
        ))
        .await
        .unwrap_or_default();
    let clock_data = clock_rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();

    Ok(Json(json!({
        "success": true,
        "timezone": req.timezone,
        "ntp_servers": ntp_servers,
        "routeros_v7_mode": v7_ok,
        "current_clock": clock_data,
        "message": format!("NTP Client dan Zona Waktu '{}' berhasil disinkronkan!", req.timezone)
    })))
}

/// GET /api/v1/system/ntp - Get NTP client status
pub async fn ntp_status(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let ntp_rows = client
        .run(build_command(
            "/system/ntp/client/print",
            std::iter::empty::<(&str, &str)>(),
        ))
        .await?;
    let ntp_data = ntp_rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();

    let clock_rows = client
        .run(build_command(
            "/system/clock/print",
            std::iter::empty::<(&str, &str)>(),
        ))
        .await?;
    let clock_data = clock_rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();

    Ok(Json(json!({
        "success": true,
        "ntp_client": ntp_data,
        "clock": clock_data
    })))
}


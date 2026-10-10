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
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SetWifiReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub ssid: String,
    pub passphrase: Option<String>,
    pub config_name: Option<String>,
    pub security_name: Option<String>,
    pub country: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ProvisionReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub radio_mac: Option<String>,
}

/// GET or POST /api/v1/capsman/radios - Daftar radio Access Point MikroTik (CAPs) yang terhubung
pub async fn radios(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Try legacy /caps-man/radio/print first, fallback to v7 /interface/wifi/capsman/remote-cap/print
    let rows = match client.run(build_command("/caps-man/radio/print", std::iter::empty::<(&str, &str)>())).await {
        Ok(r) => r,
        Err(_) => client.run(build_command("/interface/wifi/capsman/remote-cap/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default(),
    };

    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "radios": data })))
}

/// GET or POST /api/v1/capsman/interfaces - Daftar interface virtual WiFi yang di-manage controller
pub async fn interfaces(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = match client.run(build_command("/caps-man/interface/print", std::iter::empty::<(&str, &str)>())).await {
        Ok(r) => r,
        Err(_) => client.run(build_command("/interface/wifi/print", [("?master", "no")])).await.unwrap_or_default(),
    };

    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "interfaces": data })))
}

/// GET or POST /api/v1/capsman/registrations - Daftar seluruh HP/klien wireless yang tersambung di semua AP
pub async fn registrations(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = match client.run(build_command("/caps-man/registration-table/print", std::iter::empty::<(&str, &str)>())).await {
        Ok(r) => r,
        Err(_) => client.run(build_command("/interface/wifi/registration-table/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default(),
    };

    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "clients": data })))
}

/// POST /api/v1/capsman/set-wifi - Konfigurasi SSID & Password WiFi terpusat ke seluruh AP MikroTik
pub async fn set_wifi(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetWifiReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let sec_name = req.security_name.as_deref().unwrap_or("sec-gateway");
    let cfg_name = req.config_name.as_deref().unwrap_or("cfg-gateway");

    // 1. Setup / Update Security Profile (WPA2-PSK / WPA3)
    if let Some(pass) = req.passphrase.as_deref() {
        let existing_sec = client.run(build_command("/caps-man/security/print", [("?name", sec_name)])).await.unwrap_or_default();
        if let Some(first) = existing_sec.first() {
            if let Some(id) = first.get(".id") {
                let _ = client.run(build_command("/caps-man/security/set", [
                    (".id", id),
                    ("passphrase", pass),
                    ("authentication-types", "wpa2-psk"),
                    ("encryption", "aes-ccm")
                ])).await;
            }
        } else {
            let _ = client.run(build_command("/caps-man/security/add", [
                ("name", sec_name),
                ("passphrase", pass),
                ("authentication-types", "wpa2-psk"),
                ("encryption", "aes-ccm")
            ])).await;
        }
    }

    // 2. Setup / Update WiFi Configuration
    let existing_cfg = client.run(build_command("/caps-man/configuration/print", [("?name", cfg_name)])).await.unwrap_or_default();
    if let Some(first) = existing_cfg.first() {
        if let Some(id) = first.get(".id") {
            let mut args = vec![(".id", id), ("ssid", req.ssid.as_str())];
            if req.passphrase.is_some() {
                args.push(("security", sec_name));
            }
            if let Some(c) = req.country.as_deref() {
                args.push(("country", c));
            }
            client.run(build_command("/caps-man/configuration/set", args)).await?;
        }
    } else {
        let mut args = vec![("name", cfg_name), ("ssid", req.ssid.as_str())];
        if req.passphrase.is_some() {
            args.push(("security", sec_name));
        }
        if let Some(c) = req.country.as_deref() {
            args.push(("country", c));
        }
        client.run(build_command("/caps-man/configuration/add", args)).await?;
    }

    // 3. Ensure Provisioning Rule attaches cfg-gateway to all radios
    let prov_rules = client.run(build_command("/caps-man/provisioning/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    if prov_rules.is_empty() {
        let _ = client.run(build_command("/caps-man/provisioning/add", [
            ("action", "create-dynamic-enabled"),
            ("master-configuration", cfg_name),
            ("comment", "Managed by MikroTik Universal Gateway")
        ])).await;
    } else if let Some(first) = prov_rules.first() {
        if let Some(id) = first.get(".id") {
            let _ = client.run(build_command("/caps-man/provisioning/set", [
                (".id", id),
                ("master-configuration", cfg_name)
            ])).await;
        }
    }

    // 4. Trigger Reprovisioning across all connected APs
    let _ = client.run(build_command("/caps-man/remote-cap/provision", [("numbers", "*")])).await;

    Ok(Json(json!({
        "success": true,
        "message": format!("Konfigurasi WiFi berhasil disebarkan ke seluruh Access Point MikroTik (SSID: '{}')", req.ssid),
        "ssid": req.ssid,
        "security_profile": sec_name,
        "configuration_profile": cfg_name
    })))
}

/// POST /api/v1/capsman/provision - Paksa refresh/reprovisioning seluruh atau satu radio AP
pub async fn provision(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ProvisionReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    if let Some(mac) = req.radio_mac.as_deref() {
        client.run(build_command("/caps-man/remote-cap/provision", [("?radio-mac", mac)])).await?;
    } else {
        client.run(build_command("/caps-man/remote-cap/provision", [("numbers", "*")])).await?;
    }

    Ok(Json(json!({
        "success": true,
        "message": "Perintah reprovisioning Access Point berhasil dikirim ke controller."
    })))
}

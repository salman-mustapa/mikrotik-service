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
pub struct AddStaticDnsReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub address: String,
    pub ttl: Option<String>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

/// POST or GET /api/v1/dns/static - Daftar static DNS record
pub async fn static_records(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/dns/static/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/dns/static/add - Tambah static DNS record baru
pub async fn add_static(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddStaticDnsReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("name", req.name.as_str()), ("address", req.address.as_str())];
    if let Some(t) = req.ttl.as_deref() {
        args.push(("ttl", t));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ip/dns/static/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "Static DNS record added" })))
}

/// POST /api/v1/dns/static/remove - Hapus static DNS record
pub async fn remove_static(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dns/static/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Static DNS record removed" })))
}

/// POST /api/v1/dns/cache/flush - Bersihkan cache DNS di router
pub async fn flush_cache(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dns/cache/flush", std::iter::empty::<(&str, &str)>())).await?;
    Ok(Json(json!({ "success": true, "message": "DNS cache flushed" })))
}

#[derive(Deserialize, Debug)]
pub struct AddAdlistReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub url: String,
    pub ssl_verify: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct DeployAdlistPresetReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub preset: String, // "hagezi-light", "stevenblack", "adguard", "anti-malware"
}

/// GET or POST /api/v1/dns/adlist - Daftar AdList (RouterOS v7 Native Ad-Blocker)
pub async fn adlist(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/dns/adlist/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/dns/adlist/add - Tambah URL blocklist ke MikroTik AdList
pub async fn add_adlist(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddAdlistReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("url", req.url.as_str())];
    let ssl_str = req.ssl_verify.map(|s| if s { "yes" } else { "no" });
    if let Some(s) = ssl_str.as_deref() {
        args.push(("ssl-verify", s));
    }
    client.run(build_command("/ip/dns/adlist/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "AdList source added successfully" })))
}

/// POST /api/v1/dns/adlist/remove - Hapus sumber AdList
pub async fn remove_adlist(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dns/adlist/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "AdList source removed" })))
}

/// GET /api/v1/dns/adlist/presets - Preset blocklist terverifikasi dan aman
pub async fn adlist_presets() -> Json<Value> {
    Json(json!({
        "success": true,
        "presets": [
            {
                "id": "hagezi-light",
                "name": "HaGeZi Multi LIGHT",
                "desc": "Blokir iklan & tracking web tanpa memecah aplikasi atau streaming (Sangat aman untuk WISP & RT-RW Net)",
                "url": "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/light.txt"
            },
            {
                "id": "stevenblack",
                "name": "StevenBlack Unified Hosts",
                "desc": "Standar emas blocklist open-source global untuk adware dan malware",
                "url": "https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts"
            },
            {
                "id": "anti-malware",
                "name": "HaGeZi Threat Intelligence (TIF)",
                "desc": "Blokir situs phishing, ransomware C2, spyware, dan cryptojacking tingkat lanjut",
                "url": "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/tif.txt"
            },
            {
                "id": "adguard",
                "name": "AdGuard DNS Filter",
                "desc": "Filter iklan dan pelacak komprehensif dari AdGuard",
                "url": "https://adguardteam.github.io/HostlistsRegistry/assets/filter_1.txt"
            }
        ]
    }))
}

/// POST /api/v1/dns/adlist/deploy-preset - Deploy preset blocklist terpilih langsung ke router
pub async fn deploy_adlist_preset(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DeployAdlistPresetReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let url = match req.preset.to_lowercase().as_str() {
        "hagezi-light" | "hagezi" => "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/light.txt",
        "stevenblack" => "https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts",
        "anti-malware" | "malware" | "tif" => "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/tif.txt",
        "adguard" => "https://adguardteam.github.io/HostlistsRegistry/assets/filter_1.txt",
        _ => return Err(ApiError::BadRequest(format!("Unknown preset '{}'. Valid options: hagezi-light, stevenblack, anti-malware, adguard", req.preset))),
    };

    client.run(build_command("/ip/dns/adlist/add", [("url", url), ("ssl-verify", "no")])).await?;
    Ok(Json(json!({
        "success": true,
        "message": format!("Preset '{}' successfully deployed to MikroTik AdList ({})", req.preset, url)
    })))
}


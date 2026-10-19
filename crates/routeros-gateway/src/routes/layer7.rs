use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use routeros_core::build_command;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::{AppState, RouterTarget};

#[derive(Deserialize, Debug, Default)]
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AddLayer7Req {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub regexp: String,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct RemoveLayer7Req {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct DeployLayer7PresetReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub preset_id: String,
    #[serde(default = "default_true")]
    pub create_drop_filter: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Debug)]
pub struct ToggleServicePortReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub disabled: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct Layer7Preset {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub regexp: &'static str,
    pub description: &'static str,
}

pub const LAYER7_PRESETS: &[Layer7Preset] = &[
    Layer7Preset {
        id: "bittorrent",
        name: "BitTorrent-P2P",
        category: "P2P / Bandwidth Hog",
        regexp: "^(\\x13bittorrent protocol|azver\\x01|get /scrape\\?info_hash=)",
        description: "Mendeteksi handshake protokol BitTorrent, uTorrent, dan tracker p2p untuk mencegah bandwidth exhaustion.",
    },
    Layer7Preset {
        id: "judol_gambling",
        name: "Judol-AntiGambling",
        category: "Cybersecurity & Compliance",
        regexp: "^.+(judi|slot|gacor|maxwin|sbobet|poker88|togel|depo-pulsa).+$",
        description: "Mendeteksi keyword domain & request perjudian online, slot gacor, dan bandar togel.",
    },
    Layer7Preset {
        id: "tiktok",
        name: "TikTok-CDN",
        category: "Social Media",
        regexp: "^.+(tiktokcdn|byteoversea|musical\\.ly|tiktok).+$",
        description: "Mendeteksi host video stream TikTok & ByteDance CDN.",
    },
    Layer7Preset {
        id: "youtube_video",
        name: "YouTube-Video",
        category: "Streaming Media",
        regexp: "^.+(youtube|googlevideo|ytimg).+$",
        description: "Mendeteksi paket stream video YouTube & Google Video.",
    },
    Layer7Preset {
        id: "netflix",
        name: "Netflix-CDN",
        category: "Streaming Media",
        regexp: "^.+(netflix|nflxvideo|nflximg|nflxext).+$",
        description: "Mendeteksi request stream video Netflix.",
    },
    Layer7Preset {
        id: "speedtest",
        name: "Speedtest-Ookla",
        category: "Diagnostic Bypass / Shaping",
        regexp: "^.+(speedtest|ooklaserver).+$",
        description: "Mendeteksi paket uji kecepatan Speedtest Ookla.",
    },
];

/// GET /api/v1/firewall/layer7 - Daftar seluruh rule Layer 7 Protocol (/ip/firewall/layer7-protocol/print)
pub async fn list_layer7(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client
        .run(build_command("/ip/firewall/layer7-protocol/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();

    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/firewall/layer7/add - Tambah Layer 7 Protocol regex baru
pub async fn add_layer7(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddLayer7Req>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_safe_identifier(&req.name, "layer7 name")?;

    let mut args = vec![("name", req.name.as_str()), ("regexp", req.regexp.as_str())];
    if let Some(ref c) = req.comment {
        args.push(("comment", c.as_str()));
    }

    client.run(build_command("/ip/firewall/layer7-protocol/add", args)).await?;

    Ok(Json(json!({
        "success": true,
        "name": req.name,
        "regexp": req.regexp,
        "message": format!("Layer 7 Protocol '{}' berhasil ditambahkan!", req.name)
    })))
}

/// POST /api/v1/firewall/layer7/remove - Hapus Layer 7 Protocol berdasarkan nama atau ID
pub async fn remove_layer7(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RemoveLayer7Req>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    if let Some(ref id) = req.id {
        client.run(build_command("/ip/firewall/layer7-protocol/remove", [(".id", id.as_str())])).await?;
        return Ok(Json(json!({ "success": true, "message": format!("Layer 7 ID '{}' berhasil dihapus", id) })));
    }

    if let Some(ref name) = req.name {
        let rows = client
            .run(build_command("/ip/firewall/layer7-protocol/print", [("?name", name.as_str())]))
            .await?;
        if let Some(first) = rows.first() {
            if let Some(id) = first.get(".id") {
                client.run(build_command("/ip/firewall/layer7-protocol/remove", [(".id", id)])).await?;
                return Ok(Json(json!({ "success": true, "message": format!("Layer 7 '{}' berhasil dihapus", name) })));
            }
        }
        return Err(ApiError::NotFound(format!("Layer 7 Protocol '{}' tidak ditemukan", name)));
    }

    Err(ApiError::BadRequest("Harap berikan 'id' atau 'name'".into()))
}

/// GET /api/v1/firewall/layer7/presets - Katalog preset resmi Layer 7 Protocol
pub async fn list_layer7_presets() -> Json<Value> {
    Json(json!({
        "success": true,
        "count": LAYER7_PRESETS.len(),
        "presets": LAYER7_PRESETS
    }))
}

/// POST /api/v1/firewall/layer7/deploy-preset - 1-Klik Deploy preset Layer 7 Protocol lengkap dengan opsi drop filter rule
pub async fn deploy_layer7_preset(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DeployLayer7PresetReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let preset = LAYER7_PRESETS
        .iter()
        .find(|p| p.id == req.preset_id)
        .ok_or_else(|| ApiError::NotFound(format!("Preset '{}' tidak ditemukan", req.preset_id)))?;

    // 1. Check if Layer 7 Protocol already exists
    let existing = client
        .run(build_command("/ip/firewall/layer7-protocol/print", [("?name", preset.name)]))
        .await
        .unwrap_or_default();

    if existing.is_empty() {
        client.run(build_command(
            "/ip/firewall/layer7-protocol/add",
            [
                ("name", preset.name),
                ("regexp", preset.regexp),
                ("comment", preset.description),
            ],
        )).await?;
    }

    // 2. Optionally create drop rule in firewall filter
    let mut filter_deployed = false;
    if req.create_drop_filter {
        let comment_tag = format!("[Layer7-Guard] Drop {}", preset.name);
        let existing_filter = client
            .run(build_command("/ip/firewall/filter/print", [("?comment", comment_tag.as_str())]))
            .await
            .unwrap_or_default();

        if existing_filter.is_empty() {
            client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "forward"),
                    ("layer7-protocol", preset.name),
                    ("action", "drop"),
                    ("comment", comment_tag.as_str()),
                ],
            )).await?;
            filter_deployed = true;
        }
    }

    Ok(Json(json!({
        "success": true,
        "preset_id": preset.id,
        "layer7_name": preset.name,
        "regexp": preset.regexp,
        "drop_filter_rule_created": filter_deployed,
        "message": format!("Preset Layer 7 '{}' berhasil di-deploy ke MikroTik!", preset.name)
    })))
}

/// GET /api/v1/firewall/service-ports - Daftar seluruh Application Layer Gateway (ALG) Service Ports (/ip/firewall/service-port)
pub async fn list_service_ports(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client
        .run(build_command("/ip/firewall/service-port/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();

    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "service_ports": data })))
}

/// POST /api/v1/firewall/service-port/toggle - Aktifkan atau nonaktifkan ALG Service Port (misal menonaktifkan SIP ALG untuk VoIP)
pub async fn toggle_service_port(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ToggleServicePortReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_safe_identifier(&req.name, "service port name")?;

    let rows = client
        .run(build_command("/ip/firewall/service-port/print", [("?name", req.name.as_str())]))
        .await?;

    if let Some(first) = rows.first() {
        if let Some(id) = first.get(".id") {
            let dis_val = if req.disabled { "yes" } else { "no" };
            client.run(build_command(
                "/ip/firewall/service-port/set",
                [(".id", id), ("disabled", dis_val)],
            )).await?;

            return Ok(Json(json!({
                "success": true,
                "name": req.name,
                "disabled": req.disabled,
                "message": format!("Service Port '{}' berhasil di-set disabled={}", req.name, req.disabled)
            })));
        }
    }

    Err(ApiError::NotFound(format!("Service Port '{}' tidak ditemukan", req.name)))
}

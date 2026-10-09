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
pub struct EnableAntiTetheringReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub hotspot_interface: String, // e.g. "bridge-hotspot" or "ether2"
    pub enforce_shared_users_one: Option<bool>, // default true
}

/// POST /api/v1/security/anti-tethering/enable - Deploys TTL=1 packet mangling to block
/// Wi-Fi QR code sharing, Bluetooth tethering, and USB tethering from client phones
pub async fn enable_anti_tethering(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<EnableAntiTetheringReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let comment = "[Anti-Tethering] Force TTL=1 to block Wi-Fi sharing & Bluetooth tethering";

    // 1. Injects TTL=1 rule on postrouting out-interface
    client.run(build_command(
        "/ip/firewall/mangle/add",
        [
            ("chain", "postrouting"),
            ("out-interface", req.hotspot_interface.as_str()),
            ("action", "change-ttl"),
            ("new-ttl", "set:1"),
            ("passthrough", "no"),
            ("comment", comment),
        ]
    )).await?;

    // 2. Optionally enforce shared-users=1 on all Hotspot User Profiles
    let mut profiles_updated = 0;
    if req.enforce_shared_users_one.unwrap_or(true) {
        let profiles = client.run(build_command("/ip/hotspot/user/profile/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
        for p in profiles {
            if let Some(id) = p.get(".id") {
                let _ = client.run(build_command(
                    "/ip/hotspot/user/profile/set",
                    [
                        (".id", id),
                        ("shared-users", "1"),
                    ]
                )).await;
                profiles_updated += 1;
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "hotspot_interface": req.hotspot_interface,
        "mechanism": "TTL=1 (Paket akan hangus saat HP klien mencoba mem-forward ke HP lain)",
        "shared_users_enforced": profiles_updated,
        "message": "Proteksi Anti-Tethering (Anti Bagi-Bagi Wi-Fi via QR / Bluetooth) berhasil diaktifkan"
    })))
}

/// POST /api/v1/security/anti-tethering/disable - Removes anti-tethering TTL firewall rules
pub async fn disable_anti_tethering(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/firewall/mangle/print", std::iter::empty::<(&str, &str)>())).await?;
    let mut removed = 0;

    for r in rows {
        if let Some(c) = r.get("comment") {
            if c.contains("[Anti-Tethering]") {
                if let Some(id) = r.get(".id") {
                    let _ = client.run(build_command("/ip/firewall/mangle/remove", [(".id", id)])).await;
                    removed += 1;
                }
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "rules_removed": removed,
        "message": "Proteksi Anti-Tethering berhasil dinonaktifkan"
    })))
}

/// POST /api/v1/security/anti-tethering/status - Checks whether anti-tethering protection is active
pub async fn status_anti_tethering(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/firewall/mangle/print", std::iter::empty::<(&str, &str)>())).await?;
    let active_rule = rows.into_iter().find(|r| {
        r.get("comment").is_some_and(|c| c.contains("[Anti-Tethering]"))
    });

    match active_rule {
        Some(r) => Ok(Json(json!({
            "success": true,
            "anti_tethering_active": true,
            "out_interface": r.get("out-interface"),
            "details": r.attrs
        }))),
        None => Ok(Json(json!({
            "success": true,
            "anti_tethering_active": false,
            "message": "Proteksi Anti-Tethering belum aktif di router ini"
        })))
    }
}

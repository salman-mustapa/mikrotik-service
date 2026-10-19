use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures::{Stream, StreamExt};
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
pub struct SampleTrafficReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub interface: String,
}

pub async fn all(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn sample_traffic(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SampleTrafficReq>,
) -> Result<Json<Value>, ApiError> {
    crate::routes::sanitize::validate_safe_identifier(&req.interface, "interface")?;
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/monitor-traffic", [
        ("interface", req.interface.as_str()),
        ("once", ""),
    ])).await?;
    let first = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();
    Ok(Json(json!({ "success": true, "data": first })))
}

/// GET /api/v1/interfaces/stream?interface=ether1&host=...&user=...
pub async fn stream_traffic(
    State(st): State<Arc<AppState>>,
    Query(mut q): Query<Vec<(String, String)>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let extract = |key: &str, q: &mut Vec<(String, String)>| {
        q.iter().position(|(k, _)| k == key).map(|pos| q.remove(pos).1)
    };

    let host = extract("host", &mut q);
    let port = extract("port", &mut q).and_then(|p| p.parse::<u16>().ok());
    let user = extract("user", &mut q);
    let _ = extract("password", &mut q); // Deprecated: passwords must not be passed in query strings
    let router_id = extract("router_id", &mut q);

    let iface = extract("interface", &mut q)
        .ok_or_else(|| ApiError::BadRequest("missing `interface` query parameter".into()))?;

    crate::routes::sanitize::validate_safe_identifier(&iface, "interface")?;

    let target = match (host, user) {
        (Some(h), Some(u)) => Some(RouterTarget {
            host: Some(h),
            port,
            user: Some(u),
            password: None,
        }),
        _ => None,
    };

    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let sub = client.listen(build_command("/interface/monitor-traffic", [("interface", iface.as_str())])).await?;

    let stream = sub.map(|item| {
        Ok(match item {
            Ok(s) => Event::default().json_data(&s.attrs).unwrap_or_default(),
            Err(e) => Event::default().event("error").data(e.to_string()),
        })
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[derive(Deserialize, Debug)]
pub struct CreateVlanReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub vlan_id: u16,
    pub interface: String,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct RemoveVlanReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

pub async fn vlans(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/interface/vlan/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn add_vlan(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateVlanReq>,
) -> Result<Json<Value>, ApiError> {
    crate::routes::sanitize::validate_safe_identifier(&req.name, "vlan name")?;
    crate::routes::sanitize::validate_safe_identifier(&req.interface, "interface")?;
    if req.vlan_id == 0 || req.vlan_id > 4094 {
        return Err(ApiError::BadRequest("vlan_id must be between 1 and 4094".into()));
    }
    if let Some(c) = req.comment.as_deref() {
        if c.len() > 128 || c.contains(['\r', '\n']) {
            return Err(ApiError::BadRequest("comment contains invalid characters or is too long".into()));
        }
    }

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let vlan_id_str = req.vlan_id.to_string();
    let mut args = vec![
        ("name", req.name.as_str()),
        ("vlan-id", vlan_id_str.as_str()),
        ("interface", req.interface.as_str()),
    ];
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }

    client.run(build_command("/interface/vlan/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": format!("VLAN sub-interface '{}' created on '{}'", req.name, req.interface) })))
}

pub async fn remove_vlan(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RemoveVlanReq>,
) -> Result<Json<Value>, ApiError> {
    crate::routes::sanitize::validate_safe_identifier(&req.id, "vlan id")?;
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/interface/vlan/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "VLAN interface removed" })))
}

#[derive(Deserialize, Debug)]
pub struct SyncPppoeHotspotReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub physical_interface: String,
    #[serde(default = "default_coexist_mode")]
    pub mode: String, // "vlan_trunk" or "single_port"
    pub vlan_id_hotspot: Option<u16>,
    pub vlan_id_pppoe: Option<u16>,
    pub hotspot_pool_name: Option<String>,
    pub hotspot_pool_ranges: Option<String>,
    pub pppoe_pool_name: Option<String>,
    pub pppoe_pool_ranges: Option<String>,
    #[serde(default = "default_true")]
    pub clamp_mss: bool,
}

fn default_coexist_mode() -> String {
    "vlan_trunk".into()
}

fn default_true() -> bool {
    true
}

/// POST /api/v1/network/sync-pppoe-hotspot - Orchestrates multi-VLAN trunking or single-port PPPoE + Hotspot coexistence
pub async fn sync_pppoe_hotspot(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SyncPppoeHotspotReq>,
) -> Result<Json<Value>, ApiError> {
    crate::routes::sanitize::validate_safe_identifier(&req.physical_interface, "physical_interface")?;

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut configured_items = Vec::new();

    // 1. MSS Clamping if requested
    if req.clamp_mss {
        let mangle_rows = client.run(build_command(
            "/ip/firewall/mangle/print",
            [("comment", "PPPoE-MSS-Clamp")],
        )).await.unwrap_or_default();

        if mangle_rows.is_empty() {
            client.run(build_command(
                "/ip/firewall/mangle/add",
                [
                    ("chain", "forward"),
                    ("protocol", "tcp"),
                    ("tcp-flags", "syn"),
                    ("action", "change-mss"),
                    ("new-mss", "clamp-to-pmtu"),
                    ("comment", "PPPoE-MSS-Clamp"),
                ],
            )).await?;
            configured_items.push("TCP MSS Clamping (PMTU) rule installed in firewall mangle".to_string());
        }
    }

    // 2. IP Pools creation if provided
    if let (Some(hp_name), Some(hp_ranges)) = (&req.hotspot_pool_name, &req.hotspot_pool_ranges) {
        crate::routes::sanitize::validate_safe_identifier(hp_name, "hotspot_pool_name")?;
        let _ = client.run(build_command(
            "/ip/pool/add",
            [("name", hp_name.as_str()), ("ranges", hp_ranges.as_str()), ("comment", "Hotspot Client Pool")],
        )).await;
        configured_items.push(format!("IP Pool '{}' ({}) configured for Hotspot", hp_name, hp_ranges));
    }

    if let (Some(pp_name), Some(pp_ranges)) = (&req.pppoe_pool_name, &req.pppoe_pool_ranges) {
        crate::routes::sanitize::validate_safe_identifier(pp_name, "pppoe_pool_name")?;
        let _ = client.run(build_command(
            "/ip/pool/add",
            [("name", pp_name.as_str()), ("ranges", pp_ranges.as_str()), ("comment", "PPPoE Client Pool")],
        )).await;
        configured_items.push(format!("IP Pool '{}' ({}) configured for PPPoE", pp_name, pp_ranges));
    }

    // 3. Mode Handling
    if req.mode == "vlan_trunk" {
        let v_hs = req.vlan_id_hotspot.unwrap_or(100);
        let v_ppp = req.vlan_id_pppoe.unwrap_or(200);

        if v_hs == 0 || v_hs > 4094 || v_ppp == 0 || v_ppp > 4094 {
            return Err(ApiError::BadRequest("VLAN IDs must be between 1 and 4094".into()));
        }

        let hs_vlan_name = format!("{}.{}", req.physical_interface, v_hs);
        let ppp_vlan_name = format!("{}.{}", req.physical_interface, v_ppp);
        let v_hs_str = v_hs.to_string();
        let v_ppp_str = v_ppp.to_string();

        let _ = client.run(build_command(
            "/interface/vlan/add",
            [
                ("name", hs_vlan_name.as_str()),
                ("vlan-id", v_hs_str.as_str()),
                ("interface", req.physical_interface.as_str()),
                ("comment", "Trunk Hotspot VLAN"),
            ],
        )).await;

        let _ = client.run(build_command(
            "/interface/vlan/add",
            [
                ("name", ppp_vlan_name.as_str()),
                ("vlan-id", v_ppp_str.as_str()),
                ("interface", req.physical_interface.as_str()),
                ("comment", "Trunk PPPoE VLAN"),
            ],
        )).await;

        configured_items.push(format!("VLAN sub-interface '{}' (ID {}) created on {}", hs_vlan_name, v_hs, req.physical_interface));
        configured_items.push(format!("VLAN sub-interface '{}' (ID {}) created on {}", ppp_vlan_name, v_ppp, req.physical_interface));
    } else {
        // Single port coexistence: bypass PPPoE pool from Hotspot redirection
        if let Some(pp_ranges) = &req.pppoe_pool_ranges {
            let _ = client.run(build_command(
                "/ip/hotspot/ip-binding/add",
                [
                    ("address", pp_ranges.as_str()),
                    ("type", "bypassed"),
                    ("comment", "Bypass-PPPoE-Subnet-From-Hotspot"),
                ],
            )).await;
            configured_items.push(format!("Hotspot IP Binding Bypass rule added for PPPoE subnet {}", pp_ranges));
        }
    }

    Ok(Json(json!({
        "success": true,
        "mode": req.mode,
        "interface": req.physical_interface,
        "configured_actions": configured_items,
        "message": "Sinkronisasi PPPoE dan Hotspot pada interface berhasil diaplikasikan"
    })))
}


use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use routeros_core::build_command;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::routes::topology::detect_device_vendor;
use crate::state::{AppState, RouterTarget};

#[derive(Deserialize, Debug, Default)]
pub struct FilterReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    #[serde(default)]
    pub filter: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

pub async fn servers(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/dhcp-server/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

#[derive(Deserialize, Debug)]
pub struct AddDhcpServerReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub interface: String,
    pub address_pool: Option<String>,
    pub lease_time: Option<String>,
    pub disabled: Option<bool>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SetDhcpServerReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
    pub name: Option<String>,
    pub interface: Option<String>,
    pub address_pool: Option<String>,
    pub lease_time: Option<String>,
    pub disabled: Option<bool>,
    pub comment: Option<String>,
}

/// POST /api/v1/dhcp/server/add - Tambah DHCP Server baru pada interface
pub async fn add_server(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddDhcpServerReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_safe_identifier(&req.name, "name")?;
    crate::routes::sanitize::validate_safe_identifier(&req.interface, "interface")?;
    if let Some(pool) = req.address_pool.as_deref() {
        crate::routes::sanitize::validate_safe_identifier(pool, "address_pool")?;
    }
    if let Some(lt) = req.lease_time.as_deref() {
        crate::routes::sanitize::validate_validity(lt)?;
    }

    let mut args = vec![
        ("name", req.name.as_str()),
        ("interface", req.interface.as_str()),
    ];
    if let Some(pool) = req.address_pool.as_deref() {
        args.push(("address-pool", pool));
    }
    if let Some(lt) = req.lease_time.as_deref() {
        args.push(("lease-time", lt));
    }
    let dis_str;
    if let Some(d) = req.disabled {
        dis_str = if d { "yes" } else { "no" };
        args.push(("disabled", dis_str));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ip/dhcp-server/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP server created successfully" })))
}

/// POST /api/v1/dhcp/server/set - Ubah parameter DHCP Server
pub async fn set_server(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetDhcpServerReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![(".id", req.id.as_str())];
    if let Some(ref n) = req.name {
        args.push(("name", n.as_str()));
    }
    if let Some(ref i) = req.interface {
        args.push(("interface", i.as_str()));
    }
    if let Some(ref p) = req.address_pool {
        args.push(("address-pool", p.as_str()));
    }
    if let Some(ref lt) = req.lease_time {
        args.push(("lease-time", lt.as_str()));
    }
    let dis_str;
    if let Some(d) = req.disabled {
        dis_str = if d { "yes" } else { "no" };
        args.push(("disabled", dis_str));
    }
    if let Some(ref c) = req.comment {
        args.push(("comment", c.as_str()));
    }
    client.run(build_command("/ip/dhcp-server/set", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP server updated successfully" })))
}

/// POST /api/v1/dhcp/server/remove - Hapus DHCP Server
pub async fn remove_server(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dhcp-server/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP server removed successfully" })))
}

pub async fn leases(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/dhcp-server/lease/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;

    let mut bound_count = 0usize;
    let mut dynamic_count = 0usize;
    let mut static_count = 0usize;

    let enriched: Vec<Value> = rows.into_iter().map(|r| {
        let mac = r.get("mac-address")
            .or_else(|| r.get("active-mac-address"))
            .unwrap_or("")
            .to_string();
        let ip = r.get("active-address")
            .or_else(|| r.get("address"))
            .unwrap_or("")
            .to_string();
        let host_name = r.get("host-name").unwrap_or("").to_string();
        let status = r.get("status").unwrap_or("waiting").to_string();
        let is_bound = status == "bound";
        let is_dynamic = r.get("dynamic").map(|v| v == "true").unwrap_or(true);
        let is_static = !is_dynamic;

        if is_bound { bound_count += 1; }
        if is_dynamic { dynamic_count += 1; } else { static_count += 1; }

        let vendor = detect_device_vendor(&mac, &host_name);

        let mut map = serde_json::Map::new();
        for (k, v) in r.attrs {
            map.insert(k, Value::String(v));
        }
        map.insert("vendor".to_string(), Value::String(vendor));
        map.insert("is_static".to_string(), Value::Bool(is_static));
        map.insert("is_online".to_string(), Value::Bool(is_bound));
        map.insert("mac_address".to_string(), Value::String(mac));
        map.insert("ip_address".to_string(), Value::String(ip));
        Value::Object(map)
    }).collect();

    let total = enriched.len();
    Ok(Json(json!({
        "success": true,
        "count": total,
        "summary": {
            "total_leases": total,
            "active_bound": bound_count,
            "dynamic_leases": dynamic_count,
            "static_leases": static_count
        },
        "data": enriched
    })))
}

#[derive(Deserialize, Debug)]
pub struct MakeStaticReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: Option<String>,
    pub address: Option<String>,
    pub mac_address: Option<String>,
}

pub async fn make_static(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<MakeStaticReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let target_id = if let Some(ref id) = req.id {
        id.clone()
    } else if let Some(ref addr) = req.address {
        let rows = client.run(build_command("/ip/dhcp-server/lease/print", [("?address", addr.as_str())])).await?;
        rows.first()
            .and_then(|r| r.get(".id").map(str::to_owned))
            .ok_or_else(|| ApiError::BadRequest(format!("No DHCP lease found for IP address '{}'", addr)))?
    } else if let Some(ref mac) = req.mac_address {
        let rows = client.run(build_command("/ip/dhcp-server/lease/print", [("?mac-address", mac.as_str())])).await?;
        rows.first()
            .and_then(|r| r.get(".id").map(str::to_owned))
            .ok_or_else(|| ApiError::BadRequest(format!("No DHCP lease found for MAC '{}'", mac)))?
    } else {
        return Err(ApiError::BadRequest("Must provide 'id', 'address', or 'mac_address' to make lease static".into()));
    };

    client.run(build_command("/ip/dhcp-server/lease/make-static", [(".id", target_id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Lease successfully converted to static", "id": target_id })))
}

pub async fn remove_lease(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dhcp-server/lease/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "Lease removed" })))
}

#[derive(Deserialize, Debug)]
pub struct AddLeaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    pub mac_address: String,
    pub server: Option<String>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SetLeaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
    pub address: Option<String>,
    pub mac_address: Option<String>,
    pub comment: Option<String>,
    pub disabled: Option<bool>,
}

/// POST /api/v1/dhcp/lease/add - Add static DHCP lease
pub async fn add_lease(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddLeaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_ip_or_cidr(&req.address)?;
    crate::routes::sanitize::validate_mac_address(&req.mac_address)?;
    if let Some(ref s) = req.server {
        crate::routes::sanitize::validate_safe_identifier(s, "server")?;
    }

    let mut args = vec![
        ("address", req.address.as_str()),
        ("mac-address", req.mac_address.as_str()),
    ];
    if let Some(ref s) = req.server {
        args.push(("server", s.as_str()));
    }
    if let Some(ref c) = req.comment {
        args.push(("comment", c.as_str()));
    }
    client.run(build_command("/ip/dhcp-server/lease/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP static lease added" })))
}

/// POST /api/v1/dhcp/lease/set - Update DHCP lease IP, MAC, or comment
pub async fn set_lease(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetLeaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    if let Some(ref a) = req.address {
        crate::routes::sanitize::validate_ip_or_cidr(a)?;
    }
    if let Some(ref m) = req.mac_address {
        crate::routes::sanitize::validate_mac_address(m)?;
    }

    let mut args = vec![(".id", req.id.as_str())];
    if let Some(ref a) = req.address {
        args.push(("address", a.as_str()));
    }
    if let Some(ref m) = req.mac_address {
        args.push(("mac-address", m.as_str()));
    }
    if let Some(ref c) = req.comment {
        args.push(("comment", c.as_str()));
    }
    let dis_str;
    if let Some(d) = req.disabled {
        dis_str = if d { "yes" } else { "no" };
        args.push(("disabled", dis_str));
    }
    client.run(build_command("/ip/dhcp-server/lease/set", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP lease updated" })))
}

#[derive(Deserialize, Debug)]
pub struct AddDhcpNetworkReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    pub gateway: Option<String>,
    pub dns_server: Option<String>,
    pub domain: Option<String>,
    pub comment: Option<String>,
}

pub async fn networks(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/dhcp-server/network/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn add_network(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddDhcpNetworkReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_ip_or_cidr(&req.address)?;
    if let Some(gw) = req.gateway.as_deref() {
        crate::routes::sanitize::validate_ip_or_cidr(gw)?;
    }
    if let Some(dns) = req.dns_server.as_deref() {
        for ip in dns.split(',') {
            crate::routes::sanitize::validate_ip_or_cidr(ip.trim())?;
        }
    }

    let mut args = vec![("address", req.address.as_str())];
    if let Some(gw) = req.gateway.as_deref() {
        args.push(("gateway", gw));
    }
    if let Some(dns) = req.dns_server.as_deref() {
        args.push(("dns-server", dns));
    }
    if let Some(dom) = req.domain.as_deref() {
        args.push(("domain", dom));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ip/dhcp-server/network/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP network added" })))
}

pub async fn remove_network(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dhcp-server/network/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP network removed" })))
}

#[derive(Deserialize, Debug)]
pub struct AddDhcpAlertReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub interface: String,
    pub valid_server: Option<String>,
    pub alert_timeout: Option<String>,
}

/// GET or POST /api/v1/dhcp/alerts - Daftar pemantauan Rogue DHCP Server Alert
pub async fn alerts(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/dhcp-server/alert/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/dhcp/alert/add - Pasang alarm deteksi Rogue DHCP Server pada interface
pub async fn add_alert(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddDhcpAlertReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("interface", req.interface.as_str())];
    if let Some(vs) = req.valid_server.as_deref() {
        args.push(("valid-server", vs));
    }
    if let Some(at) = req.alert_timeout.as_deref() {
        args.push(("alert-timeout", at));
    }
    client.run(build_command("/ip/dhcp-server/alert/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP alert watcher added for interface" })))
}

/// POST /api/v1/dhcp/alert/remove - Hapus alarm deteksi DHCP
pub async fn remove_alert(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dhcp-server/alert/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP alert watcher removed" })))
}

#[derive(Deserialize, Debug)]
pub struct AddDhcpRelayReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub interface: String,
    pub dhcp_server: String,
    pub local_address: Option<String>,
    pub delay_threshold: Option<String>,
    pub disabled: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct SetDhcpRelayReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
    pub name: Option<String>,
    pub interface: Option<String>,
    pub dhcp_server: Option<String>,
    pub local_address: Option<String>,
    pub disabled: Option<bool>,
}

/// GET or POST /api/v1/dhcp/relays - Daftar DHCP Relay agent (/ip/dhcp-relay/print)
pub async fn relays(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/dhcp-relay/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/dhcp/relay/add - Tambah DHCP Relay agent baru (/ip/dhcp-relay/add)
pub async fn add_relay(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddDhcpRelayReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_safe_identifier(&req.name, "name")?;
    crate::routes::sanitize::validate_safe_identifier(&req.interface, "interface")?;
    crate::routes::sanitize::validate_ip_or_cidr(&req.dhcp_server)?;
    if let Some(ref la) = req.local_address {
        crate::routes::sanitize::validate_ip_or_cidr(la)?;
    }

    let mut args = vec![
        ("name", req.name.as_str()),
        ("interface", req.interface.as_str()),
        ("dhcp-server", req.dhcp_server.as_str()),
    ];
    if let Some(ref la) = req.local_address {
        args.push(("local-address", la.as_str()));
    }
    if let Some(ref dt) = req.delay_threshold {
        args.push(("delay-threshold", dt.as_str()));
    }
    let dis_str;
    if let Some(d) = req.disabled {
        dis_str = if d { "yes" } else { "no" };
        args.push(("disabled", dis_str));
    }
    client.run(build_command("/ip/dhcp-relay/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP Relay agent added successfully" })))
}

/// POST /api/v1/dhcp/relay/set - Ubah konfigurasi DHCP Relay agent (/ip/dhcp-relay/set)
pub async fn set_relay(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetDhcpRelayReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![(".id", req.id.as_str())];
    if let Some(ref n) = req.name {
        args.push(("name", n.as_str()));
    }
    if let Some(ref i) = req.interface {
        args.push(("interface", i.as_str()));
    }
    if let Some(ref ds) = req.dhcp_server {
        args.push(("dhcp-server", ds.as_str()));
    }
    if let Some(ref la) = req.local_address {
        args.push(("local-address", la.as_str()));
    }
    let dis_str;
    if let Some(d) = req.disabled {
        dis_str = if d { "yes" } else { "no" };
        args.push(("disabled", dis_str));
    }
    client.run(build_command("/ip/dhcp-relay/set", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP Relay updated successfully" })))
}

/// POST /api/v1/dhcp/relay/remove - Hapus DHCP Relay agent (/ip/dhcp-relay/remove)
pub async fn remove_relay(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dhcp-relay/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP Relay agent removed successfully" })))
}

#[derive(Deserialize, Debug)]
pub struct AddDhcpClientReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub interface: String,
    pub add_default_route: Option<String>, // "yes", "no", "special-class"
    pub use_peer_dns: Option<bool>,
    pub use_peer_ntp: Option<bool>,
    pub comment: Option<String>,
    pub disabled: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct SetDhcpClientReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
    pub interface: Option<String>,
    pub add_default_route: Option<String>,
    pub use_peer_dns: Option<bool>,
    pub use_peer_ntp: Option<bool>,
    pub comment: Option<String>,
    pub disabled: Option<bool>,
}

/// GET or POST /api/v1/dhcp/clients - Daftar DHCP Client interface (/ip/dhcp-client/print)
pub async fn clients(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/dhcp-client/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/dhcp/client/add - Tambah DHCP Client pada interface WAN (/ip/dhcp-client/add)
pub async fn add_client(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddDhcpClientReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_safe_identifier(&req.interface, "interface")?;

    let mut args = vec![("interface", req.interface.as_str())];
    if let Some(ref r) = req.add_default_route {
        args.push(("add-default-route", r.as_str()));
    }
    let dns_str;
    if let Some(d) = req.use_peer_dns {
        dns_str = if d { "yes" } else { "no" };
        args.push(("use-peer-dns", dns_str));
    }
    let ntp_str;
    if let Some(n) = req.use_peer_ntp {
        ntp_str = if n { "yes" } else { "no" };
        args.push(("use-peer-ntp", ntp_str));
    }
    let dis_str;
    if let Some(dis) = req.disabled {
        dis_str = if dis { "yes" } else { "no" };
        args.push(("disabled", dis_str));
    }
    if let Some(ref c) = req.comment {
        args.push(("comment", c.as_str()));
    }
    client.run(build_command("/ip/dhcp-client/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP Client successfully added" })))
}

/// POST /api/v1/dhcp/client/set - Ubah parameter DHCP Client (/ip/dhcp-client/set)
pub async fn set_client(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetDhcpClientReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![(".id", req.id.as_str())];
    if let Some(ref i) = req.interface {
        args.push(("interface", i.as_str()));
    }
    if let Some(ref r) = req.add_default_route {
        args.push(("add-default-route", r.as_str()));
    }
    let dns_str;
    if let Some(d) = req.use_peer_dns {
        dns_str = if d { "yes" } else { "no" };
        args.push(("use-peer-dns", dns_str));
    }
    let ntp_str;
    if let Some(n) = req.use_peer_ntp {
        ntp_str = if n { "yes" } else { "no" };
        args.push(("use-peer-ntp", ntp_str));
    }
    let dis_str;
    if let Some(dis) = req.disabled {
        dis_str = if dis { "yes" } else { "no" };
        args.push(("disabled", dis_str));
    }
    if let Some(ref c) = req.comment {
        args.push(("comment", c.as_str()));
    }
    client.run(build_command("/ip/dhcp-client/set", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP Client updated successfully" })))
}

/// POST /api/v1/dhcp/client/remove - Hapus DHCP Client (/ip/dhcp-client/remove)
pub async fn remove_client(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dhcp-client/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP Client removed successfully" })))
}

/// POST /api/v1/dhcp/client/renew - Perbarui lease IP DHCP Client (/ip/dhcp-client/renew)
pub async fn renew_client(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dhcp-client/renew", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP Client lease renewed" })))
}

/// POST /api/v1/dhcp/client/release - Lepas (release) IP DHCP Client (/ip/dhcp-client/release)
pub async fn release_client(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ip/dhcp-client/release", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "DHCP Client lease released" })))
}




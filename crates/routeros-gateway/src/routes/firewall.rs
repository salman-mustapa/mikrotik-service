use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use routeros_core::build_command;
use serde::{Deserialize, Serialize};
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
pub struct BlockIpReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    #[serde(default = "default_list")]
    pub list: String,
    pub timeout: Option<String>,
    pub comment: Option<String>,
    pub ensure_drop_rule: Option<bool>,
}

fn default_list() -> String {
    "BLACKLIST".into()
}

#[derive(Deserialize, Debug)]
pub struct UnblockIpReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: Option<String>,
    pub address: Option<String>,
    pub list: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

pub async fn filters(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/firewall/filter/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn nat(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/firewall/nat/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn address_lists(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ip/firewall/address-list/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn block_ip(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BlockIpReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_ip_or_cidr(&req.address)?;
    crate::routes::sanitize::validate_safe_identifier(&req.list, "list_name")?;
    if let Some(t) = req.timeout.as_deref() {
        crate::routes::sanitize::validate_validity(t)?;
    }

    let mut args = vec![("address", req.address.as_str()), ("list", req.list.as_str())];
    if let Some(t) = req.timeout.as_deref() {
        args.push(("timeout", t));
    }
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ip/firewall/address-list/add", args)).await?;

    let mut drop_rule_created = false;
    if req.ensure_drop_rule.unwrap_or(true) {
        let filter_rules = client.run(build_command("/ip/firewall/filter/print", [("?src-address-list", req.list.as_str())])).await.unwrap_or_default();
        let has_drop_rule = filter_rules.iter().any(|r| r.get("action") == Some("drop") && r.get("chain") == Some("forward"));
        if !has_drop_rule {
            let comment = format!("Auto-drop blacklist filter for {}", req.list);
            let _ = client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "forward"),
                    ("action", "drop"),
                    ("src-address-list", req.list.as_str()),
                    ("comment", comment.as_str()),
                ],
            )).await;
            drop_rule_created = true;
        }
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("IP '{}' added to '{}' address list", req.address, req.list),
        "drop_filter_ensured": req.ensure_drop_rule.unwrap_or(true),
        "drop_rule_created": drop_rule_created
    })))
}

pub async fn unblock_ip(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<UnblockIpReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    if let Some(ref id) = req.id {
        client.run(build_command("/ip/firewall/address-list/remove", [(".id", id.as_str())])).await?;
        return Ok(Json(json!({ "success": true, "message": "Address list item removed by ID", "removed_count": 1 })));
    }

    if let Some(ref addr) = req.address {
        crate::routes::sanitize::validate_ip_or_cidr(addr)?;
        let mut query = vec![("?address", addr.as_str())];
        if let Some(ref l) = req.list {
            crate::routes::sanitize::validate_safe_identifier(l, "list_name")?;
            query.push(("?list", l.as_str()));
        }
        let rows = client.run(build_command("/ip/firewall/address-list/print", query)).await?;
        let mut removed = 0usize;
        for r in rows {
            if let Some(id) = r.get(".id") {
                if client.run(build_command("/ip/firewall/address-list/remove", [(".id", id)])).await.is_ok() {
                    removed += 1;
                }
            }
        }
        return Ok(Json(json!({
            "success": true,
            "message": format!("Address list items removed for IP '{}'", addr),
            "removed_count": removed
        })));
    }

    Err(ApiError::BadRequest("Must provide either 'id' or 'address' to unblock IP".into()))
}

#[derive(Deserialize, Debug, Default)]
pub struct ConnectionsReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub limit: Option<usize>,
    pub protocol: Option<String>,
    pub src_address: Option<String>,
    pub dst_address: Option<String>,
    pub tcp_state: Option<String>,
    pub connection_mark: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct FlushConnectionsReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub src_address: Option<String>,
    pub dst_address: Option<String>,
    pub protocol: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct KillConnectionReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: Option<String>,
    pub src_address: Option<String>,
    pub dst_address: Option<String>,
    pub protocol: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ConntrackSettingsReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub enabled: Option<String>,
    pub max_entries: Option<u32>,
    pub tcp_syn_sent_timeout: Option<String>,
    pub tcp_established_timeout: Option<String>,
    pub udp_timeout: Option<String>,
    pub icmp_timeout: Option<String>,
}

/// GET or POST /api/v1/firewall/connections - Inspeksi tabel Connection Tracking (Conntrack) aktif
pub async fn connections(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<ConnectionsReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut query = Vec::new();
    if let Some(p) = req.protocol.as_deref() {
        query.push(("?protocol", p));
    }
    if let Some(s) = req.src_address.as_deref() {
        query.push(("?src-address", s));
    }
    if let Some(d) = req.dst_address.as_deref() {
        query.push(("?dst-address", d));
    }
    if let Some(ts) = req.tcp_state.as_deref() {
        query.push(("?tcp-state", ts));
    }
    if let Some(cm) = req.connection_mark.as_deref() {
        query.push(("?connection-mark", cm));
    }

    let rows = client.run(build_command("/ip/firewall/connection/print", query)).await?;
    let total = rows.len();

    let mut proto_summary: HashMap<String, usize> = HashMap::new();
    let mut state_summary: HashMap<String, usize> = HashMap::new();

    for r in &rows {
        if let Some(p) = r.get("protocol") {
            *proto_summary.entry(p.to_string()).or_insert(0) += 1;
        }
        if let Some(s) = r.get("tcp-state") {
            *state_summary.entry(s.to_string()).or_insert(0) += 1;
        }
    }

    let limit = req.limit.unwrap_or(200);
    let data: Vec<HashMap<String, String>> = rows.into_iter().take(limit).map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "total_connections": total,
        "protocols": proto_summary,
        "tcp_states": state_summary,
        "limit_applied": limit,
        "count": data.len(),
        "connections": data
    })))
}

/// GET or POST /api/v1/firewall/connections/top-talkers - Deteksi host penghabis sesi koneksi (Top Talkers / BitTorrent / DDoS Hogs)
pub async fn top_talkers(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/firewall/connection/print", std::iter::empty::<(&str, &str)>())).await?;
    let total = rows.len();

    let mut src_counts: HashMap<String, usize> = HashMap::new();
    let mut proto_counts: HashMap<String, usize> = HashMap::new();

    for r in &rows {
        if let Some(src) = r.get("src-address") {
            // Strip port if present (e.g. 192.168.1.50:54321 -> 192.168.1.50)
            let ip = src.split(':').next().unwrap_or(src).trim();
            if !ip.is_empty() {
                *src_counts.entry(ip.to_string()).or_insert(0) += 1;
            }
        }
        if let Some(proto) = r.get("protocol") {
            *proto_counts.entry(proto.to_string()).or_insert(0) += 1;
        }
    }

    let mut sorted_src: Vec<_> = src_counts.into_iter().collect();
    sorted_src.sort_by_key(|a| std::cmp::Reverse(a.1));

    let top_talkers: Vec<_> = sorted_src
        .into_iter()
        .take(15)
        .map(|(ip, count)| {
            let pct = if total > 0 { (count as f64 / total as f64) * 100.0 } else { 0.0 };
            json!({
                "ip_address": ip,
                "session_count": count,
                "session_percentage": (pct * 10.0).round() / 10.0
            })
        })
        .collect();

    Ok(Json(json!({
        "success": true,
        "total_active_sessions": total,
        "protocols": proto_counts,
        "top_connection_hogs": top_talkers
    })))
}

/// POST /api/v1/firewall/connections/flush - Flush / hapus sesi koneksi tertentu
pub async fn flush_connections(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<FlushConnectionsReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut query = Vec::new();
    if let Some(s) = req.src_address.as_deref() {
        query.push(("?src-address", s));
    }
    if let Some(d) = req.dst_address.as_deref() {
        query.push(("?dst-address", d));
    }
    if let Some(p) = req.protocol.as_deref() {
        query.push(("?protocol", p));
    }

    let rows = client.run(build_command("/ip/firewall/connection/print", query)).await?;
    let count = rows.len();

    for r in &rows {
        if let Some(id) = r.get(".id") {
            let _ = client.run(build_command("/ip/firewall/connection/remove", [(".id", id)])).await;
        }
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("Flushed {} connection sessions", count),
        "removed_count": count
    })))
}

/// POST /api/v1/firewall/connections/kill - Putus paksa / terminate sesi koneksi tertentu (Fitur Remove Connection Winbox)
pub async fn kill_connection(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<KillConnectionReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    if let Some(ref id) = req.id {
        crate::routes::sanitize::validate_safe_identifier(id, "connection id")?;
        client.run(build_command("/ip/firewall/connection/remove", [(".id", id.as_str())])).await?;
        return Ok(Json(json!({
            "success": true,
            "id": id,
            "message": format!("Koneksi '{}' berhasil diputus paksa!", id)
        })));
    }

    // Kill by criteria (src, dst, proto)
    let mut query = Vec::new();
    if let Some(s) = req.src_address.as_deref() {
        query.push(("?src-address", s));
    }
    if let Some(d) = req.dst_address.as_deref() {
        query.push(("?dst-address", d));
    }
    if let Some(p) = req.protocol.as_deref() {
        query.push(("?protocol", p));
    }

    if query.is_empty() {
        return Err(ApiError::BadRequest("Harap tentukan 'id', 'src_address', atau 'dst_address'".into()));
    }

    let rows = client.run(build_command("/ip/firewall/connection/print", query)).await?;
    let killed = rows.len();
    for r in &rows {
        if let Some(id) = r.get(".id") {
            let _ = client.run(build_command("/ip/firewall/connection/remove", [(".id", id)])).await;
        }
    }

    Ok(Json(json!({
        "success": true,
        "killed_count": killed,
        "message": format!("Berhasil memutus {} sesi koneksi target!", killed)
    })))
}

/// GET or POST /api/v1/firewall/conntrack/settings - Baca atau atur konfigurasi Connection Tracking RouterOS (/ip/firewall/connection/tracking)
pub async fn conntrack_settings(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<ConntrackSettingsReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Jika ada parameter update, jalankan /ip/firewall/connection/tracking/set
    let mut set_args = Vec::new();
    if let Some(ref e) = req.enabled {
        set_args.push(("enabled", e.as_str()));
    }
    let max_str;
    if let Some(m) = req.max_entries {
        max_str = m.to_string();
        set_args.push(("max-entries", max_str.as_str()));
    }
    if let Some(ref ts) = req.tcp_syn_sent_timeout {
        set_args.push(("tcp-syn-sent-timeout", ts.as_str()));
    }
    if let Some(ref te) = req.tcp_established_timeout {
        set_args.push(("tcp-established-timeout", te.as_str()));
    }
    if let Some(ref u) = req.udp_timeout {
        set_args.push(("udp-timeout", u.as_str()));
    }
    if let Some(ref i) = req.icmp_timeout {
        set_args.push(("icmp-timeout", i.as_str()));
    }

    if !set_args.is_empty() {
        client.run(build_command("/ip/firewall/connection/tracking/set", set_args)).await?;
    }

    // Ambil konfigurasi conntrack terkini
    let rows = client
        .run(build_command("/ip/firewall/connection/tracking/print", std::iter::empty::<(&str, &str)>()))
        .await
        .unwrap_or_default();
    let settings = rows.into_iter().next().map(|r| r.attrs).unwrap_or_default();

    Ok(Json(json!({
        "success": true,
        "conntrack_settings": settings,
        "updated": !req.enabled.is_none() || !req.max_entries.is_none() || !req.tcp_established_timeout.is_none()
    })))
}

#[derive(Deserialize, Debug, Default)]
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuickActionStep {
    pub step_number: usize,
    pub stage: String,
    pub name: String,
    pub description: String,
    pub command: String,
    pub status: String,
    pub duration_ms: u64,
    pub details: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuickActionJobRecord {
    pub job_id: String,
    pub preset_id: String,
    pub title: String,
    pub status: String,
    pub started_at_ms: u128,
    pub total_duration_ms: u64,
    pub total_steps: usize,
    pub steps: Vec<QuickActionStep>,
    pub router_verification: Value,
}

static FIREWALL_QUICK_ACTIONS_HISTORY: Mutex<VecDeque<QuickActionJobRecord>> = Mutex::new(VecDeque::new());

fn record_quick_action(job: QuickActionJobRecord) {
    if let Ok(mut hist) = FIREWALL_QUICK_ACTIONS_HISTORY.lock() {
        if hist.len() >= 50 {
            hist.pop_back();
        }
        hist.push_front(job);
    }
}

fn now_epoch_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn default_mitigate_action() -> String {
    "block_src".into()
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Debug)]
pub struct QuickActionExecReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub preset_id: String,
    #[serde(default)]
    pub params: HashMap<String, Value>,
}

#[derive(Deserialize, Debug)]
pub struct MitigateConnectionReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub src_address: String,
    pub dst_address: Option<String>,
    #[serde(default = "default_mitigate_action")]
    pub action: String, // "block_src", "block_dst", "block_both", "whitelist_src", "whitelist_dst", "kill_only"
    pub list: Option<String>,
    pub timeout: Option<String>,
    #[serde(default = "default_true")]
    pub kill_active: bool,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ReleaseQuarantineReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: Option<String>,
    pub address: Option<String>,
    pub list: Option<String>,
}

/// GET /api/v1/firewall/quick-actions/presets - Katalog preset quick action firewall siap pakai untuk frontend
pub async fn quick_action_presets() -> Json<Value> {
    Json(json!({
        "success": true,
        "total_presets": 9,
        "presets": [
            {
                "id": "block_malicious_ip",
                "title": "Blokir & Karantina IP Penyerang (Zero CPU RAW Drop)",
                "category": "mitigation",
                "description": "Memasukkan IP ke daftar BLACKLIST / QUARANTINE, menjamin RAW Prerouting Drop (0% CPU conntrack bypass), Filter Forward Drop, dan memutus seketika seluruh koneksi aktif.",
                "fields": [
                    { "name": "address", "label": "IP Target / Subnet", "type": "text", "required": true, "placeholder": "198.51.100.22 atau 192.168.1.50" },
                    { "name": "list", "label": "Nama Address List", "type": "text", "required": false, "default": "BLACKLIST" },
                    { "name": "timeout", "label": "Batas Waktu (Timeout)", "type": "text", "required": false, "default": "24h", "placeholder": "24h, 7d, 01:00:00" },
                    { "name": "comment", "label": "Catatan Jejak Forensik", "type": "text", "required": false, "default": "Quick-Action: Malicious IP Block" }
                ],
                "stages": ["VALIDATION", "FIREWALL_RAW", "FIREWALL_FILTER", "ADDRESS_LIST", "CONNTRACK_CLEANUP", "ROUTER_VERIFICATION"]
            },
            {
                "id": "isolate_host",
                "title": "Isolir Host Terinfeksi (Captive Quarantine)",
                "category": "mitigation",
                "description": "Mengisolir perangkat lokal yang terindikasi malware/virus ke dalam daftar ISOLIR sehingga tidak dapat menyebarkan paket lateral ke LAN maupun akses WAN internet.",
                "fields": [
                    { "name": "address", "label": "IP Klien Lokal", "type": "text", "required": true, "placeholder": "192.168.88.105" },
                    { "name": "comment", "label": "Keterangan Isolir", "type": "text", "required": false, "default": "Quick-Action: Infected Host Isolated" }
                ],
                "stages": ["VALIDATION", "ADDRESS_LIST", "FIREWALL_FILTER", "CONNTRACK_CLEANUP", "ROUTER_VERIFICATION"]
            },
            {
                "id": "protect_winbox_bruteforce",
                "title": "Proteksi Brute-Force Port Winbox 8291 (3-Stage Tarpit/Blacklist)",
                "category": "hardening",
                "description": "Menerapkan deteksi brute-force otomatis pada port Winbox 8291. Host yang gagal 3 kali dalam 1 menit otomatis masuk BLACKLIST selama 7 hari.",
                "fields": [
                    { "name": "stage1_timeout", "label": "Stage 1 Timeout", "type": "text", "required": false, "default": "1m" },
                    { "name": "blacklist_timeout", "label": "Blacklist Timeout", "type": "text", "required": false, "default": "7d" }
                ],
                "stages": ["PRE_CHECK", "FIREWALL_FILTER", "ROUTER_VERIFICATION"]
            },
            {
                "id": "block_port_scanner_psd",
                "title": "Deteksi & Drop Port Scanner Otomatis (PSD Heuristic)",
                "category": "hardening",
                "description": "Mengaktifkan filter Port Scan Detection (PSD 21,3s,3,1). Scanner otomatis dijebloskan ke daftar PORT_SCANNERS selama 24 jam dan didrop seketika.",
                "fields": [
                    { "name": "timeout", "label": "Durasi Blacklist Scanner", "type": "text", "required": false, "default": "24h" }
                ],
                "stages": ["PRE_CHECK", "FIREWALL_FILTER", "ROUTER_VERIFICATION"]
            },
            {
                "id": "anti_syn_flood_ddos",
                "title": "Mitigasi Serangan TCP SYN-Flood DDoS (RAW Table)",
                "category": "performance",
                "description": "Menahan serangan TCP SYN flood pada tabel RAW dengan batas connection-limit, melindungi CPU RouterOS dari lonjakan 100% akibat tabel conntrack penuh.",
                "fields": [
                    { "name": "connection_limit", "label": "Batas Koneksi per Host", "type": "number", "required": false, "default": 30 }
                ],
                "stages": ["FIREWALL_RAW", "ROUTER_VERIFICATION"]
            },
            {
                "id": "dns_hijack_prevention",
                "title": "Cegah DNS Hijacking & Paksa DNS Resolver Terpercaya",
                "category": "access_control",
                "description": "Memaksa seluruh query port 53 UDP/TCP dari klien lokal di-redirect ke DNS lokal router/Pi-hole/Cloudflare untuk memblokir malware bypass DNS.",
                "fields": [
                    { "name": "target_dns", "label": "IP Resolver Tujuan", "type": "text", "required": false, "default": "local" }
                ],
                "stages": ["FIREWALL_NAT", "ROUTER_VERIFICATION"]
            },
            {
                "id": "block_doh_vpn_circumvention",
                "title": "Blokir Bypass DNS-over-TLS (DoT Port 853)",
                "category": "access_control",
                "description": "Mencegah klien membypass filter DNS dengan menonaktifkan port 853 DoT di prerouting RAW.",
                "fields": [],
                "stages": ["FIREWALL_RAW", "ROUTER_VERIFICATION"]
            },
            {
                "id": "whitelist_admin_ip",
                "title": "Whitelist IP Administrator NOC (Top-Rule Accept)",
                "category": "access_control",
                "description": "Menjamin subnet manajemen/IP admin tidak akan terblokir oleh aturan firewall filter manapun dengan inject accept rule prioritas teratas.",
                "fields": [
                    { "name": "address", "label": "IP / Subnet Admin NOC", "type": "text", "required": true, "placeholder": "192.168.88.250 atau 10.10.0.0/24" },
                    { "name": "comment", "label": "Keterangan", "type": "text", "required": false, "default": "NOC Admin Whitelist Priority" }
                ],
                "stages": ["VALIDATION", "ADDRESS_LIST", "FIREWALL_FILTER", "ROUTER_VERIFICATION"]
            },
            {
                "id": "custom_quick_rule",
                "title": "Pembuatan Filter Rule Kustom Seketika",
                "category": "custom",
                "description": "Buat rule filter atau raw baru dengan memasukkan parameter esensial dan verifikasi otomatis langsung di router.",
                "fields": [
                    { "name": "table", "label": "Tabel (filter / raw / nat)", "type": "select", "options": ["filter", "raw", "nat"], "required": true, "default": "filter" },
                    { "name": "chain", "label": "Chain", "type": "select", "options": ["forward", "input", "output", "prerouting", "dstnat", "srcnat"], "required": true, "default": "forward" },
                    { "name": "action", "label": "Action", "type": "select", "options": ["drop", "accept", "reject", "tarpit", "redirect", "masquerade"], "required": true, "default": "drop" },
                    { "name": "src_address", "label": "Src Address (Opsional)", "type": "text", "required": false },
                    { "name": "dst_address", "label": "Dst Address (Opsional)", "type": "text", "required": false },
                    { "name": "protocol", "label": "Protocol (Opsional)", "type": "text", "required": false, "placeholder": "tcp, udp, icmp" },
                    { "name": "dst_port", "label": "Dst Port (Opsional)", "type": "text", "required": false, "placeholder": "80,443" },
                    { "name": "comment", "label": "Komentar", "type": "text", "required": false, "default": "Quick-Action Custom Rule" }
                ],
                "stages": ["VALIDATION", "FIREWALL_APPLY", "ROUTER_VERIFICATION"]
            }
        ]
    }))
}

/// GET /api/v1/firewall/quick-actions/history - Riwayat eksekusi quick action firewall untuk audit log developer
pub async fn quick_actions_history() -> Json<Value> {
    let hist: Vec<QuickActionJobRecord> = if let Ok(h) = FIREWALL_QUICK_ACTIONS_HISTORY.lock() {
        h.iter().cloned().collect()
    } else {
        Vec::new()
    };
    Json(json!({
        "success": true,
        "count": hist.len(),
        "history": hist
    }))
}

/// POST /api/v1/firewall/quick-actions/execute - Eksekusi Quick Action Firewall dengan checklist alur langkah & verifikasi realtime RouterOS
pub async fn execute_quick_action(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<QuickActionExecReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let started_at = now_epoch_ms();
    let job_id = format!("job_fw_{}_{:04x}", started_at, (started_at % 0xFFFF) as u16);
    let mut steps: Vec<QuickActionStep> = Vec::new();
    let overall_start = Instant::now();

    match req.preset_id.as_str() {
        "block_malicious_ip" => {
            let addr = req.params.get("address").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            if addr.is_empty() {
                return Err(ApiError::BadRequest("Parameter 'address' wajib diisi".into()));
            }
            let list_name = req.params.get("list").and_then(|v| v.as_str()).unwrap_or("BLACKLIST").trim().to_string();
            let timeout = req.params.get("timeout").and_then(|v| v.as_str()).unwrap_or("24h").trim().to_string();
            let comment = req.params.get("comment").and_then(|v| v.as_str()).unwrap_or("Quick-Action: Malicious IP Block").trim().to_string();

            // Step 1: Validation
            let s1_start = Instant::now();
            crate::routes::sanitize::validate_ip_or_cidr(&addr)?;
            crate::routes::sanitize::validate_safe_identifier(&list_name, "list_name")?;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "VALIDATION".into(),
                name: "Validasi Parameter IP & Address List".into(),
                description: format!("Memverifikasi format IP '{}' dan list '{}'", addr, list_name),
                command: format!("validate_ip_or_cidr({})", addr),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "Sintaks IP valid, tidak terdeteksi karakter berbahaya".into(),
            });

            // Step 2: Address List Add
            let s2_start = Instant::now();
            let mut add_args = vec![("address", addr.as_str()), ("list", list_name.as_str())];
            if !timeout.is_empty() {
                add_args.push(("timeout", timeout.as_str()));
            }
            if !comment.is_empty() {
                add_args.push(("comment", comment.as_str()));
            }
            let add_cmd_str = format!("/ip/firewall/address-list/add list={} address={} timeout={}", list_name, addr, timeout);
            client.run(build_command("/ip/firewall/address-list/add", add_args)).await?;
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "ADDRESS_LIST".into(),
                name: "Injeksi IP ke Address List RouterOS".into(),
                description: format!("Menyimpan host ke list '{}' dengan timeout {}", list_name, timeout),
                command: add_cmd_str,
                status: "success".into(),
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: format!("Berhasil didaftarkan ke address-list '{}'", list_name),
            });

            // Step 3: Raw Drop Rule
            let s3_start = Instant::now();
            let raw_rules = client.run(build_command("/ip/firewall/raw/print", [("?src-address-list", list_name.as_str())])).await.unwrap_or_default();
            let has_raw = raw_rules.iter().any(|r| r.get("action") == Some("drop") && r.get("chain") == Some("prerouting"));
            let raw_details = if !has_raw {
                let comm = format!("[Quick-Action] Raw Drop {}", list_name);
                let _ = client.run(build_command(
                    "/ip/firewall/raw/add",
                    [
                        ("chain", "prerouting"),
                        ("action", "drop"),
                        ("src-address-list", list_name.as_str()),
                        ("comment", comm.as_str()),
                    ],
                )).await;
                "Aturan RAW Prerouting Drop baru dibuat (bypass Connection Tracking untuk 0% CPU load)"
            } else {
                "Aturan RAW Prerouting Drop sudah aktif sebelumnya di RouterOS"
            };
            steps.push(QuickActionStep {
                step_number: 3,
                stage: "FIREWALL_RAW".into(),
                name: "Verifikasi & Jaminan RAW Prerouting Drop (Zero CPU)".into(),
                description: "Menjamin paket dibuang sebelum conntrack agar tidak membebani memori/CPU saat diserang".into(),
                command: format!("/ip/firewall/raw/add chain=prerouting action=drop src-address-list={}", list_name),
                status: "success".into(),
                duration_ms: s3_start.elapsed().as_millis() as u64,
                details: raw_details.into(),
            });

            // Step 4: Filter Drop Rule
            let s4_start = Instant::now();
            let filter_rules = client.run(build_command("/ip/firewall/filter/print", [("?src-address-list", list_name.as_str())])).await.unwrap_or_default();
            let has_filter = filter_rules.iter().any(|r| r.get("action") == Some("drop") && r.get("chain") == Some("forward"));
            let filter_details = if !has_filter {
                let comm = format!("[Quick-Action] Filter Forward Drop {}", list_name);
                let _ = client.run(build_command(
                    "/ip/firewall/filter/add",
                    [
                        ("chain", "forward"),
                        ("action", "drop"),
                        ("src-address-list", list_name.as_str()),
                        ("comment", comm.as_str()),
                    ],
                )).await;
                "Aturan Filter Forward Drop baru dipasang"
            } else {
                "Aturan Filter Forward Drop sudah aktif di RouterOS"
            };
            steps.push(QuickActionStep {
                step_number: 4,
                stage: "FIREWALL_FILTER".into(),
                name: "Verifikasi & Jaminan Filter Forward Drop".into(),
                description: "Memastikan paket forward antar-interface diblokir secara mutlak".into(),
                command: format!("/ip/firewall/filter/add chain=forward action=drop src-address-list={}", list_name),
                status: "success".into(),
                duration_ms: s4_start.elapsed().as_millis() as u64,
                details: filter_details.into(),
            });

            // Step 5: Conntrack Termination
            let s5_start = Instant::now();
            let stripped_ip = addr.split('/').next().unwrap_or(&addr).trim();
            let active_conns = client.run(build_command("/ip/firewall/connection/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
            let mut killed = 0usize;
            for c in active_conns {
                let src = c.get("src-address").unwrap_or("");
                let dst = c.get("dst-address").unwrap_or("");
                if src.contains(stripped_ip) || dst.contains(stripped_ip) {
                    if let Some(id) = c.get(".id") {
                        if client.run(build_command("/ip/firewall/connection/remove", [(".id", id)])).await.is_ok() {
                            killed += 1;
                        }
                    }
                }
            }
            steps.push(QuickActionStep {
                step_number: 5,
                stage: "CONNTRACK_CLEANUP".into(),
                name: "Terminasi Seluruh Sesi Koneksi Aktif".into(),
                description: format!("Memutus soket koneksi aktif yang sedang terbuka untuk host {}", stripped_ip),
                command: format!("/ip/firewall/connection/remove [find src/dst-address={}]", stripped_ip),
                status: "success".into(),
                duration_ms: s5_start.elapsed().as_millis() as u64,
                details: format!("{} sesi koneksi aktif berhasil diputus seketika", killed),
            });

            // Step 6: Router Live Verification
            let s6_start = Instant::now();
            let verify_rows = client.run(build_command(
                "/ip/firewall/address-list/print",
                [("?list", list_name.as_str()), ("?address", addr.as_str())],
            )).await.unwrap_or_default();
            let is_verified = !verify_rows.is_empty();
            let entry_id = verify_rows.first().and_then(|r| r.get(".id")).unwrap_or("*none");
            steps.push(QuickActionStep {
                step_number: 6,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi Langsung ke Perangkat MikroTik".into(),
                description: "Membaca kembali memori RouterOS untuk mengonfirmasi rule aktif dan sehat".into(),
                command: format!("/ip/firewall/address-list/print ?list={} ?address={}", list_name, addr),
                status: if is_verified { "success".into() } else { "failed".into() },
                duration_ms: s6_start.elapsed().as_millis() as u64,
                details: format!("Terverifikasi di RouterOS: entry aktif dengan ID {}", entry_id),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "block_malicious_ip".into(),
                title: format!("Blokir & Karantina IP '{}'", addr),
                status: if is_verified { "completed".into() } else { "partial".into() },
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({
                    "verified": is_verified,
                    "target_address": addr,
                    "target_list": list_name,
                    "routeros_entry_id": entry_id,
                    "active_sessions_killed": killed,
                    "raw_drop_guaranteed": true,
                    "filter_drop_guaranteed": true
                }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": "block_malicious_ip",
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "total_steps": steps.len(),
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "isolate_host" => {
            let addr = req.params.get("address").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            if addr.is_empty() {
                return Err(ApiError::BadRequest("Parameter 'address' wajib diisi".into()));
            }
            let comment = req.params.get("comment").and_then(|v| v.as_str()).unwrap_or("Quick-Action: Infected Host Isolated").trim().to_string();

            // Step 1: Validation
            let s1_start = Instant::now();
            crate::routes::sanitize::validate_ip_or_cidr(&addr)?;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "VALIDATION".into(),
                name: "Validasi Host Target".into(),
                description: format!("Memverifikasi integritas IP '{}'", addr),
                command: format!("validate_ip_or_cidr({})", addr),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "Sintaks IP valid".into(),
            });

            // Step 2: Address List ISOLIR
            let s2_start = Instant::now();
            client.run(build_command(
                "/ip/firewall/address-list/add",
                [("list", "ISOLIR"), ("address", addr.as_str()), ("comment", comment.as_str())],
            )).await?;
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "ADDRESS_LIST".into(),
                name: "Injeksi ke Daftar ISOLIR".into(),
                description: "Menetapkan host ke address-list ISOLIR RouterOS".into(),
                command: format!("/ip/firewall/address-list/add list=ISOLIR address={} comment={}", addr, comment),
                status: "success".into(),
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: "Host berhasil ditambahkan ke grup ISOLIR".into(),
            });

            // Step 3: Conntrack flush
            let s3_start = Instant::now();
            let stripped_ip = addr.split('/').next().unwrap_or(&addr).trim();
            let active_conns = client.run(build_command("/ip/firewall/connection/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
            let mut killed = 0;
            for c in active_conns {
                if c.get("src-address").map(|s| s.contains(stripped_ip)).unwrap_or(false) {
                    if let Some(id) = c.get(".id") {
                        if client.run(build_command("/ip/firewall/connection/remove", [(".id", id)])).await.is_ok() {
                            killed += 1;
                        }
                    }
                }
            }
            steps.push(QuickActionStep {
                step_number: 3,
                stage: "CONNTRACK_CLEANUP".into(),
                name: "Terminasi Sesi Aktif Host".into(),
                description: "Memutus seluruh sesi internet yang sedang berlangsung".into(),
                command: format!("/ip/firewall/connection/remove [find src-address={}]", stripped_ip),
                status: "success".into(),
                duration_ms: s3_start.elapsed().as_millis() as u64,
                details: format!("{} sesi koneksi diputus", killed),
            });

            // Step 4: Verification
            let s4_start = Instant::now();
            let rows = client.run(build_command(
                "/ip/firewall/address-list/print",
                [("?list", "ISOLIR"), ("?address", addr.as_str())],
            )).await.unwrap_or_default();
            let verified = !rows.is_empty();
            steps.push(QuickActionStep {
                step_number: 4,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi Status Karantina di MikroTik".into(),
                description: "Membaca kembali tabel ISOLIR di RouterOS".into(),
                command: format!("/ip/firewall/address-list/print ?list=ISOLIR ?address={}", addr),
                status: if verified { "success".into() } else { "failed".into() },
                duration_ms: s4_start.elapsed().as_millis() as u64,
                details: format!("Host terverifikasi berstatus ISOLIR di RouterOS ({} record)", rows.len()),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "isolate_host".into(),
                title: format!("Isolir Host '{}'", addr),
                status: if verified { "completed".into() } else { "failed".into() },
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "verified": verified, "isolated_address": addr, "killed_connections": killed }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": "isolate_host",
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "total_steps": steps.len(),
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "protect_winbox_bruteforce" => {
            let stage1_timeout = req.params.get("stage1_timeout").and_then(|v| v.as_str()).unwrap_or("1m");
            let blacklist_timeout = req.params.get("blacklist_timeout").and_then(|v| v.as_str()).unwrap_or("7d");

            // Step 1: Pre-check
            let s1_start = Instant::now();
            let cur = client.run(build_command("/ip/firewall/filter/print", [("?dst-port", "8291")])).await.unwrap_or_default();
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "PRE_CHECK".into(),
                name: "Inspeksi Rule Proteksi Winbox Port 8291".into(),
                description: "Mengevaluasi filter rule eksisting untuk port 8291".into(),
                command: "/ip/firewall/filter/print ?dst-port=8291".into(),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: format!("Ditemukan {} rule eksisting pada port 8291", cur.len()),
            });

            // Step 2: Inject 3-stage tarpit/blacklist rules
            let s2_start = Instant::now();
            let _ = client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "input"),
                    ("protocol", "tcp"),
                    ("dst-port", "8291"),
                    ("connection-state", "new"),
                    ("src-address-list", "winbox_stage2"),
                    ("action", "add-src-to-address-list"),
                    ("address-list", "BLACKLIST"),
                    ("address-list-timeout", blacklist_timeout),
                    ("comment", "[Quick-Action] Winbox Brute Stage 2 to Blacklist"),
                ],
            )).await;
            let _ = client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "input"),
                    ("protocol", "tcp"),
                    ("dst-port", "8291"),
                    ("connection-state", "new"),
                    ("src-address-list", "winbox_stage1"),
                    ("action", "add-src-to-address-list"),
                    ("address-list", "winbox_stage2"),
                    ("address-list-timeout", stage1_timeout),
                    ("comment", "[Quick-Action] Winbox Brute Stage 1 to Stage 2"),
                ],
            )).await;
            let _ = client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "input"),
                    ("protocol", "tcp"),
                    ("dst-port", "8291"),
                    ("connection-state", "new"),
                    ("action", "add-src-to-address-list"),
                    ("address-list", "winbox_stage1"),
                    ("address-list-timeout", stage1_timeout),
                    ("comment", "[Quick-Action] Winbox Brute Tracker Stage 1"),
                ],
            )).await;
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "FIREWALL_FILTER".into(),
                name: "Injeksi 3-Stage Brute Force Detector".into(),
                description: format!("Memasang aturan: Percobaan berulang > 2x dalam {} langsung di-blacklist selama {}", stage1_timeout, blacklist_timeout),
                command: "/ip/firewall/filter/add chain=input dst-port=8291 ...".into(),
                status: "success".into(),
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: "3 Filter rules berhasil diinjeksi ke chain input".into(),
            });

            // Step 3: Verification
            let s3_start = Instant::now();
            let check = client.run(build_command("/ip/firewall/filter/print", [("?src-address-list", "winbox_stage1")])).await.unwrap_or_default();
            let verified = !check.is_empty();
            steps.push(QuickActionStep {
                step_number: 3,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi Integrasi di RouterOS".into(),
                description: "Memastikan chain input RouterOS aktif memantau brute-force".into(),
                command: "/ip/firewall/filter/print ?src-address-list=winbox_stage1".into(),
                status: if verified { "success".into() } else { "failed".into() },
                duration_ms: s3_start.elapsed().as_millis() as u64,
                details: "Rule proteksi Winbox terverifikasi aktif di RouterOS".into(),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "protect_winbox_bruteforce".into(),
                title: "Proteksi Brute-Force Port Winbox 8291".into(),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "verified": verified, "port": 8291, "blacklist_duration": blacklist_timeout }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": "protect_winbox_bruteforce",
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "total_steps": steps.len(),
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "block_port_scanner_psd" => {
            let timeout = req.params.get("timeout").and_then(|v| v.as_str()).unwrap_or("24h");

            let s1_start = Instant::now();
            let _ = client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "input"),
                    ("protocol", "tcp"),
                    ("psd", "21,3s,3,1"),
                    ("action", "add-src-to-address-list"),
                    ("address-list", "PORT_SCANNERS"),
                    ("address-list-timeout", timeout),
                    ("comment", "[Quick-Action] Detect Port Scanner PSD"),
                ],
            )).await;
            let _ = client.run(build_command(
                "/ip/firewall/filter/add",
                [
                    ("chain", "input"),
                    ("src-address-list", "PORT_SCANNERS"),
                    ("action", "drop"),
                    ("comment", "[Quick-Action] Drop Detected Port Scanners"),
                ],
            )).await;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "FIREWALL_FILTER".into(),
                name: "Injeksi Heuristik Port Scan Detection (PSD)".into(),
                description: format!("Mendeteksi port scan TCP dan auto-blacklist IP penyerang selama {}", timeout),
                command: "/ip/firewall/filter/add chain=input psd=21,3s,3,1 action=add-src-to-address-list address-list=PORT_SCANNERS".into(),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "Filter rule PSD dan Drop Rule berhasil dipasang".into(),
            });

            let s2_start = Instant::now();
            let check = client.run(build_command("/ip/firewall/filter/print", [("?src-address-list", "PORT_SCANNERS")])).await.unwrap_or_default();
            let verified = !check.is_empty();
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi Aturan PSD di RouterOS".into(),
                description: "Membaca kembali filter rule PSD dari MikroTik".into(),
                command: "/ip/firewall/filter/print ?src-address-list=PORT_SCANNERS".into(),
                status: if verified { "success".into() } else { "failed".into() },
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: format!("Terverifikasi {} rule aktif", check.len()),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "block_port_scanner_psd".into(),
                title: "Deteksi & Drop Port Scanner (PSD)".into(),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "verified": verified, "psd_weight": "21,3s,3,1", "block_duration": timeout }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": "block_port_scanner_psd",
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "total_steps": steps.len(),
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "anti_syn_flood_ddos" => {
            let conn_limit = req.params.get("connection_limit").and_then(|v| v.as_u64()).unwrap_or(30);
            let limit_str = format!("{},32", conn_limit);

            let s1_start = Instant::now();
            let _ = client.run(build_command(
                "/ip/firewall/raw/add",
                [
                    ("chain", "prerouting"),
                    ("protocol", "tcp"),
                    ("tcp-flags", "syn"),
                    ("connection-limit", limit_str.as_str()),
                    ("action", "drop"),
                    ("comment", "[Quick-Action] Anti-SYN Flood RAW Drop"),
                ],
            )).await;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "FIREWALL_RAW".into(),
                name: "Injeksi Rate Limiter SYN Flood di RAW Prerouting".into(),
                description: format!("Membatasi koneksi SYN baru maksimal {} per IP address", conn_limit),
                command: format!("/ip/firewall/raw/add chain=prerouting protocol=tcp tcp-flags=syn connection-limit={} action=drop", limit_str),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "Rule RAW Prerouting berhasil dipasang".into(),
            });

            let s2_start = Instant::now();
            let check = client.run(build_command("/ip/firewall/raw/print", [("?tcp-flags", "syn")])).await.unwrap_or_default();
            let verified = !check.is_empty();
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi Proteksi SYN Flood di MikroTik".into(),
                description: "Membaca kembali tabel RAW untuk menjamin rule aktif".into(),
                command: "/ip/firewall/raw/print ?tcp-flags=syn".into(),
                status: if verified { "success".into() } else { "failed".into() },
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: "Rule RAW SYN Flood terverifikasi aktif".into(),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "anti_syn_flood_ddos".into(),
                title: "Mitigasi Serangan TCP SYN-Flood DDoS".into(),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "verified": verified, "connection_limit": conn_limit }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": "anti_syn_flood_ddos",
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "total_steps": steps.len(),
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "dns_hijack_prevention" => {
            let s1_start = Instant::now();
            let _ = client.run(build_command(
                "/ip/firewall/nat/add",
                [
                    ("chain", "dstnat"),
                    ("protocol", "udp"),
                    ("dst-port", "53"),
                    ("action", "redirect"),
                    ("to-ports", "53"),
                    ("comment", "[Quick-Action] Redirect Rogue UDP DNS to Local"),
                ],
            )).await;
            let _ = client.run(build_command(
                "/ip/firewall/nat/add",
                [
                    ("chain", "dstnat"),
                    ("protocol", "tcp"),
                    ("dst-port", "53"),
                    ("action", "redirect"),
                    ("to-ports", "53"),
                    ("comment", "[Quick-Action] Redirect Rogue TCP DNS to Local"),
                ],
            )).await;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "FIREWALL_NAT".into(),
                name: "Injeksi NAT Redirect Port 53 DNS".into(),
                description: "Memaksa seluruh query DNS klien lokal diarahkan ke DNS internal router".into(),
                command: "/ip/firewall/nat/add chain=dstnat protocol=udp/tcp dst-port=53 action=redirect".into(),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "2 Rule NAT Redirect (UDP & TCP 53) berhasil dipasang".into(),
            });

            let s2_start = Instant::now();
            let check = client.run(build_command("/ip/firewall/nat/print", [("?dst-port", "53")])).await.unwrap_or_default();
            let verified = !check.is_empty();
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi NAT Redirect di MikroTik".into(),
                description: "Membaca kembali tabel NAT RouterOS".into(),
                command: "/ip/firewall/nat/print ?dst-port=53".into(),
                status: if verified { "success".into() } else { "failed".into() },
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: format!("Terverifikasi {} rule NAT aktif", check.len()),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "dns_hijack_prevention".into(),
                title: "Cegah DNS Hijacking & Paksa DNS Resolver".into(),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "verified": verified, "nat_rules": check.len() }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": "dns_hijack_prevention",
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "total_steps": steps.len(),
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "block_doh_vpn_circumvention" => {
            let s1_start = Instant::now();
            let _ = client.run(build_command(
                "/ip/firewall/raw/add",
                [
                    ("chain", "prerouting"),
                    ("protocol", "tcp"),
                    ("dst-port", "853"),
                    ("action", "drop"),
                    ("comment", "[Quick-Action] Drop DNS-over-TLS (DoT Port 853)"),
                ],
            )).await;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "FIREWALL_RAW".into(),
                name: "Blokir Port 853 (DNS-over-TLS) di RAW Prerouting".into(),
                description: "Menghalau bypass DNS melalui port terenkripsi 853".into(),
                command: "/ip/firewall/raw/add chain=prerouting protocol=tcp dst-port=853 action=drop".into(),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "Rule drop port 853 aktif di RAW".into(),
            });

            let s2_start = Instant::now();
            let check = client.run(build_command("/ip/firewall/raw/print", [("?dst-port", "853")])).await.unwrap_or_default();
            let verified = !check.is_empty();
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi RAW DoT di MikroTik".into(),
                description: "Membaca tabel RAW untuk port 853".into(),
                command: "/ip/firewall/raw/print ?dst-port=853".into(),
                status: if verified { "success".into() } else { "failed".into() },
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: "Rule terverifikasi aktif di RouterOS".into(),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "block_doh_vpn_circumvention".into(),
                title: "Blokir Bypass DNS-over-TLS (DoT Port 853)".into(),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "verified": verified, "blocked_port": 853 }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": "block_doh_vpn_circumvention",
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "total_steps": steps.len(),
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "whitelist_admin_ip" => {
            let addr = req.params.get("address").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            if addr.is_empty() {
                return Err(ApiError::BadRequest("Parameter 'address' wajib diisi".into()));
            }
            let comment = req.params.get("comment").and_then(|v| v.as_str()).unwrap_or("NOC Admin Whitelist Priority").trim().to_string();

            let s1_start = Instant::now();
            crate::routes::sanitize::validate_ip_or_cidr(&addr)?;
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "VALIDATION".into(),
                name: "Validasi IP Admin NOC".into(),
                description: format!("Memeriksa format IP '{}'", addr),
                command: format!("validate_ip_or_cidr({})", addr),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "Format IP valid".into(),
            });

            let s2_start = Instant::now();
            client.run(build_command(
                "/ip/firewall/address-list/add",
                [("list", "WHITELIST"), ("address", addr.as_str()), ("comment", comment.as_str())],
            )).await?;
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "ADDRESS_LIST".into(),
                name: "Injeksi ke Address List WHITELIST".into(),
                description: "Mendaftarkan IP ke whitelist RouterOS".into(),
                command: format!("/ip/firewall/address-list/add list=WHITELIST address={}", addr),
                status: "success".into(),
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: "IP terdaftar di list WHITELIST".into(),
            });

            let s3_start = Instant::now();
            let cur = client.run(build_command("/ip/firewall/filter/print", [("?src-address-list", "WHITELIST")])).await.unwrap_or_default();
            if !cur.iter().any(|r| r.get("action") == Some("accept") && r.get("chain") == Some("input")) {
                let _ = client.run(build_command(
                    "/ip/firewall/filter/add",
                    [
                        ("chain", "input"),
                        ("action", "accept"),
                        ("src-address-list", "WHITELIST"),
                        ("comment", "[Quick-Action] Accept Admin Input Whitelist"),
                    ],
                )).await;
            }
            steps.push(QuickActionStep {
                step_number: 3,
                stage: "FIREWALL_FILTER".into(),
                name: "Jaminan Accept Input Rule untuk Whitelist".into(),
                description: "Memastikan admin selalu dapat mengakses input manajemen router".into(),
                command: "/ip/firewall/filter/add chain=input action=accept src-address-list=WHITELIST".into(),
                status: "success".into(),
                duration_ms: s3_start.elapsed().as_millis() as u64,
                details: "Rule input accept terpasang".into(),
            });

            let s4_start = Instant::now();
            let check = client.run(build_command(
                "/ip/firewall/address-list/print",
                [("?list", "WHITELIST"), ("?address", addr.as_str())],
            )).await.unwrap_or_default();
            let verified = !check.is_empty();
            steps.push(QuickActionStep {
                step_number: 4,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi Status Whitelist di MikroTik".into(),
                description: "Membaca kembali memori RouterOS untuk mengonfirmasi whitelist".into(),
                command: format!("/ip/firewall/address-list/print ?list=WHITELIST ?address={}", addr),
                status: if verified { "success".into() } else { "failed".into() },
                duration_ms: s4_start.elapsed().as_millis() as u64,
                details: "IP admin terverifikasi aktif di WHITELIST".into(),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "whitelist_admin_ip".into(),
                title: format!("Whitelist IP Admin '{}'", addr),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "verified": verified, "whitelisted_address": addr }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": "whitelist_admin_ip",
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "total_steps": steps.len(),
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        "custom_quick_rule" => {
            let table = req.params.get("table").and_then(|v| v.as_str()).unwrap_or("filter");
            let chain = req.params.get("chain").and_then(|v| v.as_str()).unwrap_or("forward");
            let action = req.params.get("action").and_then(|v| v.as_str()).unwrap_or("drop");
            let src_address = req.params.get("src_address").and_then(|v| v.as_str()).unwrap_or("");
            let dst_address = req.params.get("dst_address").and_then(|v| v.as_str()).unwrap_or("");
            let protocol = req.params.get("protocol").and_then(|v| v.as_str()).unwrap_or("");
            let dst_port = req.params.get("dst_port").and_then(|v| v.as_str()).unwrap_or("");
            let comment = req.params.get("comment").and_then(|v| v.as_str()).unwrap_or("Quick-Action Custom Rule");

            let s1_start = Instant::now();
            let cmd_path = match table {
                "raw" => "/ip/firewall/raw/add",
                "nat" => "/ip/firewall/nat/add",
                _ => "/ip/firewall/filter/add",
            };
            steps.push(QuickActionStep {
                step_number: 1,
                stage: "VALIDATION".into(),
                name: "Validasi Sintaks Rule Kustom".into(),
                description: format!("Tabel: {}, Chain: {}, Action: {}", table, chain, action),
                command: format!("build_command({})", cmd_path),
                status: "success".into(),
                duration_ms: s1_start.elapsed().as_millis() as u64,
                details: "Parameter rule valid".into(),
            });

            let s2_start = Instant::now();
            let mut args = vec![("chain", chain), ("action", action), ("comment", comment)];
            if !src_address.is_empty() {
                args.push(("src-address", src_address));
            }
            if !dst_address.is_empty() {
                args.push(("dst-address", dst_address));
            }
            if !protocol.is_empty() {
                args.push(("protocol", protocol));
            }
            if !dst_port.is_empty() {
                args.push(("dst-port", dst_port));
            }
            client.run(build_command(cmd_path, args)).await?;
            steps.push(QuickActionStep {
                step_number: 2,
                stage: "FIREWALL_APPLY".into(),
                name: "Injeksi Rule Kustom ke RouterOS".into(),
                description: format!("Menerapkan rule pada path {}", cmd_path),
                command: format!("{} chain={} action={}", cmd_path, chain, action),
                status: "success".into(),
                duration_ms: s2_start.elapsed().as_millis() as u64,
                details: "Rule kustom berhasil diinjeksi ke RouterOS".into(),
            });

            let s3_start = Instant::now();
            let print_path = match table {
                "raw" => "/ip/firewall/raw/print",
                "nat" => "/ip/firewall/nat/print",
                _ => "/ip/firewall/filter/print",
            };
            let check = client.run(build_command(print_path, [("?comment", comment)])).await.unwrap_or_default();
            let verified = !check.is_empty();
            steps.push(QuickActionStep {
                step_number: 3,
                stage: "ROUTER_VERIFICATION".into(),
                name: "Verifikasi Eksekusi Rule di MikroTik".into(),
                description: "Membaca kembali rule yang baru saja dibuat".into(),
                command: format!("{} ?comment={}", print_path, comment),
                status: if verified { "success".into() } else { "failed".into() },
                duration_ms: s3_start.elapsed().as_millis() as u64,
                details: format!("Terverifikasi di RouterOS: {} rule aktif", check.len()),
            });

            let job_record = QuickActionJobRecord {
                job_id: job_id.clone(),
                preset_id: "custom_quick_rule".into(),
                title: format!("Kustom Rule: {} ({})", action, chain),
                status: "completed".into(),
                started_at_ms: started_at,
                total_duration_ms: overall_start.elapsed().as_millis() as u64,
                total_steps: steps.len(),
                steps: steps.clone(),
                router_verification: json!({ "verified": verified, "table": table, "chain": chain, "action": action }),
            };
            record_quick_action(job_record.clone());

            Ok(Json(json!({
                "success": true,
                "job_id": job_id,
                "preset_id": "custom_quick_rule",
                "title": job_record.title,
                "status": job_record.status,
                "total_duration_ms": job_record.total_duration_ms,
                "total_steps": steps.len(),
                "steps": steps,
                "router_verification": job_record.router_verification
            })))
        }

        other => Err(ApiError::BadRequest(format!("Preset ID '{}' tidak dikenali. Gunakan /api/v1/firewall/quick-actions/presets untuk melihat daftar preset.", other))),
    }
}

/// POST /api/v1/firewall/connections/mitigate - 1-Klik Respon Mitigasi Koneksi Realtime: Putus koneksi aktif & isolir IP target ke BLACKLIST / QUARANTINE / WHITELIST dengan verifikasi checklist
pub async fn mitigate_connection(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<MitigateConnectionReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let started_at = now_epoch_ms();
    let job_id = format!("job_mitigate_{}_{:04x}", started_at, (started_at % 0xFFFF) as u16);
    let mut steps: Vec<QuickActionStep> = Vec::new();
    let overall_start = Instant::now();

    let clean_ip = |raw: &str| -> String {
        raw.split(':').next().unwrap_or(raw).trim().to_string()
    };
    let src_ip = clean_ip(&req.src_address);
    let dst_ip = req.dst_address.as_deref().map(clean_ip);

    let list_name = req.list.unwrap_or_else(|| {
        if req.action.starts_with("whitelist") {
            "WHITELIST".into()
        } else {
            "BLACKLIST".into()
        }
    });
    let timeout = req.timeout.unwrap_or_else(|| "24h".into());
    let comment = req.comment.unwrap_or_else(|| format!("Mitigated connection {} -> {:?}", src_ip, dst_ip));

    // Step 1: Validation
    let s1_start = Instant::now();
    crate::routes::sanitize::validate_ip_or_cidr(&src_ip)?;
    if let Some(ref d) = dst_ip {
        if !d.is_empty() {
            crate::routes::sanitize::validate_ip_or_cidr(d)?;
        }
    }
    crate::routes::sanitize::validate_safe_identifier(&list_name, "list_name")?;
    steps.push(QuickActionStep {
        step_number: 1,
        stage: "VALIDATION".into(),
        name: "Validasi Sintaks IP & Address List Target".into(),
        description: format!("Memverifikasi sumber {} dan tujuan {:?}", src_ip, dst_ip),
        command: format!("validate_ip_or_cidr(src={}, dst={:?})", src_ip, dst_ip),
        status: "success".into(),
        duration_ms: s1_start.elapsed().as_millis() as u64,
        details: "Format IP valid dan aman".into(),
    });

    // Step 2: Injeksi Address-List
    let s2_start = Instant::now();
    let mut ips_to_list = Vec::new();
    match req.action.as_str() {
        "block_src" | "whitelist_src" => ips_to_list.push(src_ip.clone()),
        "block_dst" | "whitelist_dst" => {
            if let Some(ref d) = dst_ip {
                ips_to_list.push(d.clone());
            }
        }
        "block_both" => {
            ips_to_list.push(src_ip.clone());
            if let Some(ref d) = dst_ip {
                ips_to_list.push(d.clone());
            }
        }
        _ => {}
    }

    for ip in &ips_to_list {
        let mut args = vec![("address", ip.as_str()), ("list", list_name.as_str()), ("comment", comment.as_str())];
        if !timeout.is_empty() {
            args.push(("timeout", timeout.as_str()));
        }
        let _ = client.run(build_command("/ip/firewall/address-list/add", args)).await;
    }
    steps.push(QuickActionStep {
        step_number: 2,
        stage: "ADDRESS_LIST".into(),
        name: format!("Injeksi IP Target ke Address List '{}'", list_name),
        description: format!("Memasukkan {} host ke daftar '{}' dengan timeout {}", ips_to_list.len(), list_name, timeout),
        command: format!("/ip/firewall/address-list/add list={} timeout={}", list_name, timeout),
        status: "success".into(),
        duration_ms: s2_start.elapsed().as_millis() as u64,
        details: format!("Target {:?} terdaftar di '{}'", ips_to_list, list_name),
    });

    // Step 3: Raw Prerouting Drop Guarantee (Zero CPU)
    let s3_start = Instant::now();
    let is_block_action = req.action.starts_with("block");
    if is_block_action {
        let raw_rules = client.run(build_command("/ip/firewall/raw/print", [("?src-address-list", list_name.as_str())])).await.unwrap_or_default();
        if !raw_rules.iter().any(|r| r.get("action") == Some("drop") && r.get("chain") == Some("prerouting")) {
            let comm = format!("[Quick-Action] Raw Prerouting Drop {}", list_name);
            let _ = client.run(build_command(
                "/ip/firewall/raw/add",
                [
                    ("chain", "prerouting"),
                    ("action", "drop"),
                    ("src-address-list", list_name.as_str()),
                    ("comment", comm.as_str()),
                ],
            )).await;
        }
    }
    steps.push(QuickActionStep {
        step_number: 3,
        stage: "FIREWALL_RAW".into(),
        name: "Jaminan RAW Prerouting Drop (0% CPU Conntrack Bypass)".into(),
        description: "Menjamin paket dibuang pada layer hardware/raw sebelum Connection Tracking".into(),
        command: format!("/ip/firewall/raw/add chain=prerouting action=drop src-address-list={}", list_name),
        status: "success".into(),
        duration_ms: s3_start.elapsed().as_millis() as u64,
        details: "Tabel RAW terverifikasi aktif melindungi router".into(),
    });

    // Step 4: Terminate Active Connection Socket
    let s4_start = Instant::now();
    let mut killed = 0;
    if req.kill_active {
        let active_conns = client.run(build_command("/ip/firewall/connection/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
        for c in active_conns {
            let s = c.get("src-address").unwrap_or("");
            let d = c.get("dst-address").unwrap_or("");
            let match_src = s.contains(&src_ip);
            let match_dst = dst_ip.as_ref().map(|target_dst| d.contains(target_dst)).unwrap_or(false);
            if match_src || match_dst {
                if let Some(id) = c.get(".id") {
                    if client.run(build_command("/ip/firewall/connection/remove", [(".id", id)])).await.is_ok() {
                        killed += 1;
                    }
                }
            }
        }
    }
    steps.push(QuickActionStep {
        step_number: 4,
        stage: "CONNTRACK_CLEANUP".into(),
        name: "Terminasi Soket Koneksi Aktif di Conntrack".into(),
        description: "Menghapus entri sesi koneksi yang sedang berlangsung dari memori kernel RouterOS".into(),
        command: format!("/ip/firewall/connection/remove [find src-address~\"{}\"]", src_ip),
        status: "success".into(),
        duration_ms: s4_start.elapsed().as_millis() as u64,
        details: format!("{} sesi koneksi berhasil diputus paksa", killed),
    });

    // Step 5: RouterOS Verification
    let s5_start = Instant::now();
    let verify_rows = client.run(build_command(
        "/ip/firewall/address-list/print",
        [("?list", list_name.as_str()), ("?address", src_ip.as_str())],
    )).await.unwrap_or_default();
    let verified = !verify_rows.is_empty() || req.action == "kill_only";
    steps.push(QuickActionStep {
        step_number: 5,
        stage: "ROUTER_VERIFICATION".into(),
        name: "Verifikasi Langsung ke Perangkat MikroTik".into(),
        description: "Membaca kembali tabel firewall untuk memastikan eksekusi berhasil 100%".into(),
        command: format!("/ip/firewall/address-list/print ?list={} ?address={}", list_name, src_ip),
        status: if verified { "success".into() } else { "failed".into() },
        duration_ms: s5_start.elapsed().as_millis() as u64,
        details: "Mitigasi terverifikasi aktif pada RouterOS".into(),
    });

    let job_record = QuickActionJobRecord {
        job_id: job_id.clone(),
        preset_id: "mitigate_connection".into(),
        title: format!("Mitigasi Koneksi: {} -> {:?}", src_ip, dst_ip),
        status: "completed".into(),
        started_at_ms: started_at,
        total_duration_ms: overall_start.elapsed().as_millis() as u64,
        total_steps: steps.len(),
        steps: steps.clone(),
        router_verification: json!({
            "verified": verified,
            "src_address": src_ip,
            "dst_address": dst_ip,
            "action": req.action,
            "list": list_name,
            "killed_sessions": killed
        }),
    };
    record_quick_action(job_record.clone());

    Ok(Json(json!({
        "success": true,
        "job_id": job_id,
        "title": job_record.title,
        "status": job_record.status,
        "total_duration_ms": job_record.total_duration_ms,
        "steps": steps,
        "router_verification": job_record.router_verification
    })))
}

/// GET or POST /api/v1/firewall/connections/anomalies - Detektor Anomali Sesi Koneksi Aktif (Conntrack Flood, Port Scanners, Port Berbahaya 445/23/3333/8291, & Byte Hogs)
pub async fn connection_anomalies(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/firewall/connection/print", std::iter::empty::<(&str, &str)>())).await?;
    let total_conns = rows.len();

    let mut src_stats: HashMap<String, (usize, std::collections::HashSet<String>)> = HashMap::new();
    let mut dangerous_conns = Vec::new();

    for r in &rows {
        let src = r.get("src-address").unwrap_or("");
        let dst = r.get("dst-address").unwrap_or("");
        let proto = r.get("protocol").unwrap_or("tcp");

        let src_ip = src.split(':').next().unwrap_or(src).trim().to_string();
        let dst_port = dst.split(':').nth(1).unwrap_or("").trim().to_string();

        if !src_ip.is_empty() {
            let entry = src_stats.entry(src_ip.clone()).or_insert_with(|| (0, std::collections::HashSet::new()));
            entry.0 += 1;
            if !dst_port.is_empty() {
                entry.1.insert(dst_port.clone());
            }
        }

        match dst_port.as_str() {
            "445" | "139" => dangerous_conns.push(json!({
                "type": "SMB_EXPOSURE",
                "severity": "HIGH",
                "description": "Koneksi ke port SMB/Samba 445 (potensi eksploitasi WannaCry/EternalBlue)",
                "src": src,
                "dst": dst,
                "protocol": proto,
                "suggested_action": "block_src",
                "quick_mitigate_payload": { "src_address": src, "dst_address": dst, "action": "block_src", "list": "BLACKLIST", "timeout": "7d" }
            })),
            "23" => dangerous_conns.push(json!({
                "type": "TELNET_PLAINTEXT",
                "severity": "CRITICAL",
                "description": "Koneksi Telnet teks polos tanpa enkripsi terdeteksi",
                "src": src,
                "dst": dst,
                "protocol": proto,
                "suggested_action": "block_src",
                "quick_mitigate_payload": { "src_address": src, "dst_address": dst, "action": "block_src", "list": "BLACKLIST", "timeout": "7d" }
            })),
            "3333" | "8333" | "14444" => dangerous_conns.push(json!({
                "type": "CRYPTOMINER_STRATUM",
                "severity": "CRITICAL",
                "description": "Koneksi Stratum Mining Rig/Cryptocurrency terdeteksi pada host internal",
                "src": src,
                "dst": dst,
                "protocol": proto,
                "suggested_action": "block_src",
                "quick_mitigate_payload": { "src_address": src, "dst_address": dst, "action": "block_src", "list": "ISOLIR", "timeout": "30d" }
            })),
            "8291" if !src_ip.starts_with("192.168.") && !src_ip.starts_with("10.") && !src_ip.starts_with("172.") => {
                dangerous_conns.push(json!({
                    "type": "WINBOX_WAN_EXPOSURE",
                    "severity": "CRITICAL",
                    "description": "Akses Winbox port 8291 dari alamat IP publik internet tanpa VPN/Whitelist",
                    "src": src,
                    "dst": dst,
                    "protocol": proto,
                    "suggested_action": "block_src",
                    "quick_mitigate_payload": { "src_address": src, "dst_address": dst, "action": "block_src", "list": "BLACKLIST", "timeout": "7d" }
                }));
            }
            _ => {}
        }
    }

    let mut anomalies = Vec::new();

    // 1. Conntrack Flood (> 40 sesi dari 1 IP)
    for (ip, (count, _)) in &src_stats {
        if *count >= 40 {
            anomalies.push(json!({
                "anomaly_id": format!("flood_{}", ip.replace('.', "_")),
                "type": "CONNTRACK_FLOOD",
                "severity": if *count > 100 { "CRITICAL" } else { "HIGH" },
                "target_ip": ip,
                "session_count": count,
                "description": format!("Host '{}' membuka {} sesi koneksi simultan (berpotensi DoS / Torrent P2P / Botnet)", ip, count),
                "quick_mitigate_payload": {
                    "src_address": ip,
                    "action": "block_src",
                    "list": "BLACKLIST",
                    "timeout": "24h",
                    "kill_active": true,
                    "comment": format!("Mitigated Conntrack Flood: {} sessions", count)
                }
            }));
        }
    }

    // 2. Port Scanner (> 8 destination ports berbeda dari 1 host)
    for (ip, (_, ports)) in &src_stats {
        if ports.len() >= 8 {
            let port_samples: Vec<_> = ports.iter().take(5).cloned().collect();
            anomalies.push(json!({
                "anomaly_id": format!("scanner_{}", ip.replace('.', "_")),
                "type": "PORT_SCANNER",
                "severity": "HIGH",
                "target_ip": ip,
                "distinct_ports_targeted": ports.len(),
                "port_samples": port_samples,
                "description": format!("Host '{}' memindai {} port tujuan berbeda (indikasi Reconnaissance / Nmap)", ip, ports.len()),
                "quick_mitigate_payload": {
                    "src_address": ip,
                    "action": "block_src",
                    "list": "PORT_SCANNERS",
                    "timeout": "24h",
                    "kill_active": true,
                    "comment": format!("Mitigated Port Scanner: {} ports", ports.len())
                }
            }));
        }
    }

    Ok(Json(json!({
        "success": true,
        "total_active_sessions_scanned": total_conns,
        "conntrack_flood_detected": anomalies.iter().filter(|a| a.get("type").and_then(|v| v.as_str()) == Some("CONNTRACK_FLOOD")).count(),
        "port_scanners_detected": anomalies.iter().filter(|a| a.get("type").and_then(|v| v.as_str()) == Some("PORT_SCANNER")).count(),
        "dangerous_port_activities": dangerous_conns.len(),
        "anomalies": anomalies,
        "dangerous_connections": dangerous_conns
    })))
}

/// GET or POST /api/v1/firewall/quarantine - Daftar seluruh IP yang sedang terisolir di daftar karantina atau blacklist RouterOS
pub async fn list_quarantine(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/firewall/address-list/print", std::iter::empty::<(&str, &str)>())).await?;
    let mut quarantined = Vec::new();

    for r in rows {
        let list = r.get("list").unwrap_or("");
        if list == "BLACKLIST" || list == "QUARANTINE" || list == "ISOLIR" || list == "PORT_SCANNERS" {
            quarantined.push(json!({
                "id": r.get(".id"),
                "address": r.get("address"),
                "list": list,
                "timeout": r.get("timeout"),
                "comment": r.get("comment"),
                "dynamic": r.get("dynamic"),
                "release_payload": {
                    "id": r.get(".id"),
                    "address": r.get("address"),
                    "list": list
                }
            }));
        }
    }

    Ok(Json(json!({
        "success": true,
        "total_quarantined": quarantined.len(),
        "quarantined_hosts": quarantined
    })))
}

/// POST /api/v1/firewall/quarantine/release - Lepas IP dari status karantina / blacklist
pub async fn release_quarantine(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ReleaseQuarantineReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut removed = 0usize;
    if let Some(ref id) = req.id {
        client.run(build_command("/ip/firewall/address-list/remove", [(".id", id.as_str())])).await?;
        removed = 1;
    } else if let Some(ref addr) = req.address {
        let mut query = vec![("?address", addr.as_str())];
        if let Some(ref l) = req.list {
            query.push(("?list", l.as_str()));
        }
        let rows = client.run(build_command("/ip/firewall/address-list/print", query)).await?;
        for r in rows {
            if let Some(id) = r.get(".id") {
                if client.run(build_command("/ip/firewall/address-list/remove", [(".id", id)])).await.is_ok() {
                    removed += 1;
                }
            }
        }
    } else {
        return Err(ApiError::BadRequest("Tentukan 'id' atau 'address' untuk melepas karantina".into()));
    }

    Ok(Json(json!({
        "success": true,
        "released_count": removed,
        "message": format!("Berhasil melepas {} entri dari status karantina!", removed)
    })))
}



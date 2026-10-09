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

#[derive(Deserialize, Debug, Clone)]
pub struct WanConfig {
    pub interface: String,      // Nama interface misal "ether1"
    pub gateway: String,        // IP Gateway ISP misal "192.168.1.1"
    #[serde(default = "default_weight")]
    pub weight: u32,            // Rasio beban misal 1 (50Mbps), 2 (100Mbps)
    pub name: Option<String>,   // Alias label misal "WAN1_Indihome"
}

fn default_weight() -> u32 {
    1
}

fn default_matcher() -> String {
    "both-addresses".into()
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Debug)]
pub struct PccSetupReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub lan_interface: String,  // Interface LAN/Lokal misal "bridge" atau "bridge-lan"
    pub wans: Vec<WanConfig>,   // Daftar 2 atau lebih koneksi WAN
    #[serde(default = "default_matcher")]
    pub matcher: String,        // "both-addresses", "both-addresses-and-ports", "src-address"
    #[serde(default = "default_true")]
    pub auto_failover: bool,    // Tambah check-gateway=ping dan failover metric
    #[serde(default = "default_true")]
    pub add_masquerade: bool,   // Otomatis pasang NAT masquerade tiap WAN
}

#[derive(Deserialize, Debug, Default)]
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

/// POST /api/v1/load-balance/pcc/setup
/// 1-Klik Wizard Multi-WAN PCC (Per Connection Classifier) Load Balancing.
/// Mendukung rasio bobot (weights), otomatisasi mangle connection mark, routing mark,
/// check-gateway failover ping, dan NAT Masquerade. Semua ditandai dengan komentar [PCC-LoadBalance].
pub async fn pcc_setup(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<PccSetupReq>,
) -> Result<Json<Value>, ApiError> {
    if req.wans.len() < 2 {
        return Err(ApiError::BadRequest("PCC Load Balancing membutuhkan minimal 2 interface WAN".into()));
    }

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // 1. Bersihkan aturan [PCC-LoadBalance] lama terlebih dahulu agar tidak duplikat
    cleanup_pcc_rules(&client).await?;

    // 2. Hitung total slot classifier berdasarkan bobot (weights)
    let total_slots: u32 = req.wans.iter().map(|w| w.weight.max(1)).sum();
    let mut slot_assignments = Vec::new(); // (slot_index, wan_index, wan_alias)

    let mut current_slot = 0u32;
    let mut wan_aliases = Vec::new();

    for (i, wan) in req.wans.iter().enumerate() {
        let alias = wan.name.clone().unwrap_or_else(|| format!("WAN{}", i + 1));
        wan_aliases.push(alias.clone());

        let w_count = wan.weight.max(1);
        for _ in 0..w_count {
            slot_assignments.push((current_slot, i, alias.clone()));
            current_slot += 1;
        }
    }

    let mut deployed_rules = Vec::new();

    // 3. Mangle: Accept traffic ke subnet lokal
    let m_local = build_command(
        "/ip/firewall/mangle/add",
        [
            ("chain", "prerouting"),
            ("dst-address-type", "local"),
            ("action", "accept"),
            ("comment", "[PCC-LoadBalance] Local Subnet Accept"),
        ],
    );
    client.run(m_local).await?;
    deployed_rules.push("Mangle: Local Subnet Accept");

    // 4. Mangle: Tandai koneksi masuk dari masing-masing interface WAN
    for (i, wan) in req.wans.iter().enumerate() {
        let alias = &wan_aliases[i];
        let conn_mark = format!("{}_conn", alias);

        let m_inbound = build_command(
            "/ip/firewall/mangle/add",
            [
                ("chain", "prerouting"),
                ("in-interface", wan.interface.as_str()),
                ("connection-mark", "no-mark"),
                ("action", "mark-connection"),
                ("new-connection-mark", conn_mark.as_str()),
                ("passthrough", "yes"),
                ("comment", format!("[PCC-LoadBalance] Inbound {}", alias).as_str()),
            ],
        );
        client.run(m_inbound).await?;
        deployed_rules.push(format!("Mangle: Inbound {}", alias));

        // Output routing mark untuk router reply
        let route_mark = format!("to_{}", alias);
        let m_out = build_command(
            "/ip/firewall/mangle/add",
            [
                ("chain", "output"),
                ("connection-mark", conn_mark.as_str()),
                ("action", "mark-routing"),
                ("new-routing-mark", route_mark.as_str()),
                ("passthrough", "no"),
                ("comment", format!("[PCC-LoadBalance] Output Route {}", alias).as_str()),
            ],
        );
        client.run(m_out).await?;
        deployed_rules.push(format!("Mangle: Output Route {}", alias));
    }

    // 5. Mangle: PCC Hashing Classifier dari interface LAN
    for (slot, _wan_idx, alias) in &slot_assignments {
        let pcc_val = format!("{}:{}/{}", req.matcher, total_slots, slot);
        let conn_mark = format!("{}_conn", alias);

        let m_pcc = build_command(
            "/ip/firewall/mangle/add",
            [
                ("chain", "prerouting"),
                ("in-interface", req.lan_interface.as_str()),
                ("connection-mark", "no-mark"),
                ("dst-address-type", "!local"),
                ("per-connection-classifier", pcc_val.as_str()),
                ("action", "mark-connection"),
                ("new-connection-mark", conn_mark.as_str()),
                ("passthrough", "yes"),
                ("comment", format!("[PCC-LoadBalance] PCC {}/{} ke {}", total_slots, slot, alias).as_str()),
            ],
        );
        client.run(m_pcc).await?;
        deployed_rules.push(format!("Mangle: PCC {}/{} ke {}", total_slots, slot, alias));
    }

    // 6. Mangle: Routing Mark dari interface LAN untuk koneksi yang sudah ditandai
    for (i, _wan) in req.wans.iter().enumerate() {
        let alias = &wan_aliases[i];
        let conn_mark = format!("{}_conn", alias);
        let route_mark = format!("to_{}", alias);

        let m_route = build_command(
            "/ip/firewall/mangle/add",
            [
                ("chain", "prerouting"),
                ("in-interface", req.lan_interface.as_str()),
                ("connection-mark", conn_mark.as_str()),
                ("action", "mark-routing"),
                ("new-routing-mark", route_mark.as_str()),
                ("passthrough", "no"),
                ("comment", format!("[PCC-LoadBalance] Mark Route to {}", alias).as_str()),
            ],
        );
        client.run(m_route).await?;
        deployed_rules.push(format!("Mangle: Routing Mark to {}", alias));
    }

    // 7. IP Route: Tambahkan tabel rute untuk masing-masing routing mark
    for (i, wan) in req.wans.iter().enumerate() {
        let alias = &wan_aliases[i];
        let route_mark = format!("to_{}", alias);

        let mut r_args = vec![
            ("dst-address", "0.0.0.0/0"),
            ("gateway", wan.gateway.as_str()),
            ("routing-mark", route_mark.as_str()),
            ("distance", "1"),
            ("comment", format!("[PCC-LoadBalance] Route {}", alias).as_str()),
        ];
        if req.auto_failover {
            r_args.push(("check-gateway", "ping"));
        }
        client.run(build_command("/ip/route/add", r_args)).await?;
        deployed_rules.push(format!("IP Route: Default mark to_{}", alias));
    }

    // 8. IP Route: Tambahkan fallback failover metric tanpa routing mark
    if req.auto_failover {
        for (i, wan) in req.wans.iter().enumerate() {
            let alias = &wan_aliases[i];
            let dist_str = (i + 1).to_string();
            let r_fallback = build_command(
                "/ip/route/add",
                [
                    ("dst-address", "0.0.0.0/0"),
                    ("gateway", wan.gateway.as_str()),
                    ("check-gateway", "ping"),
                    ("distance", dist_str.as_str()),
                    ("comment", format!("[PCC-LoadBalance] Failover Backup Metric {}", dist_str).as_str()),
                ],
            );
            client.run(r_fallback).await?;
            deployed_rules.push(format!("IP Route: Failover Metric {} {}", dist_str, alias));
        }
    }

    // 9. NAT: Masquerade untuk masing-masing WAN
    if req.add_masquerade {
        for (i, wan) in req.wans.iter().enumerate() {
            let alias = &wan_aliases[i];
            let n_masq = build_command(
                "/ip/firewall/nat/add",
                [
                    ("chain", "srcnat"),
                    ("out-interface", wan.interface.as_str()),
                    ("action", "masquerade"),
                    ("comment", format!("[PCC-LoadBalance] Masquerade {}", alias).as_str()),
                ],
            );
            client.run(n_masq).await?;
            deployed_rules.push(format!("NAT Masquerade: {}", alias));
        }
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("PCC Load Balancing berhasil dikonfigurasi untuk {} WAN dengan rasio {} slot", req.wans.len(), total_slots),
        "total_wans": req.wans.len(),
        "total_slots": total_slots,
        "matcher": req.matcher,
        "wan_distribution": req.wans.iter().enumerate().map(|(i, w)| {
            let w_val = w.weight.max(1);
            let pct = (w_val as f64 / total_slots as f64) * 100.0;
            json!({
                "alias": wan_aliases[i],
                "interface": w.interface,
                "gateway": w.gateway,
                "weight": w_val,
                "percentage": format!("{:.1}%", pct)
            })
        }).collect::<Vec<_>>(),
        "deployed_rules_count": deployed_rules.len(),
        "deployed_rules": deployed_rules
    })))
}

/// POST /api/v1/load-balance/status
/// Memantau efektivitas pembagian beban trafik antar interface WAN secara real-time.
pub async fn status(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // 1. Ambil seluruh mangle rules [PCC-LoadBalance]
    let mangle_rows = client.run(build_command("/ip/firewall/mangle/print", std::iter::empty::<(&str, &str)>())).await?;
    let pcc_mangles: Vec<_> = mangle_rows.into_iter().filter(|r| {
        r.get("comment").map(|c| c.contains("[PCC-LoadBalance]")).unwrap_or(false)
    }).collect();

    // 2. Ambil counters interface
    let iface_rows = client.run(build_command("/interface/print", std::iter::empty::<(&str, &str)>())).await?;
    let iface_map: HashMap<String, (u64, u64)> = iface_rows.into_iter().map(|r| {
        let name = r.get("name").unwrap_or_default().to_string();
        let rx = r.get("rx-byte").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        let tx = r.get("tx-byte").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        (name, (rx, tx))
    }).collect();

    let is_active = !pcc_mangles.is_empty();

    // Ekstraksi traffic per routing-mark
    let mut wan_stats = HashMap::new();
    for m in &pcc_mangles {
        let comment = m.get("comment").unwrap_or_default();
        let bytes = m.get("bytes").and_then(|b| b.parse::<u64>().ok()).unwrap_or(0);
        let packets = m.get("packets").and_then(|p| p.parse::<u64>().ok()).unwrap_or(0);

        if comment.contains("PCC") {
            let entry = wan_stats.entry(comment.to_string()).or_insert((0u64, 0u64));
            entry.0 += bytes;
            entry.1 += packets;
        }
    }

    Ok(Json(json!({
        "success": true,
        "is_active": is_active,
        "pcc_rules_installed": pcc_mangles.len(),
        "wan_interfaces_traffic": iface_map,
        "pcc_classifier_counters": wan_stats.into_iter().map(|(k, (b, p))| {
            json!({
                "rule": k,
                "bytes": b,
                "packets": p
            })
        }).collect::<Vec<_>>()
    })))
}

/// POST /api/v1/load-balance/remove
/// Menghapus seluruh aturan PCC Load Balancing ([PCC-LoadBalance]) dan mengembalikan router ke rute standar.
pub async fn remove(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let removed_count = cleanup_pcc_rules(&client).await?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Seluruh aturan PCC Load Balancing ({}) berhasil dihapus. Router kembali ke rute normal.", removed_count),
        "removed_rules_count": removed_count
    })))
}

async fn cleanup_pcc_rules(client: &routeros_core::client::RouterOsClient) -> Result<usize, ApiError> {
    let mut total_removed = 0usize;

    // 1. Bersihkan Mangle
    let m_rows = client.run(build_command("/ip/firewall/mangle/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    for r in m_rows {
        if r.get("comment").map(|c| c.contains("[PCC-LoadBalance]")).unwrap_or(false) {
            if let Some(id) = r.get(".id") {
                let _ = client.run(build_command("/ip/firewall/mangle/remove", [(".id", id)])).await;
                total_removed += 1;
            }
        }
    }

    // 2. Bersihkan Route
    let r_rows = client.run(build_command("/ip/route/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    for r in r_rows {
        if r.get("comment").map(|c| c.contains("[PCC-LoadBalance]")).unwrap_or(false) {
            if let Some(id) = r.get(".id") {
                let _ = client.run(build_command("/ip/route/remove", [(".id", id)])).await;
                total_removed += 1;
            }
        }
    }

    // 3. Bersihkan NAT Masquerade
    let n_rows = client.run(build_command("/ip/firewall/nat/print", std::iter::empty::<(&str, &str)>())).await.unwrap_or_default();
    for r in n_rows {
        if r.get("comment").map(|c| c.contains("[PCC-LoadBalance]")).unwrap_or(false) {
            if let Some(id) = r.get(".id") {
                let _ = client.run(build_command("/ip/firewall/nat/remove", [(".id", id)])).await;
                total_removed += 1;
            }
        }
    }

    Ok(total_removed)
}

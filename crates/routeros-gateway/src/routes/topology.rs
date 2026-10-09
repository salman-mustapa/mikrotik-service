use std::collections::{HashMap, HashSet};
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
pub struct TopologyReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub filter_type: Option<String>, // "all", "hotspot", "pppoe", "dhcp", "wifi"
}

#[derive(Serialize, Debug, Clone)]
pub struct TopologyNode {
    pub id: String,
    pub label: String,
    pub node_type: String, // "router", "interface", "neighbor", "device"
    pub sub_type: String,  // "wan", "lan", "bridge", "hotspot", "pppoe", "dhcp", "wifi"
    pub ip: Option<String>,
    pub mac: Option<String>,
    pub interface: Option<String>,
    pub status: String,    // "online", "idle", "active", "warning"
    pub extra: HashMap<String, String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct TopologyEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: String, // "physical", "logical", "tunnel", "wireless"
    pub speed: Option<String>,
}

/// POST /api/v1/network/topology-graph - Fast relational graph generation linking Router -> Interfaces -> Neighbors -> Connected Clients
pub async fn topology_graph(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<TopologyReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Execute 8 relational queries concurrently in single persistent session (tokio::join!)
    let ident_fut = client.run(build_command("/system/identity/print", std::iter::empty::<(&str, &str)>()));
    let res_fut = client.run(build_command("/system/resource/print", std::iter::empty::<(&str, &str)>()));
    let iface_fut = client.run(build_command("/interface/print", std::iter::empty::<(&str, &str)>()));
    let ip_fut = client.run(build_command("/ip/address/print", std::iter::empty::<(&str, &str)>()));
    let dhcp_fut = client.run(build_command("/ip/dhcp-server/lease/print", std::iter::empty::<(&str, &str)>()));
    let hs_act_fut = client.run(build_command("/ip/hotspot/active/print", std::iter::empty::<(&str, &str)>()));
    let ppp_act_fut = client.run(build_command("/ppp/active/print", std::iter::empty::<(&str, &str)>()));
    let wifi_fut = client.run(build_command("/interface/wireless/registration-table/print", std::iter::empty::<(&str, &str)>()));

    let (ident_res, res_res, iface_res, ip_res, dhcp_res, hs_act_res, ppp_act_res, wifi_res) =
        tokio::join!(ident_fut, res_fut, iface_fut, ip_fut, dhcp_fut, hs_act_fut, ppp_act_fut, wifi_fut);

    let mut nodes: Vec<TopologyNode> = Vec::new();
    let mut edges: Vec<TopologyEdge> = Vec::new();
    let filter = req.filter_type.as_deref().unwrap_or("all").to_lowercase();

    // 1. Root Node: The Router
    let router_name = ident_res.ok()
        .and_then(|r| r.into_iter().next())
        .and_then(|r| r.get("name").map(|s| s.to_string()))
        .unwrap_or_else(|| "MikroTik-Core".into());

    let mut router_extra = HashMap::new();
    if let Ok(r_rows) = res_res {
        if let Some(r) = r_rows.first() {
            if let Some(cpu) = r.get("cpu-load") { router_extra.insert("cpu_load".into(), format!("{}%", cpu)); }
            if let Some(ver) = r.get("version") { router_extra.insert("version".into(), ver.to_string()); }
            if let Some(mem) = r.get("free-memory") {
                let bytes: u64 = mem.parse().unwrap_or(0);
                router_extra.insert("free_ram_mb".into(), format!("{} MB", bytes / (1024 * 1024)));
            }
            if let Some(uptime) = r.get("uptime") { router_extra.insert("uptime".into(), uptime.to_string()); }
        }
    }

    nodes.push(TopologyNode {
        id: "node_router".into(),
        label: router_name,
        node_type: "router".into(),
        sub_type: "core".into(),
        ip: None,
        mac: None,
        interface: None,
        status: "online".into(),
        extra: router_extra,
    });

    // 2. Interface Nodes & Router-to-Interface Edges
    let mut iface_map = HashSet::new();
    if let Ok(ifaces) = iface_res {
        for row in ifaces {
            let name = row.get("name").unwrap_or("").to_string();
            let if_type = row.get("type").unwrap_or("ether").to_string();
            let running = row.get("running").map(|v| v == "true").unwrap_or(false);
            let disabled = row.get("disabled").map(|v| v == "true").unwrap_or(false);

            if disabled || name.is_empty() {
                continue;
            }

            // Keep physical, bridge, and tunnels
            if ["ether", "bridge", "wlan", "vlan", "wireguard", "pppoe-in", "sstp-in"].iter().any(|t| if_type.contains(t)) {
                let node_id = format!("if_{}", name);
                iface_map.insert(name.clone());

                let mut extra = HashMap::new();
                extra.insert("type".into(), if_type.clone());
                if let Some(rx) = row.get("rx-byte") { extra.insert("rx_bytes".into(), rx.to_string()); }
                if let Some(tx) = row.get("tx-byte") { extra.insert("tx_bytes".into(), tx.to_string()); }

                nodes.push(TopologyNode {
                    id: node_id.clone(),
                    label: name.clone(),
                    node_type: "interface".into(),
                    sub_type: if_type.clone(),
                    ip: None,
                    mac: row.get("mac-address").map(|s| s.to_string()),
                    interface: Some(name.clone()),
                    status: if running { "online".into() } else { "idle".into() },
                    extra,
                });

                edges.push(TopologyEdge {
                    id: format!("edge_router_{}", name),
                    source: "node_router".into(),
                    target: node_id,
                    edge_type: "physical".into(),
                    speed: row.get("actual-mtu").map(|s| format!("MTU {}", s)),
                });
            }
        }
    }

    // Correlate Clients (DHCP Leases, Hotspot, PPPoE, WiFi)
    let mut seen_ips = HashSet::new();
    let mut seen_macs = HashSet::new();

    // 3. Hotspot Active Clients
    if filter == "all" || filter == "hotspot" {
        if let Ok(hs_rows) = hs_act_res {
            for row in hs_rows {
                let user = row.get("user").unwrap_or("guest").to_string();
                let ip = row.get("address").unwrap_or("").to_string();
                let mac = row.get("mac-address").unwrap_or("").to_uppercase();
                let uptime = row.get("uptime").unwrap_or("").to_string();

                if ip.is_empty() { continue; }
                seen_ips.insert(ip.clone());
                if !mac.is_empty() { seen_macs.insert(mac.clone()); }

                let node_id = format!("client_hs_{}", if !mac.is_empty() { &mac } else { &ip });
                let mut extra = HashMap::new();
                extra.insert("category".into(), "hotspot".into());
                extra.insert("username".into(), user.clone());
                extra.insert("uptime".into(), uptime);
                if let Some(bytes_in) = row.get("bytes-in") { extra.insert("bytes_in".into(), bytes_in.to_string()); }
                if let Some(bytes_out) = row.get("bytes-out") { extra.insert("bytes_out".into(), bytes_out.to_string()); }

                nodes.push(TopologyNode {
                    id: node_id.clone(),
                    label: format!("🎟️ {} ({})", user, ip),
                    node_type: "device".into(),
                    sub_type: "hotspot".into(),
                    ip: Some(ip.clone()),
                    mac: Some(mac.clone()),
                    interface: None,
                    status: "active".into(),
                    extra,
                });

                // Link to bridge or core
                let target_if = if iface_map.contains("bridge") { "if_bridge" } else if iface_map.contains("bridge1") { "if_bridge1" } else { "node_router" };
                edges.push(TopologyEdge {
                    id: format!("edge_{}_{}", target_if, node_id),
                    source: target_if.into(),
                    target: node_id,
                    edge_type: "logical".into(),
                    speed: Some("Hotspot Auth".into()),
                });
            }
        }
    }

    // 4. PPPoE Active Clients
    if filter == "all" || filter == "pppoe" {
        if let Ok(ppp_rows) = ppp_act_res {
            for row in ppp_rows {
                let name = row.get("name").unwrap_or("pppoe-client").to_string();
                let ip = row.get("address").unwrap_or("").to_string();
                let caller_id = row.get("caller-id").unwrap_or("").to_string();
                let uptime = row.get("uptime").unwrap_or("").to_string();

                if ip.is_empty() { continue; }
                seen_ips.insert(ip.clone());

                let node_id = format!("client_ppp_{}", name);
                let mut extra = HashMap::new();
                extra.insert("category".into(), "pppoe".into());
                extra.insert("service".into(), row.get("service").unwrap_or("pppoe").to_string());
                extra.insert("caller_id".into(), caller_id.clone());
                extra.insert("uptime".into(), uptime);

                nodes.push(TopologyNode {
                    id: node_id.clone(),
                    label: format!("🌐 PPPoE: {} ({})", name, ip),
                    node_type: "device".into(),
                    sub_type: "pppoe".into(),
                    ip: Some(ip.clone()),
                    mac: Some(caller_id),
                    interface: None,
                    status: "active".into(),
                    extra,
                });

                edges.push(TopologyEdge {
                    id: format!("edge_router_{}", node_id),
                    source: "node_router".into(),
                    target: node_id,
                    edge_type: "tunnel".into(),
                    speed: Some("PPPoE Session".into()),
                });
            }
        }
    }

    // 5. DHCP Leases (Remaining clients not yet mapped as hotspot/pppoe)
    if filter == "all" || filter == "dhcp" {
        if let Ok(dhcp_rows) = dhcp_res {
            for row in dhcp_rows {
                let ip = row.get("address").unwrap_or("").to_string();
                let mac = row.get("mac-address").unwrap_or("").to_uppercase();
                let host_name = row.get("host-name").unwrap_or("").to_string();
                let status = row.get("status").unwrap_or("bound").to_string();

                if ip.is_empty() || seen_ips.contains(&ip) || (!mac.is_empty() && seen_macs.contains(&mac)) {
                    continue;
                }
                seen_ips.insert(ip.clone());

                let label = if !host_name.is_empty() {
                    format!("💻 {} ({})", host_name, ip)
                } else {
                    format!("📱 Device ({})", ip)
                };

                let node_id = format!("client_dhcp_{}", if !mac.is_empty() { &mac } else { &ip });
                let mut extra = HashMap::new();
                extra.insert("category".into(), "dhcp".into());
                extra.insert("host_name".into(), host_name);
                extra.insert("lease_status".into(), status.clone());

                nodes.push(TopologyNode {
                    id: node_id.clone(),
                    label,
                    node_type: "device".into(),
                    sub_type: "dhcp".into(),
                    ip: Some(ip.clone()),
                    mac: Some(mac.clone()),
                    interface: None,
                    status: if status == "bound" { "online".into() } else { "idle".into() },
                    extra,
                });

                let target_if = if iface_map.contains("bridge") { "if_bridge" } else if iface_map.contains("bridge1") { "if_bridge1" } else { "node_router" };
                edges.push(TopologyEdge {
                    id: format!("edge_{}_{}", target_if, node_id),
                    source: target_if.into(),
                    target: node_id,
                    edge_type: "physical".into(),
                    speed: Some("DHCP Lease".into()),
                });
            }
        }
    }

    // 6. Wireless Clients Signal Mapping
    if filter == "all" || filter == "wifi" {
        if let Ok(wifi_rows) = wifi_res {
            for row in wifi_rows {
                let mac = row.get("mac-address").unwrap_or("").to_uppercase();
                let iface = row.get("interface").unwrap_or("wlan1").to_string();
                let signal = row.get("signal-strength").unwrap_or("-65dBm").to_string();
                let tx_rate = row.get("tx-rate").unwrap_or("").to_string();
                let rx_rate = row.get("rx-rate").unwrap_or("").to_string();

                if mac.is_empty() || seen_macs.contains(&mac) {
                    continue;
                }
                seen_macs.insert(mac.clone());

                let node_id = format!("client_wifi_{}", mac);
                let mut extra = HashMap::new();
                extra.insert("category".into(), "wifi".into());
                extra.insert("signal".into(), signal.clone());
                extra.insert("tx_rate".into(), tx_rate);
                extra.insert("rx_rate".into(), rx_rate);

                nodes.push(TopologyNode {
                    id: node_id.clone(),
                    label: format!("📶 WiFi ({})", mac),
                    node_type: "device".into(),
                    sub_type: "wifi".into(),
                    ip: None,
                    mac: Some(mac.clone()),
                    interface: Some(iface.clone()),
                    status: "online".into(),
                    extra,
                });

                let if_source = format!("if_{}", iface);
                let source = if iface_map.contains(&iface) { if_source.as_str() } else { "node_router" };
                edges.push(TopologyEdge {
                    id: format!("edge_wifi_{}_{}", source, node_id),
                    source: source.into(),
                    target: node_id,
                    edge_type: "wireless".into(),
                    speed: Some(signal),
                });
            }
        }
    }

    let total_nodes = nodes.len();
    let total_edges = edges.len();

    Ok(Json(json!({
        "success": true,
        "nodes_count": total_nodes,
        "edges_count": total_edges,
        "nodes": nodes,
        "edges": edges
    })))
}

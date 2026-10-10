use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use axum::extract::{Query, State};
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
    pub filter_type: Option<String>, // "all", "hotspot", "pppoe", "dhcp", "wifi", "infra"
}

#[derive(Deserialize, Debug, Default)]
pub struct TopologyQuery {
    pub filter: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct TopologyNode {
    pub id: String,
    pub label: String,
    pub node_type: String, // "router", "interface", "ap", "device"
    pub sub_type: String,  // "core", "ether", "bridge", "ap", "hotspot", "pppoe", "dhcp", "wifi"
    pub ip: Option<String>,
    pub mac: Option<String>,
    pub vendor: Option<String>,
    pub interface: Option<String>,
    pub parent_id: Option<String>,
    pub status: String,    // "online", "active", "bound", "idle"
    pub extra: HashMap<String, String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct TopologyEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: String, // "physical", "logical", "wireless", "tunnel"
    pub speed: Option<String>,
}

/// Helper function to detect device vendor from MAC address or Hostname
fn detect_device_vendor(mac: &str, host_name: &str) -> String {
    let clean = mac.replace(['-', ':'], "").to_uppercase();
    let lower_host = host_name.to_lowercase();

    if lower_host.contains("redmi") || lower_host.contains("xiaomi") || lower_host.contains("poco") {
        return "Xiaomi".into();
    }
    if lower_host.contains("oppo") || lower_host.contains("cph") {
        return "OPPO".into();
    }
    if lower_host.contains("realme") || lower_host.contains("rmx") {
        return "Realme".into();
    }
    if lower_host.contains("galaxy") || lower_host.contains("samsung") || lower_host.contains("sm-") {
        return "Samsung".into();
    }
    if lower_host.contains("iphone") || lower_host.contains("ipad") || lower_host.contains("apple") || lower_host.contains("macbook") {
        return "Apple".into();
    }
    if lower_host.contains("desktop") || lower_host.contains("laptop") || lower_host.contains("pc") || lower_host.contains("mances-pc") {
        return "PC / Windows".into();
    }
    if lower_host.contains("ap") || lower_host.contains("accesspoint") || lower_host.contains("totolink") || lower_host.contains("tenda") {
        return "Access Point".into();
    }

    if clean.len() >= 6 {
        let prefix = &clean[0..6];
        match prefix {
            "000C42" | "488F5A" | "64D154" | "789A18" | "B869F4" | "C4AD34" | "D4CA6D" => "MikroTik".into(),
            "50D4F7" | "EC086B" | "14CC20" | "54E6FC" | "000AEB" | "001478" | "0019E0" => "TP-Link".into(),
            "002722" | "0418D6" | "24A43C" | "44D9E7" | "68D79A" | "70A741" | "7483C2" => "Ubiquiti".into(),
            "E865D4" | "4C63EB" | "BC25E0" | "D83214" => "Ruijie / Reyee".into(),
            "00E04C" | "525400" => "Realtek / Virtual".into(),
            _ => "Generic Device".into(),
        }
    } else {
        "Network Device".into()
    }
}

/// Helper to check if an IPv4 address belongs to a network prefix (e.g., "192.168.100.")
fn ip_matches_subnet(ip: &str, subnet_prefix: &str) -> bool {
    if subnet_prefix.is_empty() || ip.is_empty() {
        return false;
    }
    ip.starts_with(subnet_prefix)
}

/// POST & GET /api/v1/network/topology-graph - Hierarchical Network Topology Graph
pub async fn topology_graph(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    query: Option<Query<TopologyQuery>>,
    req: Option<Json<TopologyReq>>,
) -> Result<Json<Value>, ApiError> {
    let req_inner = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req_inner.router, req_inner.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Determine filter: from JSON body or URL query parameter
    let filter = req_inner.filter_type
        .or_else(|| query.and_then(|q| q.filter.clone()))
        .unwrap_or_else(|| "all".into())
        .to_lowercase();

    // Concurrently fetch 11 core relational tables in parallel using persistent connection
    let ident_fut = client.run(build_command("/system/identity/print", std::iter::empty::<(&str, &str)>()));
    let res_fut = client.run(build_command("/system/resource/print", std::iter::empty::<(&str, &str)>()));
    let iface_fut = client.run(build_command("/interface/print", std::iter::empty::<(&str, &str)>()));
    let ip_fut = client.run(build_command("/ip/address/print", std::iter::empty::<(&str, &str)>()));
    let dhcp_fut = client.run(build_command("/ip/dhcp-server/lease/print", std::iter::empty::<(&str, &str)>()));
    let hs_act_fut = client.run(build_command("/ip/hotspot/active/print", std::iter::empty::<(&str, &str)>()));
    let hs_host_fut = client.run(build_command("/ip/hotspot/host/print", std::iter::empty::<(&str, &str)>()));
    let ppp_act_fut = client.run(build_command("/ppp/active/print", std::iter::empty::<(&str, &str)>()));
    let wifi_fut = client.run(build_command("/interface/wireless/registration-table/print", std::iter::empty::<(&str, &str)>()));
    let neighbor_fut = client.run(build_command("/ip/neighbor/print", std::iter::empty::<(&str, &str)>()));
    let arp_fut = client.run(build_command("/ip/arp/print", std::iter::empty::<(&str, &str)>()));

    let (ident_res, res_res, iface_res, ip_res, dhcp_res, hs_act_res, hs_host_res, ppp_act_res, wifi_res, neighbor_res, arp_res) =
        tokio::join!(ident_fut, res_fut, iface_fut, ip_fut, dhcp_fut, hs_act_fut, hs_host_fut, ppp_act_fut, wifi_fut, neighbor_fut, arp_fut);

    let mut nodes: Vec<TopologyNode> = Vec::new();
    let mut edges: Vec<TopologyEdge> = Vec::new();

    // 1. Root Node: The Core Router
    let router_name = ident_res.ok()
        .and_then(|r| r.into_iter().next())
        .and_then(|r| r.get("name").map(|s| s.to_string()))
        .unwrap_or_else(|| "MikroTik Core".into());

    let mut router_extra = HashMap::new();
    if let Ok(r_rows) = res_res {
        if let Some(r) = r_rows.first() {
            if let Some(cpu) = r.get("cpu-load") { router_extra.insert("cpu_load".into(), format!("{}%", cpu)); }
            if let Some(ver) = r.get("version") { router_extra.insert("version".into(), ver.to_string()); }
            if let Some(board) = r.get("board-name") { router_extra.insert("board_name".into(), board.to_string()); }
            if let Some(uptime) = r.get("uptime") { router_extra.insert("uptime".into(), uptime.to_string()); }
            if let Some(mem) = r.get("free-memory") {
                let bytes: u64 = mem.parse().unwrap_or(0);
                router_extra.insert("free_ram".into(), format!("{} MB", bytes / (1024 * 1024)));
            }
        }
    }

    nodes.push(TopologyNode {
        id: "node_router".into(),
        label: router_name,
        node_type: "router".into(),
        sub_type: "core".into(),
        ip: None,
        mac: None,
        vendor: Some("MikroTik".into()),
        interface: None,
        parent_id: None,
        status: "online".into(),
        extra: router_extra,
    });

    // 2. Build Subnet-to-Interface Map from /ip/address/print
    // Example: "192.168.100.1/24" on "ether5" => ("192.168.100.", "ether5")
    let mut subnet_to_iface: Vec<(String, String)> = Vec::new();
    let mut iface_ip_map: HashMap<String, String> = HashMap::new();

    if let Ok(ip_rows) = ip_res {
        for row in ip_rows {
            let addr = row.get("address").unwrap_or("").to_string();
            let iface = row.get("interface").unwrap_or("").to_string();
            if addr.is_empty() || iface.is_empty() {
                continue;
            }
            iface_ip_map.insert(iface.clone(), addr.clone());
            // Extract /24 prefix: "192.168.100.1/24" -> "192.168.100."
            let raw_ip = addr.split('/').next().unwrap_or("");
            let parts: Vec<&str> = raw_ip.split('.').collect();
            if parts.len() == 4 {
                let prefix = format!("{}.{}.{}.", parts[0], parts[1], parts[2]);
                subnet_to_iface.push((prefix, iface));
            }
        }
    }

    // 3. ARP Table mapping (IP -> Interface & MAC)
    let mut arp_iface_map: HashMap<String, String> = HashMap::new();
    let mut arp_mac_map: HashMap<String, String> = HashMap::new();
    if let Ok(arp_rows) = arp_res {
        for row in arp_rows {
            let ip = row.get("address").unwrap_or("").to_string();
            let mac = row.get("mac-address").unwrap_or("").to_uppercase();
            let iface = row.get("interface").unwrap_or("").to_string();
            if !ip.is_empty() && !iface.is_empty() {
                arp_iface_map.insert(ip.clone(), iface);
            }
            if !ip.is_empty() && !mac.is_empty() {
                arp_mac_map.insert(ip, mac);
            }
        }
    }

    // 4. Interface Nodes & Router-to-Interface Connections
    let mut iface_nodes_map: HashMap<String, String> = HashMap::new();

    if let Ok(ifaces) = iface_res {
        for row in ifaces {
            let name = row.get("name").unwrap_or("").to_string();
            let if_type = row.get("type").unwrap_or("ether").to_string();
            let running = row.get("running").map(|v| v == "true").unwrap_or(false);
            let disabled = row.get("disabled").map(|v| v == "true").unwrap_or(false);

            if disabled || name.is_empty() {
                continue;
            }

            let is_relevant = match filter.as_str() {
                "all" => ["ether", "bridge", "wlan", "vlan", "pppoe", "wireguard"].iter().any(|t| if_type.contains(t)),
                "pppoe" => if_type.contains("pppoe") || name.contains("pppoe"),
                "hotspot" => name == "MANYTAL" || if_type == "bridge" || if_type.contains("wlan"),
                "dhcp" => if_type == "ether" || if_type == "bridge" || if_type.contains("wlan"),
                "wifi" => if_type.contains("wlan") || if_type.contains("wireless"),
                _ => true,
            };

            if is_relevant {
                let node_id = format!("if_{}", name);
                iface_nodes_map.insert(name.clone(), node_id.clone());

                let mut extra = HashMap::new();
                extra.insert("type".into(), if_type.clone());
                if let Some(ip) = iface_ip_map.get(&name) {
                    extra.insert("ip_assigned".into(), ip.clone());
                }
                if let Some(rx) = row.get("rx-byte") { extra.insert("rx_bytes".into(), rx.to_string()); }
                if let Some(tx) = row.get("tx-byte") { extra.insert("tx_bytes".into(), tx.to_string()); }

                nodes.push(TopologyNode {
                    id: node_id.clone(),
                    label: name.clone(),
                    node_type: "interface".into(),
                    sub_type: if_type.clone(),
                    ip: iface_ip_map.get(&name).cloned(),
                    mac: row.get("mac-address").map(|s| s.to_string()),
                    vendor: Some("MikroTik".into()),
                    interface: Some(name.clone()),
                    parent_id: Some("node_router".into()),
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

    // Helper closure to resolve parent interface node from client IP
    let resolve_parent = |client_ip: &str, direct_if: Option<&str>| -> String {
        // 1. Direct interface argument if known
        if let Some(i) = direct_if {
            if let Some(node_id) = iface_nodes_map.get(i) {
                return node_id.clone();
            }
        }
        // 2. ARP table resolution
        if let Some(arp_if) = arp_iface_map.get(client_ip) {
            if let Some(node_id) = iface_nodes_map.get(arp_if) {
                return node_id.clone();
            }
        }
        // 3. Subnet prefix matching (e.g. 192.168.100.x -> ether5)
        for (prefix, iface) in &subnet_to_iface {
            if ip_matches_subnet(client_ip, prefix) {
                if let Some(node_id) = iface_nodes_map.get(iface) {
                    return node_id.clone();
                }
            }
        }
        // Fallback to bridge, hotspot interface MANYTAL, or core router
        if let Some(bridge_node) = iface_nodes_map.get("bridge").or_else(|| iface_nodes_map.get("MANYTAL")) {
            bridge_node.clone()
        } else {
            "node_router".into()
        }
    };

    // 5. Access Point & Neighbor Detection (e.g. Access Point plugged into ether5)
    let mut ap_map: HashMap<String, String> = HashMap::new(); // key = interface_name, value = ap_node_id

    // Check neighbors first
    if let Ok(neighbors) = neighbor_res {
        for row in neighbors {
            let iface = row.get("interface").unwrap_or("").to_string();
            let identity = row.get("identity").unwrap_or("Access Point").to_string();
            let ip = row.get("address").unwrap_or("").to_string();
            let mac = row.get("mac-address").unwrap_or("").to_uppercase();

            if iface.is_empty() {
                continue;
            }

            let parent_if_node = iface_nodes_map.get(&iface).cloned().unwrap_or_else(|| "node_router".into());
            let ap_node_id = format!("ap_{}_{}", iface, if !mac.is_empty() { &mac } else { "dev" });

            let mut extra = HashMap::new();
            extra.insert("identity".into(), identity.clone());
            extra.insert("device_type".into(), "Access Point / Wireless AP".into());
            if !ip.is_empty() { extra.insert("ip".into(), ip.clone()); }

            nodes.push(TopologyNode {
                id: ap_node_id.clone(),
                label: format!("📡 AP: {} ({})", identity, if !ip.is_empty() { &ip } else { &iface }),
                node_type: "ap".into(),
                sub_type: "ap".into(),
                ip: if !ip.is_empty() { Some(ip) } else { None },
                mac: if !mac.is_empty() { Some(mac.clone()) } else { None },
                vendor: Some(detect_device_vendor(&mac, &identity)),
                interface: Some(iface.clone()),
                parent_id: Some(parent_if_node.clone()),
                status: "online".into(),
                extra,
            });

            edges.push(TopologyEdge {
                id: format!("edge_{}_{}", parent_if_node, ap_node_id),
                source: parent_if_node,
                target: ap_node_id.clone(),
                edge_type: "physical".into(),
                speed: Some("Ethernet Port Link".into()),
            });

            ap_map.insert(iface, ap_node_id);
        }
    }

    // Auto-detect AP on ether5 if subnet 192.168.100.x is on ether5 and AP wasn't in neighbor discovery
    if !ap_map.contains_key("ether5") {
        if let Some(if_node) = iface_nodes_map.get("ether5") {
            let ap_id = "ap_ether5_wlan".to_string();
            let mut extra = HashMap::new();
            extra.insert("device_type".into(), "Access Point (WiFi Hub)".into());
            extra.insert("port".into(), "ether5".into());
            if let Some(assigned) = iface_ip_map.get("ether5") {
                extra.insert("gateway_ip".into(), assigned.clone());
            }

            nodes.push(TopologyNode {
                id: ap_id.clone(),
                label: "📡 Access Point (ether5)".into(),
                node_type: "ap".into(),
                sub_type: "ap".into(),
                ip: Some("192.168.100.3".into()), // Known AP IP on port 5
                mac: None,
                vendor: Some("Access Point".into()),
                interface: Some("ether5".into()),
                parent_id: Some(if_node.clone()),
                status: "online".into(),
                extra,
            });

            edges.push(TopologyEdge {
                id: "edge_ether5_ap".into(),
                source: if_node.clone(),
                target: ap_id.clone(),
                edge_type: "physical".into(),
                speed: Some("LAN Cable (Port 5)".into()),
            });

            ap_map.insert("ether5".into(), ap_id);
        }
    }

    let mut seen_ips = HashSet::new();
    let mut seen_macs = HashSet::new();

    // 6. Hotspot Active Clients & Hosts
    if filter == "all" || filter == "hotspot" {
        if let Ok(hs_rows) = hs_act_res {
            for row in hs_rows {
                let user = row.get("user").unwrap_or("guest").to_string();
                let ip = row.get("address").unwrap_or("").to_string();
                let mac = row.get("mac-address").unwrap_or("").to_uppercase();
                let server = row.get("server").unwrap_or("").to_string();
                let uptime = row.get("uptime").unwrap_or("").to_string();

                if ip.is_empty() { continue; }
                seen_ips.insert(ip.clone());
                if !mac.is_empty() { seen_macs.insert(mac.clone()); }

                let node_id = format!("client_hs_{}", if !mac.is_empty() { &mac } else { &ip });
                let parent_node = resolve_parent(&ip, if !server.is_empty() { Some(&server) } else { None });

                let mut extra = HashMap::new();
                extra.insert("category".into(), "hotspot".into());
                extra.insert("username".into(), user.clone());
                extra.insert("uptime".into(), uptime);
                extra.insert("server".into(), server.clone());
                if let Some(bytes_in) = row.get("bytes-in") { extra.insert("bytes_in".into(), bytes_in.to_string()); }
                if let Some(bytes_out) = row.get("bytes-out") { extra.insert("bytes_out".into(), bytes_out.to_string()); }

                nodes.push(TopologyNode {
                    id: node_id.clone(),
                    label: format!("🎟️ {} ({})", user, ip),
                    node_type: "device".into(),
                    sub_type: "hotspot".into(),
                    ip: Some(ip.clone()),
                    mac: Some(mac.clone()),
                    vendor: Some(detect_device_vendor(&mac, &user)),
                    interface: Some(server),
                    parent_id: Some(parent_node.clone()),
                    status: "active".into(),
                    extra,
                });

                edges.push(TopologyEdge {
                    id: format!("edge_{}_{}", parent_node, node_id),
                    source: parent_node,
                    target: node_id,
                    edge_type: "logical".into(),
                    speed: Some("Hotspot Auth".into()),
                });
            }
        }

        // Also check unauthenticated / bypassed Hotspot Hosts from /ip/hotspot/host/print
        if let Ok(hs_hosts) = hs_host_res {
            for row in hs_hosts {
                let ip = row.get("address").unwrap_or("").to_string();
                let mac = row.get("mac-address").unwrap_or("").to_uppercase();
                let authorized = row.get("authorized").map(|v| v == "true").unwrap_or(false);
                let bypassed = row.get("bypassed").map(|v| v == "true").unwrap_or(false);
                let server = row.get("server").unwrap_or("").to_string();

                if ip.is_empty() || seen_ips.contains(&ip) || (!mac.is_empty() && seen_macs.contains(&mac)) {
                    continue;
                }
                seen_ips.insert(ip.clone());
                if !mac.is_empty() { seen_macs.insert(mac.clone()); }

                let node_id = format!("client_hshost_{}", if !mac.is_empty() { &mac } else { &ip });
                let parent_node = resolve_parent(&ip, if !server.is_empty() { Some(&server) } else { None });

                let mut extra = HashMap::new();
                extra.insert("category".into(), "hotspot-host".into());
                extra.insert("authorized".into(), authorized.to_string());
                extra.insert("bypassed".into(), bypassed.to_string());
                if !server.is_empty() { extra.insert("server".into(), server.clone()); }

                nodes.push(TopologyNode {
                    id: node_id.clone(),
                    label: format!("📱 Host: {} ({})", detect_device_vendor(&mac, ""), ip),
                    node_type: "device".into(),
                    sub_type: "hotspot".into(),
                    ip: Some(ip.clone()),
                    mac: Some(mac.clone()),
                    vendor: Some(detect_device_vendor(&mac, "")),
                    interface: Some(server),
                    parent_id: Some(parent_node.clone()),
                    status: if authorized { "online".into() } else if bypassed { "bypassed".into() } else { "idle".into() },
                    extra,
                });

                edges.push(TopologyEdge {
                    id: format!("edge_{}_{}", parent_node, node_id),
                    source: parent_node,
                    target: node_id,
                    edge_type: "logical".into(),
                    speed: Some(if bypassed { "Bypassed" } else { "Hotspot Host" }.into()),
                });
            }
        }
    }

    // 7. PPPoE Active Clients
    if filter == "all" || filter == "pppoe" {
        let mut ppp_count = 0;
        if let Ok(ppp_rows) = ppp_act_res {
            for row in ppp_rows {
                let name = row.get("name").unwrap_or("pppoe-client").to_string();
                let ip = row.get("address").unwrap_or("").to_string();
                let caller_id = row.get("caller-id").unwrap_or("").to_string();
                let uptime = row.get("uptime").unwrap_or("").to_string();

                if ip.is_empty() { continue; }
                seen_ips.insert(ip.clone());
                ppp_count += 1;

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
                    mac: Some(caller_id.clone()),
                    vendor: Some(detect_device_vendor(&caller_id, &name)),
                    interface: Some("pppoe".into()),
                    parent_id: Some("node_router".into()),
                    status: "active".into(),
                    extra,
                });

                edges.push(TopologyEdge {
                    id: format!("edge_router_{}", node_id),
                    source: "node_router".into(),
                    target: node_id,
                    edge_type: "tunnel".into(),
                    speed: Some("PPPoE Virtual Tunnel".into()),
                });
            }
        }

        // If filter is pppoe and there are 0 active clients, show PPPoE Server Concentrator status node
        if filter == "pppoe" && ppp_count == 0 {
            let ppp_srv_id = "node_pppoe_server".to_string();
            nodes.push(TopologyNode {
                id: ppp_srv_id.clone(),
                label: "🌐 PPPoE Server Concentrator (0 Active Tunnels)".into(),
                node_type: "interface".into(),
                sub_type: "pppoe".into(),
                ip: None,
                mac: None,
                vendor: Some("MikroTik".into()),
                interface: Some("pppoe-server".into()),
                parent_id: Some("node_router".into()),
                status: "idle".into(),
                extra: HashMap::new(),
            });
            edges.push(TopologyEdge {
                id: "edge_router_pppoe_srv".into(),
                source: "node_router".into(),
                target: ppp_srv_id,
                edge_type: "tunnel".into(),
                speed: Some("Ready & Listening".into()),
            });
        }
    }

    // 8. DHCP Leases (Hierarchical correlation to port / AP)
    if filter == "all" || filter == "dhcp" {
        if let Ok(dhcp_rows) = dhcp_res {
            for row in dhcp_rows {
                let ip = row.get("address").unwrap_or("").to_string();
                let mac = row.get("mac-address").unwrap_or("").to_uppercase();
                let host_name = row.get("host-name").unwrap_or("").to_string();
                let status = row.get("status").unwrap_or("bound").to_string();
                let server = row.get("server").unwrap_or("").to_string();

                if ip.is_empty() || seen_ips.contains(&ip) || (!mac.is_empty() && seen_macs.contains(&mac)) {
                    continue;
                }
                seen_ips.insert(ip.clone());

                let vendor = detect_device_vendor(&mac, &host_name);
                let label = if !host_name.is_empty() {
                    format!("📱 {} ({})", host_name, ip)
                } else {
                    format!("💻 {} ({})", vendor, ip)
                };

                let node_id = format!("client_dhcp_{}", if !mac.is_empty() { &mac } else { &ip });

                // Find parent interface node
                let mut parent_node = resolve_parent(&ip, if !server.is_empty() { Some(&server) } else { None });

                // If parent interface has an Access Point (e.g. ether5 has AP), link device to the AP instead!
                for (iface_name, ap_id) in &ap_map {
                    if let Some(if_node) = iface_nodes_map.get(iface_name) {
                        if if_node == &parent_node {
                            parent_node = ap_id.clone();
                            break;
                        }
                    }
                }

                let mut extra = HashMap::new();
                extra.insert("category".into(), "dhcp".into());
                extra.insert("host_name".into(), host_name);
                extra.insert("vendor".into(), vendor.clone());
                extra.insert("lease_status".into(), status.clone());
                if !server.is_empty() { extra.insert("dhcp_server".into(), server); }

                nodes.push(TopologyNode {
                    id: node_id.clone(),
                    label,
                    node_type: "device".into(),
                    sub_type: "dhcp".into(),
                    ip: Some(ip.clone()),
                    mac: Some(mac.clone()),
                    vendor: Some(vendor),
                    interface: None,
                    parent_id: Some(parent_node.clone()),
                    status: if status == "bound" { "online".into() } else { "idle".into() },
                    extra,
                });

                edges.push(TopologyEdge {
                    id: format!("edge_{}_{}", parent_node, node_id),
                    source: parent_node,
                    target: node_id,
                    edge_type: "physical".into(),
                    speed: Some("DHCP Lease (Bound)".into()),
                });
            }
        }
    }

    // 9. Wireless WiFi Clients
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

                let vendor = detect_device_vendor(&mac, "");
                let node_id = format!("client_wifi_{}", mac);
                let parent_node = iface_nodes_map.get(&iface).cloned().unwrap_or_else(|| "node_router".into());

                let mut extra = HashMap::new();
                extra.insert("category".into(), "wifi".into());
                extra.insert("signal".into(), signal.clone());
                extra.insert("tx_rate".into(), tx_rate);
                extra.insert("rx_rate".into(), rx_rate);
                extra.insert("vendor".into(), vendor.clone());

                nodes.push(TopologyNode {
                    id: node_id.clone(),
                    label: format!("📶 WiFi: {} ({})", vendor, mac),
                    node_type: "device".into(),
                    sub_type: "wifi".into(),
                    ip: None,
                    mac: Some(mac.clone()),
                    vendor: Some(vendor),
                    interface: Some(iface.clone()),
                    parent_id: Some(parent_node.clone()),
                    status: "online".into(),
                    extra,
                });

                edges.push(TopologyEdge {
                    id: format!("edge_{}_{}", parent_node, node_id),
                    source: parent_node,
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

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use routeros_core::build_command;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::{AppState, RouterTarget};

#[derive(Deserialize, Debug, Default)]
pub struct ConnectedDevicesReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub search: Option<String>,
    pub filter_status: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct DeviceAggregation {
    mac: String,
    ip: String,
    hostname: String,
    interface: String,
    
    // DHCP info
    has_dhcp: bool,
    dhcp_status: String,
    dhcp_static: bool,
    dhcp_expires: String,
    
    // Hotspot info
    has_hotspot: bool,
    hotspot_authorized: bool,
    hotspot_bypassed: bool,
    hotspot_user: String,
    hotspot_to_address: String,
    hotspot_uptime: String,
    hotspot_bytes_in: String,
    hotspot_bytes_out: String,

    // Hotspot Binding info
    has_binding: bool,
    binding_type: String,

    // Wireless info
    is_wireless: bool,
    signal_strength: String,
    tx_rate: String,
    rx_rate: String,

    // ARP info
    has_arp: bool,
    arp_interface: String,
}

/// Unified Connected Devices & Cross-Layer Correlator:
/// Simultanously queries DHCP leases, Hotspot hosts, Hotspot active users,
/// IP bindings, ARP tables, and Wireless registrations across the single pooled socket.
/// Stitches all layers together by MAC and IP address, giving an instant, comprehensive
/// relationship map of every connected device.
pub async fn connected_devices(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<ConnectedDevicesReq>>,
) -> Result<Json<Value>, ApiError> {
    let t0 = Instant::now();
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // 1. Dispatch 6 concurrent queries over the multiplexed TCP socket
    let dhcp_fut = client.run(build_command("/ip/dhcp-server/lease/print", std::iter::empty::<(&str, &str)>()));
    let hs_host_fut = client.run(build_command("/ip/hotspot/host/print", std::iter::empty::<(&str, &str)>()));
    let hs_active_fut = client.run(build_command("/ip/hotspot/active/print", std::iter::empty::<(&str, &str)>()));
    let hs_bind_fut = client.run(build_command("/ip/hotspot/ip-binding/print", std::iter::empty::<(&str, &str)>()));
    let arp_fut = client.run(build_command("/ip/arp/print", std::iter::empty::<(&str, &str)>()));
    let wlan_fut = client.run(build_command("/interface/wireless/registration-table/print", std::iter::empty::<(&str, &str)>()));

    let (dhcp_res, hs_host_res, hs_active_res, hs_bind_res, arp_res, wlan_res) =
        tokio::join!(dhcp_fut, hs_host_fut, hs_active_fut, hs_bind_fut, arp_fut, wlan_fut);

    let dhcp_rows = dhcp_res.ok().unwrap_or_default();
    let hs_host_rows = hs_host_res.ok().unwrap_or_default();
    let hs_active_rows = hs_active_res.ok().unwrap_or_default();
    let hs_bind_rows = hs_bind_res.ok().unwrap_or_default();
    let arp_rows = arp_res.ok().unwrap_or_default();
    let wlan_rows = wlan_res.ok().unwrap_or_default();

    // Key = MAC address in UPPERCASE
    let mut map: HashMap<String, DeviceAggregation> = HashMap::new();

    let normalize_mac = |m: &str| -> String {
        m.trim().to_uppercase()
    };

    // 2. Index Hotspot Active Users by IP & User
    let mut active_by_ip: HashMap<String, (String, String, String, String)> = HashMap::new(); // ip -> (user, uptime, bytes_in, bytes_out)
    for row in hs_active_rows {
        if let Some(ip) = row.get("address") {
            let user = row.get("user").unwrap_or("").to_string();
            let uptime = row.get("uptime").unwrap_or("").to_string();
            let bin = row.get("bytes-in").unwrap_or("0").to_string();
            let bout = row.get("bytes-out").unwrap_or("0").to_string();
            active_by_ip.insert(ip.to_string(), (user, uptime, bin, bout));
        }
    }

    // 3. Process DHCP Leases
    for row in dhcp_rows {
        if let Some(raw_mac) = row.get("mac-address") {
            let mac = normalize_mac(raw_mac);
            if mac.is_empty() { continue; }
            let entry = map.entry(mac.clone()).or_insert_with(|| DeviceAggregation {
                mac: mac.clone(),
                ..Default::default()
            });
            entry.has_dhcp = true;
            if let Some(ip) = row.get("address") {
                entry.ip = ip.to_string();
            }
            if let Some(host) = row.get("host-name") {
                entry.hostname = host.to_string();
            }
            if let Some(status) = row.get("status") {
                entry.dhcp_status = status.to_string();
            }
            if let Some(exp) = row.get("expires-after") {
                entry.dhcp_expires = exp.to_string();
            }
            if let Some(dyn_flag) = row.get("dynamic") {
                entry.dhcp_static = dyn_flag != "true";
            }
        }
    }

    // 4. Process Hotspot Hosts
    for row in hs_host_rows {
        if let Some(raw_mac) = row.get("mac-address") {
            let mac = normalize_mac(raw_mac);
            if mac.is_empty() { continue; }
            let entry = map.entry(mac.clone()).or_insert_with(|| DeviceAggregation {
                mac: mac.clone(),
                ..Default::default()
            });
            entry.has_hotspot = true;
            if entry.ip.is_empty() {
                if let Some(ip) = row.get("address") {
                    entry.ip = ip.to_string();
                }
            }
            if let Some(to) = row.get("to-address") {
                entry.hotspot_to_address = to.to_string();
            }
            if let Some(server) = row.get("server") {
                if entry.interface.is_empty() {
                    entry.interface = server.to_string();
                }
            }
            entry.hotspot_authorized = row.get("authorized").map(|v| v == "true").unwrap_or(false);
            entry.hotspot_bypassed = row.get("bypassed").map(|v| v == "true").unwrap_or(false);
            if let Some(up) = row.get("uptime") {
                entry.hotspot_uptime = up.to_string();
            }
            if let Some(bin) = row.get("bytes-in") {
                entry.hotspot_bytes_in = bin.to_string();
            }
            if let Some(bout) = row.get("bytes-out") {
                entry.hotspot_bytes_out = bout.to_string();
            }
            if let Some(comment) = row.get("comment") {
                if entry.hostname.is_empty() {
                    entry.hostname = comment.to_string();
                }
            }

            // Check if active user corresponds to this host IP
            if let Some((user, uptime, bin, bout)) = active_by_ip.get(&entry.ip) {
                entry.hotspot_user = user.clone();
                if entry.hotspot_uptime.is_empty() {
                    entry.hotspot_uptime = uptime.clone();
                }
                if entry.hotspot_bytes_in.is_empty() || entry.hotspot_bytes_in == "0" {
                    entry.hotspot_bytes_in = bin.clone();
                }
                if entry.hotspot_bytes_out.is_empty() || entry.hotspot_bytes_out == "0" {
                    entry.hotspot_bytes_out = bout.clone();
                }
            }
        }
    }

    // 5. Process Hotspot IP Bindings
    for row in hs_bind_rows {
        if let Some(raw_mac) = row.get("mac-address") {
            let mac = normalize_mac(raw_mac);
            if mac.is_empty() { continue; }
            let entry = map.entry(mac.clone()).or_insert_with(|| DeviceAggregation {
                mac: mac.clone(),
                ..Default::default()
            });
            entry.has_binding = true;
            if let Some(t) = row.get("type") {
                entry.binding_type = t.to_string();
            }
            if let Some(comm) = row.get("comment") {
                if entry.hostname.is_empty() {
                    entry.hostname = comm.to_string();
                }
            }
        }
    }

    // 6. Process Wireless Registrations
    for row in wlan_rows {
        if let Some(raw_mac) = row.get("mac-address") {
            let mac = normalize_mac(raw_mac);
            if mac.is_empty() { continue; }
            let entry = map.entry(mac.clone()).or_insert_with(|| DeviceAggregation {
                mac: mac.clone(),
                ..Default::default()
            });
            entry.is_wireless = true;
            if let Some(sig) = row.get("signal-strength") {
                entry.signal_strength = sig.to_string();
            }
            if let Some(tx) = row.get("tx-rate") {
                entry.tx_rate = tx.to_string();
            }
            if let Some(rx) = row.get("rx-rate") {
                entry.rx_rate = rx.to_string();
            }
            if let Some(iface) = row.get("interface") {
                entry.interface = iface.to_string();
            }
        }
    }

    // 7. Process ARP Table
    for row in arp_rows {
        if let Some(raw_mac) = row.get("mac-address") {
            let mac = normalize_mac(raw_mac);
            if mac.is_empty() { continue; }
            let entry = map.entry(mac.clone()).or_insert_with(|| DeviceAggregation {
                mac: mac.clone(),
                ..Default::default()
            });
            entry.has_arp = true;
            if entry.ip.is_empty() {
                if let Some(ip) = row.get("address") {
                    entry.ip = ip.to_string();
                }
            }
            if let Some(iface) = row.get("interface") {
                entry.arp_interface = iface.to_string();
                if entry.interface.is_empty() {
                    entry.interface = iface.to_string();
                }
            }
        }
    }

    // 8. Compile and determine Unified Summary Badges
    let search_lower = req.search.as_deref().unwrap_or("").to_lowercase();
    let filter_status = req.filter_status.as_deref().unwrap_or("all");

    let mut device_list = Vec::new();
    let mut total_authorized = 0;
    let mut total_pending = 0;
    let mut total_bypassed = 0;
    let mut total_dhcp_only = 0;
    let mut total_wireless = 0;

    for (_, dev) in map {
        // Classify status
        let (status_badge, status_label) = if dev.hotspot_authorized || !dev.hotspot_user.is_empty() {
            total_authorized += 1;
            ("AUTHORIZED_VOUCHER", "Hotspot Login Aktif")
        } else if dev.hotspot_bypassed || dev.binding_type == "bypassed" {
            total_bypassed += 1;
            ("BYPASSED", "Bypass IP Binding (No Login)")
        } else if dev.has_hotspot && !dev.hotspot_authorized {
            total_pending += 1;
            ("PENDING_HOTSPOT", "Host Terdeteksi (Belum Login)")
        } else if dev.has_dhcp {
            total_dhcp_only += 1;
            ("DHCP_ONLY", "Klien LAN DHCP Murni")
        } else {
            ("ARP_DETECTED", "Entri ARP Saja")
        };

        if dev.is_wireless {
            total_wireless += 1;
        }

        // Apply filters
        if !search_lower.is_empty() {
            let m_mac = dev.mac.to_lowercase().contains(&search_lower);
            let m_ip = dev.ip.to_lowercase().contains(&search_lower);
            let m_host = dev.hostname.to_lowercase().contains(&search_lower);
            let m_user = dev.hotspot_user.to_lowercase().contains(&search_lower);
            if !m_mac && !m_ip && !m_host && !m_user {
                continue;
            }
        }

        if filter_status == "authorized" && status_badge != "AUTHORIZED_VOUCHER" {
            continue;
        }
        if filter_status == "unauthorized" && status_badge != "PENDING_HOTSPOT" {
            continue;
        }
        if filter_status == "bypassed" && status_badge != "BYPASSED" {
            continue;
        }
        if filter_status == "dhcp_only" && status_badge != "DHCP_ONLY" {
            continue;
        }

        device_list.push(json!({
            "mac_address": dev.mac,
            "ip_address": dev.ip,
            "hostname": if dev.hostname.is_empty() { "Unknown Device".into() } else { dev.hostname },
            "interface": dev.interface,
            "status": {
                "badge": status_badge,
                "label": status_label,
            },
            "dhcp": {
                "detected": dev.has_dhcp,
                "status": dev.dhcp_status,
                "is_static": dev.dhcp_static,
                "expires_after": dev.dhcp_expires,
            },
            "hotspot": {
                "detected": dev.has_hotspot,
                "authorized": dev.hotspot_authorized,
                "bypassed": dev.hotspot_bypassed,
                "username": dev.hotspot_user,
                "to_address": dev.hotspot_to_address,
                "uptime": dev.hotspot_uptime,
                "bytes_in": dev.hotspot_bytes_in,
                "bytes_out": dev.hotspot_bytes_out,
            },
            "binding": {
                "has_binding": dev.has_binding,
                "type": dev.binding_type,
            },
            "wireless": {
                "is_wireless": dev.is_wireless,
                "signal_strength": dev.signal_strength,
                "tx_rate": dev.tx_rate,
                "rx_rate": dev.rx_rate,
            },
            "arp": {
                "detected": dev.has_arp,
                "interface": dev.arp_interface,
            }
        }));
    }

    let elapsed = t0.elapsed();

    Ok(Json(json!({
        "success": true,
        "execution_time_ms": elapsed.as_millis() as u64,
        "summary": {
            "total_connected_devices": device_list.len(),
            "authorized_hotspot_users": total_authorized,
            "pending_captive_hosts": total_pending,
            "bypassed_devices": total_bypassed,
            "dhcp_only_devices": total_dhcp_only,
            "wireless_clients": total_wireless,
        },
        "devices": device_list
    })))
}

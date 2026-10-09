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
pub struct AutoBypassApReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub mac_address: String,
    pub address: Option<String>,
    pub comment: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct InfrastructureDevice {
    pub mac: String,
    pub ip: Option<String>,
    pub identity: Option<String>,
    pub vendor: String,
    pub device_type: String, // "Access Point", "Switch", "Router", "IP Camera", "Client"
    pub interface: Option<String>,
    pub detection_method: String, // "Neighbor Discovery (LLDP/MNDP)", "ARP/Host Analysis"
    pub is_bypassed_in_hotspot: bool,
}

/// Helper function to identify hardware vendor and category from MAC prefix
fn identify_vendor(mac: &str) -> (&'static str, &'static str) {
    let clean = mac.replace(['-', ':'], "").to_uppercase();
    if clean.len() < 6 {
        return ("Unknown", "Client");
    }
    let prefix = &clean[0..6];

    match prefix {
        // MikroTik
        "000C42" | "488F5A" | "64D154" | "789A18" | "B869F4" | "C4AD34" | "D4CA6D" | "E48D8C" => {
            ("MikroTik", "Router / Access Point")
        }
        // Ubiquiti Networks
        "002722" | "0418D6" | "24A43C" | "44D9E7" | "68D79A" | "70A741" | "7483C2" | "788A20" | "802AA8" | "AC8BA3" | "B4FBE4" | "DC9FDB" | "E063DA" | "F09FC2" => {
            ("Ubiquiti (UniFi / airMAX)", "Access Point")
        }
        // TP-Link
        "50D4F7" | "EC086B" | "14CC20" | "54E6FC" | "000AEB" | "001478" | "0019E0" | "002127" | "0023CD" | "002586" | "1C3BF3" | "30B5C2" | "50C7BF" | "60E327" | "704F57" | "98DAC4" | "C006C3" | "C4E984" => {
            ("TP-Link", "Access Point / Router")
        }
        // Tenda
        "C83A35" | "502B73" | "0495E6" | "CC2D21" | "E865D4" => {
            ("Tenda", "Access Point / Router")
        }
        // Ruijie / Reyee
        "001AA9" | "00D0F8" | "14144B" | "58696C" | "708912" | "E45602" => {
            ("Ruijie / Reyee", "Access Point")
        }
        // Dahua / Hikvision IP Cameras
        "3CEF8C" | "A0BD1D" | "BC325F" | "48EA63" | "4C11BF" => {
            ("Hikvision / Dahua", "IP Camera")
        }
        _ => ("Generic / OEM", "Network Device"),
    }
}

/// POST /api/v1/network/infrastructure/scan - Correlates Neighbor Discovery (MNDP/CDP/LLDP),
/// Hotspot IP-Bindings, ARP, and Hotspot Hosts to discover bridged Access Points and infrastructure nodes
pub async fn scan_infrastructure(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Concurrent fetch: neighbors, hotspot ip-bindings, arp table, hotspot hosts
    let neighbor_fut = client.run(build_command("/ip/neighbor/print", std::iter::empty::<(&str, &str)>()));
    let binding_fut = client.run(build_command("/ip/hotspot/ip-binding/print", std::iter::empty::<(&str, &str)>()));
    let arp_fut = client.run(build_command("/ip/arp/print", std::iter::empty::<(&str, &str)>()));
    let hs_host_fut = client.run(build_command("/ip/hotspot/host/print", std::iter::empty::<(&str, &str)>()));

    let (neighbor_res, binding_res, arp_res, hs_host_res) = tokio::join!(neighbor_fut, binding_fut, arp_fut, hs_host_fut);

    // 1. Gather already bypassed MACs in Hotspot IP-Binding
    let mut bypassed_macs = HashSet::new();
    if let Ok(bindings) = binding_res {
        for b in bindings {
            if let Some(mac) = b.get("mac-address") {
                let clean_mac = mac.to_uppercase();
                let b_type = b.get("type").unwrap_or("bypassed");
                if b_type == "bypassed" {
                    bypassed_macs.insert(clean_mac);
                }
            }
        }
    }

    let mut devices: Vec<InfrastructureDevice> = Vec::new();
    let mut seen_macs: HashSet<String> = HashSet::new();

    // 2. Process Layer-2 Neighbors (MNDP, CDP, LLDP)
    if let Ok(neighbors) = neighbor_res {
        for n in neighbors {
            let mac = n.get("mac-address").unwrap_or("").to_uppercase();
            if mac.is_empty() { continue; }
            seen_macs.insert(mac.clone());

            let identity = n.get("identity").or_else(|| n.get("system-description")).map(|s| s.to_string());
            let ip = n.get("address").or_else(|| n.get("ipv4-address")).map(|s| s.to_string());
            let iface = n.get("interface").map(|s| s.to_string());
            let platform = n.get("platform").unwrap_or("");

            let (vendor, dev_type) = if !platform.is_empty() {
                (platform, "Router / AP")
            } else {
                identify_vendor(&mac)
            };

            devices.push(InfrastructureDevice {
                mac: mac.clone(),
                ip,
                identity,
                vendor: vendor.to_string(),
                device_type: dev_type.to_string(),
                interface: iface,
                detection_method: "Neighbor Discovery (LLDP/MNDP)".into(),
                is_bypassed_in_hotspot: bypassed_macs.contains(&mac),
            });
        }
    }

    // 3. Process ARP entries matching known networking vendors (Ubiquiti, TP-Link, Ruijie, Tenda)
    if let Ok(arps) = arp_res {
        for a in arps {
            let mac = a.get("mac-address").unwrap_or("").to_uppercase();
            if mac.is_empty() || seen_macs.contains(&mac) { continue; }

            let (vendor, dev_type) = identify_vendor(&mac);
            // If vendor is recognized as AP/Switch/Camera or not generic
            if vendor != "Generic / OEM" {
                seen_macs.insert(mac.clone());
                let ip = a.get("address").map(|s| s.to_string());
                let iface = a.get("interface").map(|s| s.to_string());

                devices.push(InfrastructureDevice {
                    mac: mac.clone(),
                    ip,
                    identity: a.get("comment").map(|s| s.to_string()),
                    vendor: vendor.to_string(),
                    device_type: dev_type.to_string(),
                    interface: iface,
                    detection_method: "ARP Vendor Signature Analysis".into(),
                    is_bypassed_in_hotspot: bypassed_macs.contains(&mac),
                });
            }
        }
    }

    // 4. Process Hotspot Hosts marked as bridge or bypassed
    if let Ok(hosts) = hs_host_res {
        for h in hosts {
            let mac = h.get("mac-address").unwrap_or("").to_uppercase();
            if mac.is_empty() || seen_macs.contains(&mac) { continue; }

            let is_bridge = h.get("bridge").is_some_and(|v| v == "yes" || v == "true");
            let is_bypassed = h.get("bypassed").is_some_and(|v| v == "yes" || v == "true") || bypassed_macs.contains(&mac);
            let (vendor, dev_type) = identify_vendor(&mac);

            if is_bridge || is_bypassed || vendor != "Generic / OEM" {
                seen_macs.insert(mac.clone());
                let ip = h.get("address").map(|s| s.to_string());
                let iface = h.get("interface").map(|s| s.to_string());

                devices.push(InfrastructureDevice {
                    mac: mac.clone(),
                    ip,
                    identity: h.get("comment").map(|s| s.to_string()),
                    vendor: vendor.to_string(),
                    device_type: if is_bridge { "Bridged AP / Switch".into() } else { dev_type.to_string() },
                    interface: iface,
                    detection_method: "Hotspot Host Bridge Correlator".into(),
                    is_bypassed_in_hotspot: is_bypassed,
                });
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "total_infrastructure_devices": devices.len(),
        "devices": devices,
        "recommendation": "Gunakan POST /api/v1/network/infrastructure/auto-bypass-ap untuk membypass MAC AP agar Web Management AP dapat dibuka tanpa login voucher"
    })))
}

/// POST /api/v1/network/infrastructure/auto-bypass-ap - Automatically registers an Access Point
/// or management device into Hotspot IP-Binding with type=bypassed
pub async fn auto_bypass_ap(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AutoBypassApReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let clean_mac = req.mac_address.to_uppercase();
    let comment = req.comment.unwrap_or_else(|| "[Auto-AP] Access Point Bypass".into());

    let mut args = vec![
        ("mac-address", clean_mac.as_str()),
        ("type", "bypassed"),
        ("comment", comment.as_str()),
    ];

    if let Some(addr) = req.address.as_deref() {
        args.push(("address", addr));
    }

    client.run(build_command("/ip/hotspot/ip-binding/add", args)).await?;

    Ok(Json(json!({
        "success": true,
        "mac_address": clean_mac,
        "binding_type": "bypassed",
        "message": format!("Perangkat AP dengan MAC {} berhasil di-bypass dari Captive Portal Hotspot", clean_mac)
    })))
}

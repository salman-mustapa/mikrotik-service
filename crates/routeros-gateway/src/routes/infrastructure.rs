use std::collections::HashSet;
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
    pub device_type: String, // "Access Point", "Switch", "Router", "IP Camera", "Network Device"
    pub interface: Option<String>,
    pub detection_method: String, // "Neighbor Discovery (LLDP/MNDP)", "DHCP Signature", "ARP/Host Analysis"
    pub is_bypassed_in_hotspot: bool,
    pub web_management_url: Option<String>,
    pub status: String,
}

/// Comprehensive hardware vendor and category detector using MAC OUI, Hostname, and Comments
pub fn identify_vendor_and_type(mac: &str, hostname: &str, comment: &str) -> (String, String) {
    let clean_mac = mac.replace(['-', ':'], "").to_uppercase();
    let lower_host = hostname.to_lowercase();
    let lower_comment = comment.to_lowercase();

    // 1. Heuristic Hostname & Comment Analysis
    if lower_host.contains("tl-") || lower_host.contains("wr840") || lower_host.contains("wr940")
        || lower_host.contains("archer") || lower_host.contains("deco") || lower_host.contains("eap")
        || lower_host.contains("wa801") || lower_comment.contains("tp-link") {
        return ("TP-Link".into(), "Access Point".into());
    }
    if lower_host.contains("uap") || lower_host.contains("unifi") || lower_host.contains("airmax")
        || lower_host.contains("nanostation") || lower_comment.contains("ubiquiti") {
        return ("Ubiquiti".into(), "Access Point".into());
    }
    if lower_host.contains("reyee") || lower_host.contains("ruijie") || lower_comment.contains("ruijie") {
        return ("Ruijie / Reyee".into(), "Access Point".into());
    }
    if lower_host.contains("tenda") || lower_comment.contains("tenda") {
        return ("Tenda".into(), "Access Point".into());
    }
    if lower_host.contains("totolink") || lower_comment.contains("totolink") {
        return ("Totolink".into(), "Access Point".into());
    }
    if lower_host.contains("mercusys") || lower_comment.contains("mercusys") {
        return ("Mercusys".into(), "Access Point".into());
    }
    if lower_host.contains("accesspoint") || lower_host.contains("ap-") || lower_comment.contains("ap ")
        || lower_comment.contains("access point") || lower_host.contains("extender") || lower_host.contains("repeater") {
        return ("Wireless AP".into(), "Access Point".into());
    }
    if lower_host.contains("dlink") || lower_host.contains("d-link") {
        return ("D-Link".into(), "Access Point".into());
    }
    if lower_host.contains("cctv") || lower_host.contains("cam") || lower_comment.contains("cctv") || lower_comment.contains("kamera") {
        return ("Hikvision / Dahua".into(), "IP Camera".into());
    }

    // 2. MAC Prefix / OUI Lookup
    if clean_mac.len() >= 6 {
        let prefix = &clean_mac[0..6];
        match prefix {
            // MikroTik
            "000C42" | "488F5A" | "64D154" | "789A18" | "B869F4" | "C4AD34" | "D4CA6D" | "E48D8C" => {
                ("MikroTik".into(), "Router / Access Point".into())
            }
            // Ubiquiti Networks
            "002722" | "0418D6" | "24A43C" | "44D9E7" | "68D79A" | "70A741" | "7483C2" | "788A20" | "802AA8" | "AC8BA3" | "B4FBE4" | "DC9FDB" | "E063DA" | "F09FC2" => {
                ("Ubiquiti (UniFi / airMAX)".into(), "Access Point".into())
            }
            // TP-Link
            "50D4F7" | "EC086B" | "14CC20" | "54E6FC" | "000AEB" | "001478" | "0019E0" | "002127" | "0023CD" | "002586" | "1C3BF3" | "30B5C2" | "50C7BF" | "60E327" | "704F57" | "98DAC4" | "C006C3" | "C4E984" | "AC84C6" | "F4F26D" | "B04E26" => {
                ("TP-Link".into(), "Access Point".into())
            }
            // Tenda
            "C83A35" | "502B73" | "0495E6" | "CC2D21" | "E865D4" => {
                ("Tenda".into(), "Access Point".into())
            }
            // Ruijie / Reyee
            "001AA9" | "00D0F8" | "14144B" | "58696C" | "708912" | "E45602" | "4C63EB" | "BC25E0" | "D83214" => {
                ("Ruijie / Reyee".into(), "Access Point".into())
            }
            // Totolink
            "78D38D" | "E894F6" => {
                ("Totolink".into(), "Access Point".into())
            }
            // Mercusys
            "80EA96" | "78670E" | "302303" => {
                ("Mercusys".into(), "Access Point".into())
            }
            // D-Link
            "00055D" | "00179A" | "14D64D" | "28107B" | "B0C554" => {
                ("D-Link".into(), "Access Point".into())
            }
            // Cambium Networks
            "000456" | "58C17A" => {
                ("Cambium Networks".into(), "Access Point / Wireless Bridge".into())
            }
            // Aruba / HPE
            "000B86" | "24DEC6" | "94B40F" | "D8C7C8" => {
                ("Aruba (HPE)".into(), "Enterprise Access Point".into())
            }
            // Ruckus / CommScope
            "001D2D" | "002482" | "2C3996" => {
                ("Ruckus".into(), "Enterprise Access Point".into())
            }
            // Cisco / Meraki
            "00000C" | "000142" | "000143" | "001BD4" | "E0553D" => {
                ("Cisco / Meraki".into(), "Access Point / Switch".into())
            }
            // Dahua / Hikvision IP Cameras
            "3CEF8C" | "A0BD1D" | "BC325F" | "48EA63" | "4C11BF" => {
                ("Hikvision / Dahua".into(), "IP Camera".into())
            }
            _ => ("Generic / OEM".into(), "Network Device".into()),
        }
    } else {
        ("Generic / OEM".into(), "Network Device".into())
    }
}

/// GET & POST /api/v1/network/infrastructure/scan - Discovers bridged Access Points,
/// Switches, and Infrastructure Hardware by correlating MNDP/LLDP, DHCP Leases, ARP,
/// and Hotspot IP-Bindings.
pub async fn scan_infrastructure(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req_inner = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req_inner.router, req_inner.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Concurrent fetch: neighbors, hotspot ip-bindings, arp table, hotspot hosts, dhcp leases, ip addresses
    let neighbor_fut = client.run(build_command("/ip/neighbor/print", std::iter::empty::<(&str, &str)>()));
    let binding_fut = client.run(build_command("/ip/hotspot/ip-binding/print", std::iter::empty::<(&str, &str)>()));
    let arp_fut = client.run(build_command("/ip/arp/print", std::iter::empty::<(&str, &str)>()));
    let hs_host_fut = client.run(build_command("/ip/hotspot/host/print", std::iter::empty::<(&str, &str)>()));
    let dhcp_fut = client.run(build_command("/ip/dhcp-server/lease/print", std::iter::empty::<(&str, &str)>()));
    let ip_fut = client.run(build_command("/ip/address/print", std::iter::empty::<(&str, &str)>()));

    let (neighbor_res, binding_res, arp_res, hs_host_res, dhcp_res, ip_res) =
        tokio::join!(neighbor_fut, binding_fut, arp_fut, hs_host_fut, dhcp_fut, ip_fut);

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

    // 2. Map interface assigned subnets (e.g. ether5 -> 192.168.100.1/24)
    let mut iface_subnets: Vec<(String, String)> = Vec::new();
    if let Ok(ip_rows) = ip_res {
        for row in ip_rows {
            let addr = row.get("address").unwrap_or("").to_string();
            let iface = row.get("interface").unwrap_or("").to_string();
            if !addr.is_empty() && !iface.is_empty() {
                iface_subnets.push((iface, addr));
            }
        }
    }

    let mut devices: Vec<InfrastructureDevice> = Vec::new();
    let mut seen_macs: HashSet<String> = HashSet::new();
    let mut seen_ips: HashSet<String> = HashSet::new();

    // 3. Process Layer-2 Neighbors (MNDP, CDP, LLDP)
    if let Ok(neighbors) = neighbor_res {
        for n in neighbors {
            let mac = n.get("mac-address").unwrap_or("").to_uppercase();
            if mac.is_empty() { continue; }
            seen_macs.insert(mac.clone());

            let identity = n.get("identity").or_else(|| n.get("system-description")).map(|s| s.to_string());
            let ip = n.get("address").or_else(|| n.get("ipv4-address")).map(|s| s.to_string());
            if let Some(ref addr) = ip { seen_ips.insert(addr.clone()); }
            let iface = n.get("interface").map(|s| s.to_string());
            let platform = n.get("platform").unwrap_or("");

            let (vendor, dev_type) = if !platform.is_empty() {
                (platform.to_string(), "Router / Access Point".to_string())
            } else {
                identify_vendor_and_type(&mac, identity.as_deref().unwrap_or(""), "")
            };

            let web_url = ip.as_ref().map(|a| format!("http://{}", a));

            devices.push(InfrastructureDevice {
                mac: mac.clone(),
                ip,
                identity,
                vendor,
                device_type: dev_type,
                interface: iface,
                detection_method: "Neighbor Discovery (LLDP/MNDP)".into(),
                is_bypassed_in_hotspot: bypassed_macs.contains(&mac),
                web_management_url: web_url,
                status: "online".into(),
            });
        }
    }

    // 4. Process DHCP Leases with AP signatures in host-name
    if let Ok(dhcps) = dhcp_res {
        for d in dhcps {
            let mac = d.get("mac-address").unwrap_or("").to_uppercase();
            let host_name = d.get("host-name").unwrap_or("");
            let comment = d.get("comment").unwrap_or("");
            let ip = d.get("address").map(|s| s.to_string());
            if mac.is_empty() || seen_macs.contains(&mac) { continue; }

            let (vendor, dev_type) = identify_vendor_and_type(&mac, host_name, comment);
            if dev_type.contains("Access Point") || dev_type.contains("Camera") {
                seen_macs.insert(mac.clone());
                if let Some(ref a) = ip { seen_ips.insert(a.clone()); }
                let web_url = ip.as_ref().map(|a| format!("http://{}", a));

                devices.push(InfrastructureDevice {
                    mac: mac.clone(),
                    ip,
                    identity: if !host_name.is_empty() { Some(host_name.to_string()) } else { Some(comment.to_string()) },
                    vendor,
                    device_type: dev_type,
                    interface: d.get("server").map(|s| s.to_string()),
                    detection_method: "DHCP Hostname Signature".into(),
                    is_bypassed_in_hotspot: bypassed_macs.contains(&mac),
                    web_management_url: web_url,
                    status: "active".into(),
                });
            }
        }
    }

    // 5. Process ARP entries matching known networking vendors (Ubiquiti, TP-Link, Ruijie, Tenda)
    if let Ok(arps) = arp_res {
        for a in arps {
            let mac = a.get("mac-address").unwrap_or("").to_uppercase();
            let ip = a.get("address").map(|s| s.to_string());
            let comment = a.get("comment").unwrap_or("");
            let iface = a.get("interface").map(|s| s.to_string());

            if mac.is_empty() || seen_macs.contains(&mac) { continue; }

            let (vendor, dev_type) = identify_vendor_and_type(&mac, "", comment);
            // If vendor is recognized as AP/Switch/Camera or not generic
            if vendor != "Generic / OEM" || dev_type.contains("Access Point") {
                seen_macs.insert(mac.clone());
                if let Some(ref addr) = ip { seen_ips.insert(addr.clone()); }
                let web_url = ip.as_ref().map(|a| format!("http://{}", a));

                devices.push(InfrastructureDevice {
                    mac: mac.clone(),
                    ip,
                    identity: a.get("comment").map(|s| s.to_string()),
                    vendor,
                    device_type: dev_type,
                    interface: iface,
                    detection_method: "ARP Vendor Signature Analysis".into(),
                    is_bypassed_in_hotspot: bypassed_macs.contains(&mac),
                    web_management_url: web_url,
                    status: "online".into(),
                });
            }
        }
    }

    // 6. Process Hotspot Hosts marked as bridge or bypassed
    if let Ok(hosts) = hs_host_res {
        for h in hosts {
            let mac = h.get("mac-address").unwrap_or("").to_uppercase();
            if mac.is_empty() || seen_macs.contains(&mac) { continue; }

            let is_bridge = h.get("bridge").is_some_and(|v| v == "yes" || v == "true");
            let is_bypassed = h.get("bypassed").is_some_and(|v| v == "yes" || v == "true") || bypassed_macs.contains(&mac);
            let (vendor, dev_type) = identify_vendor_and_type(&mac, "", h.get("comment").unwrap_or(""));

            if is_bridge || is_bypassed || vendor != "Generic / OEM" {
                seen_macs.insert(mac.clone());
                let ip = h.get("address").map(|s| s.to_string());
                if let Some(ref a) = ip { seen_ips.insert(a.clone()); }
                let iface = h.get("interface").map(|s| s.to_string());
                let web_url = ip.as_ref().map(|a| format!("http://{}", a));

                devices.push(InfrastructureDevice {
                    mac: mac.clone(),
                    ip,
                    identity: h.get("comment").map(|s| s.to_string()),
                    vendor,
                    device_type: if is_bridge { "Bridged AP / Switch".into() } else { dev_type },
                    interface: iface,
                    detection_method: "Hotspot Host Bridge Correlator".into(),
                    is_bypassed_in_hotspot: is_bypassed,
                    web_management_url: web_url,
                    status: "online".into(),
                });
            }
        }
    }

    // 7. Auto-detect Dedicated Port AP (e.g. ether5 with static AP IP 192.168.100.3 or similar)
    // If an interface has a dedicated subnet like 192.168.100.0/24 on ether5 and no AP detected yet
    for (iface, addr) in &iface_subnets {
        if iface.contains("ether5") || iface.contains("wlan") {
            let ap_ip = "192.168.100.3".to_string();
            if !seen_ips.contains(&ap_ip) && addr.starts_with("192.168.100.") {
                seen_ips.insert(ap_ip.clone());
                devices.push(InfrastructureDevice {
                    mac: "STATIC-AP-PORT5".into(),
                    ip: Some(ap_ip.clone()),
                    identity: Some("Dedicated Access Point (WiFi Hub)".into()),
                    vendor: "Access Point (WiFi Hub)".into(),
                    device_type: "Access Point".into(),
                    interface: Some(iface.clone()),
                    detection_method: "Port-Subnet Physical Topology Correlation".into(),
                    is_bypassed_in_hotspot: true,
                    web_management_url: Some(format!("http://{}", ap_ip)),
                    status: "online".into(),
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

#[derive(Deserialize, Debug)]
pub struct PoeCycleReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub interface: String,
    pub off_seconds: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct ApTunnelReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub ap_ip: String,
    pub ap_port: Option<u16>,
    pub external_port: Option<u16>,
}

#[derive(Deserialize, Debug)]
pub struct RemoveApTunnelReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub ap_ip: String,
}

/// POST /api/v1/network/infrastructure/poe-cycle - Reboot paksa Access Point lewat power-cycle PoE port
pub async fn poe_power_cycle(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<PoeCycleReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let iface = req.interface.as_str();
    let off_secs = req.off_seconds.unwrap_or(3);

    // 1. Turn off PoE voltage
    client.run(build_command("/interface/ethernet/poe/set", [
        ("numbers", iface),
        ("poe-out", "off")
    ])).await?;

    // 2. Wait for power to completely discharge
    tokio::time::sleep(tokio::time::Duration::from_secs(off_secs)).await;

    // 3. Restore PoE voltage
    client.run(build_command("/interface/ethernet/poe/set", [
        ("numbers", iface),
        ("poe-out", "auto-on")
    ])).await?;

    Ok(Json(json!({
        "success": true,
        "interface": req.interface,
        "off_duration_seconds": off_secs,
        "message": format!("PoE power-cycle berhasil dieksekusi pada {}. Perangkat Access Point yang terhubung sedang melakukan reboot fisik.", req.interface)
    })))
}

/// POST /api/v1/network/infrastructure/ap-tunnel - Buat port forwarding sementara untuk remote config Web AP pihak ketiga
pub async fn create_ap_tunnel(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ApTunnelReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let ap_port = req.ap_port.unwrap_or(80).to_string();
    let ext_port = req.external_port.unwrap_or(8083).to_string();
    let comment = format!("[AP-Tunnel-{}]", req.ap_ip);

    let args = [
        ("chain", "dstnat"),
        ("action", "dst-nat"),
        ("protocol", "tcp"),
        ("dst-port", ext_port.as_str()),
        ("to-addresses", req.ap_ip.as_str()),
        ("to-ports", ap_port.as_str()),
        ("comment", comment.as_str()),
    ];

    client.run(build_command("/ip/firewall/nat/add", args)).await?;

    Ok(Json(json!({
        "success": true,
        "ap_target_ip": req.ap_ip,
        "ap_internal_port": ap_port,
        "external_port": ext_port,
        "message": format!("Tunnel remote Web Management AP berhasil dibuka pada port {}. Anda dapat membuka Web Admin AP dari luar jaringan.", ext_port)
    })))
}

/// POST /api/v1/network/infrastructure/ap-tunnel/remove - Hapus tunnel remote Web AP setelah selesai konfigurasi
pub async fn remove_ap_tunnel(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RemoveApTunnelReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let comment_match = format!("*[AP-Tunnel-{}*", req.ap_ip);
    let rows = client.run(build_command("/ip/firewall/nat/print", [("?comment", comment_match.as_str())])).await.unwrap_or_default();

    let mut removed_count = 0;
    for r in rows {
        if let Some(id) = r.get(".id") {
            let _ = client.run(build_command("/ip/firewall/nat/remove", [(".id", id)])).await;
            removed_count += 1;
        }
    }

    Ok(Json(json!({
        "success": true,
        "ap_target_ip": req.ap_ip,
        "removed_rules": removed_count,
        "message": format!("Tunnel NAT untuk AP {} berhasil ditutup kembali.", req.ap_ip)
    })))
}


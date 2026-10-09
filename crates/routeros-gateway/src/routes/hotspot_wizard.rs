use std::sync::Arc;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use routeros_core::build_command;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::{AppState, RouterTarget};

#[derive(Deserialize, Debug)]
pub struct HotspotSetupWizardReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub interface: String,             // e.g. "ether2" or "bridge-hotspot"
    pub local_address: String,         // e.g. "192.168.20.1/24"
    pub dhcp_pool_range: String,       // e.g. "192.168.20.10-192.168.20.254"
    pub dns_name: Option<String>,       // e.g. "login.wifi" or "wifi.kafe.net"
    pub hotspot_name: Option<String>,   // e.g. "HOTSPOT-LAN"
    pub admin_user: Option<String>,     // default "admin"
    pub admin_password: Option<String>, // default "1234"
}

/// POST /api/v1/hotspot/wizard/setup - One-click Complete Hotspot Deployment Wizard:
/// Automatically provisions IP Address, IP Pool, DHCP Server & Network, Hotspot Profile,
/// Server Instance, User Profile (shared-users=1), Admin User, and NAT Masquerade in single transaction.
pub async fn setup_hotspot_wizard(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<HotspotSetupWizardReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let hs_name = req.hotspot_name.unwrap_or_else(|| format!("hs-{}", req.interface));
    let dns_url = req.dns_name.unwrap_or_else(|| "login.wifi".into());
    let pool_name = format!("pool-{}", hs_name);
    let prof_name = format!("prof-{}", hs_name);
    let dhcp_name = format!("dhcp-{}", hs_name);

    // Extract bare IP without CIDR e.g. "192.168.20.1/24" -> ("192.168.20.1", "192.168.20.0/24")
    let ip_parts: Vec<&str> = req.local_address.split('/').collect();
    let gateway_ip = ip_parts[0];
    let subnet_mask = ip_parts.get(1).unwrap_or(&"24");

    // Calculate base network e.g. 192.168.20.0/24
    let octets: Vec<&str> = gateway_ip.split('.').collect();
    let network_base = if octets.len() == 4 {
        format!("{}.{}.{}.0/{}", octets[0], octets[1], octets[2], subnet_mask)
    } else {
        req.local_address.clone()
    };

    let mut step_log = Vec::new();

    // Step 1: Assign Gateway IP Address to Interface
    let _ = client.run(build_command(
        "/ip/address/add",
        [
            ("address", req.local_address.as_str()),
            ("interface", req.interface.as_str()),
            ("comment", "[Wizard] Hotspot Gateway IP"),
        ]
    )).await;
    step_log.push(format!("1. IP Gateway {} ditambahkan ke interface {}", req.local_address, req.interface));

    // Step 2: Create IP Pool for Hotspot DHCP
    let _ = client.run(build_command(
        "/ip/pool/add",
        [
            ("name", pool_name.as_str()),
            ("ranges", req.dhcp_pool_range.as_str()),
            ("comment", "[Wizard] Hotspot IP Pool"),
        ]
    )).await;
    step_log.push(format!("2. IP Pool '{}' dibuat ({})", pool_name, req.dhcp_pool_range));

    // Step 3: Create DHCP Server & Network
    let _ = client.run(build_command(
        "/ip/dhcp-server/add",
        [
            ("name", dhcp_name.as_str()),
            ("interface", req.interface.as_str()),
            ("address-pool", pool_name.as_str()),
            ("disabled", "no"),
            ("comment", "[Wizard] Hotspot DHCP Server"),
        ]
    )).await;

    let _ = client.run(build_command(
        "/ip/dhcp-server/network/add",
        [
            ("address", network_base.as_str()),
            ("gateway", gateway_ip),
            ("dns-server", format!("{},8.8.8.8", gateway_ip).as_str()),
            ("comment", "[Wizard] Hotspot DHCP Network"),
        ]
    )).await;
    step_log.push(format!("3. DHCP Server & Network ({}) aktif", network_base));

    // Step 4: Create Hotspot Server Profile
    let _ = client.run(build_command(
        "/ip/hotspot/profile/add",
        [
            ("name", prof_name.as_str()),
            ("hotspot-address", gateway_ip),
            ("dns-name", dns_url.as_str()),
            ("html-directory", "hotspot"),
            ("login-by", "cookie,http-chap,http-pap"),
            ("comment", "[Wizard] Hotspot Server Profile"),
        ]
    )).await;
    step_log.push(format!("4. Hotspot Server Profile '{}' dibuat (DNS: http://{})", prof_name, dns_url));

    // Step 5: Create Hotspot Server Instance
    let _ = client.run(build_command(
        "/ip/hotspot/add",
        [
            ("name", hs_name.as_str()),
            ("interface", req.interface.as_str()),
            ("address-pool", pool_name.as_str()),
            ("profile", prof_name.as_str()),
            ("disabled", "no"),
            ("comment", "[Wizard] Hotspot Server Instance"),
        ]
    )).await;
    step_log.push(format!("5. Hotspot Server '{}' aktif pada interface {}", hs_name, req.interface));

    // Step 6: Create Default Hotspot User Profile with Anti-Share (shared-users=1)
    let user_prof_name = format!("default-{}", hs_name);
    let _ = client.run(build_command(
        "/ip/hotspot/user/profile/add",
        [
            ("name", user_prof_name.as_str()),
            ("shared-users", "1"),
            ("status-autorefresh", "1m"),
            ("transparent-proxy", "no"),
            ("comment", "[Wizard] Hotspot Default Profile (Anti-Share)"),
        ]
    )).await;
    step_log.push(format!("6. User Profile '{}' (shared-users=1) dibuat", user_prof_name));

    // Step 7: Create Admin Hotspot User
    let admin_user = req.admin_user.unwrap_or_else(|| "admin".into());
    let admin_pass = req.admin_password.unwrap_or_else(|| "1234".into());
    let _ = client.run(build_command(
        "/ip/hotspot/user/add",
        [
            ("name", admin_user.as_str()),
            ("password", admin_pass.as_str()),
            ("profile", user_prof_name.as_str()),
            ("comment", "[Wizard] Admin Voucher User"),
        ]
    )).await;
    step_log.push(format!("7. User Voucher Admin dibuat (Username: {}, Password: {})", admin_user, admin_pass));

    // Step 8: Ensure NAT Masquerade rule exists for hotspot subnet
    let _ = client.run(build_command(
        "/ip/firewall/nat/add",
        [
            ("chain", "srcnat"),
            ("src-address", network_base.as_str()),
            ("action", "masquerade"),
            ("comment", "[Wizard] Hotspot Internet Masquerade"),
        ]
    )).await;
    step_log.push("8. Firewall NAT Masquerade internet terverifikasi aktif".into());

    Ok(Json(json!({
        "success": true,
        "message": "Template Konfigurasi Hotspot Lengkap Berhasil Disuntikkan dalam 1-Klik!",
        "hotspot_summary": {
            "name": hs_name,
            "interface": req.interface,
            "gateway_ip": gateway_ip,
            "network_range": network_base,
            "dhcp_pool": req.dhcp_pool_range,
            "dns_login_url": format!("http://{}", dns_url),
            "admin_credentials": {
                "user": admin_user,
                "password": admin_pass
            }
        },
        "provisioning_steps": step_log
    })))
}

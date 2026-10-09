pub mod dhcp;
pub mod firewall;
pub mod hotspot;
pub mod interfaces;
pub mod ip;
pub mod ipv6;
pub mod ppp;
pub mod queues;
pub mod raw;
pub mod system;

use std::sync::Arc;
use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

pub fn build_api_router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        // --- Universal Raw Command & SSE Stream ---
        .route("/api/v1/command", post(raw::run_raw_command))
        .route("/api/v1/listen", get(raw::run_raw_listen))

        // --- System Management ---
        .route("/api/v1/system/resource", post(system::resource))
        .route("/api/v1/system/routerboard", post(system::routerboard))
        .route("/api/v1/system/identity", post(system::identity))
        .route("/api/v1/system/identity/set", post(system::set_identity))
        .route("/api/v1/system/check-update", post(system::check_update))
        .route("/api/v1/system/install-update", post(system::install_update))
        .route("/api/v1/system/reboot", post(system::reboot))

        // --- IPv4 & Routing ---
        .route("/api/v1/ip/addresses", post(ip::addresses))
        .route("/api/v1/ip/address/add", post(ip::add_address))
        .route("/api/v1/ip/address/remove", post(ip::remove_address))
        .route("/api/v1/ip/routes", post(ip::routes))
        .route("/api/v1/ip/dns", post(ip::dns))
        .route("/api/v1/ip/pools", post(ip::pools))

        // --- IPv6 ---
        .route("/api/v1/ipv6/addresses", post(ipv6::addresses))
        .route("/api/v1/ipv6/address/add", post(ipv6::add_address))
        .route("/api/v1/ipv6/address/remove", post(ipv6::remove_address))
        .route("/api/v1/ipv6/routes", post(ipv6::routes))
        .route("/api/v1/ipv6/pools", post(ipv6::pools))
        .route("/api/v1/ipv6/nd", post(ipv6::nd))

        // --- DHCP ---
        .route("/api/v1/dhcp/servers", post(dhcp::servers))
        .route("/api/v1/dhcp/leases", post(dhcp::leases))
        .route("/api/v1/dhcp/lease/make-static", post(dhcp::make_static))
        .route("/api/v1/dhcp/lease/remove", post(dhcp::remove_lease))

        // --- Hotspot & Vouchers ---
        .route("/api/v1/hotspot/users", post(hotspot::users))
        .route("/api/v1/hotspot/user/create", post(hotspot::create_user))
        .route("/api/v1/hotspot/user/remove", post(hotspot::remove_user))
        .route("/api/v1/hotspot/active", post(hotspot::active))
        .route("/api/v1/hotspot/kick", post(hotspot::kick))
        .route("/api/v1/hotspot/profiles", post(hotspot::profiles))
        .route("/api/v1/hotspot/ip-bindings", post(hotspot::ip_bindings))

        // --- PPP & PPPoE ---
        .route("/api/v1/ppp/secrets", post(ppp::secrets))
        .route("/api/v1/ppp/secret/create", post(ppp::create_secret))
        .route("/api/v1/ppp/secret/remove", post(ppp::remove_secret))
        .route("/api/v1/ppp/active", post(ppp::active))
        .route("/api/v1/ppp/disconnect", post(ppp::disconnect))
        .route("/api/v1/ppp/profiles", post(ppp::profiles))

        // --- Firewall & Security ---
        .route("/api/v1/firewall/filters", post(firewall::filters))
        .route("/api/v1/firewall/nat", post(firewall::nat))
        .route("/api/v1/firewall/address-lists", post(firewall::address_lists))
        .route("/api/v1/firewall/block-ip", post(firewall::block_ip))
        .route("/api/v1/firewall/unblock-ip", post(firewall::unblock_ip))

        // --- Queues (Bandwidth Limiting) ---
        .route("/api/v1/queues/simple", post(queues::simple))
        .route("/api/v1/queues/simple/add", post(queues::add_simple))
        .route("/api/v1/queues/simple/set-limit", post(queues::set_limit))
        .route("/api/v1/queues/simple/remove", post(queues::remove_simple))

        // --- Interfaces & Realtime Traffic ---
        .route("/api/v1/interfaces/all", post(interfaces::all))
        .route("/api/v1/interfaces/sample-traffic", post(interfaces::sample_traffic))
        .route("/api/v1/interfaces/stream", get(interfaces::stream_traffic))
        .with_state(state)
}

pub mod backup;
pub mod batch;
pub mod dhcp;
pub mod dns;
pub mod firewall;
pub mod hotspot;
pub mod interfaces;
pub mod ip;
pub mod ipv6;
pub mod logs;
pub mod neighbors;
pub mod ppp;
pub mod queues;
pub mod raw;
pub mod scripts;
pub mod system;
pub mod tools;
pub mod users;
pub mod wireguard;
pub mod wireless;

use std::sync::Arc;
use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

pub fn build_api_router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        // --- Universal Raw Command, Batch & SSE Stream ---
        .route("/api/v1/command", post(raw::run_raw_command))
        .route("/api/v1/batch", post(batch::execute_batch))
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

        // --- Static DNS & Cache ---
        .route("/api/v1/dns/static", post(dns::static_records))
        .route("/api/v1/dns/static/add", post(dns::add_static))
        .route("/api/v1/dns/static/remove", post(dns::remove_static))
        .route("/api/v1/dns/cache/flush", post(dns::flush_cache))

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

        // --- WireGuard (RouterOS v7) ---
        .route("/api/v1/wireguard/interfaces", post(wireguard::interfaces))
        .route("/api/v1/wireguard/peers", post(wireguard::peers))
        .route("/api/v1/wireguard/peer/add", post(wireguard::add_peer))
        .route("/api/v1/wireguard/peer/remove", post(wireguard::remove_peer))

        // --- Wireless & WiFi ---
        .route("/api/v1/wireless/interfaces", post(wireless::interfaces))
        .route("/api/v1/wireless/registrations", post(wireless::registrations))
        .route("/api/v1/wireless/security-profiles", post(wireless::security_profiles))
        .route("/api/v1/wireless/access-list", post(wireless::access_list))

        // --- Firewall & Keamanan ---
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

        // --- Network Tools & Diagnostics ---
        .route("/api/v1/tools/ping", post(tools::ping))
        .route("/api/v1/tools/traceroute", post(tools::traceroute))
        .route("/api/v1/tools/profile", post(tools::profile))
        .route("/api/v1/tools/netwatch", post(tools::netwatch))
        .route("/api/v1/tools/bandwidth-test", post(tools::bandwidth_test))

        // --- Neighbors Discovery ---
        .route("/api/v1/neighbors/all", post(neighbors::all))

        // --- Scripts & Schedulers (Cron) ---
        .route("/api/v1/scripts/all", post(scripts::scripts))
        .route("/api/v1/scripts/run", post(scripts::run_script))
        .route("/api/v1/scripts/add", post(scripts::add_script))
        .route("/api/v1/schedulers/all", post(scripts::schedulers))
        .route("/api/v1/schedulers/add", post(scripts::add_scheduler))

        // --- Backup, Export & Files ---
        .route("/api/v1/backup/create", post(backup::create_backup))
        .route("/api/v1/backup/export", post(backup::export_config))
        .route("/api/v1/files/all", post(backup::files))
        .route("/api/v1/files/remove", post(backup::remove_file))

        // --- Router Administrator Users ---
        .route("/api/v1/users/all", post(users::users))
        .route("/api/v1/users/create", post(users::create_user))
        .route("/api/v1/users/remove", post(users::remove_user))
        .route("/api/v1/users/groups", post(users::groups))

        // --- Logs ---
        .route("/api/v1/logs/all", post(logs::all))

        .with_state(state)
}

pub mod anti_tethering;
pub mod app_blocker;
pub mod audit;
pub mod backup;
pub mod batch;
pub mod bridge;
pub mod capsman;
pub mod dhcp;
pub mod dns;
pub mod doctor;
pub mod docs;
pub mod dude;
pub mod expert;
pub mod firewall;
pub mod hotspot;
pub mod hotspot_wizard;
pub mod infrastructure;
pub mod integrations;
pub mod interfaces;
pub mod ip;
pub mod ipv6;
pub mod load_balance;
pub mod logs;
pub mod maintenance;
pub mod metrics;
pub mod nat_wizard;
pub mod neighbors;
pub mod map;
pub mod network_map;
pub mod noc_tools;
pub mod overview;
pub mod ppp;
pub mod queues;
pub mod raw;
pub mod resource_detective;
pub mod routing;
pub mod scripts;
pub mod security_advisor;
pub mod system;
pub mod telegram_alerts;
pub mod tools;
pub mod topology;
pub mod traffic_shaper;
pub mod user_manager;
pub mod users;
pub mod visualizer;
pub mod voucher_template;
pub mod vouchers;
pub mod vpn;
pub mod wireguard;
pub mod wireless;
pub mod ws;
pub mod cloud;
pub mod failover_wizard;
pub mod graphing;
pub mod hardening;
pub mod isolir;
pub mod packet_sniffer;
pub mod port_knocking;
pub mod sanitize;
pub mod security_cve;
pub mod allowlist;
pub mod layer7;
pub mod provisioning;
pub mod quick_actions;

use std::sync::Arc;
use axum::routing::{get, post, put};
use axum::Router;

use crate::state::AppState;

pub fn build_api_router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        // --- Fast-Path Aggregated Snapshot & Cross-Layer Correlator ---
        .route("/api/v1/overview", get(overview::get_overview).post(overview::get_overview))
        .route("/api/v1/network/connected-devices", get(network_map::connected_devices).post(network_map::connected_devices))
        .route("/api/v1/devices", get(network_map::connected_devices).post(network_map::connected_devices))
        .route("/api/v1/connected-devices", get(network_map::connected_devices).post(network_map::connected_devices))
        .route("/api/v1/network/topology-graph", get(topology::topology_graph).post(topology::topology_graph))

        // --- Network Map: ready-to-embed floor-plan / satellite map with live traffic, queues & alerts ---
        .route("/api/v1/map/scene", get(map::scene).post(map::scene))
        .route("/api/v1/map/icons", get(map::icons))
        .route("/api/v1/map/layouts", get(map::list_layouts))
        .route("/api/v1/map/layouts/:id", get(map::get_layout).put(map::put_layout).delete(map::delete_layout))
        .route("/api/v1/map/layouts/:id/nodes/:node_id", put(map::patch_node).delete(map::delete_node))
        .route("/api/v1/map/layouts/:id/paths/:edge_id", put(map::set_path).delete(map::delete_path))
        .route("/api/v1/map/layouts/:id/auto-place", post(map::auto_place))
        .route(
            "/api/v1/map/layouts/:id/background",
            put(map::put_background).layer(map::body_limit_layer()).get(map::get_background).delete(map::delete_background),
        )

        // --- Expert Network Diagnostics & Macros ---
        .route("/api/v1/expert/quick-diagnose", post(expert::quick_diagnose))
        .route("/api/v1/expert/traffic-matrix", post(expert::traffic_matrix))
        .route("/api/v1/expert/security-audit", post(expert::security_audit))

        // --- Universal Raw Command, Batch & SSE Stream ---
        .route("/api/v1/command", post(raw::run_raw_command))
        .route("/api/v1/batch", post(batch::execute_batch))
        .route("/api/v1/listen", get(raw::run_raw_listen))

        // --- System Management & Hardening ---
        .route("/api/v1/system/resource", get(system::resource).post(system::resource))
        .route("/api/v1/system/resources", get(system::resource).post(system::resource))
        .route("/api/v1/system/routerboard", get(system::routerboard).post(system::routerboard))
        .route("/api/v1/system/identity", get(system::identity).post(system::identity))
        .route("/api/v1/system/identity/set", post(system::set_identity))
        .route("/api/v1/system/check-update", get(system::check_update).post(system::check_update))
        .route("/api/v1/system/install-update", post(system::install_update))
        .route("/api/v1/system/package/download", post(system::download_update))
        .route("/api/v1/system/package/channel", post(system::set_channel))
        .route("/api/v1/system/packages", get(system::packages).post(system::packages))
        .route("/api/v1/system/package/downgrade", post(system::package_downgrade))
        .route("/api/v1/system/package/cancel", post(system::package_cancel))
        .route("/api/v1/system/clock", get(system::clock).post(system::clock))
        .route("/api/v1/system/services", get(system::services).post(system::services))
        .route("/api/v1/system/service/toggle", post(system::toggle_service))
        .route("/api/v1/system/reboot", post(system::reboot))
        .route("/api/v1/system/shutdown", post(system::shutdown))
        .route("/api/v1/system/health", get(system::system_health).post(system::system_health))
        .route("/api/v1/system/ntp/setup", post(system::setup_ntp))
        .route("/api/v1/system/ntp", get(system::ntp_status).post(system::ntp_status))

        // --- IPv4, ARP & Routing ---
        .route("/api/v1/ip/addresses", get(ip::addresses).post(ip::addresses))
        .route("/api/v1/ip/address/add", post(ip::add_address))
        .route("/api/v1/ip/address/remove", post(ip::remove_address))
        .route("/api/v1/ip/routes", get(ip::routes).post(ip::routes))
        .route("/api/v1/ip/route/add", post(ip::add_route))
        .route("/api/v1/ip/route/remove", post(ip::remove_route))
        .route("/api/v1/ip/dns", get(ip::dns).post(ip::dns))
        .route("/api/v1/ip/pools", get(ip::pools).post(ip::pools))
        .route("/api/v1/ip/pools/used", get(ip::pools_used).post(ip::pools_used))
        .route("/api/v1/ip/pools/utilization", get(ip::pools_utilization).post(ip::pools_utilization))
        .route("/api/v1/ip/pool/add", post(ip::add_pool))
        .route("/api/v1/ip/pool/remove", post(ip::remove_pool))
        .route("/api/v1/ip/arp", get(ip::arp).post(ip::arp))
        .route("/api/v1/ip/arp/list", get(ip::arp).post(ip::arp))
        .route("/api/v1/ip/arp/add", post(ip::add_arp))
        .route("/api/v1/ip/arp/remove", post(ip::remove_arp))

        // --- Static DNS, AdList & Cache ---
        .route("/api/v1/dns/static", get(dns::static_records).post(dns::static_records))
        .route("/api/v1/dns/static/add", post(dns::add_static))
        .route("/api/v1/dns/static/remove", post(dns::remove_static))
        .route("/api/v1/dns/cache/flush", post(dns::flush_cache))
        .route("/api/v1/dns/adlist", get(dns::adlist).post(dns::adlist))
        .route("/api/v1/dns/adlist/add", post(dns::add_adlist))
        .route("/api/v1/dns/adlist/remove", post(dns::remove_adlist))
        .route("/api/v1/dns/adlist/presets", get(dns::adlist_presets))
        .route("/api/v1/dns/adlist/deploy-preset", post(dns::deploy_adlist_preset))

        // --- IPv6 ---
        .route("/api/v1/ipv6/addresses", get(ipv6::addresses).post(ipv6::addresses))
        .route("/api/v1/ipv6/address/add", post(ipv6::add_address))
        .route("/api/v1/ipv6/address/remove", post(ipv6::remove_address))
        .route("/api/v1/ipv6/routes", get(ipv6::routes).post(ipv6::routes))
        .route("/api/v1/ipv6/route/add", post(ipv6::add_route))
        .route("/api/v1/ipv6/route/remove", post(ipv6::remove_route))
        .route("/api/v1/ipv6/pools", get(ipv6::pools).post(ipv6::pools))
        .route("/api/v1/ipv6/pool/add", post(ipv6::add_pool))
        .route("/api/v1/ipv6/pool/remove", post(ipv6::remove_pool))
        .route("/api/v1/ipv6/nd", get(ipv6::nd).post(ipv6::nd))
        .route("/api/v1/ipv6/dhcp-client", get(ipv6::dhcp6_clients).post(ipv6::dhcp6_clients))
        .route("/api/v1/ipv6/dhcp-client/add", post(ipv6::add_dhcp6_client))
        .route("/api/v1/ipv6/dhcp-client/remove", post(ipv6::remove_dhcp6_client))
        .route("/api/v1/ipv6/dhcp-server", get(ipv6::dhcp6_servers).post(ipv6::dhcp6_servers))
        .route("/api/v1/ipv6/dhcp-server/add", post(ipv6::add_dhcp6_server))
        .route("/api/v1/ipv6/dhcp-server/remove", post(ipv6::remove_dhcp6_server))
        .route("/api/v1/ipv6/explain", post(ipv6::explain))
        .route("/api/v1/ipv6/wizard/quick-setup", post(ipv6::quick_setup))

        // --- DHCP & Rogue Detection ---
        .route("/api/v1/dhcp/servers", get(dhcp::servers).post(dhcp::servers))
        .route("/api/v1/dhcp/server/add", post(dhcp::add_server))
        .route("/api/v1/dhcp/server/set", post(dhcp::set_server))
        .route("/api/v1/dhcp/server/remove", post(dhcp::remove_server))
        .route("/api/v1/dhcp/networks", get(dhcp::networks).post(dhcp::networks))
        .route("/api/v1/dhcp/network/add", post(dhcp::add_network))
        .route("/api/v1/dhcp/network/remove", post(dhcp::remove_network))
        .route("/api/v1/dhcp/leases", get(dhcp::leases).post(dhcp::leases))
        .route("/api/v1/dhcp/leases/list", get(dhcp::leases).post(dhcp::leases))
        .route("/api/v1/dhcp/lease/make-static", post(dhcp::make_static))
        .route("/api/v1/dhcp/lease/add", post(dhcp::add_lease))
        .route("/api/v1/dhcp/lease/set", post(dhcp::set_lease))
        .route("/api/v1/dhcp/lease/remove", post(dhcp::remove_lease))
        .route("/api/v1/dhcp/alerts", get(dhcp::alerts).post(dhcp::alerts))
        .route("/api/v1/dhcp/alert/add", post(dhcp::add_alert))
        .route("/api/v1/dhcp/alert/remove", post(dhcp::remove_alert))
        .route("/api/v1/dhcp/relays", get(dhcp::relays).post(dhcp::relays))
        .route("/api/v1/dhcp/relay/add", post(dhcp::add_relay))
        .route("/api/v1/dhcp/relay/set", post(dhcp::set_relay))
        .route("/api/v1/dhcp/relay/remove", post(dhcp::remove_relay))
        .route("/api/v1/dhcp/clients", get(dhcp::clients).post(dhcp::clients))
        .route("/api/v1/dhcp/client/add", post(dhcp::add_client))
        .route("/api/v1/dhcp/client/set", post(dhcp::set_client))
        .route("/api/v1/dhcp/client/remove", post(dhcp::remove_client))
        .route("/api/v1/dhcp/client/renew", post(dhcp::renew_client))
        .route("/api/v1/dhcp/client/release", post(dhcp::release_client))

        // --- Hotspot, Vouchers & Hosts ---
        .route("/api/v1/hotspot/users", get(hotspot::users).post(hotspot::users))
        .route("/api/v1/hotspot/users/list", get(hotspot::users).post(hotspot::users))
        .route("/api/v1/hotspot/user/create", post(hotspot::create_user))
        .route("/api/v1/hotspot/generate-batch", post(hotspot::generate_batch))
        .route("/api/v1/hotspot/user/remove", post(hotspot::remove_user))
        .route("/api/v1/hotspot/active", get(hotspot::active).post(hotspot::active))
        .route("/api/v1/hotspot/active/list", get(hotspot::active).post(hotspot::active))
        .route("/api/v1/hotspot/kick", post(hotspot::kick))
        .route("/api/v1/hotspot/hosts", get(hotspot::hosts).post(hotspot::hosts))
        .route("/api/v1/hotspot/host/remove", post(hotspot::remove_host))
        .route("/api/v1/hotspot/host/bind", post(hotspot::bind_host))
        .route("/api/v1/hotspot/profiles", get(hotspot::profiles).post(hotspot::profiles))
        .route("/api/v1/hotspot/profile/add", post(hotspot::add_profile))
        .route("/api/v1/hotspot/profile/remove", post(hotspot::remove_profile))
        .route("/api/v1/hotspot/profiles/create-package", post(hotspot::create_package_profile))
        .route("/api/v1/hotspot/profiles/packages", get(hotspot::list_package_profiles).post(hotspot::list_package_profiles))
        .route("/api/v1/hotspot/profile/inject-expiry-script", post(hotspot::inject_expiry_script))
        .route("/api/v1/hotspot/server-profiles", get(hotspot::server_profiles).post(hotspot::server_profiles))
        .route("/api/v1/hotspot/server-profile/set", post(hotspot::set_server_profile))
        .route("/api/v1/hotspot/servers", get(hotspot::servers).post(hotspot::servers))
        .route("/api/v1/hotspot/server/set", post(hotspot::set_server))
        .route("/api/v1/hotspot/login-page/set", post(hotspot::customize_login_page))
        .route("/api/v1/hotspot/ip-bindings", get(hotspot::ip_bindings).post(hotspot::ip_bindings))
        .route("/api/v1/hotspot/walled-garden", get(hotspot::walled_garden).post(hotspot::walled_garden))
        .route("/api/v1/hotspot/walled-garden/add", post(hotspot::add_walled_garden))
        .route("/api/v1/hotspot/walled-garden/remove", post(hotspot::remove_walled_garden))
        .route("/api/v1/hotspot/walled-garden/deploy-preset", post(hotspot::deploy_walled_garden_preset))
        .route("/api/v1/hotspot/cookies", get(hotspot::cookies).post(hotspot::cookies))
        .route("/api/v1/hotspot/cookie/remove", post(hotspot::remove_cookie))
        .route("/api/v1/hotspot/user/reset-counters", post(hotspot::reset_counters))
        .route("/api/v1/hotspot/members", get(hotspot::list_members).post(hotspot::list_members))
        .route("/api/v1/hotspot/members/register", post(hotspot::register_member))
        .route("/api/v1/hotspot/members/renew", post(hotspot::renew_member))
        .route("/api/v1/hotspot/members/suspend", post(hotspot::suspend_member))
        .route("/api/v1/hotspot/members/activate", post(hotspot::activate_member))
        .route("/api/v1/hotspot/members/update", post(hotspot::update_member))
        .route("/api/v1/hotspot/members/delete", post(hotspot::delete_member))
        .route("/api/v1/hotspot/verify-login", post(hotspot::verify_login))
        .route("/api/v1/hotspot/ip-binding", get(hotspot::ip_bindings).post(hotspot::ip_bindings))
        .route("/api/v1/hotspot/ip-binding/add", post(hotspot::add_ip_binding))
        .route("/api/v1/hotspot/ip-binding/remove", post(hotspot::remove_ip_binding))
        .route("/api/v1/hotspot/active/kick", post(hotspot::kick_active))

        // --- User Manager (v6 & v7) ---
        .route("/api/v1/user-manager/users", get(user_manager::users).post(user_manager::users))
        .route("/api/v1/user-manager/user/create", post(user_manager::create_user))
        .route("/api/v1/user-manager/user/remove", post(user_manager::remove_user))
        .route("/api/v1/user-manager/sessions", get(user_manager::sessions).post(user_manager::sessions))
        .route("/api/v1/user-manager/profiles", get(user_manager::profiles).post(user_manager::profiles))
        .route("/api/v1/user-manager/profile/add", post(user_manager::add_profile))
        .route("/api/v1/user-manager/limitations", get(user_manager::limitations).post(user_manager::limitations))
        .route("/api/v1/user-manager/limitation/add", post(user_manager::add_limitation))
        .route("/api/v1/user-manager/user/assign-profile", post(user_manager::assign_profile))
        .route("/api/v1/user-manager/routers", get(user_manager::routers).post(user_manager::routers))
        .route("/api/v1/user-manager/router/add", post(user_manager::add_router))
        .route("/api/v1/user-manager/status", get(user_manager::userman_status).post(user_manager::userman_status))
        // Aliases for /api/v1/userman/...
        .route("/api/v1/userman/users", get(user_manager::users).post(user_manager::users))
        .route("/api/v1/userman/user/add", post(user_manager::create_user))
        .route("/api/v1/userman/user/remove", post(user_manager::remove_user))
        .route("/api/v1/userman/sessions", get(user_manager::sessions).post(user_manager::sessions))
        .route("/api/v1/userman/profiles", get(user_manager::profiles).post(user_manager::profiles))
        .route("/api/v1/userman/profile/add", post(user_manager::add_profile))
        .route("/api/v1/userman/limitations", get(user_manager::limitations).post(user_manager::limitations))
        .route("/api/v1/userman/limitation/add", post(user_manager::add_limitation))
        .route("/api/v1/userman/user/assign-profile", post(user_manager::assign_profile))
        .route("/api/v1/userman/routers", get(user_manager::routers).post(user_manager::routers))
        .route("/api/v1/userman/router/add", post(user_manager::add_router))
        .route("/api/v1/userman/status", get(user_manager::userman_status).post(user_manager::userman_status))

        // --- Bridge & VLANs ---
        .route("/api/v1/bridge/all", get(bridge::bridges).post(bridge::bridges))
        .route("/api/v1/bridge/add", post(bridge::add_bridge))
        .route("/api/v1/bridge/remove", post(bridge::remove_bridge))
        .route("/api/v1/bridge/ports", get(bridge::ports).post(bridge::ports))
        .route("/api/v1/bridge/port/add", post(bridge::add_port))
        .route("/api/v1/bridge/port/remove", post(bridge::remove_port))
        .route("/api/v1/bridge/vlans", get(bridge::vlans).post(bridge::vlans))
        .route("/api/v1/bridge/vlan/add", post(bridge::add_vlan))

        // --- PPP & PPPoE ISP Management ---
        .route("/api/v1/ppp/secrets", get(ppp::secrets).post(ppp::secrets))
        .route("/api/v1/ppp/secrets/list", get(ppp::secrets).post(ppp::secrets))
        .route("/api/v1/ppp/secret/create", post(ppp::create_secret))
        .route("/api/v1/ppp/secret/set", post(ppp::set_secret))
        .route("/api/v1/ppp/secret/remove", post(ppp::remove_secret))
        .route("/api/v1/ppp/customer/isolate", post(ppp::isolate_customer))
        .route("/api/v1/ppp/customer/restore", post(ppp::restore_customer))
        .route("/api/v1/ppp/active", get(ppp::active).post(ppp::active))
        .route("/api/v1/ppp/disconnect", post(ppp::disconnect))
        .route("/api/v1/ppp/servers", get(ppp::pppoe_servers).post(ppp::pppoe_servers))
        .route("/api/v1/ppp/server/create", post(ppp::create_pppoe_server))
        .route("/api/v1/ppp/profiles", get(ppp::profiles).post(ppp::profiles))
        .route("/api/v1/ppp/profile/create", post(ppp::create_profile))

        // --- VPN Servers (SSTP, L2TP, OpenVPN, EoIP, GRE & IPIP) ---
        .route("/api/v1/vpn/sstp", get(vpn::sstp_server).post(vpn::sstp_server))
        .route("/api/v1/vpn/sstp/set", post(vpn::set_sstp_server))
        .route("/api/v1/vpn/l2tp", get(vpn::l2tp_server).post(vpn::l2tp_server))
        .route("/api/v1/vpn/l2tp/set", post(vpn::set_l2tp_server))
        .route("/api/v1/vpn/ovpn", get(vpn::ovpn_server).post(vpn::ovpn_server))
        .route("/api/v1/vpn/ovpn/set", post(vpn::set_ovpn_server))
        .route("/api/v1/vpn/eoip", get(vpn::eoip_tunnels).post(vpn::eoip_tunnels))
        .route("/api/v1/vpn/eoip/add", post(vpn::add_eoip))
        .route("/api/v1/vpn/eoip/remove", post(vpn::remove_eoip))
        .route("/api/v1/vpn/gre", get(vpn::gre_tunnels).post(vpn::gre_tunnels))
        .route("/api/v1/vpn/gre/add", post(vpn::add_gre))
        .route("/api/v1/vpn/gre/remove", post(vpn::remove_gre))
        .route("/api/v1/vpn/ipip", get(vpn::ipip_tunnels).post(vpn::ipip_tunnels))
        .route("/api/v1/vpn/ipip/add", post(vpn::add_ipip))
        .route("/api/v1/vpn/ipip/remove", post(vpn::remove_ipip))

        // --- WireGuard (RouterOS v7) ---
        .route("/api/v1/wireguard/interfaces", get(wireguard::interfaces).post(wireguard::interfaces))
        .route("/api/v1/wireguard/peers", get(wireguard::peers).post(wireguard::peers))
        .route("/api/v1/wireguard/peer/add", post(wireguard::add_peer))
        .route("/api/v1/wireguard/peer/remove", post(wireguard::remove_peer))

        // --- Wireless & WiFi ---
        .route("/api/v1/wireless/interfaces", get(wireless::interfaces).post(wireless::interfaces))
        .route("/api/v1/wireless/registrations", get(wireless::registrations).post(wireless::registrations))
        .route("/api/v1/wireless/security-profiles", get(wireless::security_profiles).post(wireless::security_profiles))
        .route("/api/v1/wireless/access-list", get(wireless::access_list).post(wireless::access_list))

        // --- CAPsMAN Centralized Wireless AP Management ---
        .route("/api/v1/capsman/radios", get(capsman::radios).post(capsman::radios))
        .route("/api/v1/capsman/interfaces", get(capsman::interfaces).post(capsman::interfaces))
        .route("/api/v1/capsman/registrations", get(capsman::registrations).post(capsman::registrations))
        .route("/api/v1/capsman/set-wifi", post(capsman::set_wifi))
        .route("/api/v1/capsman/provision", post(capsman::provision))

        // --- Firewall & Keamanan ---
        .route("/api/v1/firewall/filters", get(firewall::filters).post(firewall::filters))
        .route("/api/v1/firewall/nat", get(firewall::nat).post(firewall::nat))
        .route("/api/v1/firewall/address-lists", get(firewall::address_lists).post(firewall::address_lists))
        .route("/api/v1/firewall/block-ip", post(firewall::block_ip))
        .route("/api/v1/firewall/unblock-ip", post(firewall::unblock_ip))
        .route("/api/v1/firewall/connections", get(firewall::connections).post(firewall::connections))
        .route("/api/v1/firewall/connections/top-talkers", get(firewall::top_talkers).post(firewall::top_talkers))
        .route("/api/v1/firewall/connections/flush", post(firewall::flush_connections))
        .route("/api/v1/firewall/connections/kill", post(firewall::kill_connection))
        .route("/api/v1/firewall/conntrack/settings", get(firewall::conntrack_settings).post(firewall::conntrack_settings))

        // --- Queues (Bandwidth Limiting & Live Inspection) ---
        .route("/api/v1/queues/simple", get(queues::simple).post(queues::simple))
        .route("/api/v1/queues/simple/add", post(queues::add_simple))
        .route("/api/v1/queues/simple/set-limit", post(queues::set_limit))
        .route("/api/v1/queues/simple/remove", post(queues::remove_simple))
        .route("/api/v1/queues/inspect-user", post(queues::inspect_user))
        .route("/api/v1/queues/overview-summary", get(queues::overview_summary).post(queues::overview_summary))
        .route("/api/v1/queues/test-limit", post(queues::test_limit))
        .route("/api/v1/queues/burst-calculator", post(queues::burst_calculator))

        // --- Interfaces & Realtime Traffic ---
        .route("/api/v1/interfaces/all", get(interfaces::all).post(interfaces::all))
        .route("/api/v1/interfaces", get(interfaces::all).post(interfaces::all))
        .route("/api/v1/interface/list", get(interfaces::all).post(interfaces::all))
        .route("/api/v1/interfaces/sample-traffic", post(interfaces::sample_traffic))
        .route("/api/v1/interfaces/stream", get(interfaces::stream_traffic))
        .route("/api/v1/interfaces/vlans", get(interfaces::vlans).post(interfaces::vlans))
        .route("/api/v1/interfaces/vlan/add", post(interfaces::add_vlan))
        .route("/api/v1/interfaces/vlan/remove", post(interfaces::remove_vlan))
        .route("/api/v1/network/sync-pppoe-hotspot", post(interfaces::sync_pppoe_hotspot))

        // --- Network Tools & Diagnostics ---
        .route("/api/v1/tools/ping", post(tools::ping))
        .route("/api/v1/tools/multi-ping", get(tools::multi_ping).post(tools::multi_ping))
        .route("/api/v1/tools/traceroute", post(tools::traceroute))
        .route("/api/v1/tools/profile", post(tools::profile))
        .route("/api/v1/tools/netwatch", post(tools::netwatch))
        .route("/api/v1/tools/bandwidth-test", post(tools::bandwidth_test))

        // --- Neighbors Discovery ---
        .route("/api/v1/neighbors/all", get(neighbors::all).post(neighbors::all))

        // --- Scripts & Schedulers (Cron) ---
        .route("/api/v1/scripts/all", get(scripts::scripts).post(scripts::scripts))
        .route("/api/v1/scripts/run", post(scripts::run_script))
        .route("/api/v1/scripts/add", post(scripts::add_script))
        .route("/api/v1/schedulers/all", get(scripts::schedulers).post(scripts::schedulers))
        .route("/api/v1/schedulers/add", post(scripts::add_scheduler))

        // --- Backup, Export & Files ---
        .route("/api/v1/backup/create", post(backup::create_backup))
        .route("/api/v1/backup/export", post(backup::export_config))
        .route("/api/v1/backup/restore", post(backup::restore_backup))
        .route("/api/v1/backup/import", post(backup::import_config))
        .route("/api/v1/files/all", get(backup::files).post(backup::files))
        .route("/api/v1/files/read", post(backup::read_file_content))
        .route("/api/v1/files/write", post(backup::write_file_content))
        .route("/api/v1/files/upload", post(backup::write_file_content))
        .route("/api/v1/files/remove", post(backup::remove_file))

        // --- Router Administrator Users ---
        .route("/api/v1/users/all", get(users::users).post(users::users))
        .route("/api/v1/users/create", post(users::create_user))
        .route("/api/v1/users/remove", post(users::remove_user))
        .route("/api/v1/users/groups", get(users::groups).post(users::groups))

        // --- Logs & Streaming ---
        .route("/api/v1/logs/all", get(logs::all).post(logs::all))
        .route("/api/v1/logs/stream", get(logs::stream_logs))

        // --- Security & CVE Vulnerability Advisor ---
        .route("/api/v1/security/vulnerability-audit", get(security_advisor::vulnerability_audit).post(security_advisor::vulnerability_audit))
        .route("/api/v1/security/deploy-antibruteforce", post(security_advisor::deploy_antibruteforce))

        // --- The Dude Network Monitor & Database Maintenance ---
        .route("/api/v1/dude/status", get(dude::status).post(dude::status))
        .route("/api/v1/dude/toggle", post(dude::toggle))
        .route("/api/v1/dude/export-db", post(dude::export_db))
        .route("/api/v1/dude/import-db", post(dude::import_db))
        .route("/api/v1/dude/vacuum", post(dude::vacuum))
        .route("/api/v1/dude/devices", get(dude::devices).post(dude::devices))

        // --- Disaster Recovery, Netinstall & Architecture Tools ---
        .route("/api/v1/maintenance/netinstall-prep", post(maintenance::netinstall_prep))
        .route("/api/v1/maintenance/pre-upgrade-snapshot", post(maintenance::pre_upgrade_snapshot))
        .route("/api/v1/maintenance/architecture-package-url", post(maintenance::architecture_package_url))
        .route("/api/v1/maintenance/reset-configuration", post(maintenance::reset_configuration))

        // --- Access Point & Infrastructure Device Detector ---
        .route("/api/v1/network/infrastructure/scan", get(infrastructure::scan_infrastructure).post(infrastructure::scan_infrastructure))
        .route("/api/v1/network/infrastructure/auto-bypass-ap", post(infrastructure::auto_bypass_ap))
        .route("/api/v1/network/infrastructure/poe-cycle", post(infrastructure::poe_power_cycle))
        .route("/api/v1/network/infrastructure/ap-tunnel", post(infrastructure::create_ap_tunnel))
        .route("/api/v1/network/infrastructure/ap-tunnel/remove", post(infrastructure::remove_ap_tunnel))

        // --- Telegram Bot & Netwatch Automation ---
        .route("/api/v1/telegram/send-message", post(telegram_alerts::send_message))
        .route("/api/v1/telegram/setup-netwatch", post(telegram_alerts::setup_netwatch))
        .route("/api/v1/telegram/list-monitors", get(telegram_alerts::list_monitors).post(telegram_alerts::list_monitors))
        .route("/api/v1/telegram/remove-monitor", post(telegram_alerts::remove_monitor))

        // --- Traffic Engineering, Mangle & Queue Tree ---
        .route("/api/v1/traffic/mangle/rules", get(traffic_shaper::mangle_rules).post(traffic_shaper::mangle_rules))
        .route("/api/v1/traffic/mangle/add", post(traffic_shaper::add_mangle))
        .route("/api/v1/traffic/queue-tree", get(traffic_shaper::queue_tree).post(traffic_shaper::queue_tree))
        .route("/api/v1/traffic/queue-tree/add", post(traffic_shaper::add_queue_tree))
        .route("/api/v1/traffic/preset/game-social-separation", post(traffic_shaper::deploy_game_social_separation))

        // --- NOC Pro Diagnostic Tools Suite ---
        .route("/api/v1/tools/torch", post(noc_tools::torch))
        .route("/api/v1/tools/ip-scan", post(noc_tools::ip_scan))
        .route("/api/v1/tools/romon/status", get(noc_tools::romon_status).post(noc_tools::romon_status))
        .route("/api/v1/tools/romon/set", post(noc_tools::romon_set))
        .route("/api/v1/tools/romon/discover", get(noc_tools::romon_discover).post(noc_tools::romon_discover))
        .route("/api/v1/system/watchdog", get(noc_tools::watchdog).post(noc_tools::watchdog))
        .route("/api/v1/system/ntp/config", get(noc_tools::ntp_config).post(noc_tools::ntp_config))
        .route("/api/v1/ip/dhcp-client/all", get(noc_tools::dhcp_clients).post(noc_tools::dhcp_clients))
        .route("/api/v1/ip/dhcp-client/add", post(noc_tools::add_dhcp_client))
        .route("/api/v1/ip/dhcp-client/remove", post(noc_tools::remove_dhcp_client))
        .route("/api/v1/scripts/terminal-exec", post(noc_tools::terminal_exec))

        // --- App & Content Blocker (WhatsApp, TikTok, YouTube, Judi, Torrent) ---
        .route("/api/v1/security/app-block", post(app_blocker::block_app))
        .route("/api/v1/security/app-unblock", post(app_blocker::unblock_app))
        .route("/api/v1/security/blocked-apps", get(app_blocker::list_blocked_apps).post(app_blocker::list_blocked_apps))

        // --- Anti-Tethering & Anti-WiFi Sharing (TTL=1) ---
        .route("/api/v1/security/anti-tethering/enable", post(anti_tethering::enable_anti_tethering))
        .route("/api/v1/security/anti-tethering/disable", post(anti_tethering::disable_anti_tethering))
        .route("/api/v1/security/anti-tethering/status", get(anti_tethering::status_anti_tethering).post(anti_tethering::status_anti_tethering))

        // --- NAT & Port Forwarding Wizard ---
        .route("/api/v1/firewall/port-forward", post(nat_wizard::add_port_forward))
        .route("/api/v1/firewall/port-forward/list", get(nat_wizard::list_port_forwards).post(nat_wizard::list_port_forwards))
        .route("/api/v1/firewall/port-forward/remove", post(nat_wizard::remove_port_forward))
        .route("/api/v1/firewall/hairpin-nat", post(nat_wizard::add_hairpin_nat))
        .route("/api/v1/firewall/srcnat/masquerade", post(nat_wizard::add_masquerade))

        // --- One-Click Complete Hotspot Setup Wizard ---
        .route("/api/v1/hotspot/wizard/setup", post(hotspot_wizard::setup_hotspot_wizard))

        // --- Resource Hog Detective & FastTrack CPU Optimizer ---
        .route("/api/v1/system/cpu-profiler", get(resource_detective::cpu_profiler).post(resource_detective::cpu_profiler))
        .route("/api/v1/system/fasttrack/deploy", get(resource_detective::deploy_fasttrack).post(resource_detective::deploy_fasttrack))
        .route("/api/v1/system/fasttrack/status", get(resource_detective::fasttrack_status).post(resource_detective::fasttrack_status))

        // --- Multi-WAN PCC Load Balancing Wizard & Status ---
        .route("/api/v1/load-balance/pcc/setup", post(load_balance::pcc_setup))
        .route("/api/v1/load-balance/status", get(load_balance::status).post(load_balance::status))
        .route("/api/v1/load-balance/remove", post(load_balance::remove))

        // --- BGP & OSPF Dynamic Routing Peering Telemetry ---
        .route("/api/v1/routing/bgp/sessions", get(routing::bgp_sessions).post(routing::bgp_sessions))
        .route("/api/v1/routing/bgp/peer/add", post(routing::add_bgp_peer))
        .route("/api/v1/routing/ospf/neighbors", get(routing::ospf_neighbors).post(routing::ospf_neighbors))
        .route("/api/v1/routing/routes", get(routing::routing_table).post(routing::routing_table))
        .route("/api/v1/routing/tables", get(routing::routing_tables).post(routing::routing_tables))
        .route("/api/v1/routing/table/add", post(routing::add_routing_table))
        .route("/api/v1/routing/rules", get(routing::routing_rules).post(routing::routing_rules))
        .route("/api/v1/routing/rule/add", post(routing::add_routing_rule))

        // --- Heuristic Network Doctor Diagnostic Assistant ---
        .route("/api/v1/doctor/diagnose", get(doctor::doctor_diagnose).post(doctor::doctor_diagnose))

        // --- Multi-Platform Messaging, Telegram, WhatsApp (Gowa) & Netwatch Pro ---
        .route("/api/v1/integrations/webhook/dispatch", post(integrations::webhook_dispatch))
        .route("/api/v1/integrations/telegram/send", post(integrations::telegram_send))
        .route("/api/v1/integrations/whatsapp/send", post(integrations::whatsapp_send))
        .route("/api/v1/integrations/notify", post(integrations::multi_notify))
        .route("/api/v1/integrations/netwatch/setup", post(integrations::netwatch_setup))
        .route("/api/v1/integrations/netwatch/batch-setup", post(integrations::netwatch_batch_setup))
        .route("/api/v1/integrations/netwatch/monitors", get(integrations::netwatch_list).post(integrations::netwatch_list))
        .route("/api/v1/integrations/netwatch/toggle", post(integrations::netwatch_toggle))
        .route("/api/v1/integrations/netwatch/remove", post(integrations::netwatch_remove))
        .route("/api/v1/integrations/reports/setup-scheduler", post(integrations::setup_report_scheduler))
        .route("/api/v1/integrations/hotspot-chat/snippet", get(integrations::hotspot_chat_snippet))

        // --- Enterprise Hotspot Voucher Engine 2.0 ---
        .route("/api/v1/hotspot/vouchers/generate", post(vouchers::generate_vouchers))
        .route("/api/v1/hotspot/vouchers/create-single", post(vouchers::create_single_voucher))
        .route("/api/v1/hotspot/vouchers/create", post(vouchers::create_single_voucher))
        .route("/api/v1/hotspot/vouchers/history", get(vouchers::voucher_usage_history).post(vouchers::voucher_usage_history))
        .route("/api/v1/hotspot/vouchers/tracking", get(vouchers::voucher_tracking).post(vouchers::voucher_tracking))
        .route("/api/v1/hotspot/vouchers/thermal-print", post(vouchers::thermal_print))
        .route("/api/v1/hotspot/vouchers/sell", post(vouchers::sell_voucher))
        .route("/api/v1/hotspot/vouchers/sell-and-send", post(vouchers::sell_and_send))
        .route("/api/v1/hotspot/vouchers/sales-report", get(vouchers::sales_report).post(vouchers::sales_report))
        .route("/api/v1/hotspot/vouchers/clean-expired", post(vouchers::clean_expired))
        .route("/api/v1/hotspot/voucher-template/render", post(voucher_template::render_template))

        // --- Multi-Tenant Segregated Audit Logging & Performance Analytics ---
        .route("/api/v1/audit/logs", get(audit::get_audit_logs).post(audit::get_audit_logs))
        .route("/api/v1/audit/stats", get(audit::get_performance_stats).post(audit::get_performance_stats))
        .route("/api/v1/audit/performance-stats", get(audit::get_performance_stats).post(audit::get_performance_stats))
        .route("/api/v1/audit/clear", post(audit::clear_audit_logs))

        // --- IP Cloud DDNS (Winbox & Dynamic IP Indihome/Biznet) ---
        .route("/api/v1/cloud/status", get(cloud::cloud_status).post(cloud::cloud_status))
        .route("/api/v1/cloud/set", post(cloud::cloud_set))
        .route("/api/v1/cloud/force-update", post(cloud::cloud_force_update))

        // --- Recursive Gateway Multi-WAN Failover Wizard (MTCRE & YouTube) ---
        .route("/api/v1/routing/failover/setup", post(failover_wizard::setup_recursive_failover))
        .route("/api/v1/routing/failover/status", get(failover_wizard::failover_status).post(failover_wizard::failover_status))
        .route("/api/v1/routing/failover/remove", post(failover_wizard::remove_recursive_failover))

        // --- WISP & RT-RW Net Isolir Web Redirect & Overdue Manager ---
        .route("/api/v1/wisp/isolir/setup", post(isolir::setup_isolir_infra))
        .route("/api/v1/wisp/isolir/isolate", post(isolir::isolate_user))
        .route("/api/v1/wisp/isolir/restore", post(isolir::restore_user))
        .route("/api/v1/wisp/isolir/list", get(isolir::list_isolated).post(isolir::list_isolated))
        .route("/api/v1/wisp/isolir/teardown", post(isolir::teardown_isolir))
        .route("/api/v1/wisp/isolir/pppoe/isolate", post(isolir::isolate_pppoe_user))
        .route("/api/v1/wisp/isolir/pppoe/restore", post(isolir::restore_pppoe_user))
        .route("/api/v1/wisp/isolir/pppoe/list", get(isolir::list_pppoe_isolated).post(isolir::list_pppoe_isolated))
        .route("/api/v1/wisp/isolir/allowlist/add", post(isolir::add_isolir_allowlist))

        // --- Packet Sniffer & Wireshark Streamer (Winbox NOC Tool) ---
        .route("/api/v1/tools/sniffer/status", get(packet_sniffer::sniffer_status).post(packet_sniffer::sniffer_status))
        .route("/api/v1/tools/sniffer/start", post(packet_sniffer::sniffer_start))
        .route("/api/v1/tools/sniffer/stop", post(packet_sniffer::sniffer_stop))
        .route("/api/v1/tools/sniffer/reset", post(packet_sniffer::sniffer_reset))
        .route("/api/v1/tools/sniffer/packets", get(packet_sniffer::sniffer_packets).post(packet_sniffer::sniffer_packets))
        .route("/api/v1/tools/sniffer/hosts", get(packet_sniffer::sniffer_hosts).post(packet_sniffer::sniffer_hosts))

        // --- RouterOS Built-in Web Graphing MRTG (/graphs) ---
        .route("/api/v1/tools/graphing/status", get(graphing::graphing_status).post(graphing::graphing_status))
        .route("/api/v1/tools/graphing/enable", post(graphing::graphing_enable))
        .route("/api/v1/tools/graphing/disable", post(graphing::graphing_disable))

        // --- 3-Step Stealth Port Knocking Wizard (MTCSE Security) ---
        .route("/api/v1/security/port-knocking/setup", post(port_knocking::setup_port_knocking))
        .route("/api/v1/security/port-knocking/status", get(port_knocking::port_knocking_status).post(port_knocking::port_knocking_status))
        .route("/api/v1/security/port-knocking/remove", post(port_knocking::remove_port_knocking))

        // --- Enterprise Hardening Suite, Bogon Filter & Auto-Reboot 04:00 (MTCSE & MTCTCE) ---
        .route("/api/v1/security/hardening/deploy-bogon-filter", post(hardening::deploy_bogon_filter))
        .route("/api/v1/security/hardening/deploy-dns-guard", post(hardening::deploy_dns_guard))
        .route("/api/v1/security/hardening/deploy-raw-antiddos", post(hardening::deploy_raw_antiddos))
        .route("/api/v1/security/hardening/setup-auto-reboot", post(hardening::setup_auto_reboot))
        .route("/api/v1/security/hardening/deploy-full-hardening", post(hardening::deploy_full_hardening))
        .route("/api/v1/security/hardening/audit-status", get(hardening::audit_status).post(hardening::audit_status))

        // --- Hotspot Profile & Voucher Policies (Enterprise Standard) ---
        .route("/api/v1/hotspot/profile/lock-mac", post(hotspot::enable_lock_mac))
        .route("/api/v1/hotspot/profile/bind-mac", post(hotspot::bind_user_mac_now))
        .route("/api/v1/hotspot/profile/expiry-mode", post(hotspot::setup_expiry_mode))
        .route("/api/v1/hotspot/trial/status", get(hotspot::trial_status).post(hotspot::trial_status))
        .route("/api/v1/hotspot/trial/setup", post(hotspot::setup_trial))
        .route("/api/v1/hotspot/trial/reset-cookies", post(hotspot::reset_trial_cookies))
        .route("/api/v1/hotspot/sweeper/deploy", post(vouchers::deploy_auto_cleaner))

        // --- RouterOS Vulnerability & CVE Security Database Scanner (/api/v1/vuln/*) ---
        .route("/api/v1/vuln/scan", get(security_cve::scan_vulnerabilities).post(security_cve::scan_vulnerabilities))
        .route("/api/v1/vuln/catalog", get(security_cve::list_cve_catalog))
        .route("/api/v1/vuln/quick-harden", post(security_cve::quick_harden_vulnerabilities))
        .route("/api/v1/security/vulnerabilities/scan", get(security_cve::scan_vulnerabilities).post(security_cve::scan_vulnerabilities))
        .route("/api/v1/security/vulnerabilities/quick-harden", post(security_cve::quick_harden_vulnerabilities))

        // --- Comprehensive Allowlist Suite (Tier 1 Default, Tier 2 Custom, Tier 3 Advanced & Audit) ---
        .route("/api/v1/allowlist/presets", get(allowlist::list_presets))
        .route("/api/v1/allowlist/presets/deploy", post(allowlist::deploy_preset))
        .route("/api/v1/allowlist/list", get(allowlist::list_all).post(allowlist::list_all))
        .route("/api/v1/allowlist/custom/add", post(allowlist::add_custom))
        .route("/api/v1/allowlist/custom/remove", post(allowlist::remove_custom))
        .route("/api/v1/allowlist/custom/toggle", post(allowlist::toggle_custom))
        .route("/api/v1/allowlist/advanced/add", post(allowlist::add_advanced))
        .route("/api/v1/allowlist/audit", get(allowlist::audit_allowlist).post(allowlist::audit_allowlist))

        // --- Firewall Live Connection Mitigation, Anomaly Detection & Quarantine ---
        .route("/api/v1/firewall/connections/mitigate", post(firewall::mitigate_connection))
        .route("/api/v1/firewall/connections/anomalies", get(firewall::connection_anomalies).post(firewall::connection_anomalies))
        .route("/api/v1/firewall/quarantine", get(firewall::list_quarantine).post(firewall::list_quarantine))
        .route("/api/v1/firewall/quarantine/release", post(firewall::release_quarantine))

        // --- Universal Quick Actions Engine (Firewall, Hotspot, PPPoE, Bandwidth & System) ---
        .route("/api/v1/quick-actions/presets", get(quick_actions::quick_action_presets))
        .route("/api/v1/quick-actions/execute", post(quick_actions::execute_quick_action))
        .route("/api/v1/quick-actions/pipeline", post(quick_actions::execute_pipeline))
        .route("/api/v1/quick-actions/history", get(quick_actions::quick_actions_history))
        .route("/api/v1/firewall/quick-actions/presets", get(quick_actions::quick_action_presets))
        .route("/api/v1/firewall/quick-actions/execute", post(quick_actions::execute_quick_action))
        .route("/api/v1/firewall/quick-actions/pipeline", post(quick_actions::execute_pipeline))
        .route("/api/v1/firewall/quick-actions/history", get(quick_actions::quick_actions_history))

        // --- Layer 7 Protocols & Service Ports (Winbox Tabs) ---
        .route("/api/v1/firewall/layer7", get(layer7::list_layer7).post(layer7::list_layer7))
        .route("/api/v1/firewall/layer7/add", post(layer7::add_layer7))
        .route("/api/v1/firewall/layer7/remove", post(layer7::remove_layer7))
        .route("/api/v1/firewall/layer7/presets", get(layer7::list_layer7_presets))
        .route("/api/v1/firewall/layer7/deploy-preset", post(layer7::deploy_layer7_preset))
        .route("/api/v1/firewall/service-ports", get(layer7::list_service_ports).post(layer7::list_service_ports))
        .route("/api/v1/firewall/service-port/toggle", post(layer7::toggle_service_port))

        // --- Universal Golden Setup Provisioning Engine (Zero-to-Hero with Realtime Steps & History) ---
        .route("/api/v1/provisioning/templates", get(provisioning::list_templates))
        .route("/api/v1/provisioning/blueprints", get(provisioning::list_blueprints))
        .route("/api/v1/provisioning/deploy", post(provisioning::deploy_provisioning))
        .route("/api/v1/provisioning/clean-slate", post(provisioning::clean_slate_endpoint))
        .route("/api/v1/provisioning/history", get(provisioning::provisioning_history))
        .route("/api/v1/provisioning/verify", get(provisioning::verify_provisioning_endpoint).post(provisioning::verify_provisioning_endpoint))

        .with_state(state)
}

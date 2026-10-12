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

#[derive(Deserialize, Debug, Default)]
pub struct FilterReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    #[serde(default)]
    pub filter: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct AddIpv6Req {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub address: String,
    pub interface: String,
    pub advertise: Option<bool>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

pub async fn addresses(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ipv6/address/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

pub async fn add_address(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddIpv6Req>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("address", req.address.as_str()), ("interface", req.interface.as_str())];
    let adv = if req.advertise.unwrap_or(false) { "yes" } else { "no" };
    args.push(("advertise", adv));
    if let Some(c) = req.comment.as_deref() {
        args.push(("comment", c));
    }
    client.run(build_command("/ipv6/address/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "IPv6 address added" })))
}

pub async fn remove_address(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ipv6/address/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "IPv6 address removed" })))
}

pub async fn routes(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ipv6/route/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

#[derive(Deserialize, Debug)]
pub struct AddIpv6RouteReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub dst_address: String,
    pub gateway: String,
    pub distance: Option<String>,
    pub comment: Option<String>,
}

/// POST /api/v1/ipv6/route/add - Tambah static IPv6 route
pub async fn add_route(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddIpv6RouteReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("dst-address", req.dst_address.as_str()),
        ("gateway", req.gateway.as_str()),
    ];
    if let Some(ref d) = req.distance {
        args.push(("distance", d.as_str()));
    }
    if let Some(ref c) = req.comment {
        args.push(("comment", c.as_str()));
    }
    client.run(build_command("/ipv6/route/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "IPv6 route added successfully" })))
}

/// POST /api/v1/ipv6/route/remove - Hapus IPv6 route
pub async fn remove_route(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ipv6/route/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "IPv6 route removed successfully" })))
}

pub async fn pools(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ipv6/pool/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

#[derive(Deserialize, Debug)]
pub struct AddIpv6PoolReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub prefix: String,
    pub prefix_length: Option<String>,
    pub comment: Option<String>,
}

/// POST /api/v1/ipv6/pool/add - Tambah IPv6 Pool
pub async fn add_pool(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddIpv6PoolReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("name", req.name.as_str()),
        ("prefix", req.prefix.as_str()),
    ];
    if let Some(ref pl) = req.prefix_length {
        args.push(("prefix-length", pl.as_str()));
    }
    if let Some(ref c) = req.comment {
        args.push(("comment", c.as_str()));
    }
    client.run(build_command("/ipv6/pool/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "IPv6 pool added successfully" })))
}

/// POST /api/v1/ipv6/pool/remove - Hapus IPv6 Pool
pub async fn remove_pool(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ipv6/pool/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "IPv6 pool removed successfully" })))
}

pub async fn nd(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ipv6/nd/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

#[derive(Deserialize, Debug)]
pub struct AddDhcp6ClientReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub interface: String,
    pub pool_name: Option<String>,
    pub pool_prefix_length: Option<String>,
    pub add_default_route: Option<bool>,
    pub request: Option<String>,
    pub comment: Option<String>,
}

/// GET or POST /api/v1/ipv6/dhcp-client - Daftar DHCPv6 Client (/ipv6/dhcp-client/print)
pub async fn dhcp6_clients(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ipv6/dhcp-client/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/ipv6/dhcp-client/add - Tambah DHCPv6 Client untuk Prefix Delegation ISP
pub async fn add_dhcp6_client(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddDhcp6ClientReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![("interface", req.interface.as_str())];
    if let Some(ref pn) = req.pool_name {
        args.push(("pool-name", pn.as_str()));
    }
    if let Some(ref ppl) = req.pool_prefix_length {
        args.push(("pool-prefix-length", ppl.as_str()));
    }
    let req_val = req.request.as_deref().unwrap_or("prefix");
    args.push(("request", req_val));
    let def_route = if req.add_default_route.unwrap_or(true) { "yes" } else { "no" };
    args.push(("add-default-route", def_route));
    if let Some(ref c) = req.comment {
        args.push(("comment", c.as_str()));
    }
    client.run(build_command("/ipv6/dhcp-client/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCPv6 client added successfully" })))
}

/// POST /api/v1/ipv6/dhcp-client/remove - Hapus DHCPv6 Client
pub async fn remove_dhcp6_client(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ipv6/dhcp-client/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "DHCPv6 client removed successfully" })))
}

#[derive(Deserialize, Debug)]
pub struct AddDhcp6ServerReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub name: String,
    pub interface: String,
    pub address_pool: String,
    pub lease_time: Option<String>,
    pub disabled: Option<bool>,
    pub comment: Option<String>,
}

/// GET or POST /api/v1/ipv6/dhcp-server - Daftar DHCPv6 Server (/ipv6/dhcp-server/print)
pub async fn dhcp6_servers(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let rows = client.run(build_command("/ipv6/dhcp-server/print", req.filter.iter().map(|(k, v)| (k.as_str(), v.as_str())))).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
    Ok(Json(json!({ "success": true, "count": data.len(), "data": data })))
}

/// POST /api/v1/ipv6/dhcp-server/add - Tambah DHCPv6 Server pada interface LAN
pub async fn add_dhcp6_server(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AddDhcp6ServerReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let mut args = vec![
        ("name", req.name.as_str()),
        ("interface", req.interface.as_str()),
        ("address-pool", req.address_pool.as_str()),
    ];
    if let Some(ref lt) = req.lease_time {
        args.push(("lease-time", lt.as_str()));
    }
    let dis_str;
    if let Some(d) = req.disabled {
        dis_str = if d { "yes" } else { "no" };
        args.push(("disabled", dis_str));
    }
    if let Some(ref c) = req.comment {
        args.push(("comment", c.as_str()));
    }
    client.run(build_command("/ipv6/dhcp-server/add", args)).await?;
    Ok(Json(json!({ "success": true, "message": "DHCPv6 server added successfully" })))
}

/// POST /api/v1/ipv6/dhcp-server/remove - Hapus DHCPv6 Server
pub async fn remove_dhcp6_server(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    client.run(build_command("/ipv6/dhcp-server/remove", [(".id", req.id.as_str())])).await?;
    Ok(Json(json!({ "success": true, "message": "DHCPv6 server removed successfully" })))
}

#[derive(Deserialize, Debug)]
pub struct Ipv6ExplainReq {
    pub address: String, // e.g. "2001:db8:85a3::1/64" or "fe80::1%ether1" or "2404:8000::/32"
}

/// POST /api/v1/ipv6/explain - Analisis, kalkulator & penerjemah format IPv6 ramah developer
pub async fn explain(
    Json(req): Json<Ipv6ExplainReq>,
) -> Result<Json<Value>, ApiError> {
    let raw = req.address.trim();
    let (ip_part, prefix_len) = if let Some((ip, pfx)) = raw.split_once('/') {
        (ip.trim(), pfx.trim().parse::<u8>().unwrap_or(64))
    } else {
        (raw, 64)
    };

    // Bersihkan interface zone identifier jika ada (misal %ether1)
    let clean_ip = ip_part.split('%').next().unwrap_or(ip_part).to_lowercase();

    // Deteksi tipe alamat
    let (addr_type, scope, is_routable, explanation) = if clean_ip == "::1" || clean_ip == "::" {
        ("Loopback / Unspecified", "Host Internal", false, "Alamat loopback lokal router (setara 127.0.0.1 di IPv4). Tidak boleh dikirim ke kabel jaringan.")
    } else if clean_ip.starts_with("fe8") || clean_ip.starts_with("fe9") || clean_ip.starts_with("fea") || clean_ip.starts_with("feb") {
        ("Link-Local Address (LLA)", "Segmen Kabel L2 Lokal", false, "Alamat otomatis yang dibuat router pada setiap interface. Hanya berfungsi untuk komunikasi internal satu switch (Layer 2) seperti Neighbor Discovery dan Next-Hop Gateway. Paket ber-IP fe80 tidak akan pernah di-forward keluar router.")
    } else if clean_ip.starts_with("fc") || clean_ip.starts_with("fd") {
        ("Unique Local Address (ULA)", "Private Intranet", false, "Alamat IPv6 Privat internal (setara 192.168.0.0/16 di IPv4). Aman untuk jaringan lokal tertutup, tetapi tidak bisa mengakses internet publik tanpa NAT66.")
    } else if clean_ip.starts_with("ff") {
        ("Multicast Address", "Grup Perangkat", false, "Alamat multicast untuk mengirim 1 paket ke banyak host sekaligus (menggantikan broadcast IPv4). Contoh ff02::1 untuk semua node, ff02::2 untuk semua router.")
    } else if clean_ip.starts_with("2") || clean_ip.starts_with("3") {
        ("Global Unicast Address (GUA)", "Internet Global Publik", true, "Alamat IPv6 Publik Asli yang dapat diakses langsung dari seluruh dunia tanpa perlu NAT. Setiap komputer/HP yang memiliki GUA memiliki koneksi end-to-end langsung.")
    } else {
        ("Special / Reserved IPv6", "Dokumentasi / Khusus", false, "Alamat yang dicadangkan oleh IETF untuk dokumentasi (misal 2001:db8::/32) atau transisi.")
    };

    // Analisis kapasitas subnet
    let (capacity_desc, subnets_64_possible) = match prefix_len {
        48 => ("Prefix Enterprise Besar. Dapat dipecah menjadi 65.536 subnet /64 untuk seluruh divisi kantor dan gedung.", 65536),
        56 => ("Prefix Standar ISP untuk Pelanggan Rumahan/UKM. Dapat dipecah menjadi 256 subnet /64 independen (misal untuk LAN Utama, VLAN Tamu, CCTV, dsb).", 256),
        60 => ("Prefix Menengah. Dapat dipecah menjadi 16 subnet /64 independen.", 16),
        64 => ("Ukuran Subnet Standar IPv6 untuk 1 Segmen LAN. Wajib /64 agar fitur SLAAC (Auto-Configuration HP/Laptop) berjalan lancar. Mendukung 18.4 triliun triliun host.", 1),
        126 | 127 => ("Subnet Point-to-Point Link. Digunakan khusus untuk sambungan kabel antar 2 router backbone.", 0),
        128 => ("Single Host IP (/128). Mengidentifikasi 1 perangkat tunggal secara spesifik.", 0),
        _ => ("Prefix Kustom. Rentang subnet fleksibel.", 0),
    };

    let cli_example = format!(
        "/ipv6/address/add address={}/{} interface=bridge advertise=yes comment=\"LAN Subnet\"",
        clean_ip, prefix_len
    );

    Ok(Json(json!({
        "success": true,
        "input": req.address,
        "clean_ip": clean_ip,
        "prefix_length": prefix_len,
        "address_type": addr_type,
        "scope": scope,
        "is_internet_routable": is_routable,
        "explanation": explanation,
        "capacity_description": capacity_desc,
        "possible_slash_64_subnets": subnets_64_possible,
        "developer_tips": [
            "IPv6 TIDAK menggunakan NAT (Network Address Translation). Setiap perangkat di LAN mendapatkan IP publik asli.",
            "Wajib menggunakan subnet /64 di interface LAN agar fitur SLAAC (Neighbor Discovery) bekerja otomatis di Android/iPhone/Windows.",
            "Pastikan Firewall Filter mengizinkan ICMPv6 (ping6 dan router advertisement), jangan pernah blokir ICMPv6 total karena akan merusak koneksi!",
            "Jangan ragu membuat banyak subnet: Alokasi /56 dari ISP menyediakan 256 subnet /64 siap pakai."
        ],
        "routeros_cli_example": cli_example
    })))
}

#[derive(Deserialize, Debug)]
pub struct QuickIpv6SetupReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub wan_interface: String,             // e.g. "ether1"
    pub lan_interface: String,             // e.g. "bridge"
    pub pool_name: Option<String>,         // default "isp-ipv6-pool"
    pub enable_firewall: Option<bool>,     // default true
    pub comment: Option<String>,
}

/// POST /api/v1/ipv6/wizard/quick-setup - 1-Klik Otomatisasi Lengkap Setup IPv6 ISP ke LAN (Zero Calculation)
pub async fn quick_setup(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<QuickIpv6SetupReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let pool_name = req.pool_name.unwrap_or_else(|| "isp-ipv6-pool".to_string());
    let enable_fw = req.enable_firewall.unwrap_or(true);
    let comment = req.comment.unwrap_or_else(|| "[Auto-IPv6] ISP to LAN Auto Setup".to_string());

    // 1. Tambah DHCPv6 Client di WAN Interface untuk meminta Prefix Delegation (PD)
    let _ = client.run(build_command(
        "/ipv6/dhcp-client/add",
        [
            ("interface", req.wan_interface.as_str()),
            ("request", "prefix"),
            ("pool-name", pool_name.as_str()),
            ("pool-prefix-length", "64"),
            ("add-default-route", "yes"),
            ("comment", comment.as_str()),
        ]
    )).await;

    // 2. Pasang IPv6 Address pada LAN Interface mengacu ke Pool tersebut
    let lan_comment = format!("{} - LAN Gateway", comment);
    let _ = client.run(build_command(
        "/ipv6/address/add",
        [
            ("interface", req.lan_interface.as_str()),
            ("from-pool", pool_name.as_str()),
            ("address", "::1/64"),
            ("advertise", "yes"),
            ("comment", lan_comment.as_str()),
        ]
    )).await;

    // 3. Konfigurasi Neighbor Discovery (ND) agar SLAAC membagikan IP & DNS otomatis
    let _ = client.run(build_command(
        "/ipv6/nd/set",
        [
            (".id", "*0"),
            ("interface", req.lan_interface.as_str()),
            ("advertise-mac-address", "yes"),
            ("advertise-dns", "yes"),
            ("hop-limit", "64"),
        ]
    )).await;

    // 4. Jika firewall diaktifkan: pasang aturan keamanan baseline IPv6 (karena IPv6 tidak ada NAT)
    if enable_fw {
        // Accept ICMPv6
        let _ = client.run(build_command(
            "/ipv6/firewall/filter/add",
            [
                ("chain", "input"),
                ("protocol", "icmpv6"),
                ("action", "accept"),
                ("comment", "[Auto-IPv6] Accept ICMPv6 (Critical for ND)"),
            ]
        )).await;

        // Accept Established & Related Input
        let _ = client.run(build_command(
            "/ipv6/firewall/filter/add",
            [
                ("chain", "input"),
                ("connection-state", "established,related"),
                ("action", "accept"),
                ("comment", "[Auto-IPv6] Accept Established/Related"),
            ]
        )).await;

        // Accept Established & Related Forward
        let _ = client.run(build_command(
            "/ipv6/firewall/filter/add",
            [
                ("chain", "forward"),
                ("connection-state", "established,related"),
                ("action", "accept"),
                ("comment", "[Auto-IPv6] Accept Forward Established/Related"),
            ]
        )).await;

        // Drop WAN Input (Cegah orang luar tembus ke router)
        let _ = client.run(build_command(
            "/ipv6/firewall/filter/add",
            [
                ("chain", "input"),
                ("in-interface", req.wan_interface.as_str()),
                ("action", "drop"),
                ("comment", "[Auto-IPv6] Protect Router from WAN"),
            ]
        )).await;
    }

    Ok(Json(json!({
        "success": true,
        "wan_interface": req.wan_interface,
        "lan_interface": req.lan_interface,
        "pool_name": pool_name,
        "firewall_hardened": enable_fw,
        "steps_completed": [
            format!("1. DHCPv6 Client dipasang pada interface WAN '{}' (meminta Prefix Delegation /64)", req.wan_interface),
            format!("2. IPv6 Address gateway dipasang pada interface LAN '{}' dari pool '{}'", req.lan_interface, pool_name),
            format!("3. Neighbor Discovery (ND) SLAAC diaktifkan pada interface LAN '{}' (semua PC & HP langsung dapat IPv6)", req.lan_interface),
            if enable_fw { "4. Baseline IPv6 Firewall Filter otomatis dipasang (ICMPv6 diizinkan, proteksi port router dari WAN aktif)" } else { "4. Firewall filter dilewati" }
        ],
        "verification_guide": "Tunggu 5-10 detik sampai ISP memberikan prefix. Coba jalankan 'ping -6 google.com' di PC/HP klien atau buka https://ipv6-test.com di browser."
    })))
}



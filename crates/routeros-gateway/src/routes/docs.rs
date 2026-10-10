use axum::response::Html;
use axum::Json;
use serde_json::{json, Value};

/// GET /docs & /documentation - Interactive, Rich API Documentation & Endpoint Explorer
pub async fn docs_page() -> Html<&'static str> {
    Html(r##"<!DOCTYPE html>
<html lang="id">
<head>
  <meta charset="UTF-8">
  <title>Dokumentasi Resmi API - MikroTik Universal Rust Gateway</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500;700&display=swap" rel="stylesheet">
  <style>
    :root {
      --bg: #0b0f19;
      --surface: #111827;
      --surface-elevated: #1f2937;
      --border: rgba(255, 255, 255, 0.08);
      --border-focus: #06b6d4;
      --text-main: #f9fafb;
      --text-muted: #9ca3af;
      --accent-cyan: #06b6d4;
      --accent-blue: #3b82f6;
      --accent-emerald: #10b981;
      --accent-rose: #f43f5e;
      --accent-purple: #8b5cf6;
      --accent-amber: #f59e0b;
      --code-bg: #030712;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: radial-gradient(circle at 50% 0%, #172554 0%, #0b0f19 75%);
      color: var(--text-main);
      font-family: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      min-height: 100vh;
      display: flex;
      flex-direction: column;
    }
    code, pre { font-family: 'JetBrains Mono', monospace; }

    /* Top Navigation Bar */
    header {
      position: sticky;
      top: 0;
      z-index: 50;
      background: rgba(11, 15, 25, 0.88);
      backdrop-filter: blur(16px);
      border-bottom: 1px solid var(--border);
      padding: 14px 28px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 16px;
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 12px;
    }
    .brand-title {
      font-weight: 800;
      font-size: 1.15rem;
      letter-spacing: -0.5px;
      background: linear-gradient(135deg, #38bdf8, #818cf8);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }
    .badge {
      background: rgba(6, 182, 212, 0.15);
      border: 1px solid rgba(6, 182, 212, 0.35);
      color: #38bdf8;
      font-size: 0.72rem;
      font-weight: 700;
      padding: 3px 8px;
      border-radius: 999px;
    }
    .nav-actions {
      display: flex;
      align-items: center;
      gap: 10px;
    }
    .btn-link {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 7px 14px;
      border-radius: 8px;
      font-size: 0.82rem;
      font-weight: 600;
      text-decoration: none;
      transition: all 0.2s ease;
      cursor: pointer;
    }
    .btn-playground {
      background: linear-gradient(135deg, #0284c7, #06b6d4);
      color: white;
      box-shadow: 0 4px 14px rgba(6, 182, 212, 0.25);
    }
    .btn-visualizer {
      background: rgba(31, 41, 55, 0.8);
      border: 1px solid var(--border);
      color: var(--text-main);
    }
    .btn-link:hover { transform: translateY(-1px); }

    /* Layout */
    .docs-container {
      display: flex;
      flex: 1;
      max-width: 1600px;
      width: 100%;
      margin: 0 auto;
    }

    /* Sidebar Navigation */
    .sidebar {
      width: 310px;
      position: sticky;
      top: 65px;
      height: calc(100vh - 65px);
      overflow-y: auto;
      border-right: 1px solid var(--border);
      padding: 24px 18px;
      background: rgba(17, 24, 39, 0.4);
      backdrop-filter: blur(10px);
    }
    .sidebar::-webkit-scrollbar { width: 5px; }
    .sidebar::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.1); border-radius: 4px; }
    
    .search-box {
      margin-bottom: 20px;
      position: relative;
    }
    .search-input {
      width: 100%;
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 10px 14px 10px 36px;
      color: white;
      font-size: 0.85rem;
      outline: none;
      transition: border-color 0.2s;
    }
    .search-input:focus { border-color: var(--border-focus); }
    .search-icon {
      position: absolute;
      left: 12px;
      top: 50%;
      transform: translateY(-50%);
      color: var(--text-muted);
      font-size: 0.85rem;
    }

    .cat-title {
      font-size: 0.72rem;
      font-weight: 800;
      text-transform: uppercase;
      letter-spacing: 0.8px;
      color: var(--text-muted);
      margin: 18px 0 8px 6px;
    }
    .nav-item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 8px 12px;
      border-radius: 8px;
      color: #cbd5e1;
      font-size: 0.82rem;
      font-weight: 500;
      text-decoration: none;
      transition: all 0.15s ease;
      cursor: pointer;
    }
    .nav-item:hover, .nav-item.active {
      background: rgba(56, 189, 248, 0.12);
      color: #38bdf8;
    }
    .nav-counter {
      font-size: 0.7rem;
      background: rgba(255, 255, 255, 0.08);
      padding: 2px 6px;
      border-radius: 999px;
      color: var(--text-muted);
    }

    /* Content Area */
    .content {
      flex: 1;
      padding: 36px 48px;
      overflow-y: auto;
    }

    /* Hero & Quick Spec */
    .hero {
      margin-bottom: 36px;
      background: linear-gradient(135deg, rgba(30, 58, 138, 0.25), rgba(17, 24, 39, 0.6));
      border: 1px solid rgba(56, 189, 248, 0.2);
      border-radius: 16px;
      padding: 28px 32px;
    }
    .hero h1 {
      font-size: 1.85rem;
      font-weight: 800;
      letter-spacing: -0.5px;
      margin-bottom: 8px;
    }
    .hero p {
      color: #cbd5e1;
      font-size: 0.95rem;
      line-height: 1.6;
      max-width: 900px;
    }
    .spec-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
      gap: 16px;
      margin-top: 24px;
    }
    .spec-card {
      background: rgba(17, 24, 39, 0.7);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 16px;
    }
    .spec-title {
      font-size: 0.75rem;
      font-weight: 700;
      color: var(--accent-cyan);
      text-transform: uppercase;
      letter-spacing: 0.5px;
      margin-bottom: 6px;
    }
    .spec-desc {
      font-size: 0.85rem;
      color: #94a3b8;
      line-height: 1.4;
    }

    /* Endpoint Cards */
    .endpoint-card {
      background: rgba(17, 24, 39, 0.75);
      border: 1px solid var(--border);
      border-radius: 14px;
      padding: 22px;
      margin-bottom: 24px;
      transition: border-color 0.2s, box-shadow 0.2s;
    }
    .endpoint-card:hover {
      border-color: rgba(56, 189, 248, 0.35);
      box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    }
    .ep-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      flex-wrap: wrap;
      gap: 12px;
      margin-bottom: 12px;
    }
    .ep-title-row {
      display: flex;
      align-items: center;
      gap: 10px;
    }
    .method-badge {
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.75rem;
      font-weight: 800;
      padding: 4px 10px;
      border-radius: 6px;
      letter-spacing: 0.5px;
    }
    .method-post { background: #1e3a8a; color: #93c5fd; border: 1px solid #3b82f6; }
    .method-get { background: #064e3b; color: #6ee7b7; border: 1px solid #10b981; }
    .method-sse { background: #701a75; color: #f5d0fe; border: 1px solid #c026d3; }
    .method-ws { background: #78350f; color: #fde68a; border: 1px solid #f59e0b; }
    
    .ep-path {
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.95rem;
      font-weight: 600;
      color: #f1f5f9;
    }
    .ep-desc {
      color: #94a3b8;
      font-size: 0.88rem;
      margin-bottom: 16px;
      line-height: 1.5;
    }

    .ep-details-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 16px;
      margin-top: 12px;
    }
    @media (max-width: 1024px) {
      .ep-details-grid { grid-template-columns: 1fr; }
      .sidebar { display: none; }
      .content { padding: 20px; }
    }
    
    .box-label {
      font-size: 0.72rem;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.6px;
      color: var(--text-muted);
      margin-bottom: 6px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .code-box {
      background: var(--code-bg);
      border: 1px solid rgba(255, 255, 255, 0.07);
      border-radius: 8px;
      padding: 12px;
      font-size: 0.8rem;
      color: #38bdf8;
      overflow-x: auto;
      max-height: 220px;
    }
    .copy-btn {
      background: rgba(255, 255, 255, 0.08);
      border: none;
      color: #cbd5e1;
      padding: 3px 8px;
      border-radius: 5px;
      font-size: 0.7rem;
      cursor: pointer;
      transition: background 0.2s;
    }
    .copy-btn:hover { background: rgba(56, 189, 248, 0.25); color: white; }

    .try-btn {
      background: rgba(14, 165, 233, 0.15);
      border: 1px solid rgba(14, 165, 233, 0.3);
      color: #38bdf8;
      padding: 4px 10px;
      border-radius: 6px;
      font-size: 0.75rem;
      font-weight: 600;
      text-decoration: none;
      display: inline-flex;
      align-items: center;
      gap: 4px;
      cursor: pointer;
      transition: all 0.2s;
    }
    .try-btn:hover {
      background: #0284c7;
      color: white;
    }
  </style>
</head>
<body>

  <!-- Top Navigation Bar -->
  <header>
    <div class="brand">
      <span style="font-size: 1.35rem;">🦀</span>
      <span class="brand-title">MikroTik Universal Gateway</span>
      <span class="badge">API v0.2.0 Reference</span>
    </div>
    <div class="nav-actions">
      <a href="/" class="btn-link btn-playground">
        <span>🎮</span> Live Playground
      </a>
      <a href="/topology" target="_blank" class="btn-link btn-visualizer">
        <span>🌐</span> Visualizer Relasi
      </a>
      <a href="/health" target="_blank" class="btn-link" style="color: var(--accent-emerald); font-size: 0.75rem;">
        ● Sistem Sehat (200 OK)
      </a>
    </div>
  </header>

  <div class="docs-container">
    <!-- Left Category Navigation -->
    <aside class="sidebar">
      <div class="search-box">
        <span class="search-icon">🔍</span>
        <input type="text" id="filter-input" class="search-input" placeholder="Cari endpoint atau fitur..." oninput="filterEndpoints()">
      </div>

      <div class="cat-title">Navigasi Kategori</div>
      <div id="category-nav">
        <!-- Dynamically generated or static list -->
        <a class="nav-item active" onclick="jumpCategory('all')"><span>Semua Endpoint</span> <span class="nav-counter" id="total-count">0</span></a>
        <a class="nav-item" onclick="jumpCategory('fast-path')"><span>🚀 Ringkasan &amp; Fast-Path</span></a>
        <a class="nav-item" onclick="jumpCategory('security')"><span>🛡️ Keamanan &amp; Audit</span></a>
        <a class="nav-item" onclick="jumpCategory('wizards')"><span>🧙‍♂️ Wizard &amp; Otomasi</span></a>
        <a class="nav-item" onclick="jumpCategory('load-balance')"><span>⚖️ Multi-WAN PCC</span></a>
        <a class="nav-item" onclick="jumpCategory('hotspot')"><span>🎟️ Hotspot &amp; Voucher</span></a>
        <a class="nav-item" onclick="jumpCategory('ppp')"><span>🏢 PPPoE &amp; ISP</span></a>
        <a class="nav-item" onclick="jumpCategory('wireguard')"><span>🌐 WireGuard &amp; VPN</span></a>
        <a class="nav-item" onclick="jumpCategory('firewall')"><span>🧱 Firewall &amp; NAT</span></a>
        <a class="nav-item" onclick="jumpCategory('queues')"><span>📊 Queues &amp; Bandwidth</span></a>
        <a class="nav-item" onclick="jumpCategory('interfaces')"><span>📈 Interfaces &amp; Traffic</span></a>
        <a class="nav-item" onclick="jumpCategory('tools')"><span>🔬 NOC Diagnostic Suite</span></a>
        <a class="nav-item" onclick="jumpCategory('telegram')"><span>✈️ Telegram Alerts</span></a>
        <a class="nav-item" onclick="jumpCategory('dude')"><span>📡 The Dude Monitor</span></a>
        <a class="nav-item" onclick="jumpCategory('system')"><span>⚙️ Sistem &amp; Hardware</span></a>
        <a class="nav-item" onclick="jumpCategory('backup')"><span>💾 Backup &amp; File</span></a>
        <a class="nav-item" onclick="jumpCategory('raw')"><span>⌨️ Raw Command &amp; Stream</span></a>
      </div>
    </aside>

    <!-- Main Content Area -->
    <main class="content">
      <!-- Architecture & Security Banner -->
      <section class="hero">
        <h1>Buku Petunjuk &amp; Dokumentasi API Gateway</h1>
        <p>
          MikroTik Universal Rust Gateway menyediakan lapisan antarmuka REST, Server-Sent Events (SSE), dan WebSocket berkinerja tinggi (sub-millisecond) untuk mengontrol router MikroTik secara multi-router, aman, dan tanpa ketergantungan library pihak ketiga.
        </p>

        <div class="spec-grid">
          <div class="spec-card">
            <div class="spec-title">🔐 Autentikasi Gateway</div>
            <div class="spec-desc">
              Semua endpoint <code>/api/v1/*</code> diamankan dengan Bearer Token:
              <br><code style="color: #38bdf8;">Authorization: Bearer &lt;GATEWAY_TOKEN&gt;</code>
            </div>
          </div>
          <div class="spec-card">
            <div class="spec-title">🎯 Multi-Router Dynamic Targeting</div>
            <div class="spec-desc">
              Koneksikan router manapun secara dinamis via Header:
              <br><code>X-Router-Host: &lt;IP/Domain&gt;</code>
              <br><code>X-Router-Port: &lt;Port-API&gt;</code>
              <br><code>X-Router-User: &lt;User&gt;</code>
              <br><code>X-Router-Pass: &lt;Password&gt;</code>
            </div>
          </div>
          <div class="spec-card">
            <div class="spec-title">⚡ Persistent Connection Pool</div>
            <div class="spec-desc">
              Engine Rust mempertahankan koneksi TCP soket biner RouterOS secara persisten di memori. Request berulang selesai dalam <strong>&lt; 5 milidetik</strong>.
            </div>
          </div>
        </div>
      </section>

      <!-- Endpoints List -->
      <section id="endpoints-container">
        <!-- Rendered by JS -->
      </section>
    </main>
  </div>

  <script>
    const ENDPOINTS = [
      // Fast-Path & Overview
      {
        cat: 'fast-path',
        method: 'POST',
        path: '/api/v1/overview',
        desc: 'Mengambil snapshot agregasi lengkap jaringan (System resource, routerboard, IP address, DHCP leases, Hotspot online, PPP online, dan interfaces) dalam satu request sub-milidetik.',
        req: '{}',
        res: '{\n  "success": true,\n  "system": { "cpu-load": "4", "free-memory": "118231040" },\n  "identity": "Core-Gateway",\n  "hotspot_active_count": 14,\n  "pppoe_active_count": 5,\n  "dhcp_leases_count": 28\n}'
      },
      {
        cat: 'fast-path',
        method: 'POST',
        path: '/api/v1/network/connected-devices',
        desc: 'Korelasikan perangkat di semua lapisan (DHCP, ARP, Hotspot, Wireless, dan PPPoE) menjadi satu daftar terpadu dengan status online, IP, MAC, dan hostname.',
        req: '{}',
        res: '{\n  "success": true,\n  "devices": [\n    {\n      "mac": "16:98:72:A8:21:4D",\n      "ip": "172.16.10.68",\n      "source": "hotspot",\n      "user": "euryv",\n      "online": true\n    }\n  ]\n}'
      },
      {
        cat: 'fast-path',
        method: 'POST',
        path: '/api/v1/network/topology-graph',
        desc: 'Membangun graf relasi simpul (Nodes) dan tautan (Edges) jaringan untuk divisualisasikan dalam bentuk topologi interaktif.',
        req: '{}',
        res: '{\n  "nodes": [\n    { "id": "router-core", "label": "MikroTik Core", "type": "router" },\n    { "id": "dev-1", "label": "euryv (172.16.10.68)", "type": "hotspot" }\n  ],\n  "edges": [\n    { "source": "router-core", "target": "dev-1", "label": "MANYTAL" }\n  ]\n}'
      },
      {
        cat: 'fast-path',
        method: 'POST',
        path: '/api/v1/expert/quick-diagnose',
        desc: 'Diagnosa kilat router: memeriksa utilisasi CPU, sisa memori, beban antarmuka trafik, dan peringatan potensi kelebihan beban.',
        req: '{}',
        res: '{\n  "success": true,\n  "status": "HEALTHY",\n  "cpu_load": 5,\n  "memory_free_pct": 74.2,\n  "warnings": []\n}'
      },

      // Security & Audit
      {
        cat: 'security',
        method: 'POST',
        path: '/api/v1/security/vulnerability-audit',
        desc: 'Audit keamanan sistem dan port: memeriksa port sensitif terbuka (Telnet, FTP, WWW) dan memeriksa kelemahan arsitektur router.',
        req: '{}',
        res: '{\n  "success": true,\n  "score": 85,\n  "risk_level": "LOW",\n  "open_vulnerable_services": ["telnet", "ftp"],\n  "recommendations": ["Matikan telnet dan ftp, gunakan SSH atau Winbox terenkripsi"]\n}'
      },
      {
        cat: 'security',
        method: 'POST',
        path: '/api/v1/security/deploy-antibruteforce',
        desc: 'Otomatis pasang aturan firewall Anti-Bruteforce berlapis (SSH, Winbox, FTP) dengan pemblokiran IP penyerang ke address-list blacklist.',
        req: '{\n  "service": "all",\n  "ban_time": "1d"\n}',
        res: '{\n  "success": true,\n  "message": "Aturan Anti-Bruteforce SSH, FTP, dan Winbox berhasil diterapkan."\n}'
      },
      {
        cat: 'security',
        method: 'POST',
        path: '/api/v1/security/app-block',
        desc: 'Blokir konten dan aplikasi spesifik (WhatsApp, TikTok, YouTube, Judi Online, Torrent) via TLS-Host dan Layer-7 regex filter.',
        req: '{\n  "app_type": "whatsapp",\n  "action": "drop"\n}',
        res: '{\n  "success": true,\n  "message": "Blokir WhatsApp berhasil diaktifkan pada firewall."\n}'
      },
      {
        cat: 'security',
        method: 'POST',
        path: '/api/v1/security/anti-tethering/enable',
        desc: 'Terapkan proteksi anti-tethering / anti-sharing hotspot dengan membatasi Mangle Change-TTL menjadi 1.',
        req: '{\n  "hotspot_interface": "bridge"\n}',
        res: '{\n  "success": true,\n  "message": "Proteksi Anti-Tethering (TTL=1) aktif pada interface bridge."\n}'
      },

      // Wizards & Automation
      {
        cat: 'wizards',
        method: 'POST',
        path: '/api/v1/hotspot/wizard/setup',
        desc: '1-Klik Wizard Setup Hotspot Lengkap: otomatis membuat Bridge, IP Address, DHCP Pool, DHCP Server, DNS, dan Hotspot Server.',
        req: '{\n  "interface": "ether2",\n  "local_address": "192.168.50.1/24",\n  "dhcp_pool_range": "192.168.50.10-192.168.50.254",\n  "dns_name": "login.wifi"\n}',
        res: '{\n  "success": true,\n  "message": "Hotspot Server login.wifi berhasil dibuat lengkap."\n}'
      },
      {
        cat: 'wizards',
        method: 'POST',
        path: '/api/v1/firewall/port-forward',
        desc: '1-Klik Dst-NAT Port Forwarding: membuka dan mengarahkan port publik ke port internal server/CCTV.',
        req: '{\n  "dst_port": "8080",\n  "to_addresses": "192.168.88.50",\n  "to_ports": "80",\n  "protocol": "tcp"\n}',
        res: '{\n  "success": true,\n  "message": "Port forward 8080 -> 192.168.88.50:80 berhasil dibuat."\n}'
      },
      {
        cat: 'wizards',
        method: 'POST',
        path: '/api/v1/traffic/preset/game-social-separation',
        desc: '1-Klik Pemisah Trafik Game vs Sosmed: otomatis menyuntikkan Mangle tanda koneksi port game prioritas tinggi dan Queue Tree.',
        req: '{\n  "total_bandwidth": "50M",\n  "game_reserved": "10M"\n}',
        res: '{\n  "success": true,\n  "message": "Pemisah Trafik Game & Sosmed berhasil diterapkan."\n}'
      },

      // Multi-WAN PCC
      {
        cat: 'load-balance',
        method: 'POST',
        path: '/api/v1/load-balance/pcc/setup',
        desc: 'Setup Multi-WAN PCC (Per Connection Classifier) Load Balancing otomatis dengan failover dan routing marks.',
        req: '{\n  "lan_interface": "bridge",\n  "wans": [\n    { "interface": "ether1", "gateway": "192.168.1.1", "weight": 1 },\n    { "interface": "ether2", "gateway": "192.168.2.1", "weight": 1 }\n  ]\n}',
        res: '{\n  "success": true,\n  "message": "2-WAN PCC Load Balancing berhasil dikonfigurasi."\n}'
      },
      {
        cat: 'load-balance',
        method: 'POST',
        path: '/api/v1/load-balance/status',
        desc: 'Cek status dan metrik kesehatan gateway Multi-WAN.',
        req: '{}',
        res: '{\n  "success": true,\n  "wans": [\n    { "interface": "ether1", "status": "UP", "gateway": "192.168.1.1" },\n    { "interface": "ether2", "status": "UP", "gateway": "192.168.2.1" }\n  ]\n}'
      },

      // Hotspot & Voucher
      {
        cat: 'hotspot',
        method: 'POST',
        path: '/api/v1/hotspot/generate-batch',
        desc: 'Generate massal voucher hotspot (kode acak / angka), lengkap dengan prefix dan profil paket.',
        req: '{\n  "qty": 10,\n  "prefix": "VIP-",\n  "profile": "default",\n  "uptime_limit": "1d"\n}',
        res: '{\n  "success": true,\n  "count": 10,\n  "vouchers": ["VIP-7821", "VIP-9942", "VIP-3129", "..."]\n}'
      },
      {
        cat: 'hotspot',
        method: 'POST',
        path: '/api/v1/hotspot/active',
        desc: 'Mengambil daftar seluruh pengguna hotspot yang saat ini sedang login aktif beserta durasi dan trafik rx/tx.',
        req: '{}',
        res: '{\n  "success": true,\n  "data": [\n    {\n      "user": "euryv",\n      "address": "172.16.10.68",\n      "mac-address": "16:98:72:A8:21:4D",\n      "uptime": "23h20m21s",\n      "bytes-in": "462533414",\n      "bytes-out": "5896492795"\n    }\n  ]\n}'
      },
      {
        cat: 'hotspot',
        method: 'POST',
        path: '/api/v1/hotspot/kick',
        desc: 'Memutuskan (kick / disconnect) sesi login pengguna hotspot aktif berdasarkan ID sesi.',
        req: '{\n  "id": "*AC100A44"\n}',
        res: '{\n  "success": true,\n  "message": "User berhasil diputus dari hotspot."\n}'
      },

      // PPP & PPPoE
      {
        cat: 'ppp',
        method: 'POST',
        path: '/api/v1/ppp/secrets',
        desc: 'Daftar akun PPPoE / PPP secrets (pelanggan ISP / RT RW Net).',
        req: '{}',
        res: '{\n  "success": true,\n  "data": [\n    { "name": "user01", "service": "pppoe", "profile": "Paket-20Mbps" }\n  ]\n}'
      },
      {
        cat: 'ppp',
        method: 'POST',
        path: '/api/v1/ppp/customer/isolate',
        desc: 'Isolir pelanggan PPPoE yang menunggak bayar: otomatis memindahkan profil ke paket ISOLIR dan memutuskan sesi aktif.',
        req: '{\n  "name": "user01",\n  "isolate_profile": "ISOLIR"\n}',
        res: '{\n  "success": true,\n  "message": "Pelanggan user01 berhasil diisolir."\n}'
      },
      {
        cat: 'ppp',
        method: 'POST',
        path: '/api/v1/ppp/customer/restore',
        desc: 'Pulihkan pelanggan PPPoE yang telah bayar tagihan kembali ke paket awal.',
        req: '{\n  "name": "user01",\n  "active_profile": "Paket-20Mbps"\n}',
        res: '{\n  "success": true,\n  "message": "Pelanggan user01 berhasil dipulihkan."\n}'
      },

      // WireGuard & VPN
      {
        cat: 'wireguard',
        method: 'POST',
        path: '/api/v1/wireguard/interfaces',
        desc: 'Daftar interface WireGuard (RouterOS v7).',
        req: '{}',
        res: '{\n  "success": true,\n  "data": [{ "name": "wg0", "listen-port": "13231", "public-key": "..." }]\n}'
      },
      {
        cat: 'wireguard',
        method: 'POST',
        path: '/api/v1/wireguard/peer/add',
        desc: 'Tambahkan peer WireGuard baru (client kantor / remote worker).',
        req: '{\n  "interface": "wg0",\n  "public_key": "xYz123...",\n  "allowed_address": "10.0.0.2/32"\n}',
        res: '{\n  "success": true,\n  "message": "Peer WireGuard berhasil didaftarkan."\n}'
      },

      // Firewall & NAT
      {
        cat: 'firewall',
        method: 'POST',
        path: '/api/v1/firewall/block-ip',
        desc: 'Blokir IP tertentu dengan memasukkannya ke Address-List blacklist drop.',
        req: '{\n  "address": "192.168.88.99",\n  "list": "Blacklist",\n  "timeout": "1d"\n}',
        res: '{\n  "success": true,\n  "message": "IP 192.168.88.99 dimasukkan ke Blacklist."\n}'
      },
      {
        cat: 'firewall',
        method: 'POST',
        path: '/api/v1/firewall/filters',
        desc: 'Ambil seluruh daftar aturan Firewall Filter beserta hit counter bytes/packets.',
        req: '{}',
        res: '{\n  "success": true,\n  "data": [{ "chain": "input", "action": "accept", "bytes": "48921" }]\n}'
      },

      // Queues & Bandwidth
      {
        cat: 'queues',
        method: 'POST',
        path: '/api/v1/queues/inspect-user',
        desc: 'Inspeksi batas bandwidth, kecepatan saat ini, dan penggunaan pengguna berdasarkan IP atau nama.',
        req: '{\n  "query": "192.168.88.50"\n}',
        res: '{\n  "success": true,\n  "found": true,\n  "max_limit": "10M/20M",\n  "rate": "1.2M/5.4M"\n}'
      },
      {
        cat: 'queues',
        method: 'POST',
        path: '/api/v1/queues/overview-summary',
        desc: 'Rekapitulasi total bandwidth antrian dan daftar Top Downloaders saat ini.',
        req: '{}',
        res: '{\n  "success": true,\n  "total_queues": 24,\n  "top_downloaders": [\n    { "name": "queue-user1", "rate": "9.8M" }\n  ]\n}'
      },

      // Interfaces & Streaming
      {
        cat: 'interfaces',
        method: 'POST',
        path: '/api/v1/interfaces/all',
        desc: 'Daftar lengkap antarmuka (Ethernet, SFP, Bridge, WLAN, VLAN) beserta status running, MTU, dan MAC.',
        req: '{}',
        res: '{\n  "success": true,\n  "data": [\n    { "name": "ether1", "type": "ether", "running": "true", "mac-address": "..." }\n  ]\n}'
      },
      {
        cat: 'interfaces',
        method: 'GET',
        path: '/api/v1/interfaces/stream',
        desc: 'Server-Sent Events (SSE) streaming kecepatan rx/tx bps real-time per antarmuka (query: ?interface=ether1&token=...).',
        req: 'GET /api/v1/interfaces/stream?interface=ether1&token=...',
        res: 'event: traffic\ndata: {"interface":"ether1","rx_bps":12500400,"tx_bps":4300100}\n\n'
      },

      // Tools & NOC
      {
        cat: 'tools',
        method: 'POST',
        path: '/api/v1/tools/ping',
        desc: 'Eksekusi ICMP Ping dari router MikroTik ke target IP / host tujuan dengan data latency RTT dan packet loss.',
        req: '{\n  "address": "8.8.8.8",\n  "count": 3\n}',
        res: '{\n  "success": true,\n  "packets_sent": 3,\n  "packets_received": 3,\n  "packet_loss": 0,\n  "min_rtt": "12ms",\n  "avg_rtt": "15ms",\n  "max_rtt": "18ms"\n}'
      },
      {
        cat: 'tools',
        method: 'POST',
        path: '/api/v1/tools/torch',
        desc: 'NOC Torch: sniffer aliran paket real-time (Src IP, Dst IP, Port, Protokol, Tx/Rx rate).',
        req: '{\n  "interface": "ether1",\n  "duration": 3\n}',
        res: '{\n  "success": true,\n  "flows": [\n    { "src": "192.168.88.50:54321", "dst": "142.250.190.46:443", "tx": "1.2Mbps" }\n  ]\n}'
      },
      {
        cat: 'tools',
        method: 'POST',
        path: '/api/v1/tools/romon/status',
        desc: 'Cek status Router Management Overlay Network (RoMON) untuk manajemen router jarak jauh Layer-2.',
        req: '{}',
        res: '{\n  "success": true,\n  "enabled": true,\n  "romon_id": "00:0C:42:1B:32:00"\n}'
      },

      // Telegram Alerts
      {
        cat: 'telegram',
        method: 'POST',
        path: '/api/v1/telegram/setup-netwatch',
        desc: 'Setup Netwatch otomatis terintegrasi bot Telegram: saat host ISP/Server down, router langsung kirim notifikasi Telegram.',
        req: '{\n  "bot_token": "123456:ABC-DEF",\n  "chat_id": "-10012345678",\n  "target_ip": "1.1.1.1",\n  "host_name": "Gateway-ISP-1"\n}',
        res: '{\n  "success": true,\n  "message": "Netwatch Telegram monitoring untuk Gateway-ISP-1 berhasil diset."\n}'
      },

      // The Dude
      {
        cat: 'dude',
        method: 'POST',
        path: '/api/v1/dude/status',
        desc: 'Cek status server monitor The Dude pada RouterOS.',
        req: '{}',
        res: '{\n  "success": true,\n  "enabled": true,\n  "status": "running",\n  "db_size": "14.2MB"\n}'
      },

      // System & Hardware
      {
        cat: 'system',
        method: 'POST',
        path: '/api/v1/system/resource',
        desc: 'Informasi komprehensif resource router: CPU model, CPU load, total memory, free memory, RouterOS version, uptime, dan board name.',
        req: '{}',
        res: '{\n  "success": true,\n  "data": {\n    "cpu-load": "3",\n    "free-memory": "119283712",\n    "total-memory": "134217728",\n    "uptime": "2d14h20m",\n    "version": "7.14.3",\n    "board-name": "hAP ac2"\n  }\n}'
      },
      {
        cat: 'system',
        method: 'POST',
        path: '/api/v1/system/identity',
        desc: 'Ambil nama identitas router saat ini.',
        req: '{}',
        res: '{\n  "success": true,\n  "name": "MikroTik-Main-Router"\n}'
      },
      {
        cat: 'system',
        method: 'POST',
        path: '/api/v1/system/check-update',
        desc: 'Periksa ketersediaan update firmware RouterOS dari server MikroTik.',
        req: '{}',
        res: '{\n  "success": true,\n  "installed_version": "7.14.3",\n  "latest_version": "7.16.1",\n  "status": "New version is available"\n}'
      },

      // Backup & Files
      {
        cat: 'backup',
        method: 'POST',
        path: '/api/v1/backup/create',
        desc: 'Membuat file backup binary terenkripsi (.backup) dari konfigurasi router.',
        req: '{\n  "name": "auto-backup-daily",\n  "password": "StrongBackupPassword"\n}',
        res: '{\n  "success": true,\n  "message": "Backup auto-backup-daily.backup berhasil dibuat."\n}'
      },
      {
        cat: 'backup',
        method: 'POST',
        path: '/api/v1/backup/export',
        desc: 'Export seluruh konfigurasi router ke file skrip teks (.rsc) yang terbaca.',
        req: '{\n  "file": "config-export.rsc"\n}',
        res: '{\n  "success": true,\n  "message": "Export config-export.rsc berhasil dibuat."\n}'
      },

      // Raw Command & Stream
      {
        cat: 'raw',
        method: 'POST',
        path: '/api/v1/command',
        desc: 'Universal Raw Command Executor: eksekusi perintah RouterOS API arbitrer apapun dengan parameter bebas langsung ke router.',
        req: '{\n  "command": "/system/resource/print",\n  "params": {}\n}',
        res: '{\n  "success": true,\n  "data": [\n    { "cpu": "MIPS", "cpu-load": "2" }\n  ]\n}'
      },
      {
        cat: 'raw',
        method: 'POST',
        path: '/api/v1/batch',
        desc: 'Batch Command Executor: eksekusi daftar beberapa perintah RouterOS secara atomik berurutan dalam satu koneksi soket.',
        req: '{\n  "commands": [\n    { "command": "/system/identity/print" },\n    { "command": "/system/resource/print" }\n  ]\n}',
        res: '{\n  "success": true,\n  "results": [ ... ]\n}'
      },
      {
        cat: 'raw',
        method: 'WS',
        path: '/ws',
        desc: 'Full-duplex WebSocket Server untuk interaksi dua arah real-time, streaming tag, dan kontrol router tanpa overhead HTTP.',
        req: 'ws://.../ws?token=<GATEWAY_TOKEN>',
        res: '{"type":"welcome","client_id":"..."}'
      }
    ];

    let currentCat = 'all';

    function renderEndpoints(list) {
      const c = document.getElementById('endpoints-container');
      c.innerHTML = '';

      if (list.length === 0) {
        c.innerHTML = '<div style="text-align: center; padding: 48px; color: var(--text-muted);">Tidak ada endpoint yang cocok dengan pencarian.</div>';
        return;
      }

      list.forEach(ep => {
        const card = document.createElement('div');
        card.className = 'endpoint-card';
        card.id = ep.path.replace(/\//g, '-');

        const methodClass = ep.method === 'POST' ? 'method-post' : (ep.method === 'GET' ? 'method-get' : (ep.method === 'SSE' ? 'method-sse' : 'method-ws'));

        card.innerHTML = `
          <div class="ep-header">
            <div class="ep-title-row">
              <span class="method-badge ${methodClass}">${ep.method}</span>
              <span class="ep-path">${ep.path}</span>
            </div>
            <div style="display: flex; gap: 8px;">
              <button class="try-btn" onclick="copyCurl('${ep.path}', ${JSON.stringify(ep.req).replace(/"/g, '&quot;')})">📋 Copy cURL</button>
              <button class="try-btn" onclick="copyJs('${ep.path}', ${JSON.stringify(ep.req).replace(/"/g, '&quot;')})">📜 Copy Fetch JS</button>
              <a href="/?exec=${encodeURIComponent(ep.path)}" class="try-btn" style="background: rgba(16, 185, 129, 0.2); border-color: rgba(16, 185, 129, 0.4); color: #34d399;">⚡ Coba di Playground</a>
            </div>
          </div>
          <div class="ep-desc">${ep.desc}</div>
          <div class="ep-details-grid">
            <div>
              <div class="box-label">
                <span>Request Payload (JSON Body)</span>
                <button class="copy-btn" onclick="copyText('${encodeURIComponent(ep.req)}')">Copy</button>
              </div>
              <pre class="code-box">${escapeHtml(ep.req)}</pre>
            </div>
            <div>
              <div class="box-label">
                <span>Contoh Response (JSON Output)</span>
                <button class="copy-btn" onclick="copyText('${encodeURIComponent(ep.res)}')">Copy</button>
              </div>
              <pre class="code-box" style="color: #4ade80;">${escapeHtml(ep.res)}</pre>
            </div>
          </div>
        `;
        c.appendChild(card);
      });
    }

    function escapeHtml(str) {
      return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    }

    function copyText(encoded) {
      navigator.clipboard.writeText(decodeURIComponent(encoded));
      alert("Teks berhasil disalin ke clipboard!");
    }

    function copyCurl(path, payload) {
      const token = localStorage.getItem('ros_token') || 'change-me-to-a-long-random-string';
      const host = localStorage.getItem('ros_host') || '192.168.88.1';
      const port = localStorage.getItem('ros_port') || '8728';
      const user = localStorage.getItem('ros_user') || 'admin';
      const pass = localStorage.getItem('ros_pass') || '';

      const curl = `curl -X POST "https://ros-gateway.samrifa.com${path}" \\
  -H "Authorization: Bearer ${token}" \\
  -H "X-Router-Host: ${host}" \\
  -H "X-Router-Port: ${port}" \\
  -H "X-Router-User: ${user}" \\
  -H "X-Router-Pass: ${pass}" \\
  -H "Content-Type: application/json" \\
  -d '${payload.replace(/\n\s*/g, '')}'`;

      navigator.clipboard.writeText(curl);
      alert("Snippet cURL lengkap berhasil disalin ke clipboard!");
    }

    function copyJs(path, payload) {
      const token = localStorage.getItem('ros_token') || 'change-me-to-a-long-random-string';
      const host = localStorage.getItem('ros_host') || '192.168.88.1';
      const port = localStorage.getItem('ros_port') || '8728';
      const user = localStorage.getItem('ros_user') || 'admin';
      const pass = localStorage.getItem('ros_pass') || '';

      const js = `const res = await fetch("https://ros-gateway.samrifa.com${path}", {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer ' + '${token}',
    'X-Router-Host': '${host}',
    'X-Router-Port': '${port}',
    'X-Router-User': '${user}',
    'X-Router-Pass': '${pass}',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify(${payload})
});
const data = await res.json();
console.log(data);`;

      navigator.clipboard.writeText(js);
      alert("Snippet JavaScript (Fetch) berhasil disalin ke clipboard!");
    }

    function jumpCategory(cat) {
      currentCat = cat;
      document.querySelectorAll('.sidebar .nav-item').forEach(el => el.classList.remove('active'));
      event.currentTarget.classList.add('active');
      filterEndpoints();
    }

    function filterEndpoints() {
      const q = document.getElementById('filter-input').value.toLowerCase().trim();
      const filtered = ENDPOINTS.filter(ep => {
        const matchCat = (currentCat === 'all' || ep.cat === currentCat);
        const matchQuery = !q || ep.path.toLowerCase().includes(q) || ep.desc.toLowerCase().includes(q) || ep.cat.toLowerCase().includes(q);
        return matchCat && matchQuery;
      });
      renderEndpoints(filtered);
    }

    window.addEventListener('DOMContentLoaded', () => {
      document.getElementById('total-count').textContent = ENDPOINTS.length;
      renderEndpoints(ENDPOINTS);
    });
  </script>
</body>
</html>"##)
}

/// GET /api/v1/spec - Machine-readable JSON Catalog of all Gateway Endpoints
pub async fn api_spec_json() -> Json<Value> {
    Json(json!({
        "openapi": "3.0.3",
        "info": {
            "title": "MikroTik Universal Rust Gateway API",
            "version": "0.2.0",
            "description": "High-performance sub-millisecond MikroTik management gateway"
        },
        "servers": [
            { "url": "https://ros-gateway.samrifa.com" },
            { "url": "http://127.0.0.1:8080" }
        ],
        "security": [
            { "BearerAuth": [] }
        ],
        "components": {
            "securitySchemes": {
                "BearerAuth": {
                    "type": "http",
                    "scheme": "bearer"
                }
            },
            "parameters": {
                "RouterHost": {
                    "name": "X-Router-Host",
                    "in": "header",
                    "required": false,
                    "schema": { "type": "string" },
                    "description": "MikroTik router IP or domain name"
                },
                "RouterPort": {
                    "name": "X-Router-Port",
                    "in": "header",
                    "required": false,
                    "schema": { "type": "integer", "default": 8728 },
                    "description": "MikroTik API port"
                },
                "RouterUser": {
                    "name": "X-Router-User",
                    "in": "header",
                    "required": false,
                    "schema": { "type": "string" },
                    "description": "MikroTik admin username"
                },
                "RouterPass": {
                    "name": "X-Router-Pass",
                    "in": "header",
                    "required": false,
                    "schema": { "type": "string" },
                    "description": "MikroTik user password"
                }
            }
        }
    }))
}

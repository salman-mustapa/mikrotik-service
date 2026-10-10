use axum::response::Html;
use axum::Json;
use serde_json::{json, Value};

/// GET / - MikroTik NOC Operations Command Center & Developer Console (Light Theme)
pub async fn console_page() -> Html<&'static str> {
    Html(r##"<!DOCTYPE html>
<html lang="id">
<head>
  <meta charset="UTF-8">
  <title>MikroTik Universal Gateway - NOC Operations Console &amp; Developer Playground</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600;700&display=swap" rel="stylesheet">
  <style>
    :root {
      --bg: #f8fafc;
      --card-bg: #ffffff;
      --card-border: #e2e8f0;
      --text-main: #0f172a;
      --text-muted: #64748b;
      --text-sub: #475569;
      --accent-indigo: #4f46e5;
      --accent-blue: #0284c7;
      --accent-emerald: #059669;
      --accent-amber: #d97706;
      --accent-rose: #e11d48;
      --code-bg: #f8fafc;
      --code-border: #e2e8f0;
      --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
      --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.06), 0 2px 4px -2px rgba(0, 0, 0, 0.06);
    }
    * { box-sizing: border-box; margin: 0; padding: 0; font-family: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
    body {
      background: var(--bg);
      color: var(--text-main);
      min-height: 100vh;
      display: flex;
      flex-direction: column;
    }
    code, pre, .mono { font-family: 'JetBrains Mono', monospace; }

    /* Top Navigation Header */
    header {
      position: sticky;
      top: 0;
      z-index: 50;
      background: #ffffff;
      border-bottom: 1px solid var(--card-border);
      box-shadow: var(--shadow-sm);
      padding: 12px 24px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 16px;
      flex-wrap: wrap;
    }
    .brand-wrap {
      display: flex;
      align-items: center;
      gap: 12px;
      text-decoration: none;
      color: inherit;
    }
    .brand-logo {
      font-size: 1.35rem;
    }
    .brand-title {
      font-weight: 800;
      font-size: 1.1rem;
      letter-spacing: -0.3px;
      color: var(--text-main);
    }
    .badge-endpoints {
      background: #eff6ff;
      border: 1px solid #bfdbfe;
      color: #1d4ed8;
      font-size: 0.72rem;
      font-weight: 700;
      padding: 3px 10px;
      border-radius: 999px;
    }
    .badge-engine {
      background: #ecfdf5;
      border: 1px solid #a7f3d0;
      color: #047857;
      font-size: 0.72rem;
      font-weight: 700;
      padding: 3px 10px;
      border-radius: 999px;
      display: flex;
      align-items: center;
      gap: 5px;
    }
    .dot-live {
      width: 7px;
      height: 7px;
      background: #10b981;
      border-radius: 50%;
    }

    .nav-links {
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .nav-btn {
      background: #ffffff;
      border: 1px solid var(--card-border);
      color: var(--text-muted);
      padding: 6px 14px;
      border-radius: 7px;
      font-size: 0.8rem;
      font-weight: 600;
      text-decoration: none;
      display: inline-flex;
      align-items: center;
      gap: 6px;
      transition: all 0.15s;
    }
    .nav-btn:hover { background: #f1f5f9; color: var(--text-main); }
    .nav-btn.active {
      background: var(--accent-indigo);
      color: white;
      border-color: var(--accent-indigo);
    }
    .nav-btn-highlight {
      background: #eff6ff;
      border: 1px solid #93c5fd;
      color: #1d4ed8;
    }
    .nav-btn-highlight:hover {
      background: #dbeafe;
    }

    /* Router Target Credentials Bar */
    .creds-container {
      background: #ffffff;
      border-bottom: 1px solid var(--card-border);
      padding: 10px 24px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      flex-wrap: wrap;
    }
    .creds-inputs {
      display: flex;
      align-items: center;
      gap: 8px;
      flex-wrap: wrap;
    }
    .cred-group {
      display: flex;
      align-items: center;
      background: #f8fafc;
      border: 1px solid var(--card-border);
      border-radius: 6px;
      padding: 3px 8px;
    }
    .cred-label {
      font-size: 0.72rem;
      font-weight: 700;
      color: var(--text-muted);
      margin-right: 6px;
    }
    .cred-field {
      border: none;
      background: transparent;
      font-size: 0.76rem;
      font-family: 'JetBrains Mono', monospace;
      color: var(--text-main);
      outline: none;
    }
    .btn-sync {
      background: var(--accent-indigo);
      color: white;
      border: none;
      padding: 6px 14px;
      border-radius: 6px;
      font-size: 0.78rem;
      font-weight: 700;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 5px;
      transition: background 0.15s;
    }
    .btn-sync:hover { background: #4338ca; }

    /* Main Container */
    main {
      flex: 1;
      max-width: 1650px;
      width: 100%;
      margin: 0 auto;
      padding: 20px 24px;
      display: flex;
      flex-direction: column;
      gap: 20px;
    }

    /* Quick KPI Cards Grid */
    .kpi-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
      gap: 16px;
    }
    .kpi-card {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 16px 20px;
      box-shadow: var(--shadow-sm);
      display: flex;
      flex-direction: column;
      justify-content: space-between;
      gap: 10px;
    }
    .kpi-title {
      font-size: 0.76rem;
      font-weight: 700;
      color: var(--text-muted);
      text-transform: uppercase;
      letter-spacing: 0.5px;
      display: flex;
      align-items: center;
      justify-content: space-between;
    }
    .kpi-val {
      font-size: 1.45rem;
      font-weight: 800;
      color: var(--text-main);
      letter-spacing: -0.5px;
    }
    .kpi-sub {
      font-size: 0.78rem;
      color: var(--text-muted);
    }
    .progress-bar-wrap {
      background: #e2e8f0;
      border-radius: 999px;
      height: 6px;
      overflow: hidden;
      margin-top: 4px;
    }
    .progress-fill {
      background: var(--accent-indigo);
      height: 100%;
      width: 0%;
      transition: width 0.3s ease;
    }

    /* Split Console Layout */
    .console-split {
      display: grid;
      grid-template-columns: 1.15fr 0.85fr;
      gap: 20px;
      align-items: start;
    }
    @media (max-width: 1100px) {
      .console-split { grid-template-columns: 1fr; }
    }

    /* Left Panel: Request Configuration */
    .panel-card {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 20px;
      box-shadow: var(--shadow-sm);
      display: flex;
      flex-direction: column;
      gap: 16px;
    }
    .panel-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding-bottom: 12px;
      border-bottom: 1px solid var(--card-border);
    }
    .panel-title {
      font-size: 0.95rem;
      font-weight: 800;
      color: var(--text-main);
      display: flex;
      align-items: center;
      gap: 8px;
    }

    /* Filter Pills */
    .category-pills {
      display: flex;
      flex-wrap: wrap;
      gap: 6px;
    }
    .cat-btn {
      background: #f8fafc;
      border: 1px solid var(--card-border);
      color: var(--text-muted);
      padding: 4px 10px;
      border-radius: 6px;
      font-size: 0.74rem;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.15s;
    }
    .cat-btn:hover { background: #e2e8f0; color: var(--text-main); }
    .cat-btn.active {
      background: #eff6ff;
      border-color: #93c5fd;
      color: #1d4ed8;
      font-weight: 700;
    }

    /* Endpoint Selector Row */
    .endpoint-select-row {
      display: flex;
      gap: 8px;
      align-items: center;
    }
    .method-badge {
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.72rem;
      font-weight: 800;
      padding: 6px 12px;
      border-radius: 6px;
      cursor: pointer;
      user-select: none;
    }
    .method-badge.m-get { background: #ecfdf5; border: 1px solid #a7f3d0; color: #047857; }
    .method-badge.m-post { background: #eff6ff; border: 1px solid #bfdbfe; color: #1d4ed8; }

    select.ep-dropdown {
      flex: 1;
      padding: 7px 12px;
      border-radius: 6px;
      border: 1px solid var(--card-border);
      background: #ffffff;
      font-size: 0.82rem;
      color: var(--text-main);
      font-family: 'JetBrains Mono', monospace;
      outline: none;
    }

    /* Code Tabs & Snippets */
    .tab-bar {
      display: flex;
      gap: 4px;
      border-bottom: 1px solid var(--card-border);
      padding-bottom: 6px;
    }
    .code-tab-btn {
      background: transparent;
      border: none;
      padding: 4px 10px;
      border-radius: 4px;
      font-size: 0.74rem;
      font-weight: 600;
      color: var(--text-muted);
      cursor: pointer;
    }
    .code-tab-btn.active {
      background: #f1f5f9;
      color: var(--text-main);
      font-weight: 700;
    }

    .code-box {
      background: var(--code-bg);
      border: 1px solid var(--code-border);
      border-radius: 8px;
      padding: 12px;
      font-size: 0.76rem;
      overflow-x: auto;
      color: var(--text-main);
      max-height: 180px;
    }

    /* Request Payload Textarea */
    .payload-area {
      width: 100%;
      height: 110px;
      background: var(--code-bg);
      border: 1px solid var(--card-border);
      border-radius: 8px;
      padding: 10px 12px;
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.78rem;
      color: var(--text-main);
      outline: none;
      resize: vertical;
    }
    .payload-area:focus { border-color: var(--accent-indigo); }

    .btn-exec {
      background: var(--accent-indigo);
      color: white;
      border: none;
      padding: 10px 20px;
      border-radius: 8px;
      font-size: 0.86rem;
      font-weight: 700;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
      transition: background 0.15s;
    }
    .btn-exec:hover { background: #4338ca; }

    /* Right Panel: Live Response Console */
    .resp-console {
      background: #ffffff;
      border: 1px solid var(--card-border);
      border-radius: 12px;
      display: flex;
      flex-direction: column;
      box-shadow: var(--shadow-sm);
      overflow: hidden;
      min-height: 520px;
    }
    .resp-header {
      background: #f8fafc;
      padding: 12px 16px;
      border-bottom: 1px solid var(--card-border);
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .resp-meta {
      display: flex;
      align-items: center;
      gap: 10px;
      font-size: 0.76rem;
      font-family: 'JetBrains Mono', monospace;
    }
    .status-badge {
      background: #ecfdf5;
      color: #047857;
      border: 1px solid #a7f3d0;
      padding: 2px 8px;
      border-radius: 4px;
      font-weight: 700;
    }
    .latency-badge {
      color: var(--text-muted);
      font-weight: 600;
    }
    .btn-copy {
      background: #ffffff;
      border: 1px solid var(--card-border);
      padding: 4px 10px;
      border-radius: 6px;
      font-size: 0.72rem;
      font-weight: 600;
      color: var(--text-main);
      cursor: pointer;
    }
    .btn-copy:hover { background: #f1f5f9; }

    .resp-body {
      flex: 1;
      padding: 16px;
      background: #fafafa;
      overflow: auto;
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.78rem;
      line-height: 1.5;
      color: #1e293b;
      max-height: 560px;
      white-space: pre-wrap;
      word-break: break-all;
    }
  </style>
</head>
<body>

  <!-- Top Enterprise Header -->
  <header>
    <a href="/" class="brand-wrap">
      <span class="brand-logo">🦀</span>
      <div>
        <div class="brand-title">MikroTik Universal Gateway</div>
        <div style="font-size: 0.72rem; color: var(--text-muted);">Enterprise NOC Management Platform</div>
      </div>
    </a>

    <div style="display: flex; align-items: center; gap: 8px;">
      <span class="badge-endpoints">155+ Enterprise Endpoints</span>
      <span class="badge-engine"><span class="dot-live"></span>Rust Socket Engine: &lt;2ms</span>
    </div>

    <div class="nav-links">
      <a href="/" class="nav-btn active">🎮 Console &amp; Playground</a>
      <a href="/topology" class="nav-btn nav-btn-highlight">🗺️ Visualizer Topologi</a>
      <a href="/docs" class="nav-btn">📖 API Docs &amp; Schema</a>
    </div>
  </header>

  <!-- Router Credentials Sync Bar -->
  <div class="creds-container">
    <div class="creds-inputs">
      <div class="cred-group">
        <span class="cred-label">Router Host:</span>
        <input type="text" id="target-host" class="cred-field" style="width: 140px;" value="ath.vpnbersama.us">
      </div>
      <div class="cred-group">
        <span class="cred-label">API Port:</span>
        <input type="number" id="target-port" class="cred-field" style="width: 60px;" value="51121">
      </div>
      <div class="cred-group">
        <span class="cred-label">User:</span>
        <input type="text" id="target-user" class="cred-field" style="width: 80px;" value="salman">
      </div>
      <div class="cred-group">
        <span class="cred-label">Password:</span>
        <input type="password" id="target-pass" class="cred-field" style="width: 90px;">
      </div>
      <div class="cred-group">
        <span class="cred-label">Bearer Token:</span>
        <input type="text" id="gw-token" class="cred-field" style="width: 150px;" value="change-me-to-a-long-random-string">
      </div>
    </div>
    <button class="btn-sync" onclick="testConnectionAndSnapshot()">⚡ Sinkronisasi &amp; Snapshot</button>
  </div>

  <main>
    <!-- KPI Overview Cards Grid -->
    <div class="kpi-grid">
      <div class="kpi-card">
        <div class="kpi-title">Identitas Router <span id="kpi-board" style="color: var(--accent-indigo);">RB951Ui-2HnD</span></div>
        <div class="kpi-val" id="kpi-identity">Router Manyta</div>
        <div class="kpi-sub" id="kpi-ros">RouterOS v6.48.6 | Uptime: 1w4d</div>
      </div>

      <div class="kpi-card">
        <div class="kpi-title">CPU Load &amp; Frekuensi <span id="kpi-freq">600 MHz</span></div>
        <div class="kpi-val" id="kpi-cpu">38%</div>
        <div class="progress-bar-wrap">
          <div class="progress-fill" id="kpi-cpu-bar" style="width: 38%;"></div>
        </div>
      </div>

      <div class="kpi-card">
        <div class="kpi-title">Memori RAM Bebas <span id="kpi-total-ram">128 MB</span></div>
        <div class="kpi-val" id="kpi-free-ram">89 MB</div>
        <div class="kpi-sub" id="kpi-mem-percent">Penggunaan RAM: 30.1%</div>
      </div>

      <div class="kpi-card">
        <div class="kpi-title">Klien &amp; Relasi Jaringan <span style="color: var(--accent-emerald);">● Online</span></div>
        <div class="kpi-val" id="kpi-clients">17 Perangkat</div>
        <div class="kpi-sub" id="kpi-clients-sub">DHCP: 7 | Hotspot: 10 | PPPoE: 0</div>
      </div>
    </div>

    <!-- Interactive Console & Playground Split -->
    <div class="console-split">
      <!-- Left: Request Configuration -->
      <div class="panel-card">
        <div class="panel-header">
          <div class="panel-title">
            <span>⚡ Interactive Endpoint Playground</span>
          </div>
          <div style="font-size: 0.74rem; color: var(--text-muted);">
            Dual HTTP GET &amp; POST Support
          </div>
        </div>

        <!-- Category Filter Pills -->
        <div class="category-pills">
          <button class="cat-btn active" onclick="filterCategory('all', this)">Semua (155+)</button>
          <button class="cat-btn" onclick="filterCategory('notify', this)">💬 Bot &amp; Netwatch</button>
          <button class="cat-btn" onclick="filterCategory('voucher', this)">🎟️ Voucher 2.0</button>
          <button class="cat-btn" onclick="filterCategory('fast', this)">🚀 Fast-Path</button>
          <button class="cat-btn" onclick="filterCategory('net', this)">🌐 Topologi</button>
          <button class="cat-btn" onclick="filterCategory('system', this)">⚙️ Sistem</button>
          <button class="cat-btn" onclick="filterCategory('dhcp', this)">💻 DHCP</button>
          <button class="cat-btn" onclick="filterCategory('hotspot', this)">🔥 Hotspot</button>
          <button class="cat-btn" onclick="filterCategory('pppoe', this)">🌐 PPPoE</button>
          <button class="cat-btn" onclick="filterCategory('ip', this)">📡 IP</button>
          <button class="cat-btn" onclick="filterCategory('firewall', this)">🛡️ Firewall</button>
          <button class="cat-btn" onclick="filterCategory('tools', this)">🛠️ NOC Tools</button>
        </div>

        <!-- Endpoint Select Dropdown & Method Badge -->
        <div class="endpoint-select-row">
          <span class="method-badge m-get" id="badge-method" onclick="toggleMethod()">GET</span>
          <select id="ep-select" class="ep-dropdown" onchange="onEndpointChange()">
            <!-- Options populated dynamically by JS -->
          </select>
        </div>

        <!-- Description Box -->
        <div id="ep-desc" style="font-size: 0.78rem; color: var(--text-sub); line-height: 1.4; padding: 4px 2px;">
          Pilih endpoint untuk melihat spesifikasi dan menguji request secara live.
        </div>

        <!-- Code Snippet Tabs -->
        <div>
          <div class="tab-bar">
            <button class="code-tab-btn active" onclick="switchSnippet('curl', this)">cURL</button>
            <button class="code-tab-btn" onclick="switchSnippet('fetch', this)">JavaScript (Fetch)</button>
            <button class="code-tab-btn" onclick="switchSnippet('laravel', this)">PHP (Laravel / Guzzle)</button>
            <button class="code-tab-btn" onclick="switchSnippet('flutter', this)">Flutter (Dart / Dio)</button>
          </div>
          <pre class="code-box" id="snippet-box"></pre>
        </div>

        <!-- JSON Request Body / Query Params Box -->
        <div id="payload-container">
          <div style="font-size: 0.74rem; font-weight: 700; color: var(--text-muted); margin-bottom: 6px;">
            Request Body (JSON) atau Query Params:
          </div>
          <textarea id="req-payload" class="payload-area" placeholder="{ ... }">{}</textarea>
        </div>

        <button class="btn-exec" onclick="executeCurrentEndpoint()">
          <span>⚡ Kirim Request ke Gateway</span>
        </button>
      </div>

      <!-- Right: Live Response Console -->
      <div class="resp-console">
        <div class="resp-header">
          <div class="resp-meta">
            <span>Status:</span>
            <span class="status-badge" id="resp-status">200 OK</span>
            <span class="latency-badge" id="resp-latency">Latency: 0 ms</span>
          </div>
          <button class="btn-copy" onclick="copyResponse()">Salin Respons JSON</button>
        </div>
        <pre class="resp-body" id="resp-viewer">// Tekan 'Kirim Request' atau 'Sinkronisasi' untuk melihat respons data...</pre>
      </div>
    </div>
  </main>

  <script>
    const ENDPOINTS = [
      { id: "overview", path: "/api/v1/overview", method: "GET", cat: "fast", desc: "Snapshot agregasi cepat: CPU, memory, uptime, total user hotspot, ppp, dan leases dalam <2ms.", defaultPayload: "{}" },
      { id: "connected_devices", path: "/api/v1/network/connected-devices", method: "GET", cat: "fast", desc: "Korelasikan perangkat terhubung dari seluruh layer (DHCP, ARP, Hotspot, WiFi, PPPoE).", defaultPayload: "{}" },
      { id: "topology", path: "/api/v1/network/topology-graph", method: "GET", cat: "net", desc: "Data graf relasi simpul (Nodes & Edges) jaringan berjenjang untuk visualizer topologi.", defaultPayload: "{}" },
      { id: "sys_resource", path: "/api/v1/system/resource", method: "GET", cat: "system", desc: "Spesifikasi resource CPU, RAM bebas, storage, dan arsitektur router.", defaultPayload: "{}" },
      { id: "sys_routerboard", path: "/api/v1/system/routerboard", method: "GET", cat: "system", desc: "Model board MikroTik, serial number, firmware saat ini dan firmware upgrade.", defaultPayload: "{}" },
      { id: "sys_identity", path: "/api/v1/system/identity", method: "GET", cat: "system", desc: "Nama identity sistem router MikroTik.", defaultPayload: "{}" },
      { id: "sys_clock", path: "/api/v1/system/clock", method: "GET", cat: "system", desc: "Waktu jam, tanggal, dan timezone internal router.", defaultPayload: "{}" },
      { id: "sys_packages", path: "/api/v1/system/packages", method: "GET", cat: "system", desc: "Daftar paket software RouterOS terpasang (wireless, routing, ppp, security).", defaultPayload: "{}" },
      { id: "sys_services", path: "/api/v1/system/services", method: "GET", cat: "system", desc: "Daftar service port router (API, SSH, Winbox, WWW) dan status disable/enable.", defaultPayload: "{}" },
      { id: "dhcp_leases", path: "/api/v1/dhcp/leases", method: "GET", cat: "dhcp", desc: "Daftar seluruh IP lease DHCP server, hostname klien, status bound, dan MAC address.", defaultPayload: "{}" },
      { id: "dhcp_servers", path: "/api/v1/dhcp/servers", method: "GET", cat: "dhcp", desc: "Daftar instance DHCP Server yang aktif dan interface induknya.", defaultPayload: "{}" },
      { id: "dhcp_networks", path: "/api/v1/dhcp/networks", method: "GET", cat: "dhcp", desc: "Daftar subnet jaringan DHCP, gateway IP, DNS server, dan domain.", defaultPayload: "{}" },
      { id: "hs_active", path: "/api/v1/hotspot/active", method: "GET", cat: "hotspot", desc: "Daftar pengguna Hotspot yang sedang login aktif, IP, MAC address, uptime, dan bytes.", defaultPayload: "{}" },
      { id: "hs_users", path: "/api/v1/hotspot/users", method: "GET", cat: "hotspot", desc: "Daftar seluruh database user Hotspot dan voucher di router.", defaultPayload: "{}" },
      { id: "hs_hosts", path: "/api/v1/hotspot/hosts", method: "GET", cat: "hotspot", desc: "Daftar seluruh perangkat host yang terdeteksi di subnet hotspot (authorized & unauthorized).", defaultPayload: "{}" },
      { id: "hs_profiles", path: "/api/v1/hotspot/profiles", method: "GET", cat: "hotspot", desc: "Profil kecepatan dan batasan user Hotspot (rate-limit, shared-users).", defaultPayload: "{}" },
      { id: "hs_ip_bindings", path: "/api/v1/hotspot/ip-bindings", method: "GET", cat: "hotspot", desc: "Daftar IP/MAC binding Hotspot untuk bypass tanpa login (AP, Smart TV, CCTV).", defaultPayload: "{}" },
      { id: "ppp_secrets", path: "/api/v1/ppp/secrets", method: "GET", cat: "pppoe", desc: "Database akun PPPoE ISP, username, password, profile, dan IP remote statis.", defaultPayload: "{}" },
      { id: "ppp_active", path: "/api/v1/ppp/active", method: "GET", cat: "pppoe", desc: "Daftar sesi PPPoE yang sedang terhubung aktif (tunnel UP) beserta uptime.", defaultPayload: "{}" },
      { id: "ppp_servers", path: "/api/v1/ppp/servers", method: "GET", cat: "pppoe", desc: "Daftar server PPPoE Concentrator di router.", defaultPayload: "{}" },
      { id: "if_all", path: "/api/v1/interfaces/all", method: "GET", cat: "net", desc: "Daftar seluruh interface jaringan (ether, bridge, wlan, pppoe, vlan) dan statistik trafik.", defaultPayload: "{}" },
      { id: "ip_addr", path: "/api/v1/ip/addresses", method: "GET", cat: "ip", desc: "Daftar alamat IP yang dikonfigurasi pada setiap interface router.", defaultPayload: "{}" },
      { id: "ip_routes", path: "/api/v1/ip/routes", method: "GET", cat: "ip", desc: "Tabel routing IPv4 lengkap (default gateway, routing dinamik dan statis).", defaultPayload: "{}" },
      { id: "ip_arp", path: "/api/v1/ip/arp", method: "GET", cat: "ip", desc: "Tabel pemetaan ARP (IP ke MAC Address) dari seluruh interface router.", defaultPayload: "{}" },
      { id: "ip_dns", path: "/api/v1/ip/dns", method: "GET", cat: "ip", desc: "Konfigurasi upstream DNS resolver dan opsi allow-remote-requests.", defaultPayload: "{}" },
      { id: "fw_filters", path: "/api/v1/firewall/filters", method: "GET", cat: "firewall", desc: "Daftar rule firewall filter (input, forward, output).", defaultPayload: "{}" },
      { id: "fw_nat", path: "/api/v1/firewall/nat", method: "GET", cat: "firewall", desc: "Daftar rule NAT (Masquerade, Port Forwarding, dst-nat).", defaultPayload: "{}" },
      { id: "fw_addr_list", path: "/api/v1/firewall/address-lists", method: "GET", cat: "firewall", desc: "Daftar Address-List firewall (Whitelist, Blacklist, isolir).", defaultPayload: "{}" },
      { id: "fw_conntrack", path: "/api/v1/firewall/connections", method: "GET", cat: "firewall", desc: "Inspeksi tabel conntrack aktif (IP sumber, tujuan, protokol, timeout).", defaultPayload: "{}" },
      { id: "fw_top_talkers", path: "/api/v1/firewall/connections/top-talkers", method: "GET", cat: "firewall", desc: "Deteksi top talkers / host penghabis sesi koneksi (BitTorrent, DDoS, botnet).", defaultPayload: "{}" },
      { id: "dns_adlist", path: "/api/v1/dns/adlist", method: "GET", cat: "ip", desc: "RouterOS v7 native AdList (Blocklist Iklan & Malware tingkat router).", defaultPayload: "{}" },
      { id: "dns_adlist_presets", path: "/api/v1/dns/adlist/presets", method: "GET", cat: "ip", desc: "Katalog preset blocklist resmi aman (HaGeZi, StevenBlack, AdGuard).", defaultPayload: "{}" },
      { id: "dhcp_alerts", path: "/api/v1/dhcp/alerts", method: "GET", cat: "dhcp", desc: "Monitoring alarm Rogue DHCP Server ilegal di jaringan lokal.", defaultPayload: "{}" },
      { id: "tools_ping", path: "/api/v1/tools/ping", method: "POST", cat: "tools", desc: "Kirim ICMP Ping langsung dari router ke target host/IP dengan hitungan latency.", defaultPayload: '{\n  "address": "8.8.8.8",\n  "count": 4\n}' },
      { id: "tools_multi_ping", path: "/api/v1/tools/multi-ping", method: "GET", cat: "tools", desc: "Ping matriks multi-target sekaligus (Gateway, DNS Cloudflare, Google, OpenDNS) dengan ringkasan SLA.", defaultPayload: '{\n  "targets": ["1.1.1.1", "8.8.8.8", "208.67.222.222"],\n  "count": 3\n}' },
      { id: "capsman_radios", path: "/api/v1/capsman/radios", method: "GET", cat: "net", desc: "Daftar seluruh radio Access Point MikroTik (CAPs) yang terhubung ke controller.", defaultPayload: "{}" },
      { id: "capsman_set_wifi", path: "/api/v1/capsman/set-wifi", method: "POST", cat: "net", desc: "Konfigurasi SSID & Password WiFi terpusat ke seluruh AP MikroTik sekaligus.", defaultPayload: '{\n  "ssid": "HOTSPOT-WARUNG",\n  "passphrase": "password123"\n}' },
      { id: "capsman_registrations", path: "/api/v1/capsman/registrations", method: "GET", cat: "net", desc: "Daftar seluruh klien wireless (HP/Laptop) yang tersambung di semua AP.", defaultPayload: "{}" },
      { id: "backup_restore", path: "/api/v1/backup/restore", method: "POST", cat: "system", desc: "Pulihkan sistem router dari file binary (.backup) dan reboot otomatis.", defaultPayload: '{\n  "name": "backup-auto.backup"\n}' },
      { id: "backup_import", path: "/api/v1/backup/import", method: "POST", cat: "system", desc: "Impor dan jalankan skrip konfigurasi RouterOS (.rsc) ke router.", defaultPayload: '{\n  "file": "config.rsc"\n}' },
      { id: "infra_poe_cycle", path: "/api/v1/network/infrastructure/poe-cycle", method: "POST", cat: "net", desc: "Reboot paksa Access Point yang hang lewat pemutusan daya PoE port sementara.", defaultPayload: '{\n  "interface": "ether5",\n  "off_seconds": 3\n}' },
      { id: "infra_ap_tunnel", path: "/api/v1/network/infrastructure/ap-tunnel", method: "POST", cat: "net", desc: "Buka port forwarding sementara untuk akses Web Admin AP pihak ketiga dari luar LAN.", defaultPayload: '{\n  "ap_ip": "192.168.100.3",\n  "external_port": 8083\n}' },

      // Integrasi Telegram, WhatsApp Gowa & Netwatch Pro
      { id: "integ_tg_send", path: "/api/v1/integrations/telegram/send", method: "POST", cat: "notify", desc: "Kirim pesan/alert instan ke Telegram Bot via socket fetch RouterOS.", defaultPayload: '{\n  "bot_token": "YOUR_BOT_TOKEN",\n  "chat_id": "123456789",\n  "text": "🚨 *ALERT NOC*: Link Internet Utama DOWN!"\n}' },
      { id: "integ_wa_send", path: "/api/v1/integrations/whatsapp/send", method: "POST", cat: "notify", desc: "Kirim notifikasi pesan WhatsApp langsung via Gowa API service.", defaultPayload: '{\n  "phone": "081234567890",\n  "message": "Halo, voucher internet hotspot Anda: *HOT-9812* berlaku 24 Jam.",\n  "gowa_url": "http://127.0.0.1:3000"\n}' },
      { id: "integ_multi_notify", path: "/api/v1/integrations/notify", method: "POST", cat: "notify", desc: "Multi-platform notification dispatcher (Telegram + WhatsApp) dengan severity level.", defaultPayload: '{\n  "channel": "all",\n  "severity": "WARNING",\n  "title": "Koneksi AP Terganggu",\n  "message": "AP Ruang Meeting down sejak 2 menit yang lalu.",\n  "telegram_bot_token": "",\n  "telegram_chat_id": "",\n  "whatsapp_phone": ""\n}' },
      { id: "integ_netwatch_setup", path: "/api/v1/integrations/netwatch/setup", method: "POST", cat: "notify", desc: "Setup probe Netwatch RouterOS otomatis dengan skrip alert instan ke Telegram & WhatsApp.", defaultPayload: '{\n  "host": "172.16.10.2",\n  "comment": "FAUJIA HOTSPOT 1",\n  "interval": "00:00:10",\n  "timeout": "1000ms",\n  "telegram_bot_token": "BOT_TOKEN",\n  "telegram_chat_id": "CHAT_ID"\n}' },
      { id: "integ_netwatch_batch", path: "/api/v1/integrations/netwatch/batch-setup", method: "POST", cat: "notify", desc: "Otomatisasi pengawasan Netwatch armada seluruh Access Point (AP Fleet) sekaligus.", defaultPayload: '{\n  "items": [\n    { "host": "172.16.10.2", "comment": "FAUJIA HOTSPOT 1" },\n    { "host": "172.16.10.3", "comment": "FAUJIA HOTSPOT 2" },\n    { "host": "172.16.10.4", "comment": "FAUJIA HOTSPOT 3" },\n    { "host": "172.16.10.5", "comment": "FAUJIA HOTSPOT 4" }\n  ],\n  "telegram_bot_token": "BOT_TOKEN",\n  "telegram_chat_id": "CHAT_ID"\n}' },
      { id: "integ_netwatch_list", path: "/api/v1/integrations/netwatch/list", method: "GET", cat: "notify", desc: "Daftar seluruh probe Netwatch, status up/down/since, dan skrip alert terpasang.", defaultPayload: "{}" },
      { id: "integ_netwatch_toggle", path: "/api/v1/integrations/netwatch/toggle", method: "POST", cat: "notify", desc: "Enable atau Disable probe Netwatch target pada router.", defaultPayload: '{\n  "id": "*1",\n  "disabled": false\n}' },
      { id: "integ_reports_sched", path: "/api/v1/integrations/reports/scheduler", method: "POST", cat: "notify", desc: "Pasang skrip & scheduler otomatis RouterOS untuk rekap laporan harian NOC ke Telegram.", defaultPayload: '{\n  "name": "noc-daily-report",\n  "start_time": "07:00:00",\n  "interval": "1d",\n  "telegram_bot_token": "BOT_TOKEN",\n  "telegram_chat_id": "CHAT_ID"\n}' },
      { id: "integ_chat_snippet", path: "/api/v1/integrations/hotspot-chat/snippet", method: "GET", cat: "notify", desc: "Dapatkan snippet embed live chat widget (Intergram Telegram & WhatsApp) untuk login.html & status.html hotspot.", defaultPayload: "{}" },

      // Hotspot Voucher Engine 2.0 (Mikhmon Killer)
      { id: "vouchers_gen", path: "/api/v1/hotspot/vouchers/generate", method: "POST", cat: "voucher", desc: "Ultra-fast parallel batch voucher generator (1-1000 voucher, ROS v6/v7 safe, custom prefix, time/quota limit).", defaultPayload: '{\n  "profile": "1JAM-3RB",\n  "count": 5,\n  "prefix": "HOT-",\n  "user_mode": "same",\n  "charset": "alphanumeric_lower",\n  "length": 6,\n  "timelimit": "1h",\n  "datalimit": "500M",\n  "price": 3000,\n  "selling_price": 3000\n}' },
      { id: "vouchers_track", path: "/api/v1/hotspot/vouchers/tracking", method: "GET", cat: "voucher", desc: "Tracking lifecycle & sisa kuota/waktu voucher presisi real-time (available, online, used, expired).", defaultPayload: "{}" },
      { id: "vouchers_thermal", path: "/api/v1/hotspot/vouchers/thermal-print", method: "POST", cat: "voucher", desc: "Formatter struk termal ESC/POS (58mm/80mm) siap cetak via printer Bluetooth / USB.", defaultPayload: '{\n  "vouchers": [\n    { "username": "HOT-a89f2", "password": "HOT-a89f2", "profile": "1JAM-3RB", "timelimit": "1h", "price": 3000 }\n  ],\n  "hotspot_name": "MANYTANET HOTSPOT",\n  "dns_name": "inetmanyta.net",\n  "paper_width": "58mm"\n}' },
      { id: "vouchers_sell_send", path: "/api/v1/hotspot/vouchers/sell-and-send", method: "POST", cat: "voucher", desc: "POS Kasir: Tandai voucher terjual dan kirim struk instan ke nomor WhatsApp pelanggan via Gowa API.", defaultPayload: '{\n  "username": "HOT-a89f2",\n  "phone": "081234567890",\n  "cashier": "Admin Kasir",\n  "hotspot_name": "MANYTANET",\n  "login_url": "http://inetmanyta.net/login"\n}' },
      { id: "vouchers_clean", path: "/api/v1/hotspot/vouchers/clean-expired", method: "POST", cat: "voucher", desc: "Pembersihan aman voucher kedaluwarsa tanpa menghapus voucher yang belum digunakan.", defaultPayload: '{\n  "force": true\n}' }
    ];

    let currentEndpoint = ENDPOINTS[0];
    let currentMethod = "GET";
    let currentSnippetType = "curl";

    function initCreds() {
      const urlParams = new URLSearchParams(window.location.search);
      if (urlParams.get('host')) document.getElementById('target-host').value = urlParams.get('host');
      else if (localStorage.getItem('ros_host')) document.getElementById('target-host').value = localStorage.getItem('ros_host');

      if (urlParams.get('port')) document.getElementById('target-port').value = urlParams.get('port');
      else if (localStorage.getItem('ros_port')) document.getElementById('target-port').value = localStorage.getItem('ros_port');

      if (urlParams.get('user')) document.getElementById('target-user').value = urlParams.get('user');
      else if (localStorage.getItem('ros_user')) document.getElementById('target-user').value = localStorage.getItem('ros_user');

      if (urlParams.get('pass')) document.getElementById('target-pass').value = urlParams.get('pass');
      else if (localStorage.getItem('ros_pass')) document.getElementById('target-pass').value = localStorage.getItem('ros_pass');

      if (localStorage.getItem('ros_token')) document.getElementById('gw-token').value = localStorage.getItem('ros_token');

      ['target-host', 'target-port', 'target-user', 'target-pass', 'gw-token'].forEach(id => {
        document.getElementById(id).addEventListener('input', () => {
          localStorage.setItem('ros_host', document.getElementById('target-host').value.trim());
          localStorage.setItem('ros_port', document.getElementById('target-port').value.trim());
          localStorage.setItem('ros_user', document.getElementById('target-user').value.trim());
          localStorage.setItem('ros_pass', document.getElementById('target-pass').value);
          localStorage.setItem('ros_token', document.getElementById('gw-token').value.trim());
          updateSnippet();
        });
      });
    }

    function populateDropdown(filterCat = 'all') {
      const sel = document.getElementById('ep-select');
      sel.innerHTML = '';
      const list = filterCat === 'all' ? ENDPOINTS : ENDPOINTS.filter(e => e.cat === filterCat);
      list.forEach(ep => {
        const opt = document.createElement('option');
        opt.value = ep.id;
        opt.textContent = `${ep.method} ${ep.path}`;
        sel.appendChild(opt);
      });
      if (list.length > 0) {
        currentEndpoint = list[0];
        onEndpointChange();
      }
    }

    function filterCategory(cat, btn) {
      document.querySelectorAll('.cat-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      populateDropdown(cat);
    }

    function onEndpointChange() {
      const val = document.getElementById('ep-select').value;
      const found = ENDPOINTS.find(e => e.id === val);
      if (found) {
        currentEndpoint = found;
        currentMethod = found.method;
        updateMethodBadge();
        document.getElementById('ep-desc').textContent = found.desc;
        document.getElementById('req-payload').value = found.defaultPayload || "{}";
        updateSnippet();
      }
    }

    function toggleMethod() {
      currentMethod = currentMethod === "GET" ? "POST" : "GET";
      updateMethodBadge();
      updateSnippet();
    }

    function updateMethodBadge() {
      const b = document.getElementById('badge-method');
      b.textContent = currentMethod;
      b.className = `method-badge m-${currentMethod.toLowerCase()}`;
    }

    function switchSnippet(type, btn) {
      currentSnippetType = type;
      document.querySelectorAll('.code-tab-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      updateSnippet();
    }

    function updateSnippet() {
      const host = document.getElementById('target-host').value.trim();
      const port = document.getElementById('target-port').value.trim();
      const user = document.getElementById('target-user').value.trim();
      const pass = document.getElementById('target-pass').value;
      const token = document.getElementById('gw-token').value.trim();
      const url = window.location.origin + currentEndpoint.path;

      let code = "";
      if (currentSnippetType === "curl") {
        if (currentMethod === "GET") {
          code = `curl -X GET "${url}" \\\n  -H "Authorization: Bearer ${token}" \\\n  -H "X-Router-Host: ${host}" \\\n  -H "X-Router-Port: ${port}" \\\n  -H "X-Router-User: ${user}" \\\n  -H "X-Router-Pass: ${pass}"`;
        } else {
          code = `curl -X POST "${url}" \\\n  -H "Authorization: Bearer ${token}" \\\n  -H "X-Router-Host: ${host}" \\\n  -H "X-Router-Port: ${port}" \\\n  -H "X-Router-User: ${user}" \\\n  -H "X-Router-Pass: ${pass}" \\\n  -H "Content-Type: application/json" \\\n  -d '${currentEndpoint.defaultPayload}'`;
        }
      } else if (currentSnippetType === "fetch") {
        code = `const response = await fetch("${url}", {\n  method: "${currentMethod}",\n  headers: {\n    "Authorization": "Bearer ${token}",\n    "X-Router-Host": "${host}",\n    "X-Router-Port": "${port}",\n    "X-Router-User": "${user}",\n    "X-Router-Pass": "${pass}",\n    "Content-Type": "application/json"\n  }${currentMethod === 'POST' ? ',\n  body: JSON.stringify(' + currentEndpoint.defaultPayload + ')' : ''}\n});\nconst result = await response.json();\nconsole.log(result);`;
      } else if (currentSnippetType === "laravel") {
        code = `$response = Http::withHeaders([\n    'Authorization' => 'Bearer ${token}',\n    'X-Router-Host' => '${host}',\n    'X-Router-Port' => '${port}',\n    'X-Router-User' => '${user}',\n    'X-Router-Pass' => '${pass}',\n])->${currentMethod.toLowerCase()}("${url}"${currentMethod === 'POST' ? ', ' + currentEndpoint.defaultPayload : ''});\n\n$data = $response->json();`;
      } else if (currentSnippetType === "flutter") {
        code = `final dio = Dio();\nfinal response = await dio.${currentMethod.toLowerCase()}(\n  "${url}",\n  options: Options(headers: {\n    "Authorization": "Bearer ${token}",\n    "X-Router-Host": "${host}",\n    "X-Router-Port": "${port}",\n    "X-Router-User": "${user}",\n    "X-Router-Pass": "${pass}",\n  }),\n  ${currentMethod === 'POST' ? 'data: ' + currentEndpoint.defaultPayload + ',' : ''}\n);\nprint(response.data);`;
      }
      document.getElementById('snippet-box').textContent = code;
    }

    async function executeCurrentEndpoint() {
      const host = document.getElementById('target-host').value.trim();
      const port = document.getElementById('target-port').value.trim();
      const user = document.getElementById('target-user').value.trim();
      const pass = document.getElementById('target-pass').value;
      const token = document.getElementById('gw-token').value.trim();
      const payloadStr = document.getElementById('req-payload').value.trim();

      const viewer = document.getElementById('resp-viewer');
      const statusBadge = document.getElementById('resp-status');
      const latencyBadge = document.getElementById('resp-latency');

      viewer.textContent = `Mengirim ${currentMethod} ke ${currentEndpoint.path}...`;
      const t0 = performance.now();

      try {
        const headers = {
          'Authorization': 'Bearer ' + token,
          'X-Router-Host': host,
          'X-Router-Port': port,
          'X-Router-User': user,
          'X-Router-Pass': pass,
          'Content-Type': 'application/json'
        };

        const opts = { method: currentMethod, headers };
        if (currentMethod === 'POST' && payloadStr) {
          opts.body = payloadStr;
        }

        const res = await fetch(currentEndpoint.path, opts);
        const t1 = performance.now();
        const latency = (t1 - t0).toFixed(1);

        statusBadge.textContent = `${res.status} ${res.statusText || 'OK'}`;
        statusBadge.style.background = res.ok ? '#ecfdf5' : '#fef2f2';
        statusBadge.style.color = res.ok ? '#047857' : '#dc2626';
        statusBadge.style.borderColor = res.ok ? '#a7f3d0' : '#fecaca';
        latencyBadge.textContent = `Latency: ${latency} ms`;

        const json = await res.json();
        viewer.textContent = JSON.stringify(json, null, 2);
      } catch (err) {
        statusBadge.textContent = 'Error';
        statusBadge.style.background = '#fef2f2';
        statusBadge.style.color = '#dc2626';
        viewer.textContent = `Gagal mengirim request: ${err.message}`;
      }
    }

    async function testConnectionAndSnapshot() {
      executeCurrentEndpoint();
      // Fetch overview to update KPI cards
      const host = document.getElementById('target-host').value.trim();
      const port = document.getElementById('target-port').value.trim();
      const user = document.getElementById('target-user').value.trim();
      const pass = document.getElementById('target-pass').value;
      const token = document.getElementById('gw-token').value.trim();

      try {
        const res = await fetch('/api/v1/overview', {
          method: 'GET',
          headers: {
            'Authorization': 'Bearer ' + token,
            'X-Router-Host': host,
            'X-Router-Port': port,
            'X-Router-User': user,
            'X-Router-Pass': pass
          }
        });
        const json = await res.json();
        if (json.success && json.data) {
          const d = json.data;
          if (d.identity) document.getElementById('kpi-identity').textContent = d.identity;
          if (d.system) {
            document.getElementById('kpi-cpu').textContent = (d.system.cpu_load_percent || 0) + '%';
            document.getElementById('kpi-cpu-bar').style.width = (d.system.cpu_load_percent || 0) + '%';
            document.getElementById('kpi-free-ram').textContent = (d.system.free_memory_mb || 0) + ' MB';
            document.getElementById('kpi-total-ram').textContent = (d.system.total_memory_mb || 128) + ' MB';
            document.getElementById('kpi-board').textContent = d.system.board_name || 'RB951Ui-2HnD';
            document.getElementById('kpi-ros').textContent = `RouterOS ${d.system.version || ''} | Uptime: ${d.system.uptime || ''}`;
          }
          if (d.hotspot_active_count !== undefined || d.dhcp_lease_count !== undefined) {
            const hs = d.hotspot_active_count || 0;
            const dhcp = d.dhcp_lease_count || 0;
            const ppp = d.ppp_active_count || 0;
            document.getElementById('kpi-clients').textContent = `${hs + dhcp + ppp} Perangkat`;
            document.getElementById('kpi-clients-sub').textContent = `DHCP: ${dhcp} | Hotspot: ${hs} | PPPoE: ${ppp}`;
          }
        }
      } catch (e) {
        console.warn('Overview snapshot failed:', e);
      }
    }

    function copyResponse() {
      const text = document.getElementById('resp-viewer').textContent;
      navigator.clipboard.writeText(text).then(() => {
        alert("Respons JSON berhasil disalin ke clipboard!");
      });
    }

    window.addEventListener('DOMContentLoaded', () => {
      initCreds();
      populateDropdown('all');
      setTimeout(testConnectionAndSnapshot, 400);
    });
  </script>
</body>
</html>"##)
}

/// GET /docs - Dedicated Enterprise API Documentation & Schema Explorer (Light Theme)
pub async fn docs_page() -> Html<&'static str> {
    Html(r##"<!DOCTYPE html>
<html lang="id">
<head>
  <meta charset="UTF-8">
  <title>MikroTik Universal Gateway - Dokumentasi &amp; Spesifikasi API Lengkap</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600;700&display=swap" rel="stylesheet">
  <style>
    :root {
      --bg: #f8fafc;
      --card-bg: #ffffff;
      --card-border: #e2e8f0;
      --text-main: #0f172a;
      --text-muted: #64748b;
      --text-sub: #334155;
      --accent-indigo: #4f46e5;
      --accent-blue: #0284c7;
      --accent-emerald: #059669;
      --accent-amber: #d97706;
      --accent-rose: #e11d48;
      --code-bg: #f1f5f9;
      --code-border: #e2e8f0;
      --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
    }
    * { box-sizing: border-box; margin: 0; padding: 0; font-family: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
    body {
      background: var(--bg);
      color: var(--text-main);
      min-height: 100vh;
      display: flex;
      flex-direction: column;
    }
    code, pre { font-family: 'JetBrains Mono', monospace; }

    /* Sticky Clean Header */
    header {
      position: sticky;
      top: 0;
      z-index: 50;
      background: #ffffff;
      border-bottom: 1px solid var(--card-border);
      padding: 12px 28px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 16px;
      box-shadow: var(--shadow-sm);
    }
    .brand-group {
      display: flex;
      align-items: center;
      gap: 12px;
      text-decoration: none;
      color: inherit;
    }
    .brand-title {
      font-weight: 800;
      font-size: 1.15rem;
      letter-spacing: -0.3px;
      color: var(--text-main);
    }
    .badge-endpoints {
      background: #eff6ff;
      border: 1px solid #bfdbfe;
      color: #1d4ed8;
      font-size: 0.72rem;
      font-weight: 700;
      padding: 3px 10px;
      border-radius: 999px;
    }
    .nav-links {
      display: flex;
      gap: 8px;
    }
    .nav-btn {
      background: #ffffff;
      border: 1px solid var(--card-border);
      color: var(--text-muted);
      padding: 6px 14px;
      border-radius: 7px;
      font-size: 0.8rem;
      font-weight: 600;
      text-decoration: none;
      display: inline-flex;
      align-items: center;
      gap: 6px;
      transition: all 0.15s;
    }
    .nav-btn:hover { background: #f1f5f9; color: var(--text-main); }
    .nav-btn.active {
      background: var(--accent-indigo);
      color: white;
      border-color: var(--accent-indigo);
    }

    /* 2-Column Documentation Layout */
    .docs-container {
      display: grid;
      grid-template-columns: 280px 1fr;
      flex: 1;
      max-width: 1650px;
      width: 100%;
      margin: 0 auto;
    }
    @media (max-width: 960px) {
      .docs-container { grid-template-columns: 1fr; }
    }

    /* Left Sidebar: Categories Navigation */
    aside.sidebar {
      background: #ffffff;
      border-right: 1px solid var(--card-border);
      padding: 20px 16px;
      position: sticky;
      top: 57px;
      height: calc(100vh - 57px);
      overflow-y: auto;
      display: flex;
      flex-direction: column;
      gap: 16px;
    }
    .search-box {
      width: 100%;
      padding: 8px 12px;
      border: 1px solid var(--card-border);
      border-radius: 6px;
      font-size: 0.8rem;
      outline: none;
      background: #f8fafc;
    }
    .search-box:focus { border-color: var(--accent-indigo); background: #ffffff; }

    .cat-menu-title {
      font-size: 0.72rem;
      font-weight: 700;
      color: var(--text-muted);
      text-transform: uppercase;
      letter-spacing: 0.5px;
      margin-bottom: 6px;
    }
    .cat-link {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 7px 10px;
      border-radius: 6px;
      font-size: 0.8rem;
      font-weight: 600;
      color: var(--text-sub);
      text-decoration: none;
      transition: all 0.15s;
    }
    .cat-link:hover { background: #f1f5f9; color: var(--text-main); }
    .cat-count {
      font-size: 0.7rem;
      background: #e2e8f0;
      padding: 1px 6px;
      border-radius: 999px;
      color: var(--text-muted);
    }

    /* Main Content Area */
    main.content {
      padding: 28px 36px;
      display: flex;
      flex-direction: column;
      gap: 24px;
    }

    /* Hero Banner */
    .hero-banner {
      background: #ffffff;
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 24px;
      box-shadow: var(--shadow-sm);
    }
    .hero-title {
      font-size: 1.45rem;
      font-weight: 800;
      color: var(--text-main);
      margin-bottom: 8px;
    }
    .hero-desc {
      font-size: 0.88rem;
      color: var(--text-muted);
      line-height: 1.5;
    }

    /* Dual Method Callout */
    .dual-callout {
      background: #eff6ff;
      border: 1px solid #bfdbfe;
      border-left: 4px solid var(--accent-indigo);
      border-radius: 8px;
      padding: 14px 18px;
      font-size: 0.82rem;
      color: #1e3a8a;
      line-height: 1.5;
    }

    /* Section & Cards */
    .doc-section {
      display: flex;
      flex-direction: column;
      gap: 16px;
    }
    .section-header {
      font-size: 1.15rem;
      font-weight: 800;
      color: var(--text-main);
      padding-bottom: 8px;
      border-bottom: 2px solid var(--card-border);
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .ep-doc-card {
      background: #ffffff;
      border: 1px solid var(--card-border);
      border-radius: 10px;
      padding: 18px 20px;
      box-shadow: var(--shadow-sm);
      display: flex;
      flex-direction: column;
      gap: 12px;
    }
    .ep-top {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 10px;
      flex-wrap: wrap;
    }
    .methods-group {
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .m-pill {
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.72rem;
      font-weight: 800;
      padding: 3px 8px;
      border-radius: 5px;
    }
    .m-get { background: #ecfdf5; border: 1px solid #a7f3d0; color: #047857; }
    .m-post { background: #eff6ff; border: 1px solid #bfdbfe; color: #1d4ed8; }
    .ep-path {
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.86rem;
      font-weight: 700;
      color: var(--text-main);
    }
    .ep-desc {
      font-size: 0.82rem;
      color: var(--text-sub);
      line-height: 1.4;
    }

    /* Param Table */
    table.params-table {
      width: 100%;
      border-collapse: collapse;
      font-size: 0.78rem;
      margin-top: 6px;
    }
    table.params-table th, table.params-table td {
      text-align: left;
      padding: 6px 10px;
      border-bottom: 1px solid var(--card-border);
    }
    table.params-table th {
      background: #f8fafc;
      color: var(--text-muted);
      font-weight: 700;
    }

    .code-preview {
      background: var(--code-bg);
      border: 1px solid var(--code-border);
      border-radius: 6px;
      padding: 10px 12px;
      font-size: 0.76rem;
      overflow-x: auto;
      color: #0f172a;
    }
  </style>
</head>
<body>

  <!-- Top Clean Header -->
  <header>
    <a href="/" class="brand-group">
      <span style="font-size: 1.35rem;">🦀</span>
      <div class="brand-title">MikroTik Gateway API Reference</div>
      <span class="badge-endpoints">155+ Endpoints Terdaftar &amp; Aktif</span>
    </a>

    <div class="nav-links">
      <a href="/" class="nav-btn">🎮 Live Console &amp; Playground</a>
      <a href="/topology" class="nav-btn">🗺️ Visualizer Topologi</a>
      <a href="/docs" class="nav-btn active">📖 API Documentation</a>
    </div>
  </header>

  <div class="docs-container">
    <!-- Left Navigation Sidebar -->
    <aside class="sidebar">
      <input type="text" id="doc-search" class="search-box" placeholder="Cari endpoint (mis: pppoe, hotspot, arp)..." oninput="filterDocCards()">

      <div>
        <div class="cat-menu-title">Kategori Endpoint</div>
        <a href="#sec-notify" class="cat-link"><span>💬 Bot &amp; Netwatch Pro</span><span class="cat-count">9</span></a>
        <a href="#sec-vouchers" class="cat-link"><span>🎟️ Hotspot Voucher 2.0</span><span class="cat-count">5</span></a>
        <a href="#sec-fast" class="cat-link"><span>🚀 Fast-Path Snapshot</span><span class="cat-count">3</span></a>
        <a href="#sec-net" class="cat-link"><span>🌐 Relasi Topologi</span><span class="cat-count">3</span></a>
        <a href="#sec-sys" class="cat-link"><span>⚙️ Sistem &amp; Board</span><span class="cat-count">13</span></a>
        <a href="#sec-dhcp" class="cat-link"><span>💻 DHCP Server &amp; Leases</span><span class="cat-count">6</span></a>
        <a href="#sec-hotspot" class="cat-link"><span>🎟️ Hotspot &amp; Voucher</span><span class="cat-count">15</span></a>
        <a href="#sec-pppoe" class="cat-link"><span>🌐 PPPoE &amp; ISP Access</span><span class="cat-count">11</span></a>
        <a href="#sec-ip" class="cat-link"><span>📡 IP &amp; Routing</span><span class="cat-count">12</span></a>
        <a href="#sec-iface" class="cat-link"><span>🔌 Interface &amp; Trafik</span><span class="cat-count">5</span></a>
        <a href="#sec-wifi" class="cat-link"><span>📶 WiFi Wireless</span><span class="cat-count">4</span></a>
        <a href="#sec-fw" class="cat-link"><span>🛡️ Firewall &amp; Keamanan</span><span class="cat-count">14</span></a>
        <a href="#sec-queues" class="cat-link"><span>📊 Queues &amp; Bandwidth</span><span class="cat-count">8</span></a>
        <a href="#sec-tools" class="cat-link"><span>🛠️ NOC Diagnostics</span><span class="cat-count">12</span></a>
        <a href="#sec-scripts" class="cat-link"><span>📝 Scripts &amp; Cron</span><span class="cat-count">5</span></a>
        <a href="#sec-raw" class="cat-link"><span>⚡ Raw Socket &amp; Batch</span><span class="cat-count">3</span></a>
      </div>
    </aside>

    <!-- Main Content Area -->
    <main class="content">
      <div class="hero-banner">
        <div class="hero-title">Integrasi API MikroTik Universal Gateway</div>
        <div class="hero-desc">
          Dokumentasi teknis resmi untuk developer backend (Laravel, Node.js, Golang, Python) dan mobile (Flutter, React Native).
          Gateway ini menghubungkan aplikasi Anda langsung ke socket RouterOS menggunakan koneksi TCP multiplexed berkecepatan tinggi (&lt;2ms).
        </div>
      </div>

      <div class="dual-callout">
        <strong>💡 Dukungan Fleksibel: HTTP GET dan POST</strong><br>
        Seluruh endpoint pengambilan data (Read-Only) dapat dipanggil menggunakan <strong>HTTP GET</strong> (mudah digunakan langsung dari browser atau cURL dengan query parameters) maupun <strong>HTTP POST</strong> (dengan JSON payload untuk menjaga keamanan kredensial).
      </div>

      <!-- Section: Telegram, WhatsApp Gowa & Netwatch Pro -->
      <section class="doc-section" id="sec-notify">
        <div class="section-header">💬 Integrasi Multi-Platform Telegram, WhatsApp (Gowa) &amp; Netwatch Pro</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/integrations/telegram/send</div>
            <a href="/?endpoint=integ_tg_send" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Kirim notifikasi atau broadcast alert langsung ke Bot Telegram via RouterOS socket fetch atau gateway dispatcher.</div>
          <div class="code-preview">POST /api/v1/integrations/telegram/send
{ "bot_token": "BOT_TOKEN", "chat_id": "CHAT_ID", "text": "🚨 *ALERT NOC*: Link Internet DOWN!" }</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/integrations/whatsapp/send</div>
            <a href="/?endpoint=integ_wa_send" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Kirim pesan WhatsApp otomatis ke pelanggan atau tim NOC menggunakan service Gowa WhatsApp API. Mendukung format Markdown WhatsApp (*tebal*, _miring_).</div>
          <div class="code-preview">POST /api/v1/integrations/whatsapp/send
{ "phone": "081234567890", "message": "Voucher: *HOT-8812* Uptime: 1 Hari", "gowa_url": "http://127.0.0.1:3000" }</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/integrations/notify</div>
            <a href="/?endpoint=integ_multi_notify" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Multi-channel dispatcher: kirim alert ke Telegram dan WhatsApp sekaligus dengan tingkatan severity level (INFO, WARNING, CRITICAL).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/integrations/netwatch/setup</div>
            <a href="/?endpoint=integ_netwatch_setup" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Setup otomatis probe Netwatch di MikroTik dengan skrip Up/Down yang mengeksekusi alert instan ke Telegram dan WhatsApp secara mandiri dari router.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/integrations/netwatch/batch-setup</div>
            <a href="/?endpoint=integ_netwatch_batch" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Otomatisasi pengawasan Netwatch armada Access Point (AP Fleet) sekaligus (misal FAUJIA HOTSPOT 1..4 pada IP 172.16.10.2..5) hanya dalam satu panggilan API.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/integrations/netwatch/list</div>
            <a href="/?endpoint=integ_netwatch_list" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Inspeksi daftar probe Netwatch terpasang di router, status UP / DOWN, durasi uptime, interval, dan skrip alert.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/integrations/reports/scheduler</div>
            <a href="/?endpoint=integ_reports_sched" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Pasang skrip &amp; scheduler otomatis RouterOS untuk rekap laporan harian NOC (Active Users Hotspot, PPPoE, CPU/RAM, Uptime) terkirim otomatis ke Telegram.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/integrations/hotspot-chat/snippet</div>
            <a href="/?endpoint=integ_chat_snippet" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Generator widget floating Live Chat (Intergram Telegram &amp; WhatsApp CS) siap pakai untuk disisipkan langsung ke halaman login.html &amp; status.html hotspot MikroTik.</div>
        </div>
      </section>

      <!-- Section: Hotspot Voucher Engine 2.0 -->
      <section class="doc-section" id="sec-vouchers">
        <div class="section-header">🎟️ Next-Gen Hotspot Voucher Engine 2.0 (Mikhmon Killer)</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/vouchers/generate</div>
            <a href="/?endpoint=vouchers_gen" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Batch generator multi-threaded berkecepatan tinggi: buat 1 hingga 1.000 voucher dalam &lt;1 detik. Kompatibel penuh dengan RouterOS v6 dan v7 tanpa bug NTP/reboot desync. Dilengkapi structured metadata comment untuk tracking status &amp; harga jual.</div>
          <div class="code-preview">POST /api/v1/hotspot/vouchers/generate
{ "profile": "1JAM-3RB", "count": 20, "prefix": "HOT-", "user_mode": "same", "timelimit": "1h", "price": 3000 }</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/vouchers/tracking</div>
            <a href="/?endpoint=vouchers_track" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Sistem tracking presisi real-time: cross-korelasi database user hotspot dengan session login aktif. Klasifikasi status akurat: available (belum dipakai), online (sedang login), used (pernah dipakai), atau expired (kedaluwarsa), lengkap dengan sisa detik dan sisa kuota bytes.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/vouchers/thermal-print</div>
            <a href="/?endpoint=vouchers_thermal" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Formatter struk thermal printer ESC/POS (standar 58mm dan 80mm) siap dikirim langsung ke printer kasir Bluetooth atau USB POS.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/vouchers/sell-and-send</div>
            <a href="/?endpoint=vouchers_sell_send" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Point of Sale (POS) Kasir: tandai voucher terjual dan kirim bukti struk login otomatis beserta link quick-login ke nomor WhatsApp pelanggan via Gowa.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/vouchers/clean-expired</div>
            <a href="/?endpoint=vouchers_clean" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Pembersihan aman voucher kedaluwarsa secara berkala tanpa menyentuh voucher aktif atau voucher yang belum laku terjual.</div>
        </div>
      </section>

      <!-- Section: Fast Path -->
      <section class="doc-section" id="sec-fast">
        <div class="section-header">🚀 1. Fast-Path Snapshot &amp; Aggregator</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/overview</div>
            <a href="/?endpoint=overview" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Snapshot agregasi tercepat: mengambil CPU, memory, uptime, total user hotspot, pppoe, dan dhcp leases dalam 1 request simultan (&lt;2ms).</div>
          <pre class="code-preview">curl -X GET "https://ros-gateway.samrifa.com/api/v1/overview" \
  -H "Authorization: Bearer change-me-to-a-long-random-string" \
  -H "X-Router-Host: ath.vpnbersama.us" \
  -H "X-Router-Port: 51121" \
  -H "X-Router-User: salman" \
  -H "X-Router-Pass: yourpassword"</pre>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/network/connected-devices</div>
            <a href="/?endpoint=connected_devices" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Korelasikan seluruh perangkat terhubung lintas layer (DHCP Leases, ARP table, Hotspot Active, Hotspot Hosts, WiFi registration table). Menghasilkan relasi IP, MAC, hostname, sinyal, dan status otorisasi.</div>
        </div>
      </section>

      <!-- Section: Topology -->
      <section class="doc-section" id="sec-net">
        <div class="section-header">🌐 2. Relasi Topologi Jaringan &amp; Cabang Port</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/network/topology-graph</div>
            <a href="/topology" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Buka Canvas Visualizer</a>
          </div>
          <div class="ep-desc">Menghasilkan simpul (nodes) dan cabang (edges) terstruktur secara hierarkis: Core Router &rarr; Interface Fisik (ether1 - ether5) &rarr; Access Point &rarr; Klien Terhubung. Dilengkapi deteksi vendor otomatis (Xiaomi, Samsung, OPPO, PC, Ubiquiti, TP-Link).</div>
          <table class="params-table">
            <tr><th>Query Param</th><th>Tipe</th><th>Nilai Opsi</th><th>Keterangan</th></tr>
            <tr><td><code>filter</code></td><td>string</td><td>all | hotspot | pppoe | dhcp | wifi</td><td>Filter relasi cabang spesifik</td></tr>
          </table>
        </div>
      </section>

      <!-- Section: System -->
      <section class="doc-section" id="sec-sys">
        <div class="section-header">⚙️ 3. Sistem &amp; Hardware Management</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/system/resource</div>
          </div>
          <div class="ep-desc">Detail status performa perangkat: CPU load %, free memory, total memory, free HDD space, versi RouterOS, dan platform arsitektur.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/system/clock</div>
          </div>
          <div class="ep-desc">Status jam internal router, tanggal, timezone, dan sinkronisasi GMT offset.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/system/reboot</div>
          </div>
          <div class="ep-desc">Dispatach perintah reboot aman ke router MikroTik.</div>
        </div>
      </section>

      <!-- Section: DHCP -->
      <section class="doc-section" id="sec-dhcp">
        <div class="section-header">💻 4. DHCP Server &amp; Leases Management</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dhcp/leases</div>
          </div>
          <div class="ep-desc">Daftar seluruh IP lease DHCP server, hostname klien, status bound, MAC address, dan interface server.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dhcp/lease/make-static</div>
          </div>
          <div class="ep-desc">Ubah dynamic lease menjadi static IP reservation berdasarkan ID.</div>
        </div>
      </section>

      <!-- Section: Hotspot -->
      <section class="doc-section" id="sec-hotspot">
        <div class="section-header">🎟️ 5. Hotspot &amp; Voucher Engine</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/active</div>
          </div>
          <div class="ep-desc">Daftar klien yang sedang login aktif di Hotspot captive portal, uptime, byte masuk dan keluar.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/hosts</div>
          </div>
          <div class="ep-desc">Daftar seluruh host yang terdeteksi di jaringan Hotspot (termasuk yang belum login, bypassed, atau unauthorized).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/generate-batch</div>
          </div>
          <div class="ep-desc">Generate ratusan voucher Hotspot unik secara massal dengan profil limit dan masa aktif tertentu.</div>
        </div>
      </section>

      <!-- Section: PPPoE -->
      <section class="doc-section" id="sec-pppoe">
        <div class="section-header">🌐 6. PPPoE &amp; ISP Access Management</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ppp/secrets</div>
          </div>
          <div class="ep-desc">Daftar pelanggan PPPoE, username, password terenkripsi, profil paket, dan remote IP.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ppp/active</div>
          </div>
          <div class="ep-desc">Daftar sesi PPPoE yang sedang terhubung aktif (tunnel status UP) dan durasi online.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ppp/customer/isolate</div>
          </div>
          <div class="ep-desc">Isolir otomatis pelanggan yang menunggak dengan memindahkannya ke profil ISOLIR dan memutuskan sesi aktif.</div>
        </div>
      </section>

      <!-- Section: Tools -->
      <section class="doc-section" id="sec-tools">
        <div class="section-header">🛠️ 7. NOC Pro Diagnostics &amp; Tools</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/tools/ping</div>
          </div>
          <div class="ep-desc">Kirim paket ICMP Ping dari router ke target IP dengan statistik min, avg, max RTT dan packet loss.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span></div>
            <div class="ep-path">/metrics</div>
          </div>
          <div class="ep-desc">Native Prometheus Exporter (text/plain v0.0.4) untuk scraping langsung dari Grafana / Prometheus / VictoriaMetrics. Zero SNMP overhead!</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/doctor/diagnose</div>
          </div>
          <div class="ep-desc">Heuristic Doctor Network Diagnostic Assistant: audit otomatis 6 pilar (CPU, RAM, WAN Ping, Throttled Queues, FastTrack, dan AP LAN) dengan skor kesehatan &amp; rekomendasi perbaikan.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/network/infrastructure/scan</div>
          </div>
          <div class="ep-desc">Mendeteksi perangkat Access Point (TP-Link, Ubiquiti, Ruijie, Tenda), Switch, dan Kamera di balik port/bridge router, lengkap dengan link Web Management dan status bypass captive portal.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/network/infrastructure/auto-bypass-ap</div>
          </div>
          <div class="ep-desc">Otomatis mendaftarkan MAC Access Point ke Hotspot IP-Binding (bypassed) agar admin dapat membuka Web Admin AP tanpa login voucher.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/routing/bgp/sessions</div>
          </div>
          <div class="ep-desc">Inspeksi sesi peering BGP Carrier (Established / Active / Idle, AS number, prefix count, uptime) dengan kompatibilitas RouterOS v6 dan v7.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/routing/ospf/neighbors</div>
          </div>
          <div class="ep-desc">Monitoring OSPF Neighbor Adjacency (Full, 2-Way, Init, Designated Router, Interface link).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/routing/routes</div>
          </div>
          <div class="ep-desc">Ringkasan tabel routing aktif dengan rincian protokol (Connected, Static, BGP, OSPF).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/tools/multi-ping</div>
          </div>
          <div class="ep-desc">Ping matriks multi-target sekaligus (Gateway, DNS Cloudflare 1.1.1.1, Google 8.8.8.8, OpenDNS) dengan kalkulasi packet loss %, RTT min/avg/max, jitter, dan status SLA global.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dns/adlist</div>
          </div>
          <div class="ep-desc">RouterOS v7 native AdList DNS ad-blocker & malware prevention table (Pi-hole tanpa hardware eksternal).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span></div>
            <div class="ep-path">/api/v1/dns/adlist/presets</div>
          </div>
          <div class="ep-desc">Katalog preset blocklist resmi aman (HaGeZi Multi Light, StevenBlack Hosts, Threat Intelligence TIF, AdGuard DNS).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dns/adlist/deploy-preset</div>
          </div>
          <div class="ep-desc">Deploy satu klik preset blocklist pilihan ke router MikroTik dengan ssl-verify bypass otomatis.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/firewall/connections</div>
          </div>
          <div class="ep-desc">Inspeksi tabel Connection Tracking (Conntrack) aktif secara realtime lengkap dengan filter protokol, IP asal, IP tujuan, dan status TCP.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/firewall/connections/top-talkers</div>
          </div>
          <div class="ep-desc">Analisis Top Talkers & Conntrack Hogs: deteksi otomatis 15 IP teratas yang menghabiskan kuota tabel sesi (BitTorrent, DDoS botnet, malware).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/firewall/connections/flush</div>
          </div>
          <div class="ep-desc">Flush / putus paksa sesi koneksi dari IP tertentu atau seluruh sesi conntrack untuk mitigasi darurat DDoS.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dhcp/alerts</div>
          </div>
          <div class="ep-desc">Monitoring deteksi Rogue DHCP Server ilegal di port LAN (mencegah tabrakan IP router rumah tetangga yang terbalik colok).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dhcp/alert/add</div>
          </div>
          <div class="ep-desc">Pasang alarm pengawas Rogue DHCP Server pada interface tertentu dengan opsi valid-server dan timeout.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/capsman/radios</div>
          </div>
          <div class="ep-desc">Daftar radio Access Point MikroTik (CAPs) yang terhubung ke controller terpusat.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/capsman/set-wifi</div>
          </div>
          <div class="ep-desc">Konfigurasi SSID & Password WiFi terpusat yang otomatis disebarkan ke seluruh Access Point MikroTik yang terhubung.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/capsman/registrations</div>
          </div>
          <div class="ep-desc">Monitoring seluruh klien wireless (HP/laptop) yang tersambung di semua Access Point, kekuatan sinyal dBm, dan rate data.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/backup/restore</div>
          </div>
          <div class="ep-desc">Pulihkan sistem router dari file binary (.backup) dan reboot otomatis menerapkan konfigurasi.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/backup/import</div>
          </div>
          <div class="ep-desc">Impor dan eksekusi skrip konfigurasi RouterOS (.rsc) langsung ke dalam database router.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/network/infrastructure/poe-cycle</div>
          </div>
          <div class="ep-desc">Reboot paksa Access Point (TP-Link, Ubiquiti, Ruijie) yang hang di tiang/tower lewat pemutusan daya PoE port sementara.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/network/infrastructure/ap-tunnel</div>
          </div>
          <div class="ep-desc">Buka tunnel port forwarding sementara agar administrator dapat mengakses Web GUI Access Point pihak ketiga dari luar LAN.</div>
        </div>
      </section>
    </main>
  </div>

  <script>
    function filterDocCards() {
      const q = document.getElementById('doc-search').value.toLowerCase().trim();
      const cards = document.querySelectorAll('.ep-doc-card');
      cards.forEach(c => {
        const text = c.textContent.toLowerCase();
        c.style.display = text.includes(q) ? 'flex' : 'none';
      });
    }
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
            "version": "0.3.0",
            "description": "High-performance sub-millisecond MikroTik management gateway with 155+ endpoints"
        },
        "servers": [
            { "url": "https://ros-gateway.samrifa.com" },
            { "url": "http://127.0.0.1:8080" }
        ],
        "security": [
            { "BearerAuth": [] }
        ]
    }))
}

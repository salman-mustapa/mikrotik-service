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
      <span class="badge-endpoints">280+ Enterprise Endpoints</span>
      <span class="badge-engine"><span class="dot-live"></span>Rust Socket Engine: &lt;2ms</span>
    </div>

    <div class="nav-links">
      <a href="/" class="nav-btn active">🎮 Console &amp; Playground</a>
      <a href="/topology" class="nav-btn nav-btn-highlight">🗺️ Visualizer Topologi</a>
      <a href="/docs" class="nav-btn">📖 API Docs &amp; Schema</a>
    </div>
  </header>

  <!-- Router Credentials & Profile Switcher Bar -->
  <div class="creds-container">
    <div class="creds-inputs">
      <div class="cred-group">
        <span class="cred-label">Profil:</span>
        <select id="profile-select" class="cred-field" style="width: 135px; font-weight: 600; cursor: pointer;" onchange="onProfileChange()">
          <option value="custom">-- Custom Host --</option>
        </select>
        <button class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem; cursor: pointer;" onclick="saveCurrentProfile()" title="Simpan router ini ke daftar profil">+ Simpan</button>
      </div>
      <div class="cred-group">
        <span class="cred-label">Router Host:</span>
        <input type="text" id="target-host" class="cred-field" style="width: 130px;" value="" placeholder="192.168.88.1">
      </div>
      <div class="cred-group">
        <span class="cred-label">API Port:</span>
        <input type="number" id="target-port" class="cred-field" style="width: 55px;" value="" placeholder="8728">
      </div>
      <div class="cred-group">
        <span class="cred-label">User:</span>
        <input type="text" id="target-user" class="cred-field" style="width: 75px;" value="" placeholder="admin">
      </div>
      <div class="cred-group">
        <span class="cred-label">Password:</span>
        <input type="password" id="target-pass" class="cred-field" style="width: 85px;" placeholder="••••••">
      </div>
      <div class="cred-group">
        <span class="cred-label">Bearer Token:</span>
        <input type="text" id="gw-token" class="cred-field" style="width: 135px;" value="change-me-to-a-long-random-string">
      </div>
    </div>
    <button class="btn-sync" onclick="testConnectionAndSnapshot()">⚡ Sinkronisasi &amp; Snapshot</button>
  </div>

  <main>
    <!-- KPI Overview Cards Grid -->
    <div class="kpi-grid">
      <div class="kpi-card">
        <div class="kpi-title">Identitas Router <span id="kpi-board" style="color: var(--accent-indigo);">-</span></div>
        <div class="kpi-val" id="kpi-identity">Belum Terhubung</div>
        <div class="kpi-sub" id="kpi-ros">Masukkan host &amp; klik Sinkronisasi</div>
      </div>

      <div class="kpi-card">
        <div class="kpi-title">CPU Load &amp; Frekuensi <span id="kpi-freq">- MHz</span></div>
        <div class="kpi-val" id="kpi-cpu">0%</div>
        <div class="progress-bar-wrap">
          <div class="progress-fill" id="kpi-cpu-bar" style="width: 0%;"></div>
        </div>
      </div>

      <div class="kpi-card">
        <div class="kpi-title">Memori RAM Bebas <span id="kpi-total-ram">- MB</span></div>
        <div class="kpi-val" id="kpi-free-ram">- MB</div>
        <div class="kpi-sub" id="kpi-mem-percent">Penggunaan RAM: -%</div>
      </div>

      <div class="kpi-card">
        <div class="kpi-title">Klien &amp; Relasi Jaringan <span id="kpi-clients-status" style="color: var(--text-muted);">● Menunggu Data</span></div>
        <div class="kpi-val" id="kpi-clients">0 Perangkat</div>
        <div class="kpi-sub" id="kpi-clients-sub">DHCP: 0 | Hotspot: 0 | PPPoE: 0</div>
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
          <button class="cat-btn active" onclick="filterCategory('all', this)">Semua (110+)</button>
          <button class="cat-btn" onclick="filterCategory('security', this)">🛡️ Keamanan &amp; Dokter</button>
          <button class="cat-btn" onclick="filterCategory('voucher', this)">🎟️ Voucher 2.0</button>
          <button class="cat-btn" onclick="filterCategory('notify', this)">💬 Bot &amp; Netwatch</button>
          <button class="cat-btn" onclick="filterCategory('loadbalance', this)">🔀 Load Balance &amp; NAT</button>
          <button class="cat-btn" onclick="filterCategory('vpn', this)">🔐 VPN &amp; WireGuard</button>
          <button class="cat-btn" onclick="filterCategory('queues', this)">📊 QoS &amp; Queues</button>
          <button class="cat-btn" onclick="filterCategory('tools', this)">🛠️ NOC Tools</button>
          <button class="cat-btn" onclick="filterCategory('maintenance', this)">💾 Backup &amp; Files</button>
          <button class="cat-btn" onclick="filterCategory('hotspot', this)">🔥 Hotspot</button>
          <button class="cat-btn" onclick="filterCategory('dhcp', this)">💻 DHCP</button>
          <button class="cat-btn" onclick="filterCategory('pppoe', this)">🌐 PPPoE</button>
          <button class="cat-btn" onclick="filterCategory('ip', this)">📡 IP &amp; Routing</button>
          <button class="cat-btn" onclick="filterCategory('system', this)">⚙️ Sistem</button>
          <button class="cat-btn" onclick="filterCategory('fast', this)">🚀 Fast-Path</button>
          <button class="cat-btn" onclick="filterCategory('net', this)">🌐 Topologi</button>
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
      { id: "integ_netwatch_setup", path: "/api/v1/integrations/netwatch/setup", method: "POST", cat: "notify", desc: "Setup probe Netwatch RouterOS otomatis dengan skrip alert instan ke Telegram & WhatsApp.", defaultPayload: '{\n  "host": "192.168.88.2",\n  "comment": "AP-HOTSPOT-1",\n  "interval": "00:00:10",\n  "timeout": "1000ms",\n  "telegram_bot_token": "BOT_TOKEN",\n  "telegram_chat_id": "CHAT_ID"\n}' },
      { id: "integ_netwatch_batch", path: "/api/v1/integrations/netwatch/batch-setup", method: "POST", cat: "notify", desc: "Otomatisasi pengawasan Netwatch armada seluruh Access Point (AP Fleet) sekaligus.", defaultPayload: '{\n  "items": [\n    { "host": "192.168.88.2", "comment": "AP-HOTSPOT-1" },\n    { "host": "192.168.88.3", "comment": "AP-HOTSPOT-2" },\n    { "host": "192.168.88.4", "comment": "AP-HOTSPOT-3" },\n    { "host": "192.168.88.5", "comment": "AP-HOTSPOT-4" }\n  ],\n  "telegram_bot_token": "BOT_TOKEN",\n  "telegram_chat_id": "CHAT_ID"\n}' },
      { id: "integ_netwatch_list", path: "/api/v1/integrations/netwatch/list", method: "GET", cat: "notify", desc: "Daftar seluruh probe Netwatch, status up/down/since, dan skrip alert terpasang.", defaultPayload: "{}" },
      { id: "integ_netwatch_toggle", path: "/api/v1/integrations/netwatch/toggle", method: "POST", cat: "notify", desc: "Enable atau Disable probe Netwatch target pada router.", defaultPayload: '{\n  "id": "*1",\n  "disabled": false\n}' },
      { id: "integ_reports_sched", path: "/api/v1/integrations/reports/scheduler", method: "POST", cat: "notify", desc: "Pasang skrip & scheduler otomatis RouterOS untuk rekap laporan harian NOC ke Telegram.", defaultPayload: '{\n  "name": "noc-daily-report",\n  "start_time": "07:00:00",\n  "interval": "1d",\n  "telegram_bot_token": "BOT_TOKEN",\n  "telegram_chat_id": "CHAT_ID"\n}' },
      { id: "integ_chat_snippet", path: "/api/v1/integrations/hotspot-chat/snippet", method: "GET", cat: "notify", desc: "Dapatkan snippet embed live chat widget (Intergram Telegram & WhatsApp) untuk login.html & status.html hotspot.", defaultPayload: "{}" },

      // Hotspot Voucher Engine 2.0 (Mikhmon Killer)
      { id: "vouchers_gen", path: "/api/v1/hotspot/vouchers/generate", method: "POST", cat: "voucher", desc: "Ultra-fast parallel batch voucher generator (1-1000 voucher, ROS v6/v7 safe, custom prefix, time/quota limit).", defaultPayload: '{\n  "profile": "1JAM-3RB",\n  "count": 5,\n  "prefix": "HOT-",\n  "user_mode": "same",\n  "charset": "alphanumeric_lower",\n  "length": 6,\n  "timelimit": "1h",\n  "datalimit": "500M",\n  "price": 3000,\n  "selling_price": 3000\n}' },
      { id: "vouchers_track", path: "/api/v1/hotspot/vouchers/tracking", method: "GET", cat: "voucher", desc: "Tracking lifecycle & sisa kuota/waktu voucher presisi real-time (available, online, used, expired).", defaultPayload: "{}" },
      { id: "vouchers_thermal", path: "/api/v1/hotspot/vouchers/thermal-print", method: "POST", cat: "voucher", desc: "Formatter struk termal ESC/POS (58mm/80mm) siap cetak via printer Bluetooth / USB.", defaultPayload: '{\n  "vouchers": [\n    { "username": "HOT-a89f2", "password": "HOT-a89f2", "profile": "1JAM-3RB", "timelimit": "1h", "price": 3000 }\n  ],\n  "hotspot_name": "HOTSPOT ENTERPRISE",\n  "dns_name": "hotspot.local",\n  "paper_width": "58mm"\n}' },
      { id: "vouchers_sell_send", path: "/api/v1/hotspot/vouchers/sell-and-send", method: "POST", cat: "voucher", desc: "POS Kasir: Tandai voucher terjual dan kirim struk instan ke nomor WhatsApp pelanggan via Gowa API.", defaultPayload: '{\n  "username": "HOT-a89f2",\n  "phone": "081234567890",\n  "cashier": "Admin Kasir",\n  "hotspot_name": "HOTSPOT ENTERPRISE",\n  "login_url": "http://hotspot.local/login"\n}' },
      { id: "vouchers_sales_report", path: "/api/v1/hotspot/vouchers/sales-report", method: "GET", cat: "voucher", desc: "Mikhmon-Style Financial & Sales Revenue Summary: rekap omset rupiah harian & bulanan, voucher terjual, dan breakdown per paket.", defaultPayload: "{}" },
      { id: "vouchers_clean", path: "/api/v1/hotspot/vouchers/clean-expired", method: "POST", cat: "voucher", desc: "Pembersihan aman voucher kedaluwarsa tanpa menghapus voucher yang belum digunakan.", defaultPayload: '{\n  "force": true\n}' },

      // Hotspot Walled Garden & Advanced Session Management
      { id: "hs_walled_garden", path: "/api/v1/hotspot/walled-garden", method: "GET", cat: "hotspot", desc: "Daftar domain dan IP subnet yang di-bypass sebelum login (Payment Gateway, Webhook, QRIS).", defaultPayload: "{}" },
      { id: "hs_walled_garden_preset", path: "/api/v1/hotspot/walled-garden/deploy-preset", method: "POST", cat: "hotspot", desc: "1-Klik deploy preset bypass Walled Garden (Midtrans, Xendit, DANA, OVO, QRIS, WhatsApp API).", defaultPayload: '{\n  "preset": "payment_gateways"\n}' },
      { id: "hs_cookies", path: "/api/v1/hotspot/cookies", method: "GET", cat: "hotspot", desc: "Daftar active MAC/User cookie Hotspot tersimpan di router.", defaultPayload: "{}" },
      { id: "hs_reset_counters", path: "/api/v1/hotspot/user/reset-counters", method: "POST", cat: "hotspot", desc: "Reset traffic bytes dan uptime counter user hotspot tanpa menghapus voucher.", defaultPayload: '{\n  "name": "HOT-a89f2"\n}' },
      { id: "hs_inject_expiry", path: "/api/v1/hotspot/profile/inject-expiry-script", method: "POST", cat: "hotspot", desc: "Pasang script On-Login first login validity tracker otomatis pada user profile hotspot.", defaultPayload: '{\n  "profile_name": "1JAM-3RB",\n  "validity": "1d"\n}' },

      // Queues Burst Calculator & Helper
      { id: "queues_burst_calc", path: "/api/v1/queues/burst-calculator", method: "POST", cat: "tools", desc: "Kalkulator & Formatter formula Burst Rate-Limit MikroTik (Max Limit, Burst, Threshold, Time, Priority).", defaultPayload: '{\n  "max_limit": "2M/2M",\n  "burst_limit": "5M/5M",\n  "burst_time": "8/8",\n  "threshold_percent": 75\n}' },

      // User Manager (RADIUS Internal MikroTik v6 & v7)
      { id: "userman_profiles", path: "/api/v1/user-manager/profiles", method: "GET", cat: "hotspot", desc: "Daftar billing profile User Manager (RADIUS internal MikroTik).", defaultPayload: "{}" },
      { id: "userman_limitations", path: "/api/v1/user-manager/limitations", method: "GET", cat: "hotspot", desc: "Daftar limitations kecepatan & kuota paket User Manager.", defaultPayload: "{}" },
      { id: "userman_assign", path: "/api/v1/user-manager/user/assign-profile", method: "POST", cat: "hotspot", desc: "Assign billing profile/paket User Manager ke user.", defaultPayload: '{\n  "user": "user123",\n  "profile": "Paket-1Bulan"\n}' },

      // --- Keamanan, Dokter Heuristik & App Blocker (cat: security) ---
      { id: "doctor_diagnose", path: "/api/v1/doctor/diagnose", method: "POST", cat: "security", desc: "AI Heuristic Network Doctor: Audit 6 pilar vital (CPU, RAM, WAN Ping, Throttled Queues, FastTrack, AP/LAN) & rekomendasi perbaikan.", defaultPayload: "{}" },
      { id: "sec_vuln_audit", path: "/api/v1/security/vulnerability-audit", method: "GET", cat: "security", desc: "Audit kerentanan CVE RouterOS, open service sniffing (Telnet, FTP, API), & Open DNS Resolver.", defaultPayload: "{}" },
      { id: "sec_antibruteforce", path: "/api/v1/security/deploy-antibruteforce", method: "POST", cat: "security", desc: "1-Klik pasang firewall filter anti brute-force attack bertingkat untuk WinBox, SSH, & API.", defaultPayload: '{\n  "protect_winbox": true,\n  "protect_ssh": true,\n  "protect_api": true,\n  "blacklist_timeout": "7d"\n}' },
      { id: "app_block", path: "/api/v1/security/app-block", method: "POST", cat: "security", desc: "1-Klik blokir Layer 7 / TLS host untuk TikTok, YouTube, WhatsApp, Judi Online, atau Torrent P2P.", defaultPayload: '{\n  "app_type": "judi_online",\n  "comment": "Blokir Judol"\n}' },
      { id: "app_unblock", path: "/api/v1/security/app-unblock", method: "POST", cat: "security", desc: "Buka kembali blokir aplikasi / domain di firewall.", defaultPayload: '{\n  "app_type": "judi_online"\n}' },
      { id: "app_blocked_list", path: "/api/v1/security/blocked-apps", method: "GET", cat: "security", desc: "Daftar seluruh rule pemblokiran aplikasi dan domain aktif di firewall.", defaultPayload: "{}" },
      { id: "anti_tethering_enable", path: "/api/v1/security/anti-tethering/enable", method: "POST", cat: "security", desc: "Proteksi anti-tethering hotspot: Paksa TTL=1 di Mangle agar klien tidak bisa berbagi hotspot via HP.", defaultPayload: '{\n  "comment": "Lock TTL to prevent tethering"\n}' },
      { id: "anti_tethering_status", path: "/api/v1/security/anti-tethering/status", method: "GET", cat: "security", desc: "Status proteksi anti-tethering hotspot (aktif / tidak).", defaultPayload: "{}" },
      { id: "anti_tethering_disable", path: "/api/v1/security/anti-tethering/disable", method: "POST", cat: "security", desc: "Nonaktifkan proteksi anti-tethering hotspot.", defaultPayload: "{}" },
      { id: "expert_diag", path: "/api/v1/expert/quick-diagnose", method: "POST", cat: "security", desc: "Expert deep diagnostic analyzer untuk performa dan latency router.", defaultPayload: "{}" },
      { id: "expert_traffic", path: "/api/v1/expert/traffic-matrix", method: "POST", cat: "security", desc: "Matriks persebaran trafik komprehensif antar interface.", defaultPayload: "{}" },

      // --- Load Balancing Multi-WAN & NAT Wizards (cat: loadbalance) ---
      { id: "lb_pcc_setup", path: "/api/v1/load-balance/pcc/setup", method: "POST", cat: "loadbalance", desc: "Automated Dual-WAN PCC Wizard: Buat otomatis rule Mangle PCC, routing marks, & failover 2-4 ISP.", defaultPayload: '{\n  "wan_interfaces": ["ether1", "ether2"],\n  "gateways": ["192.168.1.1", "192.168.2.1"],\n  "lan_interface": "bridge-lan"\n}' },
      { id: "lb_status", path: "/api/v1/load-balance/status", method: "GET", cat: "loadbalance", desc: "Status konfigurasi Multi-WAN PCC Load Balancer aktif di router.", defaultPayload: "{}" },
      { id: "lb_remove", path: "/api/v1/load-balance/remove", method: "POST", cat: "loadbalance", desc: "Hapus seluruh rule Load Balancing PCC dan bersihkan firewall Mangle.", defaultPayload: "{}" },
      { id: "nat_port_forward", path: "/api/v1/firewall/port-forward", method: "POST", cat: "loadbalance", desc: "Port Forwarding Wizard (dst-nat) lengkap dengan opsi Hairpin NAT untuk akses Web/CCTV dari luar LAN.", defaultPayload: '{\n  "protocol": "tcp",\n  "dst_port": "8080",\n  "to_addresses": "192.168.88.50",\n  "to_ports": "80",\n  "in_interface": "ether1",\n  "enable_hairpin": true,\n  "comment": "Web Server Forward"\n}' },
      { id: "nat_port_forward_list", path: "/api/v1/firewall/port-forward/list", method: "GET", cat: "loadbalance", desc: "Daftar seluruh rule port forwarding (dst-nat) aktif di router.", defaultPayload: "{}" },
      { id: "nat_masquerade", path: "/api/v1/firewall/srcnat/masquerade", method: "POST", cat: "loadbalance", desc: "Buat rule Masquerade WAN standar agar LAN dapat terhubung ke internet.", defaultPayload: '{\n  "out_interface": "ether1",\n  "comment": "Default Internet NAT"\n}' },

      // --- VPN Modern & Remote Access (cat: vpn) ---
      { id: "wg_interfaces", path: "/api/v1/wireguard/interfaces", method: "GET", cat: "vpn", desc: "Daftar interface WireGuard VPN di router (RouterOS v7).", defaultPayload: "{}" },
      { id: "wg_peers", path: "/api/v1/wireguard/peers", method: "GET", cat: "vpn", desc: "Daftar peer WireGuard (klien & site-to-site) aktif di router.", defaultPayload: "{}" },
      { id: "wg_peer_add", path: "/api/v1/wireguard/peer/add", method: "POST", cat: "vpn", desc: "Tambah peer WireGuard baru dengan public key & allowed-address.", defaultPayload: '{\n  "interface": "wireguard1",\n  "public_key": "YOUR_PEER_PUBLIC_KEY=",\n  "allowed_address": "10.0.0.2/32"\n}' },
      { id: "vpn_l2tp", path: "/api/v1/vpn/l2tp", method: "GET", cat: "vpn", desc: "Konfigurasi server L2TP VPN dan status IPsec encryption.", defaultPayload: "{}" },
      { id: "vpn_sstp", path: "/api/v1/vpn/sstp", method: "GET", cat: "vpn", desc: "Konfigurasi server SSTP VPN (Port 443 SSL).", defaultPayload: "{}" },
      { id: "vpn_ovpn", path: "/api/v1/vpn/ovpn", method: "GET", cat: "vpn", desc: "Konfigurasi server OpenVPN (TCP/UDP) di router.", defaultPayload: "{}" },
      { id: "vpn_eoip", path: "/api/v1/vpn/eoip", method: "GET", cat: "vpn", desc: "Daftar tunnel EoIP (Ethernet over IP) untuk bridging L2 antar cabang.", defaultPayload: "{}" },

      // --- QoS, Bandwidth & Queues (cat: queues) ---
      { id: "queues_simple", path: "/api/v1/queues/simple", method: "GET", cat: "queues", desc: "Daftar seluruh Simple Queues, target IP, max-limit (upload/download), & bytes terpakai.", defaultPayload: "{}" },
      { id: "queues_summary", path: "/api/v1/queues/overview-summary", method: "GET", cat: "queues", desc: "Ringkasan total kuota, agregasi kecepatan, dan status throttling seluruh antrean.", defaultPayload: "{}" },
      { id: "queues_simple_add", path: "/api/v1/queues/simple/add", method: "POST", cat: "queues", desc: "Tambah antrean Simple Queue baru dengan limit bandwidth target.", defaultPayload: '{\n  "name": "User-Kantor-1",\n  "target": "192.168.88.20/32",\n  "max_limit": "5M/10M",\n  "comment": "Staff Kantor"\n}' },
      { id: "queues_set_limit", path: "/api/v1/queues/simple/set-limit", method: "POST", cat: "queues", desc: "Ubah batas bandwidth Simple Queue target secara langsung.", defaultPayload: '{\n  "name": "User-Kantor-1",\n  "max_limit": "10M/20M"\n}' },
      { id: "queues_inspect", path: "/api/v1/queues/inspect-user", method: "POST", cat: "queues", desc: "Inspeksi utilisasi antrean bandwidth per user atau IP spesifik.", defaultPayload: '{\n  "target": "192.168.88.20"\n}' },
      { id: "traffic_preset_game", path: "/api/v1/traffic/preset/game-social-separation", method: "POST", cat: "queues", desc: "1-Klik Smart Traffic Shaper: Pisahkan trafik Game Online (prioritas tinggi) dari Sosmed/Download via Mangle & Queue Tree.", defaultPayload: '{\n  "wan_interface": "ether1",\n  "lan_interface": "bridge-lan"\n}' },
      { id: "traffic_queue_tree", path: "/api/v1/traffic/queue-tree", method: "GET", cat: "queues", desc: "Daftar Queue Tree hierarkis (Parent & Child queues) di router.", defaultPayload: "{}" },
      { id: "traffic_mangle_rules", path: "/api/v1/traffic/mangle/rules", method: "GET", cat: "queues", desc: "Daftar rule Firewall Mangle untuk penandaan paket (packet marks & routing marks).", defaultPayload: "{}" },

      // --- NOC Tools & Diagnostics (cat: tools) ---
      { id: "noc_torch", path: "/api/v1/tools/torch", method: "POST", cat: "tools", desc: "Torch live real-time bandwidth monitor per-IP, protocol, & port pada interface target.", defaultPayload: '{\n  "interface": "ether1",\n  "duration_seconds": 3\n}' },
      { id: "noc_btest", path: "/api/v1/tools/bandwidth-test", method: "POST", cat: "tools", desc: "MikroTik Bandwidth Test internal ke router tujuan (TCP/UDP speed test).", defaultPayload: '{\n  "address": "192.168.88.2",\n  "direction": "both",\n  "duration": 5\n}' },
      { id: "noc_traceroute", path: "/api/v1/tools/traceroute", method: "POST", cat: "tools", desc: "Jalankan Traceroute ICMP/UDP dari router untuk melacak hop rute ke internet.", defaultPayload: '{\n  "address": "8.8.8.8",\n  "count": 3\n}' },
      { id: "noc_ip_scan", path: "/api/v1/tools/ip-scan", method: "POST", cat: "tools", desc: "IP Scan: Deteksi seluruh perangkat aktif dan MAC address pada subnet interface.", defaultPayload: '{\n  "interface": "bridge-lan"\n}' },
      { id: "noc_profile", path: "/api/v1/tools/profile", method: "POST", cat: "tools", desc: "CPU Profile Detective (/tool profile): Profil mikro beban CPU per-proses (firewall, networking, console, queuing).", defaultPayload: '{\n  "duration": 3\n}' },
      { id: "noc_romon_discover", path: "/api/v1/tools/romon/discover", method: "GET", cat: "tools", desc: "Temukan router MikroTik tetangga di jaringan RoMON (Router Management Overlay Network).", defaultPayload: "{}" },
      { id: "noc_watchdog", path: "/api/v1/system/watchdog", method: "GET", cat: "tools", desc: "Status hardware Watchdog router (auto reboot saat kernel freeze).", defaultPayload: "{}" },
      { id: "noc_ntp_config", path: "/api/v1/system/ntp/config", method: "GET", cat: "tools", desc: "Konfigurasi NTP Client (Network Time Protocol) sinkronisasi jam router.", defaultPayload: "{}" },
      { id: "noc_terminal_exec", path: "/api/v1/scripts/terminal-exec", method: "POST", cat: "tools", desc: "Eksekusi baris perintah skrip RouterOS langsung dan ambil keluarannya.", defaultPayload: '{\n  "script": ":put [/system resource get cpu-load]"\n}' },

      // --- Backup & Maintenance (cat: maintenance) ---
      { id: "backup_create", path: "/api/v1/backup/create", method: "POST", cat: "maintenance", desc: "Buat file binary backup sistem router (.backup) di storage internal.", defaultPayload: '{\n  "name": "backup-gateway-manual"\n}' },
      { id: "backup_export", path: "/api/v1/backup/export", method: "POST", cat: "maintenance", desc: "Ekspor konfigurasi teks skrip RouterOS (.rsc) dengan hide-sensitive.", defaultPayload: '{\n  "file": "export-gateway.rsc",\n  "hide_sensitive": true\n}' },
      { id: "backup_files", path: "/api/v1/files/all", method: "GET", cat: "maintenance", desc: "Daftar seluruh file di storage router (backup, skin hotspot, cert, log).", defaultPayload: "{}" },
      { id: "maint_reboot", path: "/api/v1/system/reboot", method: "POST", cat: "maintenance", desc: "Reboot sistem router MikroTik secara graceful.", defaultPayload: "{}" },
      { id: "maint_check_update", path: "/api/v1/system/check-update", method: "GET", cat: "maintenance", desc: "Periksa ketersediaan pembaruan RouterOS versi terbaru dari server MikroTik.", defaultPayload: "{}" },
      { id: "maint_health", path: "/api/v1/system/health", method: "GET", cat: "maintenance", desc: "Sensor voltase, suhu CPU, dan temperatur board router (RouterBOARD).", defaultPayload: "{}" },
      { id: "maint_pre_upgrade", path: "/api/v1/maintenance/pre-upgrade-snapshot", method: "POST", cat: "maintenance", desc: "Snapshot pra-upgrade: Simpan backup konfigurasi dan riwayat package sebelum OS di-upgrade.", defaultPayload: "{}" },

      // --- Hotspot Wizard & Visual Templates (cat: hotspot & voucher) ---
      { id: "hs_wizard_setup", path: "/api/v1/hotspot/wizard/setup", method: "POST", cat: "hotspot", desc: "1-Klik Hotspot Wizard: Konfigurasi lengkap interface, IP pool, DNS, profile & login page otomatis.", defaultPayload: '{\n  "interface": "wlan1",\n  "dns_name": "hotspot.net",\n  "network": "192.168.10.0/24",\n  "gateway": "192.168.10.1"\n}' },
      { id: "voucher_template_render", path: "/api/v1/hotspot/voucher-template/render", method: "POST", cat: "voucher", desc: "Render lembaran cetak voucher visual berformat HTML (58mm, 80mm, atau A4 Grid) dengan QR Code.", defaultPayload: '{\n  "theme": "modern",\n  "paper_width": "80mm",\n  "hotspot_name": "WAROENG HOTSPOT",\n  "vouchers": [\n    { "username": "HOT-9012", "password": "pass-9012", "profile": "1JAM-3RB", "timelimit": "1h", "price": 3000 }\n  ]\n}' },
      { id: "raw_command", path: "/api/v1/command", method: "POST", cat: "system", desc: "Universal Raw Command Executor: Kirim perintah sembarang ke RouterOS API secara langsung.", defaultPayload: '{\n  "command": "/system/resource/print"\n}' }
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

    function initSavedProfiles() {
      const sel = document.getElementById('profile-select');
      if (!sel) return;
      sel.innerHTML = '<option value="custom">-- Custom Host --</option>';
      let profiles = [];
      try {
        profiles = JSON.parse(localStorage.getItem('ros_saved_profiles') || '[]');
      } catch (e) { profiles = []; }

      profiles.forEach((p, idx) => {
        const opt = document.createElement('option');
        opt.value = idx.toString();
        opt.textContent = `📍 ${p.name || p.host}`;
        sel.appendChild(opt);
      });

      const activeIdx = localStorage.getItem('ros_active_profile_idx');
      if (activeIdx !== null && profiles[parseInt(activeIdx)]) {
        sel.value = activeIdx;
      }
    }

    function onProfileChange() {
      const sel = document.getElementById('profile-select');
      if (!sel) return;
      if (sel.value === 'custom') return;
      const idx = parseInt(sel.value);
      try {
        const profiles = JSON.parse(localStorage.getItem('ros_saved_profiles') || '[]');
        const p = profiles[idx];
        if (p) {
          document.getElementById('target-host').value = p.host || '';
          document.getElementById('target-port').value = p.port || '';
          document.getElementById('target-user').value = p.user || '';
          document.getElementById('target-pass').value = p.pass || '';
          document.getElementById('gw-token').value = p.token || 'change-me-to-a-long-random-string';

          localStorage.setItem('ros_host', p.host || '');
          localStorage.setItem('ros_port', p.port || '');
          localStorage.setItem('ros_user', p.user || '');
          localStorage.setItem('ros_pass', p.pass || '');
          localStorage.setItem('ros_token', p.token || '');
          localStorage.setItem('ros_active_profile_idx', idx.toString());

          updateSnippet();
          testConnectionAndSnapshot();
        }
      } catch (e) {
        console.error('Failed to load profile:', e);
      }
    }

    function saveCurrentProfile() {
      const host = document.getElementById('target-host').value.trim();
      const port = document.getElementById('target-port').value.trim();
      const user = document.getElementById('target-user').value.trim();
      const pass = document.getElementById('target-pass').value;
      const token = document.getElementById('gw-token').value.trim();

      if (!host) {
        alert("Harap masukkan Router Host sebelum menyimpan profil.");
        return;
      }

      const defaultName = `${user || 'admin'}@${host}`;
      const name = prompt("Nama Profil Router:", defaultName);
      if (!name) return;

      try {
        let profiles = JSON.parse(localStorage.getItem('ros_saved_profiles') || '[]');
        const newProf = { name, host, port, user, pass, token };
        const existingIdx = profiles.findIndex(p => p.name === name);
        if (existingIdx >= 0) {
          profiles[existingIdx] = newProf;
        } else {
          profiles.push(newProf);
        }
        localStorage.setItem('ros_saved_profiles', JSON.stringify(profiles));
        initSavedProfiles();
        const sel = document.getElementById('profile-select');
        const foundIdx = profiles.findIndex(p => p.name === name);
        if (foundIdx >= 0 && sel) {
          sel.value = foundIdx.toString();
          localStorage.setItem('ros_active_profile_idx', foundIdx.toString());
        }
        alert(`Profil '${name}' berhasil disimpan!`);
      } catch (e) {
        alert("Gagal menyimpan profil: " + e.message);
      }
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
        let desc = found.desc;
        if (found.desc.includes('RouterOS v7') && window.currentRouterOSVersion && window.currentRouterOSVersion.startsWith('6.')) {
          desc += ' ⚠️ [Perhatian: Router Anda saat ini terdeteksi v' + window.currentRouterOSVersion + ', fitur ini dirancang untuk RouterOS v7+]';
        }
        document.getElementById('ep-desc').textContent = desc;
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
        if (json && json.success && json.data) {
          updateDashboardKPIs(json.data);
        }
      } catch (err) {
        statusBadge.textContent = 'Error';
        statusBadge.style.background = '#fef2f2';
        statusBadge.style.color = '#dc2626';
        viewer.textContent = `Gagal mengirim request: ${err.message}`;
      }
    }

    function updateDashboardKPIs(d) {
      if (!d) return;

      // Identitas & Tipe Board Router
      if (d.identity) {
        const idEl = document.getElementById('kpi-identity');
        if (idEl) idEl.textContent = d.identity;
      }
      const board = d.routerboard?.model || d.system?.board_name;
      if (board) {
        const bEl = document.getElementById('kpi-board');
        if (bEl) bEl.textContent = board;
      }
      if (d.system && (d.system.version || d.system.uptime)) {
        window.currentRouterOSVersion = d.system.version || '';
        const rosEl = document.getElementById('kpi-ros');
        if (rosEl) rosEl.textContent = `RouterOS ${d.system.version || ''} | Uptime: ${d.system.uptime || ''}`;
      }

      // CPU Load & Frekuensi
      if (d.system) {
        if (d.system.cpu_load_percent !== undefined) {
          const cpuEl = document.getElementById('kpi-cpu');
          if (cpuEl) cpuEl.textContent = `${d.system.cpu_load_percent}%`;
          const barEl = document.getElementById('kpi-cpu-bar');
          if (barEl) barEl.style.width = `${Math.min(100, Math.max(0, d.system.cpu_load_percent))}%`;
        }
        if (d.system.cpu_frequency_mhz) {
          const freqEl = document.getElementById('kpi-freq');
          const cores = d.system.cpu_count ? ` (${d.system.cpu_count} Core)` : '';
          if (freqEl) freqEl.textContent = `${d.system.cpu_frequency_mhz} MHz${cores}`;
        }

        // RAM Free, Total, dan Persentase Terpakai
        if (d.system.free_memory_mb !== undefined) {
          const freeEl = document.getElementById('kpi-free-ram');
          if (freeEl) freeEl.textContent = `${d.system.free_memory_mb} MB`;
        }
        if (d.system.total_memory_mb !== undefined) {
          const totEl = document.getElementById('kpi-total-ram');
          if (totEl) totEl.textContent = `${d.system.total_memory_mb} MB`;
        }
        const memPercentEl = document.getElementById('kpi-mem-percent');
        if (memPercentEl) {
          if (d.system.memory_used_percent) {
            memPercentEl.textContent = `Penggunaan RAM: ${d.system.memory_used_percent}`;
          } else if (d.system.total_memory_mb && d.system.free_memory_mb !== undefined) {
            const usedPct = (((d.system.total_memory_mb - d.system.free_memory_mb) / d.system.total_memory_mb) * 100).toFixed(1);
            memPercentEl.textContent = `Penggunaan RAM: ${usedPct}%`;
          }
        }
      }

      // Klien & Relasi Jaringan (DHCP, Hotspot, PPPoE)
      const hs = d.hotspot_active_count ?? d.hotspot?.active_total ?? 0;
      const dhcp = d.dhcp_lease_count ?? d.dhcp?.lease_total ?? 0;
      const ppp = d.ppp_active_count ?? d.ppp?.active_total ?? 0;
      const total = d.total_clients_count ?? d.clients?.total ?? (hs + dhcp + ppp);

      if (d.clients || d.hotspot || d.dhcp || d.ppp || d.hotspot_active_count !== undefined || d.dhcp_lease_count !== undefined || d.ppp_active_count !== undefined) {
        const clientsEl = document.getElementById('kpi-clients');
        if (clientsEl) clientsEl.textContent = `${total} Perangkat`;
        const clientsSubEl = document.getElementById('kpi-clients-sub');
        if (clientsSubEl) clientsSubEl.textContent = `DHCP: ${dhcp} | Hotspot: ${hs} | PPPoE: ${ppp}`;
        const statusEl = document.getElementById('kpi-clients-status');
        if (statusEl) {
          statusEl.textContent = '● Terkoneksi';
          statusEl.style.color = 'var(--accent-emerald)';
          statusEl.style.fontWeight = '600';
        }
      }
    }

    async function testConnectionAndSnapshot() {
      const host = document.getElementById('target-host').value.trim();
      const port = document.getElementById('target-port').value.trim();
      const user = document.getElementById('target-user').value.trim();
      const pass = document.getElementById('target-pass').value;
      const token = document.getElementById('gw-token').value.trim();

      if (!host && !user) {
        document.getElementById('resp-viewer').textContent = "// Masukkan Router Host & User pada form di atas, lalu klik '⚡ Sinkronisasi & Snapshot' untuk menghubungkan ke router.";
        return;
      }

      if (currentEndpoint.path === '/api/v1/overview') {
        executeCurrentEndpoint();
        return;
      }

      executeCurrentEndpoint();

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
          updateDashboardKPIs(json.data);
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
      initSavedProfiles();
      populateDropdown('all');
      const host = document.getElementById('target-host').value.trim();
      if (host) {
        setTimeout(testConnectionAndSnapshot, 400);
      }
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
      <span class="badge-endpoints">170+ Endpoints Terdaftar &amp; Aktif</span>
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
          <div class="ep-desc">Otomatisasi pengawasan Netwatch armada Access Point (AP Fleet) sekaligus (misal AP-HOTSPOT-1..4 pada IP 192.168.88.2..5) hanya dalam satu panggilan API.</div>
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
          <div class="ep-desc">Pembersihan aman voucher kedaluwarsa secara berkala tanpa menyentuh voucher aktif atau voucher yang belum laku terjual.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/voucher-template/render</div>
          </div>
          <div class="ep-desc">Dynamic HTML Voucher Template Engine: render template voucher kustom (variabel {{username}}, {{password}}, {{price}}, {{validity}}) menjadi tampilan voucher siap cetak.</div>
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
          <pre class="code-preview">curl -X GET "http://127.0.0.1:8080/api/v1/overview" \
  -H "Authorization: Bearer change-me-to-a-long-random-string" \
  -H "X-Router-Host: 192.168.88.1" \
  -H "X-Router-Port: 8728" \
  -H "X-Router-User: admin" \
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

      <!-- Section: Hotspot Member Management & Pre-Login Verifier -->
      <section class="doc-section" id="sec-members">
        <div class="section-header">👤 Hotspot Member Management &amp; Pre-Login Verifier</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/members/register</div>
          </div>
          <div class="ep-desc">Registrasi member bulanan/VIP: username &amp; password terpisah, metadata Nama, HP, masa aktif (EXP:YYYY-MM-DD), harga, dan catatan tersimpan rapi di router.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/members</div>
          </div>
          <div class="ep-desc">Daftar terstruktur seluruh member: parsing otomatis status (ACTIVE, EXPIRED, SUSPENDED), sisa hari jatuh tempo, total kuota bytes in/out, dan uptime.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/members/renew</div>
          </div>
          <div class="ep-desc">Perpanjangan masa aktif bulanan (+30 hari), otomatis update tanggal expired baru, reset counter traffic, dan aktifkan kembali akun member.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/members/suspend</div>
          </div>
          <div class="ep-desc">Isolir member yang menunggak tagihan (pindah profile isolir / disable, serta langsung kick sesi aktif seketika).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/verify-login</div>
          </div>
          <div class="ep-desc">Unified Pre-Login Authentication Engine: verifikasi kredensial captive portal (voucher vs member, password, tanggal kadaluarsa, batas kuota/uptime, dan batas shared-users) sebelum dikirim ke router.</div>
        </div>
      </section>

      <!-- Section: Multi-Tenant Segregated Audit Logging & Latency Analytics -->
      <section class="doc-section" id="sec-audit">
        <div class="section-header">🔒 Multi-Tenant Segregated Audit Logging &amp; Latency Analytics</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/audit/logs</div>
          </div>
          <div class="ep-desc">Riwayat log audit eksekusi dan latency ms: diisolasi 100% ketat berbasis hash token tenant (Zero-Leak Boundary). Tenant A tidak bisa melihat riwayat Tenant B.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/audit/stats</div>
          </div>
          <div class="ep-desc">Metrik analitik performa: rata-rata latency ms, p95 latency, tingkat sukses %, throughput, dan daftar router aktif milik tenant.</div>
        </div>
      </section>

      <!-- Section: System Health, NTP, IP Pool & DHCP Lease -->
      <section class="doc-section" id="sec-system-pro">
        <div class="section-header">🌡️ System Health, NTP Sync &amp; IP Pool Exhaustion</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/system/health</div>
          </div>
          <div class="ep-desc">Hardware telemetry: sensor suhu CPU (°C), voltase board (V), watt, dan kecepatan fan RPM (dengan graceful fallback untuk CHR/VM).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/system/ntp/setup</div>
          </div>
          <div class="ep-desc">1-Klik NTP &amp; Timezone Sync: sinkronisasi jam satelit internet (id.pool.ntp.org, time.google.com) dan set timezone agar jam router tidak reset ke 1970 saat reboot.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/system/ntp</div>
          </div>
          <div class="ep-desc">Status NTP Client: inspeksi status sinkronisasi waktu satelit dan detail pembacaan jam (clock) router saat ini.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/system/ntp/config</div>
          </div>
          <div class="ep-desc">Konfigurasi manual NTP Client: atur parameter server pool kustom, status enabled, dan nama zona waktu RouterOS.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ip/pools/utilization</div>
          </div>
          <div class="ep-desc">Deteksi dini pool exhaustion: hitung kapasitas rentang IP, pemakaian riil, persentase utilisasi, dan alert (HEALTHY, WARNING_HIGH, CRITICAL_EXHAUSTED).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dhcp/lease/add</div>
          </div>
          <div class="ep-desc">Daftarkan static DHCP lease baru permanen dengan IP, MAC address, dan comment identitas perangkat.</div>
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
            "version": "1.0.0",
            "description": "High-performance sub-millisecond MikroTik management gateway with 280+ enterprise endpoints"
        },
        "servers": [
            { "url": "http://127.0.0.1:8080" },
            { "url": "https://gateway.example.com" }
        ],
        "security": [
            { "BearerAuth": [] }
        ]
    }))
}

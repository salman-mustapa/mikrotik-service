use axum::response::Html;
use axum::Json;
use serde_json::{json, Value};

/// Unified Developer Portal, Interactive API Explorer, and Gateway Documentation
pub async fn docs_page() -> Html<&'static str> {
    Html(r##"<!DOCTYPE html>
<html lang="id">
<head>
  <meta charset="UTF-8">
  <title>MikroTik Universal Gateway - Developer Console &amp; Interactive API Explorer</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500;700&display=swap" rel="stylesheet">
  <style>
    :root {
      --bg: #070b14;
      --surface: #0e1626;
      --surface-elevated: #152238;
      --surface-card: #111c30;
      --border: rgba(255, 255, 255, 0.09);
      --border-accent: rgba(56, 189, 248, 0.3);
      --text-main: #f8fafc;
      --text-muted: #94a3b8;
      --accent-cyan: #06b6d4;
      --accent-blue: #3b82f6;
      --accent-emerald: #10b981;
      --accent-purple: #8b5cf6;
      --accent-amber: #f59e0b;
      --accent-rose: #f43f5e;
      --code-bg: #030712;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: radial-gradient(circle at 50% 0%, #172554 0%, #070b14 70%);
      color: var(--text-main);
      font-family: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      min-height: 100vh;
      display: flex;
      flex-direction: column;
    }
    code, pre { font-family: 'JetBrains Mono', monospace; }

    /* Header */
    header {
      position: sticky;
      top: 0;
      z-index: 100;
      background: rgba(7, 11, 20, 0.88);
      backdrop-filter: blur(20px);
      border-bottom: 1px solid var(--border);
      padding: 12px 24px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 16px;
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 12px;
      text-decoration: none;
      color: inherit;
    }
    .brand-title {
      font-weight: 800;
      font-size: 1.15rem;
      letter-spacing: -0.5px;
      background: linear-gradient(135deg, #38bdf8, #818cf8);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }
    .badge-core {
      background: rgba(6, 182, 212, 0.15);
      border: 1px solid rgba(6, 182, 212, 0.35);
      color: #38bdf8;
      font-size: 0.72rem;
      font-weight: 700;
      padding: 3px 8px;
      border-radius: 999px;
    }

    /* Top Tabs Navigation */
    .top-nav {
      display: flex;
      align-items: center;
      gap: 6px;
      background: rgba(15, 23, 42, 0.8);
      border: 1px solid var(--border);
      padding: 4px;
      border-radius: 10px;
    }
    .tab-btn {
      background: transparent;
      border: none;
      color: var(--text-muted);
      padding: 8px 16px;
      border-radius: 7px;
      font-size: 0.82rem;
      font-weight: 600;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 6px;
      transition: all 0.2s ease;
    }
    .tab-btn:hover { color: var(--text-main); background: rgba(255, 255, 255, 0.04); }
    .tab-btn.active {
      background: linear-gradient(135deg, #0284c7, #06b6d4);
      color: white;
      box-shadow: 0 4px 12px rgba(6, 182, 212, 0.25);
    }

    .header-actions {
      display: flex;
      align-items: center;
      gap: 10px;
    }
    .btn-action-link {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 7px 14px;
      border-radius: 8px;
      font-size: 0.8rem;
      font-weight: 600;
      text-decoration: none;
      transition: all 0.2s ease;
      cursor: pointer;
    }
    .btn-visualizer-nav {
      background: linear-gradient(135deg, #4f46e5, #7c3aed);
      color: white;
      box-shadow: 0 4px 14px rgba(124, 58, 237, 0.25);
    }
    .btn-action-link:hover { transform: translateY(-1px); }

    /* Router Settings Sticky Bar */
    .creds-bar {
      background: rgba(17, 28, 48, 0.95);
      border-bottom: 1px solid var(--border);
      padding: 10px 24px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      flex-wrap: wrap;
      gap: 12px;
      font-size: 0.8rem;
    }
    .creds-fields {
      display: flex;
      align-items: center;
      flex-wrap: wrap;
      gap: 8px;
    }
    .cred-input-wrap {
      display: flex;
      align-items: center;
      background: var(--code-bg);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 2px 8px;
    }
    .cred-label {
      color: var(--text-muted);
      font-size: 0.72rem;
      font-weight: 600;
      margin-right: 6px;
      white-space: nowrap;
    }
    .cred-input {
      background: transparent;
      border: none;
      color: white;
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.78rem;
      padding: 4px 0;
      outline: none;
    }
    .status-pill {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      background: rgba(16, 185, 129, 0.15);
      border: 1px solid rgba(16, 185, 129, 0.35);
      color: #34d399;
      font-size: 0.72rem;
      font-weight: 700;
      padding: 4px 10px;
      border-radius: 999px;
    }
    .pulse-dot {
      width: 7px;
      height: 7px;
      border-radius: 50%;
      background: var(--accent-emerald);
      animation: pulse 1.5s infinite;
    }
    @keyframes pulse { 0% { opacity: 0.4; } 50% { opacity: 1; } 100% { opacity: 0.4; } }

    /* Main Container */
    .main-viewport {
      display: flex;
      flex: 1;
      max-width: 1750px;
      width: 100%;
      margin: 0 auto;
      padding: 20px 24px;
      gap: 20px;
    }

    /* Tab 1: API Explorer Layout (2 Kolom) */
    .explorer-view {
      display: grid;
      grid-template-columns: 1.15fr 0.85fr;
      gap: 20px;
      width: 100%;
    }
    @media (max-width: 1100px) {
      .explorer-view { grid-template-columns: 1fr; }
    }

    /* Left: Categories & Endpoints List */
    .endpoints-panel {
      display: flex;
      flex-direction: column;
      gap: 16px;
    }
    .search-filter-card {
      background: var(--surface-card);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 16px;
    }
    .search-input-box {
      width: 100%;
      background: var(--code-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 10px 14px 10px 36px;
      color: white;
      font-size: 0.88rem;
      outline: none;
      position: relative;
    }
    .search-input-box:focus { border-color: var(--accent-cyan); }
    .cat-pills {
      display: flex;
      flex-wrap: wrap;
      gap: 6px;
      margin-top: 12px;
    }
    .cat-pill {
      background: rgba(30, 41, 59, 0.6);
      border: 1px solid rgba(255, 255, 255, 0.08);
      color: var(--text-muted);
      padding: 4px 10px;
      border-radius: 6px;
      font-size: 0.75rem;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.15s;
    }
    .cat-pill:hover, .cat-pill.active {
      background: rgba(56, 189, 248, 0.2);
      border-color: var(--accent-cyan);
      color: #38bdf8;
    }

    /* Endpoint Card */
    .ep-card {
      background: var(--surface-card);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 18px;
      transition: all 0.2s ease;
    }
    .ep-card:hover {
      border-color: var(--border-accent);
      box-shadow: 0 6px 20px rgba(0, 0, 0, 0.35);
    }
    .ep-head {
      display: flex;
      align-items: center;
      justify-content: space-between;
      flex-wrap: wrap;
      gap: 10px;
      margin-bottom: 8px;
    }
    .ep-badge-method {
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.72rem;
      font-weight: 800;
      padding: 3px 8px;
      border-radius: 5px;
    }
    .m-post { background: #1e3a8a; color: #93c5fd; border: 1px solid #3b82f6; }
    .m-get { background: #064e3b; color: #6ee7b7; border: 1px solid #10b981; }
    .m-ws { background: #78350f; color: #fde68a; border: 1px solid #f59e0b; }
    .ep-path {
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.9rem;
      font-weight: 700;
      color: #f1f5f9;
    }
    .ep-desc {
      color: #94a3b8;
      font-size: 0.82rem;
      line-height: 1.45;
      margin-bottom: 12px;
    }
    .ep-body-editor {
      background: var(--code-bg);
      border: 1px solid rgba(255, 255, 255, 0.08);
      border-radius: 8px;
      padding: 10px;
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.78rem;
      color: #38bdf8;
      width: 100%;
      min-height: 70px;
      max-height: 150px;
      resize: vertical;
      outline: none;
      margin-bottom: 10px;
    }
    .ep-actions {
      display: flex;
      align-items: center;
      flex-wrap: wrap;
      gap: 6px;
    }
    .btn-exec {
      background: linear-gradient(135deg, #0284c7, #06b6d4);
      color: white;
      border: none;
      padding: 6px 14px;
      border-radius: 6px;
      font-size: 0.78rem;
      font-weight: 700;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 5px;
      transition: opacity 0.2s;
    }
    .btn-exec:hover { opacity: 0.9; }
    .btn-snippet {
      background: rgba(255, 255, 255, 0.06);
      border: 1px solid rgba(255, 255, 255, 0.08);
      color: var(--text-muted);
      padding: 6px 10px;
      border-radius: 6px;
      font-size: 0.72rem;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.15s;
    }
    .btn-snippet:hover { background: rgba(56, 189, 248, 0.15); color: #38bdf8; }

    /* Right: Sticky Live Response Console */
    .console-panel {
      position: sticky;
      top: 130px;
      height: calc(100vh - 150px);
      display: flex;
      flex-direction: column;
      background: var(--surface-card);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 16px;
      overflow: hidden;
    }
    .console-head {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 10px;
      padding-bottom: 10px;
      border-bottom: 1px solid var(--border);
    }
    .console-title {
      font-weight: 700;
      font-size: 0.88rem;
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .console-timing {
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.75rem;
      color: var(--text-muted);
    }
    .console-output {
      flex: 1;
      background: var(--code-bg);
      border: 1px solid rgba(255, 255, 255, 0.08);
      border-radius: 8px;
      padding: 14px;
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.8rem;
      color: #38bdf8;
      overflow: auto;
      white-space: pre-wrap;
      word-break: break-all;
    }
    .console-footer {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-top: 10px;
      padding-top: 10px;
      border-top: 1px solid var(--border);
      font-size: 0.75rem;
      color: var(--text-muted);
    }

    /* Tab 2: Documentation & Architecture Layout */
    .docs-view {
      display: none;
      flex-direction: column;
      gap: 24px;
      width: 100%;
      max-width: 1200px;
      margin: 0 auto;
    }
    .doc-hero {
      background: linear-gradient(135deg, rgba(30, 58, 138, 0.3), rgba(15, 23, 42, 0.8));
      border: 1px solid var(--border-accent);
      border-radius: 16px;
      padding: 28px 32px;
    }
    .doc-hero h1 { font-size: 1.85rem; font-weight: 800; margin-bottom: 8px; }
    .doc-hero p { color: #cbd5e1; font-size: 0.95rem; line-height: 1.6; }
    .grid-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
    @media (max-width: 900px) { .grid-2 { grid-template-columns: 1fr; } }
    .doc-card {
      background: var(--surface-card);
      border: 1px solid var(--border);
      border-radius: 14px;
      padding: 22px;
    }
    .doc-card h2 { font-size: 1.15rem; font-weight: 700; margin-bottom: 10px; color: #f1f5f9; }
    .doc-card p { font-size: 0.88rem; color: #94a3b8; line-height: 1.6; margin-bottom: 14px; }
    .table-spec {
      width: 100%;
      border-collapse: collapse;
      font-size: 0.82rem;
      margin-top: 10px;
    }
    .table-spec th, .table-spec td {
      border: 1px solid var(--border);
      padding: 8px 12px;
      text-align: left;
    }
    .table-spec th { background: rgba(255, 255, 255, 0.04); color: #cbd5e1; font-weight: 700; }
    .table-spec td { color: #94a3b8; }
    .lang-tabs { display: flex; gap: 6px; margin-bottom: 10px; }
    .lang-tab {
      background: rgba(255, 255, 255, 0.05);
      border: 1px solid var(--border);
      color: var(--text-muted);
      padding: 5px 12px;
      border-radius: 6px;
      font-size: 0.75rem;
      cursor: pointer;
    }
    .lang-tab.active { background: #0284c7; color: white; border-color: #0284c7; }
    .code-preview {
      background: var(--code-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 14px;
      font-size: 0.8rem;
      color: #38bdf8;
      overflow-x: auto;
    }

    /* Tab 3: Topology View */
    .topology-view {
      display: none;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      width: 100%;
      min-height: 60vh;
      text-align: center;
      gap: 16px;
    }

    /* Tab 4: Settings View */
    .settings-view {
      display: none;
      flex-direction: column;
      max-width: 700px;
      margin: 0 auto;
      width: 100%;
      gap: 16px;
    }
  </style>
</head>
<body>

  <!-- Top Global Header -->
  <header>
    <a href="/" class="brand">
      <span style="font-size: 1.4rem;">🦀</span>
      <div>
        <div class="brand-title">MikroTik Universal Gateway</div>
        <div style="font-size: 0.68rem; color: var(--text-muted);">Enterprise NOC Gateway &amp; Developer Console</div>
      </div>
      <span class="badge-core">v0.2.0 Active</span>
    </a>

    <!-- Navigation Tabs -->
    <nav class="top-nav">
      <button class="tab-btn active" onclick="switchTab('explorer')">⚡ API Explorer</button>
      <button class="tab-btn" onclick="switchTab('docs')">📖 Panduan Integrasi</button>
      <button class="tab-btn" onclick="switchTab('topology')">🌐 Visualizer Topologi</button>
      <button class="tab-btn" onclick="switchTab('settings')">⚙️ Kredensial Router</button>
    </nav>

    <div class="header-actions">
      <a href="/topology" target="_blank" class="btn-action-link btn-visualizer-nav">
        <span>🌐</span> Buka Full Canvas Visualizer &rarr;
      </a>
    </div>
  </header>

  <!-- Pinned Active Router Connection Bar -->
  <section class="creds-bar">
    <div class="creds-fields">
      <div class="cred-input-wrap">
        <span class="cred-label">Bearer Token:</span>
        <input type="text" id="g-token" class="cred-input" style="width: 130px;" value="change-me-to-a-long-random-string">
      </div>
      <div class="cred-input-wrap">
        <span class="cred-label">Router Host:</span>
        <input type="text" id="r-host" class="cred-input" style="width: 120px;" value="ath.vpnbersama.us">
      </div>
      <div class="cred-input-wrap">
        <span class="cred-label">API Port:</span>
        <input type="number" id="r-port" class="cred-input" style="width: 55px;" value="51121">
      </div>
      <div class="cred-input-wrap">
        <span class="cred-label">User:</span>
        <input type="text" id="r-user" class="cred-input" style="width: 75px;" value="salman">
      </div>
      <div class="cred-input-wrap">
        <span class="cred-label">Password:</span>
        <input type="password" id="r-pass" class="cred-input" style="width: 80px;" placeholder="Password">
      </div>
    </div>
    <div style="display: flex; align-items: center; gap: 8px;">
      <span class="status-pill" id="conn-pill"><span class="pulse-dot"></span> <span id="conn-text">Tersimpan di Browser</span></span>
    </div>
  </section>

  <!-- Main Viewport -->
  <main class="main-viewport">

    <!-- VIEW 1: API EXPLORER & LIVE TESTER -->
    <div id="view-explorer" class="explorer-view">
      <!-- Left: Interactive Endpoints -->
      <div class="endpoints-panel">
        <div class="search-filter-card">
          <div style="position: relative;">
            <span style="position: absolute; left: 12px; top: 11px; color: var(--text-muted);">🔍</span>
            <input type="text" id="search-box" class="search-input-box" placeholder="Cari endpoint (misal: hotspot, ppp, ping, wireguard, traffic)..." oninput="filterList()">
          </div>
          <div class="cat-pills" id="cat-pills">
            <span class="cat-pill active" onclick="setCategory('all', this)">Semua (All)</span>
            <span class="cat-pill" onclick="setCategory('fast-path', this)">🚀 Fast-Path</span>
            <span class="cat-pill" onclick="setCategory('hotspot', this)">🎟️ Hotspot &amp; Voucher</span>
            <span class="cat-pill" onclick="setCategory('ppp', this)">🏢 PPPoE ISP</span>
            <span class="cat-pill" onclick="setCategory('security', this)">🛡️ Keamanan</span>
            <span class="cat-pill" onclick="setCategory('wizards', this)">🧙‍♂️ Wizard 1-Klik</span>
            <span class="cat-pill" onclick="setCategory('load-balance', this)">⚖️ Multi-WAN PCC</span>
            <span class="cat-pill" onclick="setCategory('queues', this)">📊 Queues &amp; Limit</span>
            <span class="cat-pill" onclick="setCategory('interfaces', this)">📈 Interfaces</span>
            <span class="cat-pill" onclick="setCategory('tools', this)">🔬 NOC Tools</span>
            <span class="cat-pill" onclick="setCategory('system', this)">⚙️ System</span>
            <span class="cat-pill" onclick="setCategory('raw', this)">⌨️ Raw &amp; Batch</span>
          </div>
        </div>

        <div id="endpoint-list-container" style="display: flex; flex-direction: column; gap: 14px;">
          <!-- Rendered dynamically by JS -->
        </div>
      </div>

      <!-- Right: Sticky Live Response Inspector -->
      <div class="console-panel">
        <div class="console-head">
          <div class="console-title">
            <span>📊</span> Live Response Console
          </div>
          <div class="console-timing" id="console-meta">Menunggu eksekusi...</div>
        </div>
        <pre class="console-output" id="console-body">// Klik tombol "▶️ Test Endpoint" pada endpoint di sebelah kiri
// Hasil eksekusi dari router MikroTik riil akan langsung tampil di sini dalam JSON...</pre>
        <div class="console-footer">
          <button class="btn-snippet" onclick="copyConsoleOutput()">📋 Salin Respons JSON</button>
          <span>Rust Socket Engine: Active</span>
        </div>
      </div>
    </div>

    <!-- VIEW 2: INTEGRATION DOCUMENTATION -->
    <div id="view-docs" class="docs-view">
      <div class="doc-hero">
        <h1>Panduan Integrasi Gateway (Headless API)</h1>
        <p>
          MikroTik Universal Gateway adalah microservice perantara berbasis Rust. Aplikasi Anda (PHP Laravel, Flutter Dart, Express, Next.js, Python, Go) cukup memanggil HTTP POST ke endpoint gateway ini. Gateway akan membuka koneksi soket biner TCP persisten ke MikroTik Anda dan mengembalikan respons JSON dalam hitungan <strong>milidetik</strong> tanpa membebani CPU router.
        </p>
      </div>

      <div class="grid-2">
        <div class="doc-card">
          <h2>🔐 Header Autentikasi Wajib</h2>
          <p>Kirimkan header berikut pada setiap pemanggilan API dari backend / aplikasi luar:</p>
          <table class="table-spec">
            <thead>
              <tr><th>Header</th><th>Wajib?</th><th>Keterangan</th></tr>
            </thead>
            <tbody>
              <tr><td><code>Authorization</code></td><td>Ya</td><td><code>Bearer &lt;GATEWAY_TOKEN&gt;</code></td></tr>
              <tr><td><code>X-Router-Host</code></td><td>Ya</td><td>IP Publik / Domain router target</td></tr>
              <tr><td><code>X-Router-Port</code></td><td>Opsional</td><td>Port API MikroTik (default <code>8728</code> atau port VPN)</td></tr>
              <tr><td><code>X-Router-User</code></td><td>Ya</td><td>Username login admin MikroTik</td></tr>
              <tr><td><code>X-Router-Pass</code></td><td>Opsional</td><td>Password login user MikroTik</td></tr>
              <tr><td><code>Content-Type</code></td><td>Ya</td><td><code>application/json</code></td></tr>
            </tbody>
          </table>
        </div>

        <div class="doc-card">
          <h2>💡 Keunggulan Arsitektur Rust Gateway</h2>
          <ul style="color: #94a3b8; font-size: 0.88rem; line-height: 1.8; padding-left: 20px;">
            <li><strong style="color: white;">Persistent Multiplexed Pool:</strong> Tidak buka-tutup soket TCP baru setiap request. Soket tetap hangat di memori Rust.</li>
            <li><strong style="color: white;">Zero CPU Freeze Guard:</strong> Dilengkapi timeout 15 detik agar router tidak pernah hang atau 100% CPU.</li>
            <li><strong style="color: white;">Sub-Milidetik:</strong> Rata-rata respons 1 - 5 milidetik karena menggunakan protokol biner tingkat rendah (wire format).</li>
            <li><strong style="color: white;">Universal &amp; Bebas Bahasa:</strong> Tidak perlu install extension PHP atau lib MikroTik di server aplikasi Anda.</li>
          </ul>
        </div>
      </div>

      <div class="doc-card">
        <h2>💻 Contoh Kode Pemanggilan Nyata (Multi-Bahasa)</h2>
        <div class="lang-tabs">
          <button class="lang-tab active" onclick="switchLang('php', this)">🐘 PHP (Laravel)</button>
          <button class="lang-tab" onclick="switchLang('js', this)">📜 JavaScript / Node.js</button>
          <button class="lang-tab" onclick="switchLang('dart', this)">💙 Dart (Flutter)</button>
          <button class="lang-tab" onclick="switchLang('python', this)">🐍 Python</button>
          <button class="lang-tab" onclick="switchLang('curl', this)">📋 cURL</button>
        </div>

        <pre class="code-preview" id="lang-code-box">use Illuminate\Support\Facades\Http;

$response = Http::withHeaders([
    'Authorization' => 'Bearer change-me-to-a-long-random-string',
    'X-Router-Host' => 'ath.vpnbersama.us',
    'X-Router-Port' => '51121',
    'X-Router-User' => 'salman',
    'X-Router-Pass' => 'password_router_anda',
    'Content-Type'  => 'application/json',
])->post('https://ros-gateway.samrifa.com/api/v1/hotspot/active');

$users = $response->json();</pre>
      </div>
    </div>

    <!-- VIEW 3: TOPOLOGY CANVAS LINK -->
    <div id="view-topology" class="topology-view">
      <div style="font-size: 3.5rem;">🌐</div>
      <h2 style="font-size: 1.7rem; font-weight: 800;">Visualizer Relasi Jaringan &amp; Topologi Interaktif</h2>
      <p style="color: var(--text-muted); max-width: 600px; font-size: 0.95rem; line-height: 1.6;">
        Buka Canvas visualizer SVG bertenaga tinggi untuk melihat hubungan antara Router Core, Interface, DHCP Leases, PPPoE online, dan Hotspot clients secara grafis dengan simulasi partikel aktif.
      </p>
      <div style="margin-top: 12px; display: flex; gap: 12px;">
        <button onclick="openVisualizerDirect()" class="btn-exec" style="padding: 10px 24px; font-size: 0.9rem;">
          🌐 Buka Visualizer Relasi Topologi &rarr;
        </button>
      </div>
    </div>

    <!-- VIEW 4: SETTINGS VIEW -->
    <div id="view-settings" class="settings-view">
      <div class="doc-card">
        <h2>⚙️ Konfigurasi Target Router &amp; Kredensial</h2>
        <p>Kredensial disimpan di <code>localStorage</code> browser Anda dan otomatis disinkronkan saat menguji API atau membuka Visualizer:</p>
        
        <div style="display: flex; flex-direction: column; gap: 12px; margin-top: 12px;">
          <div>
            <label style="font-size: 0.75rem; color: var(--text-muted); font-weight: 700; display: block; margin-bottom: 4px;">Gateway Bearer Token</label>
            <input type="text" id="cfg-token" class="search-input-box" value="change-me-to-a-long-random-string">
          </div>
          <div style="display: grid; grid-template-columns: 2fr 1fr; gap: 10px;">
            <div>
              <label style="font-size: 0.75rem; color: var(--text-muted); font-weight: 700; display: block; margin-bottom: 4px;">Router Host (IP / Domain)</label>
              <input type="text" id="cfg-host" class="search-input-box" value="ath.vpnbersama.us">
            </div>
            <div>
              <label style="font-size: 0.75rem; color: var(--text-muted); font-weight: 700; display: block; margin-bottom: 4px;">API Port</label>
              <input type="number" id="cfg-port" class="search-input-box" value="51121">
            </div>
          </div>
          <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
            <div>
              <label style="font-size: 0.75rem; color: var(--text-muted); font-weight: 700; display: block; margin-bottom: 4px;">Router Username</label>
              <input type="text" id="cfg-user" class="search-input-box" value="salman">
            </div>
            <div>
              <label style="font-size: 0.75rem; color: var(--text-muted); font-weight: 700; display: block; margin-bottom: 4px;">Router Password</label>
              <input type="password" id="cfg-pass" class="search-input-box" placeholder="Password router">
            </div>
          </div>
          <button onclick="saveAllCreds()" class="btn-exec" style="margin-top: 10px; justify-content: center; padding: 10px;">
            💾 Simpan Kredensial ke Browser
          </button>
        </div>
      </div>
    </div>

  </main>

  <script>
    const ENDPOINTS = [
      {
        id: 'ep-overview',
        cat: 'fast-path',
        method: 'POST',
        path: '/api/v1/overview',
        desc: 'Snapshot agregasi cepat: CPU, memory, uptime, total user hotspot, ppp, dan leases dalam < 2ms.',
        defaultBody: '{}'
      },
      {
        id: 'ep-devices',
        cat: 'fast-path',
        method: 'POST',
        path: '/api/v1/network/connected-devices',
        desc: 'Korelasikan perangkat terhubung dari seluruh layer (DHCP, ARP, Hotspot, WiFi, PPPoE).',
        defaultBody: '{}'
      },
      {
        id: 'ep-topology',
        cat: 'fast-path',
        method: 'POST',
        path: '/api/v1/network/topology-graph',
        desc: 'Data graf simpul relasi (Nodes & Edges) jaringan untuk visualizer topologi.',
        defaultBody: '{}'
      },
      {
        id: 'ep-hs-active',
        cat: 'hotspot',
        method: 'POST',
        path: '/api/v1/hotspot/active',
        desc: 'Daftar pengguna Hotspot yang sedang login aktif, IP, MAC address, uptime, dan bytes.',
        defaultBody: '{}'
      },
      {
        id: 'ep-hs-users',
        cat: 'hotspot',
        method: 'POST',
        path: '/api/v1/hotspot/users',
        desc: 'Daftar seluruh database user Hotspot di router.',
        defaultBody: '{}'
      },
      {
        id: 'ep-hs-batch',
        cat: 'hotspot',
        method: 'POST',
        path: '/api/v1/hotspot/generate-batch',
        desc: 'Generate massal voucher hotspot otomatis dengan prefix kode unik.',
        defaultBody: JSON.stringify({ qty: 5, prefix: "V-", profile: "default", uptime_limit: "1d" }, null, 2)
      },
      {
        id: 'ep-hs-wizard',
        cat: 'wizards',
        method: 'POST',
        path: '/api/v1/hotspot/wizard/setup',
        desc: '1-Klik Setup Hotspot: otomatis bridge, IP pool, DHCP server, DNS, dan Hotspot server.',
        defaultBody: JSON.stringify({ interface: "ether2", local_address: "192.168.50.1/24", dhcp_pool_range: "192.168.50.10-192.168.50.254", dns_name: "login.wifi" }, null, 2)
      },
      {
        id: 'ep-ppp-secrets',
        cat: 'ppp',
        method: 'POST',
        path: '/api/v1/ppp/secrets',
        desc: 'Daftar akun pelanggan PPPoE secrets di router.',
        defaultBody: '{}'
      },
      {
        id: 'ep-ppp-active',
        cat: 'ppp',
        method: 'POST',
        path: '/api/v1/ppp/active',
        desc: 'Daftar pelanggan PPPoE yang sedang online aktif.',
        defaultBody: '{}'
      },
      {
        id: 'ep-ppp-isolate',
        cat: 'ppp',
        method: 'POST',
        path: '/api/v1/ppp/customer/isolate',
        desc: 'Isolir pelanggan PPPoE yang menunggak: ubah profil ke ISOLIR dan putus sesi aktif.',
        defaultBody: JSON.stringify({ name: "user_pelanggan_1", isolate_profile: "ISOLIR" }, null, 2)
      },
      {
        id: 'ep-sec-audit',
        cat: 'security',
        method: 'POST',
        path: '/api/v1/security/vulnerability-audit',
        desc: 'Audit kerentanan keamanan port sensitif terbuka (Telnet, FTP, WWW) dan saran pengetatan.',
        defaultBody: '{}'
      },
      {
        id: 'ep-sec-brute',
        cat: 'security',
        method: 'POST',
        path: '/api/v1/security/deploy-antibruteforce',
        desc: 'Terapkan aturan firewall Anti-Bruteforce berlapis untuk SSH, FTP, dan Winbox.',
        defaultBody: JSON.stringify({ service: "all", ban_time: "1d" }, null, 2)
      },
      {
        id: 'ep-sec-appblock',
        cat: 'security',
        method: 'POST',
        path: '/api/v1/security/app-block',
        desc: 'Blokir konten / aplikasi spesifik (whatsapp, tiktok, youtube, judi, torrent).',
        defaultBody: JSON.stringify({ app_type: "whatsapp", action: "drop" }, null, 2)
      },
      {
        id: 'ep-sec-anti-tether',
        cat: 'security',
        method: 'POST',
        path: '/api/v1/security/anti-tethering/enable',
        desc: 'Aktifkan aturan Anti-Tethering (Change TTL=1) agar user hotspot tidak bisa bagi wifi hotspot.',
        defaultBody: JSON.stringify({ hotspot_interface: "bridge" }, null, 2)
      },
      {
        id: 'ep-lb-pcc',
        cat: 'load-balance',
        method: 'POST',
        path: '/api/v1/load-balance/pcc/setup',
        desc: '1-Klik Multi-WAN PCC Load Balance: otomatis buat mangle rules dan failover route.',
        defaultBody: JSON.stringify({ lan_interface: "bridge", wans: [{ interface: "ether1", gateway: "192.168.1.1", weight: 1 }, { interface: "ether2", gateway: "192.168.2.1", weight: 1 }] }, null, 2)
      },
      {
        id: 'ep-lb-status',
        cat: 'load-balance',
        method: 'POST',
        path: '/api/v1/load-balance/status',
        desc: 'Cek status metrik gateway Multi-WAN.',
        defaultBody: '{}'
      },
      {
        id: 'ep-nat-port-forward',
        cat: 'wizards',
        method: 'POST',
        path: '/api/v1/firewall/port-forward',
        desc: '1-Klik Port Forwarding (Dst-NAT) ke web server atau CCTV internal.',
        defaultBody: JSON.stringify({ dst_port: "8080", to_addresses: "192.168.88.50", to_ports: "80", protocol: "tcp" }, null, 2)
      },
      {
        id: 'ep-traffic-game',
        cat: 'queues',
        method: 'POST',
        path: '/api/v1/traffic/preset/game-social-separation',
        desc: 'Pemisah trafik game vs sosmed: otomatis set mangle port game prioritas dan Queue Tree.',
        defaultBody: JSON.stringify({ total_bandwidth: "50M", game_reserved: "10M" }, null, 2)
      },
      {
        id: 'ep-queues-inspect',
        cat: 'queues',
        method: 'POST',
        path: '/api/v1/queues/inspect-user',
        desc: 'Inspeksi limit kecepatan dan pemakaian user berdasarkan IP address.',
        defaultBody: JSON.stringify({ query: "172.16.10.68" }, null, 2)
      },
      {
        id: 'ep-queues-summary',
        cat: 'queues',
        method: 'POST',
        path: '/api/v1/queues/overview-summary',
        desc: 'Rekapitulasi total bandwidth dan daftar Top Downloaders jaringan saat ini.',
        defaultBody: '{}'
      },
      {
        id: 'ep-if-all',
        cat: 'interfaces',
        method: 'POST',
        path: '/api/v1/interfaces/all',
        desc: 'Daftar seluruh interfaces router (Ethernet, Bridge, WLAN, SFP, VLAN).',
        defaultBody: '{}'
      },
      {
        id: 'ep-tools-ping',
        cat: 'tools',
        method: 'POST',
        path: '/api/v1/tools/ping',
        desc: 'Eksekusi ICMP Ping dari router MikroTik ke target luar (RTT latency & packet loss).',
        defaultBody: JSON.stringify({ address: "8.8.8.8", count: 3 }, null, 2)
      },
      {
        id: 'ep-tools-torch',
        cat: 'tools',
        method: 'POST',
        path: '/api/v1/tools/torch',
        desc: 'Live Torch Packet Sniffer pada interface router.',
        defaultBody: JSON.stringify({ interface: "ether1", duration: 3 }, null, 2)
      },
      {
        id: 'ep-tools-romon',
        cat: 'tools',
        method: 'POST',
        path: '/api/v1/tools/romon/status',
        desc: 'Cek status Router Management Overlay Network (RoMON).',
        defaultBody: '{}'
      },
      {
        id: 'ep-sys-res',
        cat: 'system',
        method: 'POST',
        path: '/api/v1/system/resource',
        desc: 'Informasi sistem komprehensif: CPU load, free memory, routeros version, board name.',
        defaultBody: '{}'
      },
      {
        id: 'ep-sys-identity',
        cat: 'system',
        method: 'POST',
        path: '/api/v1/system/identity',
        desc: 'Ambil nama identitas router saat ini.',
        defaultBody: '{}'
      },
      {
        id: 'ep-raw-cmd',
        cat: 'raw',
        method: 'POST',
        path: '/api/v1/command',
        desc: 'Universal Raw Command: jalankan perintah RouterOS API arbitrer apapun dengan parameter bebas.',
        defaultBody: JSON.stringify({ command: "/system/resource/print", params: {} }, null, 2)
      }
    ];

    let currentCategory = 'all';

    function renderEndpointsList(items) {
      const container = document.getElementById('endpoint-list-container');
      container.innerHTML = '';

      if (items.length === 0) {
        container.innerHTML = '<div style="text-align: center; padding: 40px; color: var(--text-muted);">Tidak ada endpoint yang cocok.</div>';
        return;
      }

      items.forEach(ep => {
        const card = document.createElement('div');
        card.className = 'ep-card';
        card.id = ep.id;

        const mClass = ep.method === 'POST' ? 'm-post' : (ep.method === 'GET' ? 'm-get' : 'm-ws');

        card.innerHTML = `
          <div class="ep-head">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span class="ep-badge-method ${mClass}">${ep.method}</span>
              <span class="ep-path">${ep.path}</span>
            </div>
            <div class="ep-actions">
              <button class="btn-snippet" onclick="copySnippet('${ep.path}', 'curl', '${ep.id}')">📋 cURL</button>
              <button class="btn-snippet" onclick="copySnippet('${ep.path}', 'fetch', '${ep.id}')">📜 Fetch</button>
              <button class="btn-snippet" onclick="copySnippet('${ep.path}', 'php', '${ep.id}')">🐘 Laravel</button>
              <button class="btn-snippet" onclick="copySnippet('${ep.path}', 'dart', '${ep.id}')">💙 Flutter</button>
            </div>
          </div>
          <div class="ep-desc">${ep.desc}</div>
          <div>
            <textarea id="${ep.id}-body" class="ep-body-editor" placeholder="JSON Request Body">${ep.defaultBody}</textarea>
          </div>
          <div style="display: flex; justify-content: flex-end;">
            <button class="btn-exec" onclick="executeEndpoint('${ep.path}', '${ep.id}')">
              ▶️ Test Endpoint Ini
            </button>
          </div>
        `;
        container.appendChild(card);
      });
    }

    async function executeEndpoint(path, epId) {
      syncCredsFromBar();
      const token = document.getElementById('g-token').value;
      const host = document.getElementById('r-host').value;
      const port = document.getElementById('r-port').value;
      const user = document.getElementById('r-user').value;
      const pass = document.getElementById('r-pass').value;

      let payload = {};
      try {
        const raw = document.getElementById(`${epId}-body`).value.trim();
        if (raw) payload = JSON.parse(raw);
      } catch (e) {
        alert("Payload JSON tidak valid: " + e.message);
        return;
      }

      const consoleBox = document.getElementById('console-body');
      const consoleMeta = document.getElementById('console-meta');
      consoleBox.textContent = `Menghubungkan ke ${host}:${port} via Rust Connection Pool...\nMengirim request ke: ${path}`;
      consoleMeta.textContent = "Sedang memproses...";

      const t0 = performance.now();
      try {
        const res = await fetch(path, {
          method: 'POST',
          headers: {
            'Authorization': 'Bearer ' + token,
            'X-Router-Host': host,
            'X-Router-Port': port,
            'X-Router-User': user,
            'X-Router-Pass': pass,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify(payload)
        });
        const t1 = performance.now();
        const data = await res.json();
        const duration = (t1 - t0).toFixed(1);

        consoleMeta.innerHTML = `<span style="color: ${res.status === 200 ? '#34d399' : '#f43f5e'}; font-weight: 700;">Status: ${res.status}</span> | Latency: <strong>${duration} ms</strong>`;
        consoleBox.textContent = JSON.stringify(data, null, 2);
      } catch (err) {
        consoleMeta.textContent = "Error Jaringan / Timeout";
        consoleBox.textContent = "Gagal memanggil API: " + err.message;
      }
    }

    function copySnippet(path, type, epId) {
      syncCredsFromBar();
      const token = document.getElementById('g-token').value;
      const host = document.getElementById('r-host').value;
      const port = document.getElementById('r-port').value;
      const user = document.getElementById('r-user').value;
      const pass = document.getElementById('r-pass').value;
      const body = document.getElementById(`${epId}-body`).value.trim().replace(/\n\s*/g, '');

      let text = '';
      if (type === 'curl') {
        text = `curl -X POST "https://ros-gateway.samrifa.com${path}" \\
  -H "Authorization: Bearer ${token}" \\
  -H "X-Router-Host: ${host}" \\
  -H "X-Router-Port: ${port}" \\
  -H "X-Router-User: ${user}" \\
  -H "X-Router-Pass: ${pass}" \\
  -H "Content-Type: application/json" \\
  -d '${body}'`;
      } else if (type === 'fetch') {
        text = `const res = await fetch("https://ros-gateway.samrifa.com${path}", {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer ${token}',
    'X-Router-Host': '${host}',
    'X-Router-Port': '${port}',
    'X-Router-User': '${user}',
    'X-Router-Pass': '${pass}',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify(${body || '{}'})
});
const data = await res.json();`;
      } else if (type === 'php') {
        text = `use Illuminate\\Support\\Facades\\Http;

$response = Http::withHeaders([
    'Authorization' => 'Bearer ${token}',
    'X-Router-Host' => '${host}',
    'X-Router-Port' => '${port}',
    'X-Router-User' => '${user}',
    'X-Router-Pass' => '${pass}',
    'Content-Type'  => 'application/json',
])->post('https://ros-gateway.samrifa.com${path}', ${body || '[]'});
$data = $response->json();`;
      } else if (type === 'dart') {
        text = `import 'dart:convert';
import 'package:http/http.dart' as http;

final res = await http.post(
  Uri.parse('https://ros-gateway.samrifa.com${path}'),
  headers: {
    'Authorization': 'Bearer ${token}',
    'X-Router-Host': '${host}',
    'X-Router-Port': '${port}',
    'X-Router-User': '${user}',
    'X-Router-Pass': '${pass}',
    'Content-Type': 'application/json',
  },
  body: jsonEncode(${body || '{}'}),
);`;
      }

      navigator.clipboard.writeText(text);
      alert(`Snippet ${type.toUpperCase()} berhasil disalin ke clipboard!`);
    }

    function filterList() {
      const q = document.getElementById('search-box').value.toLowerCase().trim();
      const filtered = ENDPOINTS.filter(ep => {
        const mCat = currentCategory === 'all' || ep.cat === currentCategory;
        const mQuery = !q || ep.path.toLowerCase().includes(q) || ep.desc.toLowerCase().includes(q) || ep.cat.toLowerCase().includes(q);
        return mCat && mQuery;
      });
      renderEndpointsList(filtered);
    }

    function setCategory(cat, el) {
      currentCategory = cat;
      document.querySelectorAll('#cat-pills .cat-pill').forEach(p => p.classList.remove('active'));
      el.classList.add('active');
      filterList();
    }

    function switchTab(tabId) {
      document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
      event.currentTarget.classList.add('active');

      document.getElementById('view-explorer').style.display = tabId === 'explorer' ? 'grid' : 'none';
      document.getElementById('view-docs').style.display = tabId === 'docs' ? 'flex' : 'none';
      document.getElementById('view-topology').style.display = tabId === 'topology' ? 'flex' : 'none';
      document.getElementById('view-settings').style.display = tabId === 'settings' ? 'flex' : 'none';
    }

    function switchLang(lang, el) {
      document.querySelectorAll('.lang-tab').forEach(b => b.classList.remove('active'));
      el.classList.add('active');
      const box = document.getElementById('lang-code-box');
      if (lang === 'php') {
        box.textContent = `use Illuminate\\Support\\Facades\\Http;

$response = Http::withHeaders([
    'Authorization' => 'Bearer change-me-to-a-long-random-string',
    'X-Router-Host' => 'ath.vpnbersama.us',
    'X-Router-Port' => '51121',
    'X-Router-User' => 'salman',
    'X-Router-Pass' => 'password_anda',
    'Content-Type'  => 'application/json',
])->post('https://ros-gateway.samrifa.com/api/v1/hotspot/active');

$users = $response->json();`;
      } else if (lang === 'js') {
        box.textContent = `const response = await fetch('https://ros-gateway.samrifa.com/api/v1/hotspot/active', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer change-me-to-a-long-random-string',
    'X-Router-Host': 'ath.vpnbersama.us',
    'X-Router-Port': '51121',
    'X-Router-User': 'salman',
    'X-Router-Pass': 'password_anda',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({})
});
const users = await response.json();`;
      } else if (lang === 'dart') {
        box.textContent = `import 'dart:convert';
import 'package:http/http.dart' as http;

final res = await http.post(
  Uri.parse('https://ros-gateway.samrifa.com/api/v1/hotspot/active'),
  headers: {
    'Authorization': 'Bearer change-me-to-a-long-random-string',
    'X-Router-Host': 'ath.vpnbersama.us',
    'X-Router-Port': '51121',
    'X-Router-User': 'salman',
    'X-Router-Pass': 'password_anda',
    'Content-Type': 'application/json',
  },
  body: jsonEncode({}),
);
final users = jsonDecode(res.body);`;
      } else if (lang === 'python') {
        box.textContent = `import requests

res = requests.post(
    'https://ros-gateway.samrifa.com/api/v1/hotspot/active',
    headers={
        'Authorization': 'Bearer change-me-to-a-long-random-string',
        'X-Router-Host': 'ath.vpnbersama.us',
        'X-Router-Port': '51121',
        'X-Router-User': 'salman',
        'X-Router-Pass': 'password_anda',
    },
    json={}
)
users = res.json()`;
      } else if (lang === 'curl') {
        box.textContent = `curl -X POST "https://ros-gateway.samrifa.com/api/v1/hotspot/active" \\
  -H "Authorization: Bearer change-me-to-a-long-random-string" \\
  -H "X-Router-Host: ath.vpnbersama.us" \\
  -H "X-Router-Port: 51121" \\
  -H "X-Router-User: salman" \\
  -H "X-Router-Pass: password_anda" \\
  -H "Content-Type: application/json" \\
  -d '{}'`;
      }
    }

    function syncCredsFromBar() {
      localStorage.setItem('ros_token', document.getElementById('g-token').value);
      localStorage.setItem('ros_host', document.getElementById('r-host').value);
      localStorage.setItem('ros_port', document.getElementById('r-port').value);
      localStorage.setItem('ros_user', document.getElementById('r-user').value);
      localStorage.setItem('ros_pass', document.getElementById('r-pass').value);
    }

    function loadSavedCreds() {
      if (localStorage.getItem('ros_token')) {
        document.getElementById('g-token').value = localStorage.getItem('ros_token');
        document.getElementById('cfg-token').value = localStorage.getItem('ros_token');
      }
      if (localStorage.getItem('ros_host')) {
        document.getElementById('r-host').value = localStorage.getItem('ros_host');
        document.getElementById('cfg-host').value = localStorage.getItem('ros_host');
      }
      if (localStorage.getItem('ros_port')) {
        document.getElementById('r-port').value = localStorage.getItem('ros_port');
        document.getElementById('cfg-port').value = localStorage.getItem('ros_port');
      }
      if (localStorage.getItem('ros_user')) {
        document.getElementById('r-user').value = localStorage.getItem('ros_user');
        document.getElementById('cfg-user').value = localStorage.getItem('ros_user');
      }
      if (localStorage.getItem('ros_pass')) {
        document.getElementById('r-pass').value = localStorage.getItem('ros_pass');
        document.getElementById('cfg-pass').value = localStorage.getItem('ros_pass');
      }
    }

    function saveAllCreds() {
      document.getElementById('g-token').value = document.getElementById('cfg-token').value;
      document.getElementById('r-host').value = document.getElementById('cfg-host').value;
      document.getElementById('r-port').value = document.getElementById('cfg-port').value;
      document.getElementById('r-user').value = document.getElementById('cfg-user').value;
      document.getElementById('r-pass').value = document.getElementById('cfg-pass').value;
      syncCredsFromBar();
      alert("Kredensial berhasil disimpan di browser (localStorage)!");
      document.querySelectorAll('.tab-btn')[0].click();
    }

    function openVisualizerDirect() {
      syncCredsFromBar();
      const host = document.getElementById('r-host').value;
      const port = document.getElementById('r-port').value;
      const user = document.getElementById('r-user').value;
      const pass = document.getElementById('r-pass').value;
      const token = document.getElementById('g-token').value;
      window.open(`/topology?host=${encodeURIComponent(host)}&port=${encodeURIComponent(port)}&user=${encodeURIComponent(user)}&pass=${encodeURIComponent(pass)}&token=${encodeURIComponent(token)}`, '_blank');
    }

    function copyConsoleOutput() {
      const text = document.getElementById('console-body').textContent;
      navigator.clipboard.writeText(text);
      alert("Respons JSON disalin ke clipboard!");
    }

    window.addEventListener('DOMContentLoaded', () => {
      loadSavedCreds();
      ['g-token', 'r-host', 'r-port', 'r-user', 'r-pass'].forEach(id => {
        document.getElementById(id).addEventListener('input', syncCredsFromBar);
      });
      renderEndpointsList(ENDPOINTS);
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

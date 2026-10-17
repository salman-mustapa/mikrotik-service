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
    /* Live Real-Time WebSocket Terminal Modal */
    .modal-backdrop {
      display: none;
      position: fixed;
      top: 0; left: 0; right: 0; bottom: 0;
      background: rgba(15, 23, 42, 0.72);
      backdrop-filter: blur(8px);
      -webkit-backdrop-filter: blur(8px);
      z-index: 9999;
      align-items: center;
      justify-content: center;
      padding: 20px;
    }
    .modal-backdrop.show {
      display: flex !important;
    }
    .modal-card {
      background: #ffffff;
      border-radius: 16px;
      box-shadow: 0 25px 50px -12px rgba(15, 23, 42, 0.35), 0 0 0 1px rgba(226, 232, 240, 0.8);
      width: 100%;
      max-width: 920px;
      display: flex;
      flex-direction: column;
      overflow: hidden;
      animation: modal-pop 0.22s cubic-bezier(0.16, 1, 0.3, 1);
    }
    @keyframes modal-pop {
      from { transform: scale(0.96) translateY(8px); opacity: 0; }
      to { transform: scale(1) translateY(0); opacity: 1; }
    }
    .modal-header {
      padding: 16px 24px;
      background: #ffffff;
      border-bottom: 1px solid var(--card-border);
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .btn-close-modal {
      width: 32px;
      height: 32px;
      border-radius: 8px;
      border: 1px solid var(--card-border);
      background: #f8fafc;
      color: var(--text-muted);
      font-size: 1.35rem;
      line-height: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      cursor: pointer;
      transition: all 0.15s ease;
    }
    .btn-close-modal:hover {
      background: #fee2e2;
      border-color: #fca5a5;
      color: #dc2626;
      transform: scale(1.05);
    }
    .modal-tabs {
      display: flex;
      background: #f1f5f9;
      padding: 6px 20px 0;
      border-bottom: 1px solid var(--card-border);
      gap: 6px;
    }
    .modal-tab-btn {
      padding: 8px 16px;
      border: none;
      background: transparent;
      font-size: 0.78rem;
      font-weight: 700;
      color: var(--text-muted);
      cursor: pointer;
      border-top-left-radius: 8px;
      border-top-right-radius: 8px;
      display: flex;
      align-items: center;
      gap: 6px;
      transition: all 0.15s ease;
    }
    .modal-tab-btn:hover {
      color: var(--text-main);
    }
    .modal-tab-btn.active {
      background: #ffffff;
      color: var(--accent-indigo);
      box-shadow: 0 -2px 5px rgba(0,0,0,0.03);
    }
    .ws-controls {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      padding: 12px 24px;
      background: #f8fafc;
      border-bottom: 1px solid var(--card-border);
      flex-wrap: wrap;
    }
    .ws-input {
      background: #ffffff;
      border: 1px solid #cbd5e1;
      border-radius: 8px;
      padding: 8px 12px;
      font-size: 0.8rem;
      font-family: inherit;
      color: #0f172a;
      outline: none;
      transition: border-color 0.15s, box-shadow 0.15s;
    }
    .ws-input:focus {
      border-color: var(--accent-indigo);
      box-shadow: 0 0 0 3px rgba(79, 70, 229, 0.15);
    }
    .ws-btn {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 8px 16px;
      border-radius: 8px;
      font-size: 0.8rem;
      font-weight: 700;
      cursor: pointer;
      transition: all 0.15s ease;
      border: none;
    }
    .ws-btn-start {
      background: #4f46e5;
      color: #ffffff;
      box-shadow: 0 1px 3px rgba(79, 70, 229, 0.35);
    }
    .ws-btn-start:hover {
      background: #4338ca;
    }
    .ws-btn-stop {
      background: #ef4444 !important;
      color: #ffffff !important;
      box-shadow: 0 1px 3px rgba(239, 68, 68, 0.35);
    }
    .ws-btn-stop:hover {
      background: #dc2626 !important;
    }
    .ws-btn-clear {
      background: #ffffff;
      color: #475569;
      border: 1px solid #cbd5e1;
    }
    .ws-btn-clear:hover {
      background: #f1f5f9;
      color: #0f172a;
    }
    .ws-terminal-screen {
      background: #090d16;
      color: #38bdf8;
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.78rem;
      padding: 16px 20px;
      height: 350px;
      max-height: 350px;
      overflow-y: auto;
      line-height: 1.6;
      box-sizing: border-box;
      box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.5);
    }
    .ws-line {
      margin-bottom: 4px;
      word-break: break-all;
    }
    .ws-line.comment {
      color: #64748b;
    }
    .ws-line.success {
      color: #34d399;
      font-weight: 600;
    }
    .ws-line.error {
      color: #f87171;
      font-weight: 600;
    }
    .ws-line.normal {
      color: #38bdf8;
    }
    .modal-footer {
      padding: 12px 24px;
      background: #ffffff;
      border-top: 1px solid var(--card-border);
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 12px;
    }
    .ws-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: #94a3b8;
      display: inline-block;
      transition: all 0.2s;
    }
    .ws-dot.active {
      background: #10b981;
      box-shadow: 0 0 8px #10b981;
      animation: pulse-dot 1.5s infinite ease-in-out;
    }
    @keyframes pulse-dot {
      0%, 100% { transform: scale(1); opacity: 1; }
      50% { transform: scale(1.3); opacity: 0.7; }
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
      <span class="badge-engine" title="Internal Rust micro-socket binary processing time (&lt;2ms) for atomic RouterOS queries. ICMP ping batch takes ~3.5s due to 1s network probe intervals."><span class="dot-live"></span>Rust Socket Engine: &lt;2ms</span>
      <button class="nav-btn" onclick="openWsModal()" style="background: #ecfdf5; border-color: #a7f3d0; color: #047857; font-weight: 700; cursor: pointer;">⚡ Live WebSocket Terminal</button>
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
        <div class="kpi-title">Identitas Router <span id="kpi-board" style="color: var(--accent-indigo); text-transform: none;">-</span></div>
        <div class="kpi-val" id="kpi-identity">Belum Terhubung</div>
        <div class="kpi-sub" id="kpi-ros">Masukkan host &amp; klik Sinkronisasi</div>
      </div>

      <div class="kpi-card">
        <div class="kpi-title">CPU Load &amp; Frekuensi <span id="kpi-freq" style="text-transform: none;">- MHz</span></div>
        <div class="kpi-val" id="kpi-cpu">0%</div>
        <div class="progress-bar-wrap">
          <div class="progress-fill" id="kpi-cpu-bar" style="width: 0%;"></div>
        </div>
      </div>

      <div class="kpi-card">
        <div class="kpi-title">Memori RAM Bebas <span id="kpi-total-ram" style="text-transform: none;">- MB</span></div>
        <div class="kpi-val" id="kpi-free-ram">- MB</div>
        <div class="kpi-sub" id="kpi-mem-percent">Penggunaan RAM: -%</div>
      </div>

      <div class="kpi-card">
        <div class="kpi-title">Klien &amp; Relasi Jaringan <span id="kpi-clients-status" style="color: var(--text-muted); text-transform: none;">● Menunggu Data</span></div>
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

        <!-- Category Filter Pills (Standardized English Nomenclature) -->
        <div class="category-pills">
          <button class="cat-btn active" onclick="filterCategory('all', this)">All Endpoints (250+)</button>
          <button class="cat-btn" onclick="filterCategory('security', this)">🛡️ Security &amp; Diagnostics</button>
          <button class="cat-btn" onclick="filterCategory('fast', this)">🚀 Fast-Path &amp; Performance</button>
          <button class="cat-btn" onclick="filterCategory('voucher', this)">🎟️ Voucher Engine 2.0</button>
          <button class="cat-btn" onclick="filterCategory('notify', this)">🤖 Bot &amp; Netwatch Automation</button>
          <button class="cat-btn" onclick="filterCategory('loadbalance', this)">🔀 Load Balancing &amp; NAT</button>
          <button class="cat-btn" onclick="filterCategory('vpn', this)">🔐 VPN &amp; WireGuard</button>
          <button class="cat-btn" onclick="filterCategory('queues', this)">📊 QoS &amp; Traffic Shaping</button>
          <button class="cat-btn" onclick="filterCategory('tools', this)">🛠️ NOC Diagnostic Tools</button>
          <button class="cat-btn" onclick="filterCategory('maintenance', this)">💾 Backup &amp; Maintenance</button>
          <button class="cat-btn" onclick="filterCategory('hotspot', this)">🔥 Hotspot Captive Portal</button>
          <button class="cat-btn" onclick="filterCategory('dhcp', this)">💻 DHCP &amp; IPAM</button>
          <button class="cat-btn" onclick="filterCategory('pppoe', this)">🌐 PPPoE &amp; ISP Access</button>
          <button class="cat-btn" onclick="filterCategory('ip', this)">📡 IP &amp; Routing Tables</button>
          <button class="cat-btn" onclick="filterCategory('system', this)">⚙️ System &amp; Hardware</button>
          <button class="cat-btn" onclick="filterCategory('net', this)">🗺️ Network Topology &amp; Discovery</button>
          <button class="cat-btn" onclick="openWsModal()" style="background: #eff6ff; border-color: #bfdbfe; color: #1d4ed8; font-weight: 700;">⚡ Real-Time WebSocket</button>
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

  <!-- Live WebSocket Real-Time Terminal Modal -->
  <div id="ws-modal" class="modal-backdrop" onclick="onBackdropClick(event)">
    <div class="modal-card" onclick="event.stopPropagation()">
      <div class="modal-header">
        <div>
          <h3 style="font-size: 1.05rem; font-weight: 700; color: #1e293b; display: flex; align-items: center; gap: 8px;">
            <span>⚡ Real-Time WebSocket Streaming Engine</span>
            <span style="font-size: 0.68rem; padding: 2px 8px; border-radius: 999px; background: #dcfce7; color: #15803d; font-weight: 600;">Sub-2ms Duplex Channel</span>
          </h3>
          <p style="font-size: 0.76rem; color: #64748b; margin-top: 2px;">
            Direct bi-directional duplex stream to RouterOS via <code>/ws</code>. Mengalirkan respons data paket demi paket per detik secara instan tanpa polling HTTP.
          </p>
        </div>
        <button class="btn-close-modal" onclick="closeWsModal()" title="Tutup Modal (Esc)">&times;</button>
      </div>

      <!-- Navigation Tabs inside Modal -->
      <div class="modal-tabs">
        <button class="modal-tab-btn active" id="tab-btn-terminal" onclick="switchWsModalTab('terminal')">🖥️ Live Stream Terminal</button>
        <button class="modal-tab-btn" id="tab-btn-docs" onclick="switchWsModalTab('docs')">💻 Developer Integration Guide &amp; SDK</button>
      </div>

      <!-- Tab Content 1: Live Terminal -->
      <div id="ws-pane-terminal">
        <div class="ws-controls">
          <div style="display: flex; gap: 8px; flex-wrap: wrap; align-items: center;">
            <select id="ws-stream-type" class="ws-input" onchange="onWsStreamTypeChange()">
              <option value="ping">Live ICMP Ping Stream (RouterOS -> Target Host)</option>
              <option value="traffic">Real-Time Interface Bandwidth (RX/TX Bps)</option>
              <option value="overview">System Overview Live Heartbeat (CPU &amp; RAM)</option>
              <option value="logs">Live RouterOS Syslog Stream</option>
            </select>
            <input id="ws-target-param" class="ws-input" type="text" placeholder="Target Host / IP (e.g. 8.8.8.8)" value="8.8.8.8" style="min-width: 180px;">
          </div>
          <div style="display: flex; gap: 8px; align-items: center;">
            <button id="ws-connect-btn" class="ws-btn ws-btn-start" onclick="toggleWsStream()">
              <span>▶ Mulai Stream Real-Time</span>
            </button>
            <button class="ws-btn ws-btn-clear" onclick="clearWsTerminal()">Clear Terminal</button>
          </div>
        </div>

        <div class="ws-terminal-screen" id="ws-terminal">
          <div class="ws-line comment"># RouterOS Universal Gateway Live WebSocket Terminal</div>
          <div class="ws-line comment"># Protocol: ws://${location.host}/ws?token=&lt;bearer_token&gt;</div>
          <div class="ws-line comment"># Socket Engine: Async Rust Multiplexed Channel (&lt;2ms)</div>
          <div class="ws-line comment"># Ready. Klik 'Mulai Stream Real-Time' untuk menguji koneksi live socket duplex.</div>
        </div>
      </div>

      <!-- Tab Content 2: Developer Integration Docs -->
      <div id="ws-pane-docs" style="display: none; padding: 20px; max-height: 400px; overflow-y: auto; background: #ffffff;">
        <div style="font-size: 0.88rem; font-weight: 700; color: #1e293b; margin-bottom: 6px;">🔌 Panduan Integrasi WebSocket untuk Developer &amp; Proyek</div>
        <p style="font-size: 0.78rem; color: #64748b; line-height: 1.5; margin-bottom: 14px;">
          Koneksi WebSocket gateway (<code>/ws</code>) dirancang khusus untuk integrasi aplikasi mobile (Flutter), dashboard web (React/Vue/Next), atau backend microservice yang membutuhkan live packet streaming (ICMP ping per detik, traffic interface real-time, telemetry hardware, dan live syslog) tanpa beban HTTP polling berulang.
        </p>

        <div style="font-size: 0.78rem; font-weight: 700; color: #334155; margin-bottom: 8px;">Contoh Kode Klien Siap Pakai:</div>
        <div class="tab-bar" style="margin-bottom: 8px;">
          <button class="code-tab-btn active" id="ws-sdk-tab-flutter" onclick="switchWsSnippet('flutter', this)">Flutter (Dart / web_socket_channel)</button>
          <button class="code-tab-btn" id="ws-sdk-tab-js" onclick="switchWsSnippet('js', this)">JavaScript (Browser / Node.js)</button>
          <button class="code-tab-btn" id="ws-sdk-tab-python" onclick="switchWsSnippet('python', this)">Python (websockets)</button>
        </div>
        <pre class="code-box" id="ws-snippet-box" style="margin: 0; max-height: 220px;"></pre>
      </div>

      <div class="modal-footer">
        <div style="display: flex; align-items: center; gap: 8px;">
          <span class="ws-dot" id="ws-status-dot"></span>
          <span style="font-weight: 600; color: #475569; font-size: 0.78rem;" id="ws-status-text">Disconnected</span>
        </div>
        <div style="font-size: 0.74rem; color: #64748b;">
          💡 Endpoint: <code>ws://{location.host}/ws?token={bearer_token}</code> &bull; Tekan <kbd style="background: #f1f5f9; padding: 2px 6px; border-radius: 4px; border: 1px solid #cbd5e1; font-size: 0.7rem; color: #475569;">Esc</kbd> atau klik luar untuk menutup
        </div>
      </div>
    </div>
  </div>

  <script>
    const ENDPOINTS = [
      // --- Fast-Path & Performance (cat: fast) ---
      { id: "overview", path: "/api/v1/overview", method: "GET", cat: "fast", desc: "Snapshot agregasi cepat: CPU, memory, uptime, total user hotspot, ppp, dan leases dalam <2ms.", defaultPayload: "{}" },
      { id: "connected_devices", path: "/api/v1/network/connected-devices", method: "GET", cat: "fast", desc: "Korelasikan perangkat terhubung dari seluruh layer (DHCP, ARP, Hotspot, WiFi, PPPoE).", defaultPayload: "{}" },
      { id: "fasttrack_status", path: "/api/v1/system/fasttrack/status", method: "GET", cat: "fast", desc: "Status aturan FastTrack Hardware Acceleration RouterOS, packet counter, dan status bypass.", defaultPayload: "{}" },
      { id: "fasttrack_deploy", path: "/api/v1/system/fasttrack/deploy", method: "POST", cat: "fast", desc: "1-Klik Deploy Rule FastTrack IPv4 Connection Tracking: Bypass firewall untuk established/related packets, memangkas CPU load hingga 80%.", defaultPayload: "{}" },
      { id: "cpu_profiler", path: "/api/v1/system/cpu-profiler", method: "GET", cat: "fast", desc: "Pemeriksa profil CPU live & pendeteksi bottleneck proses RouterOS (networking, firewall, profiling).", defaultPayload: "{}" },

      // --- Security & Diagnostics (cat: security) ---
      { id: "doctor_diagnose", path: "/api/v1/doctor/diagnose", method: "POST", cat: "security", desc: "AI Heuristic Network Doctor: Audit 6 pilar vital (CPU, RAM, WAN Ping, Throttled Queues, FastTrack, AP/LAN) & rekomendasi perbaikan instan.", defaultPayload: "{}" },
      { id: "sec_vuln_audit", path: "/api/v1/security/vulnerability-audit", method: "GET", cat: "security", desc: "Audit kerentanan CVE RouterOS, open service sniffing (Telnet, FTP, API), & Open DNS Resolver.", defaultPayload: "{}" },
      { id: "sec_antibruteforce", path: "/api/v1/security/deploy-antibruteforce", method: "POST", cat: "security", desc: "1-Klik pasang firewall filter anti brute-force attack bertingkat untuk WinBox, SSH, & API.", defaultPayload: '{\n  "protect_winbox": true,\n  "protect_ssh": true,\n  "protect_api": true,\n  "blacklist_timeout": "7d"\n}' },
      { id: "app_block", path: "/api/v1/security/app-block", method: "POST", cat: "security", desc: "1-Klik blokir Layer 7 / TLS host untuk TikTok, YouTube, WhatsApp, Judi Online, atau Torrent P2P.", defaultPayload: '{\n  "app_type": "judi_online",\n  "comment": "Blokir Judol"\n}' },
      { id: "app_unblock", path: "/api/v1/security/app-unblock", method: "POST", cat: "security", desc: "Buka kembali blokir aplikasi / domain di firewall.", defaultPayload: '{\n  "app_type": "judi_online"\n}' },
      { id: "app_blocked_list", path: "/api/v1/security/blocked-apps", method: "GET", cat: "security", desc: "Daftar seluruh rule pemblokiran aplikasi dan domain aktif di firewall.", defaultPayload: "{}" },
      { id: "anti_tethering_enable", path: "/api/v1/security/anti-tethering/enable", method: "POST", cat: "security", desc: "Proteksi anti-tethering hotspot: Paksa TTL=1 di Mangle agar klien tidak bisa berbagi hotspot via HP.", defaultPayload: '{\n  "comment": "Lock TTL to prevent tethering"\n}' },
      { id: "anti_tethering_status", path: "/api/v1/security/anti-tethering/status", method: "GET", cat: "security", desc: "Status proteksi anti-tethering hotspot (aktif / tidak).", defaultPayload: "{}" },
      { id: "anti_tethering_disable", path: "/api/v1/security/anti-tethering/disable", method: "POST", cat: "security", desc: "Nonaktifkan proteksi anti-tethering hotspot.", defaultPayload: "{}" },
      { id: "sec_port_knocking_setup", path: "/api/v1/security/port-knocking/setup", method: "POST", cat: "security", desc: "1-Klik setup 3-Step Port Knocking (amankan port Winbox 8291 & SSH 22).", defaultPayload: '{\n  "knock_port_1": 7000,\n  "knock_port_2": 8000,\n  "knock_port_3": 9000,\n  "target_ports": "8291,22",\n  "access_duration": "1h"\n}' },
      { id: "sec_port_knocking_status", path: "/api/v1/security/port-knocking/status", method: "GET", cat: "security", desc: "Status port knocking dan daftar IP admin yang sedang memiliki akses aktif.", defaultPayload: "{}" },
      { id: "sec_port_knocking_remove", path: "/api/v1/security/port-knocking/remove", method: "POST", cat: "security", desc: "Hapus rule port knocking dan bersihkan konfigurasi stealth access.", defaultPayload: "{}" },
      { id: "sec_bogon_filter", path: "/api/v1/security/hardening/deploy-bogon-filter", method: "POST", cat: "security", desc: "Deploy filter drop paket IP Bogon & Martian di interface WAN.", defaultPayload: '{\n  "wan_interface": "ether1"\n}' },
      { id: "sec_dns_guard", path: "/api/v1/security/hardening/deploy-dns-guard", method: "POST", cat: "security", desc: "Blokir celah DNS Open Resolver port 53 dari internet WAN.", defaultPayload: '{\n  "wan_interface": "ether1"\n}' },
      { id: "sec_raw_antiddos", path: "/api/v1/security/hardening/deploy-raw-antiddos", method: "POST", cat: "security", desc: "Raw Firewall Anti-DDoS sebelum Connection Tracking (MTCTCE).", defaultPayload: '{\n  "wan_interface": "ether1"\n}' },
      { id: "sec_auto_reboot", path: "/api/v1/security/hardening/setup-auto-reboot", method: "POST", cat: "security", desc: "Jadwalkan auto-reboot router harian jam 04:00 subuh.", defaultPayload: '{\n  "reboot_time": "04:00:00"\n}' },
      { id: "sec_full_hardening", path: "/api/v1/security/hardening/deploy-full-hardening", method: "POST", cat: "security", desc: "1-Klik deploy seluruh paket hardening & auto-reboot sekaligus.", defaultPayload: '{\n  "wan_interface": "ether1",\n  "enable_auto_reboot": true\n}' },
      { id: "sec_hardening_audit", path: "/api/v1/security/hardening/audit-status", method: "GET", cat: "security", desc: "Audit status kepatuhan enterprise security hardening router.", defaultPayload: "{}" },
      { id: "expert_diag", path: "/api/v1/expert/quick-diagnose", method: "POST", cat: "security", desc: "Expert deep diagnostic analyzer untuk performa dan latency router.", defaultPayload: "{}" },
      { id: "expert_traffic", path: "/api/v1/expert/traffic-matrix", method: "POST", cat: "security", desc: "Matriks persebaran trafik komprehensif antar interface.", defaultPayload: "{}" },
      { id: "expert_security", path: "/api/v1/expert/security-audit", method: "POST", cat: "security", desc: "Comprehensive expert firewall and security posture analyzer.", defaultPayload: "{}" },
      { id: "fw_filters", path: "/api/v1/firewall/filters", method: "GET", cat: "security", desc: "Daftar seluruh rule Firewall Filter IPv4 (input, forward, output).", defaultPayload: "{}" },
      { id: "fw_nat", path: "/api/v1/firewall/nat", method: "GET", cat: "security", desc: "Daftar seluruh rule Firewall NAT (srcnat & dstnat).", defaultPayload: "{}" },
      { id: "fw_address_lists", path: "/api/v1/firewall/address-lists", method: "GET", cat: "security", desc: "Daftar tabel Address-List firewall (BLACKLIST, ISOLIR, WHITELIST).", defaultPayload: "{}" },
      { id: "fw_block_ip", path: "/api/v1/firewall/block-ip", method: "POST", cat: "security", desc: "Blokir IP instan: Tambahkan ke address-list BLACKLIST dan pastikan rule drop forward aktif.", defaultPayload: '{\n  "address": "192.168.88.99",\n  "list": "BLACKLIST",\n  "comment": "Abuse client",\n  "ensure_drop_rule": true\n}' },
      { id: "fw_unblock_ip", path: "/api/v1/firewall/unblock-ip", method: "POST", cat: "security", desc: "Buka blokir IP target secara langsung via alamat IP atau ID.", defaultPayload: '{\n  "address": "192.168.88.99",\n  "list": "BLACKLIST"\n}' },
      { id: "fw_conntrack", path: "/api/v1/firewall/connections", method: "GET", cat: "security", desc: "Inspeksi tabel Connection Tracking (Conntrack) aktif di router.", defaultPayload: "{}" },
      { id: "fw_top_talkers", path: "/api/v1/firewall/connections/top-talkers", method: "GET", cat: "security", desc: "Top Talkers Conntrack: Perangkat yang memiliki koneksi terbanyak.", defaultPayload: "{}" },
      { id: "fw_conntrack_flush", path: "/api/v1/firewall/connections/flush", method: "POST", cat: "security", desc: "Flush/reset tabel koneksi conntrack aktif.", defaultPayload: '{\n  "src_address": "192.168.88.99"\n}' },

      // --- Voucher Engine 2.0 (cat: voucher) ---
      { id: "vouchers_gen", path: "/api/v1/hotspot/vouchers/generate", method: "POST", cat: "voucher", desc: "Ultra-fast parallel batch voucher generator (1-1000 voucher, ROS v6/v7 safe, custom prefix, time/quota limit).", defaultPayload: '{\n  "profile": "1JAM-3RB",\n  "count": 5,\n  "prefix": "HOT-",\n  "user_mode": "same",\n  "charset": "alphanumeric_lower",\n  "length": 6,\n  "timelimit": "1h",\n  "datalimit": "500M",\n  "price": 3000,\n  "selling_price": 3000\n}' },
      { id: "vouchers_track", path: "/api/v1/hotspot/vouchers/tracking", method: "GET", cat: "voucher", desc: "Tracking lifecycle & sisa kuota/waktu voucher presisi real-time (available, online, used, expired).", defaultPayload: "{}" },
      { id: "vouchers_thermal", path: "/api/v1/hotspot/vouchers/thermal-print", method: "POST", cat: "voucher", desc: "Formatter struk termal ESC/POS (58mm/80mm) siap cetak via printer Bluetooth / USB.", defaultPayload: '{\n  "vouchers": [\n    { "username": "HOT-a89f2", "password": "HOT-a89f2", "profile": "1JAM-3RB", "timelimit": "1h", "price": 3000 }\n  ],\n  "hotspot_name": "HOTSPOT ENTERPRISE",\n  "dns_name": "hotspot.local",\n  "paper_width": "58mm"\n}' },
      { id: "vouchers_sell", path: "/api/v1/hotspot/vouchers/sell", method: "POST", cat: "voucher", desc: "Tandai voucher terjual di kasir dengan nama operator dan nominal rupiah.", defaultPayload: '{\n  "username": "HOT-a89f2",\n  "cashier": "Kasir-1",\n  "price": 3000\n}' },
      { id: "vouchers_sell_send", path: "/api/v1/hotspot/vouchers/sell-and-send", method: "POST", cat: "voucher", desc: "POS Kasir: Tandai voucher terjual dan kirim struk instan ke nomor WhatsApp pelanggan via Gowa API.", defaultPayload: '{\n  "username": "HOT-a89f2",\n  "phone": "081234567890",\n  "cashier": "Admin Kasir",\n  "hotspot_name": "HOTSPOT ENTERPRISE",\n  "login_url": "http://hotspot.local/login"\n}' },
      { id: "vouchers_sales_report", path: "/api/v1/hotspot/vouchers/sales-report", method: "GET", cat: "voucher", desc: "Mikhmon-Style Financial & Sales Revenue Summary: rekap omset rupiah harian & bulanan, voucher terjual, dan breakdown per paket.", defaultPayload: "{}" },
      { id: "vouchers_clean", path: "/api/v1/hotspot/vouchers/clean-expired", method: "POST", cat: "voucher", desc: "Pembersihan aman voucher kedaluwarsa tanpa menghapus voucher yang belum digunakan.", defaultPayload: '{\n  "force": true\n}' },
      { id: "voucher_template_render", path: "/api/v1/hotspot/voucher-template/render", method: "POST", cat: "voucher", desc: "Render lembaran cetak voucher visual berformat HTML (58mm, 80mm, atau A4 Grid) dengan QR Code.", defaultPayload: '{\n  "theme": "modern",\n  "paper_width": "80mm",\n  "hotspot_name": "WAROENG HOTSPOT",\n  "vouchers": [\n    { "username": "HOT-9012", "password": "pass-9012", "profile": "1JAM-3RB", "timelimit": "1h", "price": 3000 }\n  ]\n}' },
      { id: "mikhmon_lock_mac", path: "/api/v1/hotspot/mikhmon-v4/lock-mac/enable", method: "POST", cat: "voucher", desc: "Auto-Lock MAC Address pada Hotspot profile (Anti tukar/bagi-bagi voucher).", defaultPayload: '{\n  "profile_name": "default"\n}' },
      { id: "mikhmon_lock_mac_now", path: "/api/v1/hotspot/mikhmon-v4/lock-mac/bind-now", method: "POST", cat: "voucher", desc: "Kunci MAC address user hotspot target ke entri user saat ini juga.", defaultPayload: '{\n  "username": "HOT-a89f2",\n  "mac_address": "AA:BB:CC:DD:EE:FF"\n}' },
      { id: "mikhmon_expiry_setup", path: "/api/v1/hotspot/mikhmon-v4/expiry-mode/setup", method: "POST", cat: "voucher", desc: "Setup 3-Mode Expiry Voucher: Remove, Notice (isolir web peringatan), atau Record.", defaultPayload: '{\n  "profile_name": "default",\n  "expiry_mode": "notice",\n  "validity": "1d",\n  "notice_profile": "EXPIRED_NOTICE"\n}' },
      { id: "mikhmon_trial_status", path: "/api/v1/hotspot/mikhmon-v4/trial/status", method: "GET", cat: "voucher", desc: "Status konfigurasi login gratisan (Trial) di Hotspot Server Profile.", defaultPayload: "{}" },
      { id: "mikhmon_trial_setup", path: "/api/v1/hotspot/mikhmon-v4/trial/setup", method: "POST", cat: "voucher", desc: "Setup batasan durasi trial gratisan dan waktu reset per MAC.", defaultPayload: '{\n  "enable_trial": true,\n  "trial_uptime_limit": "15m",\n  "trial_uptime_reset": "1d"\n}' },
      { id: "mikhmon_trial_reset", path: "/api/v1/hotspot/mikhmon-v4/trial/reset-cookies", method: "POST", cat: "voucher", desc: "Reset seluruh cookies MAC trial agar pengunjung dapat mencoba login kembali.", defaultPayload: "{}" },

      // --- Bot & Netwatch Automation (cat: notify) ---
      { id: "integ_tg_send", path: "/api/v1/integrations/telegram/send", method: "POST", cat: "notify", desc: "Kirim pesan/alert instan ke Telegram Bot via socket fetch RouterOS.", defaultPayload: '{\n  "bot_token": "YOUR_BOT_TOKEN",\n  "chat_id": "123456789",\n  "text": "🚨 *ALERT NOC*: Link Internet Utama DOWN!"\n}' },
      { id: "integ_wa_send", path: "/api/v1/integrations/whatsapp/send", method: "POST", cat: "notify", desc: "Kirim notifikasi pesan WhatsApp langsung via Gowa API service.", defaultPayload: '{\n  "phone": "081234567890",\n  "message": "Halo, voucher internet hotspot Anda: *HOT-9812* berlaku 24 Jam.",\n  "gowa_url": "http://127.0.0.1:3000"\n}' },
      { id: "integ_multi_notify", path: "/api/v1/integrations/notify", method: "POST", cat: "notify", desc: "Multi-platform notification dispatcher (Telegram + WhatsApp) dengan severity level.", defaultPayload: '{\n  "channel": "all",\n  "severity": "WARNING",\n  "title": "Koneksi AP Terganggu",\n  "message": "AP Ruang Meeting down sejak 2 menit yang lalu.",\n  "telegram_bot_token": "",\n  "telegram_chat_id": "",\n  "whatsapp_phone": ""\n}' },
      { id: "integ_netwatch_setup", path: "/api/v1/integrations/netwatch/setup", method: "POST", cat: "notify", desc: "Setup probe Netwatch RouterOS otomatis dengan skrip alert instan ke Telegram & WhatsApp.", defaultPayload: '{\n  "host": "192.168.88.2",\n  "comment": "AP-HOTSPOT-1",\n  "interval": "00:00:10",\n  "timeout": "1000ms",\n  "telegram_bot_token": "BOT_TOKEN",\n  "telegram_chat_id": "CHAT_ID"\n}' },
      { id: "integ_netwatch_batch", path: "/api/v1/integrations/netwatch/batch-setup", method: "POST", cat: "notify", desc: "Otomatisasi pengawasan Netwatch armada seluruh Access Point (AP Fleet) sekaligus.", defaultPayload: '{\n  "items": [\n    { "host": "192.168.88.2", "comment": "AP-HOTSPOT-1" },\n    { "host": "192.168.88.3", "comment": "AP-HOTSPOT-2" },\n    { "host": "192.168.88.4", "comment": "AP-HOTSPOT-3" }\n  ],\n  "telegram_bot_token": "BOT_TOKEN",\n  "telegram_chat_id": "CHAT_ID"\n}' },
      { id: "integ_netwatch_list", path: "/api/v1/integrations/netwatch/monitors", method: "GET", cat: "notify", desc: "Daftar seluruh probe Netwatch, status up/down/since, dan skrip alert terpasang.", defaultPayload: "{}" },
      { id: "integ_netwatch_toggle", path: "/api/v1/integrations/netwatch/toggle", method: "POST", cat: "notify", desc: "Enable atau Disable probe Netwatch target pada router.", defaultPayload: '{\n  "id": "*1",\n  "disabled": false\n}' },
      { id: "integ_netwatch_remove", path: "/api/v1/integrations/netwatch/remove", method: "POST", cat: "notify", desc: "Hapus probe Netwatch spesifik dari router.", defaultPayload: '{\n  "id": "*1"\n}' },
      { id: "integ_reports_sched", path: "/api/v1/integrations/reports/setup-scheduler", method: "POST", cat: "notify", desc: "Pasang skrip & scheduler otomatis RouterOS untuk rekap laporan harian NOC ke Telegram.", defaultPayload: '{\n  "name": "noc-daily-report",\n  "start_time": "07:00:00",\n  "interval": "1d",\n  "telegram_bot_token": "BOT_TOKEN",\n  "telegram_chat_id": "CHAT_ID"\n}' },
      { id: "integ_chat_snippet", path: "/api/v1/integrations/hotspot-chat/snippet", method: "GET", cat: "notify", desc: "Snippet embed live chat widget (Intergram Telegram & WhatsApp) untuk login.html & status.html hotspot.", defaultPayload: "{}" },
      { id: "tg_send_message", path: "/api/v1/telegram/send-message", method: "POST", cat: "notify", desc: "Kirim pesan Telegram melalui bot bridge internal gateway.", defaultPayload: '{\n  "chat_id": "123456",\n  "message": "Tes Alert dari Gateway"\n}' },

      // --- Load Balancing & NAT (cat: loadbalance) ---
      { id: "lb_pcc_setup", path: "/api/v1/load-balance/pcc/setup", method: "POST", cat: "loadbalance", desc: "Automated Dual-WAN PCC Wizard: Buat otomatis rule Mangle PCC, routing marks, & failover 2-4 ISP.", defaultPayload: '{\n  "wan_interfaces": ["ether1", "ether2"],\n  "gateways": ["192.168.1.1", "192.168.2.1"],\n  "lan_interface": "bridge-lan"\n}' },
      { id: "lb_status", path: "/api/v1/load-balance/status", method: "GET", cat: "loadbalance", desc: "Status konfigurasi Multi-WAN PCC Load Balancer aktif di router.", defaultPayload: "{}" },
      { id: "lb_remove", path: "/api/v1/load-balance/remove", method: "POST", cat: "loadbalance", desc: "Hapus seluruh rule Load Balancing PCC dan bersihkan firewall Mangle.", defaultPayload: "{}" },
      { id: "failover_setup", path: "/api/v1/routing/failover/setup", method: "POST", cat: "loadbalance", desc: "1-Klik setup Recursive Gateway Failover via DNS ping 8.8.8.8 & 1.1.1.1 (MTCRE).", defaultPayload: '{\n  "primary_gateway": "192.168.1.1",\n  "backup_gateway": "192.168.2.1",\n  "primary_check_host": "8.8.8.8",\n  "backup_check_host": "1.1.1.1"\n}' },
      { id: "failover_status", path: "/api/v1/routing/failover/status", method: "GET", cat: "loadbalance", desc: "Cek status rute failover rekursif aktif vs rute cadangan.", defaultPayload: "{}" },
      { id: "failover_remove", path: "/api/v1/routing/failover/remove", method: "POST", cat: "loadbalance", desc: "Hapus seluruh rute failover rekursif bertag [Recursive-Failover].", defaultPayload: "{}" },
      { id: "nat_port_forward", path: "/api/v1/firewall/port-forward", method: "POST", cat: "loadbalance", desc: "Port Forwarding Wizard (dst-nat) lengkap dengan opsi Hairpin NAT untuk akses Web/CCTV dari luar LAN.", defaultPayload: '{\n  "protocol": "tcp",\n  "dst_port": "8080",\n  "to_addresses": "192.168.88.50",\n  "to_ports": "80",\n  "in_interface": "ether1",\n  "enable_hairpin": true,\n  "comment": "Web Server Forward"\n}' },
      { id: "nat_port_forward_list", path: "/api/v1/firewall/port-forward/list", method: "GET", cat: "loadbalance", desc: "Daftar seluruh rule port forwarding (dst-nat) aktif di router.", defaultPayload: "{}" },
      { id: "nat_port_forward_remove", path: "/api/v1/firewall/port-forward/remove", method: "POST", cat: "loadbalance", desc: "Hapus rule port forwarding berdasarkan ID router.", defaultPayload: '{\n  "id": "*1"\n}' },
      { id: "nat_hairpin", path: "/api/v1/firewall/hairpin-nat", method: "POST", cat: "loadbalance", desc: "Setup rule Hairpin NAT otomatis untuk subnet LAN internal.", defaultPayload: '{\n  "lan_subnet": "192.168.88.0/24",\n  "server_ip": "192.168.88.50"\n}' },
      { id: "nat_masquerade", path: "/api/v1/firewall/srcnat/masquerade", method: "POST", cat: "loadbalance", desc: "Buat rule Masquerade WAN standar agar LAN dapat terhubung ke internet.", defaultPayload: '{\n  "out_interface": "ether1",\n  "comment": "Default Internet NAT"\n}' },

      // --- VPN & WireGuard (cat: vpn) ---
      { id: "wg_interfaces", path: "/api/v1/wireguard/interfaces", method: "GET", cat: "vpn", desc: "Daftar interface WireGuard VPN di router (RouterOS v7).", defaultPayload: "{}" },
      { id: "wg_peers", path: "/api/v1/wireguard/peers", method: "GET", cat: "vpn", desc: "Daftar peer WireGuard (klien & site-to-site) aktif di router.", defaultPayload: "{}" },
      { id: "wg_peer_add", path: "/api/v1/wireguard/peer/add", method: "POST", cat: "vpn", desc: "Tambah peer WireGuard baru dengan public key & allowed-address.", defaultPayload: '{\n  "interface": "wireguard1",\n  "public_key": "YOUR_PEER_PUBLIC_KEY=",\n  "allowed_address": "10.0.0.2/32"\n}' },
      { id: "wg_peer_remove", path: "/api/v1/wireguard/peer/remove", method: "POST", cat: "vpn", desc: "Hapus peer WireGuard berdasarkan ID atau public key.", defaultPayload: '{\n  "id": "*1"\n}' },
      { id: "vpn_sstp", path: "/api/v1/vpn/sstp", method: "GET", cat: "vpn", desc: "Konfigurasi server SSTP VPN (Port 443 SSL).", defaultPayload: "{}" },
      { id: "vpn_sstp_set", path: "/api/v1/vpn/sstp/set", method: "POST", cat: "vpn", desc: "Aktifkan atau ubah opsi server SSTP VPN.", defaultPayload: '{\n  "enabled": true,\n  "port": 443\n}' },
      { id: "vpn_l2tp", path: "/api/v1/vpn/l2tp", method: "GET", cat: "vpn", desc: "Konfigurasi server L2TP VPN dan status IPsec encryption.", defaultPayload: "{}" },
      { id: "vpn_l2tp_set", path: "/api/v1/vpn/l2tp/set", method: "POST", cat: "vpn", desc: "Aktifkan atau ubah opsi server L2TP VPN dan secret IPsec.", defaultPayload: '{\n  "enabled": true,\n  "use_ipsec": "yes",\n  "ipsec_secret": "securepresharedkey"\n}' },
      { id: "vpn_ovpn", path: "/api/v1/vpn/ovpn", method: "GET", cat: "vpn", desc: "Konfigurasi server OpenVPN (TCP/UDP) di router.", defaultPayload: "{}" },
      { id: "vpn_ovpn_set", path: "/api/v1/vpn/ovpn/set", method: "POST", cat: "vpn", desc: "Aktifkan atau ubah mode server OpenVPN router.", defaultPayload: '{\n  "enabled": true,\n  "mode": "ip",\n  "port": 1194\n}' },
      { id: "vpn_eoip", path: "/api/v1/vpn/eoip", method: "GET", cat: "vpn", desc: "Daftar tunnel EoIP (Ethernet over IP) untuk bridging L2 antar cabang.", defaultPayload: "{}" },
      { id: "vpn_eoip_add", path: "/api/v1/vpn/eoip/add", method: "POST", cat: "vpn", desc: "Buat tunnel EoIP baru dengan tunnel-id dan remote-address.", defaultPayload: '{\n  "name": "eoip-cabang",\n  "tunnel_id": 10,\n  "remote_address": "203.0.113.5"\n}' },
      { id: "vpn_eoip_remove", path: "/api/v1/vpn/eoip/remove", method: "POST", cat: "vpn", desc: "Hapus interface tunnel EoIP.", defaultPayload: '{\n  "name": "eoip-cabang"\n}' },
      { id: "vpn_gre", path: "/api/v1/vpn/gre", method: "GET", cat: "vpn", desc: "Daftar GRE Tunnels (Generic Routing Encapsulation) aktif di router.", defaultPayload: "{}" },
      { id: "vpn_ipip", path: "/api/v1/vpn/ipip", method: "GET", cat: "vpn", desc: "Daftar IPIP Tunnels (IP Encapsulation within IP).", defaultPayload: "{}" },

      // --- QoS & Traffic Shaping (cat: queues) ---
      { id: "queues_simple", path: "/api/v1/queues/simple", method: "GET", cat: "queues", desc: "Daftar seluruh Simple Queues, target IP, max-limit (upload/download), & bytes terpakai.", defaultPayload: "{}" },
      { id: "queues_summary", path: "/api/v1/queues/overview-summary", method: "GET", cat: "queues", desc: "Ringkasan total kuota, agregasi kecepatan, dan status throttling seluruh antrean.", defaultPayload: "{}" },
      { id: "queues_simple_add", path: "/api/v1/queues/simple/add", method: "POST", cat: "queues", desc: "Tambah antrean Simple Queue baru dengan limit bandwidth target.", defaultPayload: '{\n  "name": "User-Kantor-1",\n  "target": "192.168.88.20/32",\n  "max_limit": "5M/10M",\n  "comment": "Staff Kantor"\n}' },
      { id: "queues_set_limit", path: "/api/v1/queues/simple/set-limit", method: "POST", cat: "queues", desc: "Ubah batas bandwidth Simple Queue target secara langsung.", defaultPayload: '{\n  "name": "User-Kantor-1",\n  "max_limit": "10M/20M"\n}' },
      { id: "queues_remove", path: "/api/v1/queues/simple/remove", method: "POST", cat: "queues", desc: "Hapus Simple Queue spesifik berdasarkan ID atau nama.", defaultPayload: '{\n  "name": "User-Kantor-1"\n}' },
      { id: "queues_inspect", path: "/api/v1/queues/inspect-user", method: "POST", cat: "queues", desc: "Inspeksi utilisasi antrean bandwidth per user atau IP spesifik.", defaultPayload: '{\n  "target": "192.168.88.20"\n}' },
      { id: "queues_test_limit", path: "/api/v1/queues/test-limit", method: "POST", cat: "queues", desc: "Uji coba validitas format limit kecepatan (misal 512k/2M).", defaultPayload: '{\n  "limit": "5M/10M"\n}' },
      { id: "queues_burst_calc", path: "/api/v1/queues/burst-calculator", method: "POST", cat: "queues", desc: "Kalkulator & Formatter formula Burst Rate-Limit MikroTik (Max Limit, Burst, Threshold, Time, Priority).", defaultPayload: '{\n  "max_limit": "2M/2M",\n  "burst_limit": "5M/5M",\n  "burst_time": "8/8",\n  "threshold_percent": 75\n}' },
      { id: "traffic_preset_game", path: "/api/v1/traffic/preset/game-social-separation", method: "POST", cat: "queues", desc: "1-Klik Smart Traffic Shaper: Pisahkan trafik Game Online (prioritas tinggi) dari Sosmed/Download via Mangle & Queue Tree.", defaultPayload: '{\n  "wan_interface": "ether1",\n  "lan_interface": "bridge-lan"\n}' },
      { id: "traffic_queue_tree", path: "/api/v1/traffic/queue-tree", method: "GET", cat: "queues", desc: "Daftar Queue Tree hierarkis (Parent & Child queues) di router.", defaultPayload: "{}" },
      { id: "traffic_queue_tree_add", path: "/api/v1/traffic/queue-tree/add", method: "POST", cat: "queues", desc: "Tambah child queue baru pada Queue Tree dengan packet mark.", defaultPayload: '{\n  "name": "Game-Traffic",\n  "parent": "global",\n  "packet_mark": "game_pkt",\n  "limit_at": "5M",\n  "max_limit": "10M",\n  "priority": 1\n}' },
      { id: "traffic_mangle_rules", path: "/api/v1/traffic/mangle/rules", method: "GET", cat: "queues", desc: "Daftar rule Firewall Mangle untuk penandaan paket (packet marks & routing marks).", defaultPayload: "{}" },
      { id: "traffic_mangle_add", path: "/api/v1/traffic/mangle/add", method: "POST", cat: "queues", desc: "Tambah penandaan paket baru di tabel firewall mangle.", defaultPayload: '{\n  "chain": "forward",\n  "protocol": "udp",\n  "dst_port": "5000-5500",\n  "action": "mark-packet",\n  "new_packet_mark": "game_pkt",\n  "passthrough": true\n}' },

      // --- NOC Diagnostic Tools (cat: tools) ---
      { id: "tools_ping", path: "/api/v1/tools/ping", method: "POST", cat: "tools", desc: "Kirim batch ICMP Ping langsung dari router ke target host/IP dengan metrik latency (min/avg/max) & packet loss.", defaultPayload: '{\n  "address": "8.8.8.8",\n  "count": 4\n}' },
      { id: "tools_multi_ping", path: "/api/v1/tools/multi-ping", method: "GET", cat: "tools", desc: "Ping matriks multi-target sekaligus (Gateway, DNS Cloudflare, Google, OpenDNS) dengan ringkasan SLA.", defaultPayload: '{\n  "targets": ["1.1.1.1", "8.8.8.8", "208.67.222.222"],\n  "count": 3\n}' },
      { id: "tools_traceroute", path: "/api/v1/tools/traceroute", method: "POST", cat: "tools", desc: "Jalankan Traceroute ICMP/UDP dari router untuk melacak hop rute ke internet.", defaultPayload: '{\n  "address": "8.8.8.8",\n  "count": 3\n}' },
      { id: "noc_torch", path: "/api/v1/tools/torch", method: "POST", cat: "tools", desc: "Torch live bandwidth monitor per-IP, protocol, & port pada interface target.", defaultPayload: '{\n  "interface": "ether1",\n  "duration_seconds": 3\n}' },
      { id: "noc_btest", path: "/api/v1/tools/bandwidth-test", method: "POST", cat: "tools", desc: "MikroTik Bandwidth Test internal ke router tujuan (TCP/UDP speed test).", defaultPayload: '{\n  "address": "192.168.88.2",\n  "direction": "both",\n  "duration": 5\n}' },
      { id: "noc_ip_scan", path: "/api/v1/tools/ip-scan", method: "POST", cat: "tools", desc: "IP Scan: Deteksi seluruh perangkat aktif dan MAC address pada subnet interface.", defaultPayload: '{\n  "interface": "bridge-lan"\n}' },
      { id: "noc_profile", path: "/api/v1/tools/profile", method: "POST", cat: "tools", desc: "CPU Profile Detective (/tool profile): Profil mikro beban CPU per-proses (firewall, networking, console, queuing).", defaultPayload: '{\n  "duration": 3\n}' },
      { id: "tools_sniffer_status", path: "/api/v1/tools/sniffer/status", method: "GET", cat: "tools", desc: "Status Packet Sniffer RouterOS (running, buffer, limit).", defaultPayload: "{}" },
      { id: "tools_sniffer_start", path: "/api/v1/tools/sniffer/start", method: "POST", cat: "tools", desc: "Mulai packet capture ke file .pcap atau streaming TZSP ke PC Wireshark.", defaultPayload: '{\n  "interface": "all",\n  "file_name": "capture.pcap",\n  "file_limit": "2000KiB"\n}' },
      { id: "tools_sniffer_stop", path: "/api/v1/tools/sniffer/stop", method: "POST", cat: "tools", desc: "Hentikan packet sniffer.", defaultPayload: "{}" },
      { id: "tools_graphing_status", path: "/api/v1/tools/graphing/status", method: "GET", cat: "tools", desc: "Status konfigurasi grafik historis web RouterOS (/graphs).", defaultPayload: "{}" },
      { id: "tools_graphing_enable", path: "/api/v1/tools/graphing/enable", method: "POST", cat: "tools", desc: "Aktifkan grafik web MRTG untuk interface, queue bandwidth, dan CPU.", defaultPayload: '{\n  "enable_interfaces": true,\n  "interface": "all",\n  "enable_queues": true,\n  "enable_resource": true\n}' },
      { id: "tools_graphing_disable", path: "/api/v1/tools/graphing/disable", method: "POST", cat: "tools", desc: "Nonaktifkan grafik historis web MRTG.", defaultPayload: "{}" },
      { id: "noc_romon_discover", path: "/api/v1/tools/romon/discover", method: "GET", cat: "tools", desc: "Temukan router MikroTik tetangga di jaringan RoMON (Router Management Overlay Network).", defaultPayload: "{}" },
      { id: "noc_romon_status", path: "/api/v1/tools/romon/status", method: "GET", cat: "tools", desc: "Status service RoMON pada router.", defaultPayload: "{}" },
      { id: "noc_watchdog", path: "/api/v1/system/watchdog", method: "GET", cat: "tools", desc: "Status hardware Watchdog router (auto reboot saat kernel freeze).", defaultPayload: "{}" },
      { id: "noc_ntp_config", path: "/api/v1/system/ntp/config", method: "GET", cat: "tools", desc: "Konfigurasi NTP Client (Network Time Protocol) sinkronisasi jam router.", defaultPayload: "{}" },
      { id: "noc_terminal_exec", path: "/api/v1/scripts/terminal-exec", method: "POST", cat: "tools", desc: "Eksekusi baris perintah skrip RouterOS langsung dan ambil keluarannya.", defaultPayload: '{\n  "script": ":put [/system resource get cpu-load]"\n}' },

      // --- Backup & Maintenance (cat: maintenance) ---
      { id: "backup_create", path: "/api/v1/backup/create", method: "POST", cat: "maintenance", desc: "Buat file binary backup sistem router (.backup) di storage internal.", defaultPayload: '{\n  "name": "backup-gateway-manual"\n}' },
      { id: "backup_export", path: "/api/v1/backup/export", method: "POST", cat: "maintenance", desc: "Ekspor konfigurasi teks skrip RouterOS (.rsc) dengan hide-sensitive.", defaultPayload: '{\n  "file": "export-gateway.rsc",\n  "hide_sensitive": true\n}' },
      { id: "backup_restore", path: "/api/v1/backup/restore", method: "POST", cat: "maintenance", desc: "Pulihkan sistem router dari file binary (.backup) dan reboot otomatis.", defaultPayload: '{\n  "name": "backup-auto.backup"\n}' },
      { id: "backup_import", path: "/api/v1/backup/import", method: "POST", cat: "maintenance", desc: "Impor dan jalankan skrip konfigurasi RouterOS (.rsc) ke router.", defaultPayload: '{\n  "file": "config.rsc"\n}' },
      { id: "backup_files", path: "/api/v1/files/all", method: "GET", cat: "maintenance", desc: "Daftar seluruh file di storage router (backup, skin hotspot, cert, log).", defaultPayload: "{}" },
      { id: "backup_file_read", path: "/api/v1/files/read", method: "POST", cat: "maintenance", desc: "Baca isi file teks atau skrip dari storage router.", defaultPayload: '{\n  "file": "config.rsc"\n}' },
      { id: "backup_file_write", path: "/api/v1/files/write", method: "POST", cat: "maintenance", desc: "Tulis atau unggah teks file (skrip, custom HTML hotspot, config) langsung ke storage MikroTik.", defaultPayload: '{\n  "file": "hotspot/custom.txt",\n  "contents": "Hello MikroTik Storage"\n}' },
      { id: "backup_file_remove", path: "/api/v1/files/remove", method: "POST", cat: "maintenance", desc: "Hapus file dari storage router.", defaultPayload: '{\n  "file": "old-backup.backup"\n}' },
      { id: "maint_reboot", path: "/api/v1/system/reboot", method: "POST", cat: "maintenance", desc: "Reboot sistem router MikroTik secara graceful.", defaultPayload: "{}" },
      { id: "maint_shutdown", path: "/api/v1/system/shutdown", method: "POST", cat: "maintenance", desc: "Mematikan router MikroTik secara graceful dan aman.", defaultPayload: "{}" },
      { id: "maint_check_update", path: "/api/v1/system/check-update", method: "GET", cat: "maintenance", desc: "Periksa ketersediaan pembaruan RouterOS versi terbaru dari server MikroTik.", defaultPayload: "{}" },
      { id: "maint_install_update", path: "/api/v1/system/install-update", method: "POST", cat: "maintenance", desc: "Unduh dan pasang update RouterOS kemudian reboot router.", defaultPayload: "{}" },
      { id: "maint_health", path: "/api/v1/system/health", method: "GET", cat: "maintenance", desc: "Sensor voltase, suhu CPU, dan temperatur board router (RouterBOARD).", defaultPayload: "{}" },
      { id: "maint_pre_upgrade", path: "/api/v1/maintenance/pre-upgrade-snapshot", method: "POST", cat: "maintenance", desc: "Snapshot pra-upgrade: Simpan backup konfigurasi dan riwayat package sebelum OS di-upgrade.", defaultPayload: "{}" },
      { id: "maint_reset_config", path: "/api/v1/maintenance/reset-configuration", method: "POST", cat: "maintenance", desc: "Factory reset konfigurasi router (opsi no-defaults & keep-users).", defaultPayload: '{\n  "no_defaults": false,\n  "keep_users": true\n}' },
      { id: "audit_logs", path: "/api/v1/audit/logs", method: "GET", cat: "maintenance", desc: "Daftar riwayat audit log seluruh eksekusi API gateway.", defaultPayload: "{}" },
      { id: "audit_stats", path: "/api/v1/audit/stats", method: "GET", cat: "maintenance", desc: "Statistik performa gateway: total request, error rate, dan p95 latency.", defaultPayload: "{}" },
      { id: "audit_clear", path: "/api/v1/audit/clear", method: "POST", cat: "maintenance", desc: "Bersihkan rekaman riwayat audit log gateway.", defaultPayload: "{}" },

      // --- Hotspot Captive Portal (cat: hotspot) ---
      { id: "hs_active", path: "/api/v1/hotspot/active", method: "GET", cat: "hotspot", desc: "Daftar pengguna Hotspot yang sedang login aktif, IP, MAC address, uptime, dan bytes.", defaultPayload: "{}" },
      { id: "hs_users", path: "/api/v1/hotspot/users", method: "GET", cat: "hotspot", desc: "Daftar seluruh database user Hotspot dan voucher di router.", defaultPayload: "{}" },
      { id: "hs_user_create", path: "/api/v1/hotspot/user/create", method: "POST", cat: "hotspot", desc: "Tambah user hotspot baru dengan username, password, dan profile.", defaultPayload: '{\n  "name": "guest1",\n  "password": "pass",\n  "profile": "default"\n}' },
      { id: "hs_user_remove", path: "/api/v1/hotspot/user/remove", method: "POST", cat: "hotspot", desc: "Hapus akun user hotspot dari database router.", defaultPayload: '{\n  "name": "guest1"\n}' },
      { id: "hs_kick", path: "/api/v1/hotspot/kick", method: "POST", cat: "hotspot", desc: "Kick / diskoneksi sesi login user hotspot yang sedang online.", defaultPayload: '{\n  "user": "guest1"\n}' },
      { id: "hs_hosts", path: "/api/v1/hotspot/hosts", method: "GET", cat: "hotspot", desc: "Daftar seluruh perangkat host yang terdeteksi di subnet hotspot (authorized & unauthorized).", defaultPayload: "{}" },
      { id: "hs_host_bind", path: "/api/v1/hotspot/host/bind", method: "POST", cat: "hotspot", desc: "Bypass perangkat host spesifik agar bisa akses internet tanpa perlu login.", defaultPayload: '{\n  "mac_address": "AA:BB:CC:DD:EE:FF",\n  "type": "bypassed"\n}' },
      { id: "hs_host_remove", path: "/api/v1/hotspot/host/remove", method: "POST", cat: "hotspot", desc: "Hapus host terdeteksi dari tabel hotspot host.", defaultPayload: '{\n  "id": "*1"\n}' },
      { id: "hs_profiles", path: "/api/v1/hotspot/profiles", method: "GET", cat: "hotspot", desc: "Profil kecepatan dan batasan user Hotspot (rate-limit, shared-users).", defaultPayload: "{}" },
      { id: "hs_profile_add", path: "/api/v1/hotspot/profile/add", method: "POST", cat: "hotspot", desc: "Buat user profile hotspot baru dengan limit kecepatan.", defaultPayload: '{\n  "name": "Paket-Vip",\n  "rate_limit": "10M/10M",\n  "shared_users": "1"\n}' },
      { id: "hs_profile_remove", path: "/api/v1/hotspot/profile/remove", method: "POST", cat: "hotspot", desc: "Hapus user profile hotspot.", defaultPayload: '{\n  "name": "Paket-Vip"\n}' },
      { id: "hs_ip_bindings", path: "/api/v1/hotspot/ip-bindings", method: "GET", cat: "hotspot", desc: "Daftar IP/MAC binding Hotspot untuk bypass tanpa login (AP, Smart TV, CCTV).", defaultPayload: "{}" },
      { id: "hs_ip_binding_add", path: "/api/v1/hotspot/ip-binding/add", method: "POST", cat: "hotspot", desc: "Tambah entri IP/MAC binding bypass hotspot baru.", defaultPayload: '{\n  "mac_address": "AA:BB:CC:DD:EE:FF",\n  "type": "bypassed",\n  "comment": "Smart TV"\n}' },
      { id: "hs_ip_binding_remove", path: "/api/v1/hotspot/ip-binding/remove", method: "POST", cat: "hotspot", desc: "Hapus entri IP binding hotspot.", defaultPayload: '{\n  "id": "*1"\n}' },
      { id: "hs_walled_garden", path: "/api/v1/hotspot/walled-garden", method: "GET", cat: "hotspot", desc: "Daftar domain dan IP subnet yang di-bypass sebelum login (Payment Gateway, Webhook, QRIS).", defaultPayload: "{}" },
      { id: "hs_walled_garden_preset", path: "/api/v1/hotspot/walled-garden/deploy-preset", method: "POST", cat: "hotspot", desc: "1-Klik deploy preset bypass Walled Garden (Midtrans, Xendit, DANA, OVO, QRIS, WhatsApp API).", defaultPayload: '{\n  "preset": "payment_gateways"\n}' },
      { id: "hs_cookies", path: "/api/v1/hotspot/cookies", method: "GET", cat: "hotspot", desc: "Daftar active MAC/User cookie Hotspot tersimpan di router.", defaultPayload: "{}" },
      { id: "hs_reset_counters", path: "/api/v1/hotspot/user/reset-counters", method: "POST", cat: "hotspot", desc: "Reset traffic bytes dan uptime counter user hotspot tanpa menghapus voucher.", defaultPayload: '{\n  "name": "HOT-a89f2"\n}' },
      { id: "hs_inject_expiry", path: "/api/v1/hotspot/profile/inject-expiry-script", method: "POST", cat: "hotspot", desc: "Pasang script On-Login first login validity tracker otomatis pada user profile hotspot.", defaultPayload: '{\n  "profile_name": "1JAM-3RB",\n  "validity": "1d"\n}' },
      { id: "hs_wizard_setup", path: "/api/v1/hotspot/wizard/setup", method: "POST", cat: "hotspot", desc: "1-Klik Hotspot Wizard: Konfigurasi lengkap interface, IP pool, DNS, profile & login page otomatis.", defaultPayload: '{\n  "interface": "wlan1",\n  "dns_name": "hotspot.net",\n  "network": "192.168.10.0/24",\n  "gateway": "192.168.10.1"\n}' },
      { id: "hs_server_profiles", path: "/api/v1/hotspot/server-profiles", method: "GET", cat: "hotspot", desc: "Daftar Hotspot Server Profiles (html-directory, login-by, dns-name).", defaultPayload: "{}" },
      { id: "hs_server_profile_set", path: "/api/v1/hotspot/server-profile/set", method: "POST", cat: "hotspot", desc: "Ubah konfigurasi Hotspot Server Profile (folder HTML, DNS name, mode login).", defaultPayload: '{\n  "name": "hsprof1",\n  "html_directory": "hotspot",\n  "dns_name": "wifi.kafe.net",\n  "login_by": "cookie,http-chap,http-pap"\n}' },
      { id: "hs_servers", path: "/api/v1/hotspot/servers", method: "GET", cat: "hotspot", desc: "Daftar instance Hotspot Server aktif dan interface pengikatnya.", defaultPayload: "{}" },
      { id: "hs_server_set", path: "/api/v1/hotspot/server/set", method: "POST", cat: "hotspot", desc: "Ubah konfigurasi Hotspot Server instance (interface, pool, profil).", defaultPayload: '{\n  "name": "hotspot1",\n  "profile": "hsprof1"\n}' },
      { id: "hs_login_page_set", path: "/api/v1/hotspot/login-page/set", method: "POST", cat: "hotspot", desc: "Konfigurasi folder skin & upload template HTML halaman login captive portal Hotspot.", defaultPayload: '{\n  "profile_name": "hsprof1",\n  "html_directory": "hotspot",\n  "dns_name": "wifi.kafe.net",\n  "login_by": "cookie,http-chap,http-pap",\n  "html_content": "<!DOCTYPE html><html><head><title>WiFi Login</title></head><body><h1>Selamat Datang</h1><form name=\\"login\\" action=\\"$(link-login-only)\\" method=\\"post\\"><input type=\\"text\\" name=\\"username\\"><input type=\\"password\\" name=\\"password\\"><button type=\\"submit\\">Login</button></form></body></html>"\n}' },
      { id: "hs_members", path: "/api/v1/hotspot/members", method: "GET", cat: "hotspot", desc: "Daftar pelanggan member langganan hotspot bulanan/tetap.", defaultPayload: "{}" },
      { id: "hs_members_register", path: "/api/v1/hotspot/members/register", method: "POST", cat: "hotspot", desc: "Daftarkan pelanggan member hotspot baru dengan data kontak dan paket langganan.", defaultPayload: '{\n  "username": "member_budi",\n  "password": "pass",\n  "profile": "Member-1Bulan",\n  "phone": "08123456789"\n}' },
      { id: "hs_members_renew", path: "/api/v1/hotspot/members/renew", method: "POST", cat: "hotspot", desc: "Perpanjang masa aktif langganan member hotspot.", defaultPayload: '{\n  "username": "member_budi",\n  "extension_days": 30\n}' },
      { id: "userman_users", path: "/api/v1/user-manager/users", method: "GET", cat: "hotspot", desc: "Daftar seluruh user di database RADIUS User Manager internal MikroTik.", defaultPayload: "{}" },
      { id: "userman_profiles", path: "/api/v1/user-manager/profiles", method: "GET", cat: "hotspot", desc: "Daftar billing profile User Manager (RADIUS internal MikroTik).", defaultPayload: "{}" },
      { id: "userman_limitations", path: "/api/v1/user-manager/limitations", method: "GET", cat: "hotspot", desc: "Daftar limitations kecepatan & kuota paket User Manager.", defaultPayload: "{}" },
      { id: "userman_assign", path: "/api/v1/user-manager/user/assign-profile", method: "POST", cat: "hotspot", desc: "Assign billing profile/paket User Manager ke user.", defaultPayload: '{\n  "user": "user123",\n  "profile": "Paket-1Bulan"\n}' },

      // --- DHCP & IPAM (cat: dhcp) ---
      { id: "dhcp_leases", path: "/api/v1/dhcp/leases", method: "GET", cat: "dhcp", desc: "Daftar seluruh IP lease DHCP server, hostname klien, status bound, dan MAC address.", defaultPayload: "{}" },
      { id: "dhcp_lease_make_static", path: "/api/v1/dhcp/lease/make-static", method: "POST", cat: "dhcp", desc: "Ubah dynamic lease menjadi static IP lease permanen.", defaultPayload: '{\n  "id": "*1"\n}' },
      { id: "dhcp_lease_add", path: "/api/v1/dhcp/lease/add", method: "POST", cat: "dhcp", desc: "Tambah static DHCP lease manual dengan MAC dan IP address target.", defaultPayload: '{\n  "address": "192.168.88.50",\n  "mac_address": "AA:BB:CC:DD:EE:FF",\n  "comment": "Server NVR CCTV"\n}' },
      { id: "dhcp_lease_set", path: "/api/v1/dhcp/lease/set", method: "POST", cat: "dhcp", desc: "Ubah parameter static lease (comment, address, dll).", defaultPayload: '{\n  "id": "*1",\n  "comment": "Updated Server"\n}' },
      { id: "dhcp_lease_remove", path: "/api/v1/dhcp/lease/remove", method: "POST", cat: "dhcp", desc: "Hapus IP lease dari tabel DHCP.", defaultPayload: '{\n  "id": "*1"\n}' },
      { id: "dhcp_servers", path: "/api/v1/dhcp/servers", method: "GET", cat: "dhcp", desc: "Daftar instance DHCP Server yang aktif dan interface induknya.", defaultPayload: "{}" },
      { id: "dhcp_server_add", path: "/api/v1/dhcp/server/add", method: "POST", cat: "dhcp", desc: "Buat DHCP Server baru pada interface.", defaultPayload: '{\n  "name": "dhcp_lan",\n  "interface": "bridge-lan",\n  "address_pool": "pool_lan",\n  "lease_time": "1d"\n}' },
      { id: "dhcp_networks", path: "/api/v1/dhcp/networks", method: "GET", cat: "dhcp", desc: "Daftar subnet jaringan DHCP, gateway IP, DNS server, dan domain.", defaultPayload: "{}" },
      { id: "dhcp_network_add", path: "/api/v1/dhcp/network/add", method: "POST", cat: "dhcp", desc: "Tambah konfigurasi network DHCP baru dengan gateway & DNS.", defaultPayload: '{\n  "address": "192.168.88.0/24",\n  "gateway": "192.168.88.1",\n  "dns_server": "192.168.88.1"\n}' },
      { id: "dhcp_alerts", path: "/api/v1/dhcp/alerts", method: "GET", cat: "dhcp", desc: "Monitoring alarm Rogue DHCP Server ilegal di jaringan lokal.", defaultPayload: "{}" },
      { id: "dhcp_alert_add", path: "/api/v1/dhcp/alert/add", method: "POST", cat: "dhcp", desc: "Pasang alert pengawasan rogue DHCP server pada interface.", defaultPayload: '{\n  "interface": "bridge-lan",\n  "valid_server": "AA:BB:CC:DD:EE:FF"\n}' },
      { id: "dhcp_relays", path: "/api/v1/dhcp/relays", method: "GET", cat: "dhcp", desc: "Daftar DHCP Relay agent yang meneruskan permintaan DHCP ke server pusat.", defaultPayload: "{}" },
      { id: "dhcp_clients", path: "/api/v1/dhcp/clients", method: "GET", cat: "dhcp", desc: "Daftar DHCP Client interface WAN (/ip/dhcp-client/print).", defaultPayload: "{}" },
      { id: "dhcp_client_add", path: "/api/v1/dhcp/client/add", method: "POST", cat: "dhcp", desc: "Tambah DHCP Client pada interface WAN router.", defaultPayload: '{\n  "interface": "ether1",\n  "add_default_route": "yes",\n  "use_peer_dns": true,\n  "comment": "ISP WAN DHCP"\n}' },
      { id: "dhcp_client_set", path: "/api/v1/dhcp/client/set", method: "POST", cat: "dhcp", desc: "Ubah parameter DHCP Client pada interface router.", defaultPayload: '{\n  "id": "*1",\n  "use_peer_dns": true\n}' },
      { id: "dhcp_client_remove", path: "/api/v1/dhcp/client/remove", method: "POST", cat: "dhcp", desc: "Hapus DHCP Client dari interface router.", defaultPayload: '{\n  "id": "*1"\n}' },
      { id: "dhcp_client_renew", path: "/api/v1/dhcp/client/renew", method: "POST", cat: "dhcp", desc: "Perbarui masa sewa (lease) IP DHCP Client WAN.", defaultPayload: '{\n  "id": "*1"\n}' },
      { id: "dhcp_client_release", path: "/api/v1/dhcp/client/release", method: "POST", cat: "dhcp", desc: "Lepaskan (release) IP DHCP Client WAN.", defaultPayload: '{\n  "id": "*1"\n}' },

      // --- PPPoE & ISP Access (cat: pppoe) ---
      { id: "ppp_secrets", path: "/api/v1/ppp/secrets", method: "GET", cat: "pppoe", desc: "Database akun PPPoE ISP, username, password, profile, dan IP remote statis.", defaultPayload: "{}" },
      { id: "ppp_secret_create", path: "/api/v1/ppp/secret/create", method: "POST", cat: "pppoe", desc: "Tambah akun PPPoE secret baru dengan username, password, dan paket profil.", defaultPayload: '{\n  "name": "user_pppoe1",\n  "password": "password123",\n  "profile": "default-encryption",\n  "service": "pppoe"\n}' },
      { id: "ppp_secret_set", path: "/api/v1/ppp/secret/set", method: "POST", cat: "pppoe", desc: "Ubah parameter akun PPPoE (password, profil, remote IP, status disabled).", defaultPayload: '{\n  "name": "user_pppoe1",\n  "profile": "Paket-20Mbps"\n}' },
      { id: "ppp_secret_remove", path: "/api/v1/ppp/secret/remove", method: "POST", cat: "pppoe", desc: "Hapus akun PPPoE dari database router.", defaultPayload: '{\n  "name": "user_pppoe1"\n}' },
      { id: "ppp_active", path: "/api/v1/ppp/active", method: "GET", cat: "pppoe", desc: "Daftar sesi PPPoE yang sedang terhubung aktif (tunnel UP) beserta uptime.", defaultPayload: "{}" },
      { id: "ppp_disconnect", path: "/api/v1/ppp/disconnect", method: "POST", cat: "pppoe", desc: "Putuskan paksa koneksi sesi PPPoE aktif.", defaultPayload: '{\n  "name": "user_pppoe1"\n}' },
      { id: "ppp_servers", path: "/api/v1/ppp/servers", method: "GET", cat: "pppoe", desc: "Daftar server PPPoE Concentrator di router.", defaultPayload: "{}" },
      { id: "ppp_server_create", path: "/api/v1/ppp/server/create", method: "POST", cat: "pppoe", desc: "Buat server PPPoE baru pada interface jaringan.", defaultPayload: '{\n  "service_name": "pppoe-server1",\n  "interface": "ether2",\n  "default_profile": "default"\n}' },
      { id: "ppp_profiles", path: "/api/v1/ppp/profiles", method: "GET", cat: "pppoe", desc: "Profil paket langganan PPP (rate limit, local/remote address pool, DNS).", defaultPayload: "{}" },
      { id: "ppp_profile_create", path: "/api/v1/ppp/profile/create", method: "POST", cat: "pppoe", desc: "Buat profil paket PPP baru.", defaultPayload: '{\n  "name": "Paket-50M",\n  "local_address": "10.10.10.1",\n  "remote_address": "pool-pppoe",\n  "rate_limit": "20M/50M"\n}' },
      { id: "isolir_setup", path: "/api/v1/wisp/isolir/setup", method: "POST", cat: "pppoe", desc: "Setup NAT Redirect port 80 dan firewall drop isolir untuk pelanggan jatuh tempo.", defaultPayload: '{\n  "isolir_server_ip": "192.168.88.254",\n  "isolir_server_port": 80\n}' },
      { id: "isolir_isolate", path: "/api/v1/wisp/isolir/isolate", method: "POST", cat: "pppoe", desc: "Isolir pelanggan menunggak ke address-list ISOLIR_LIST.", defaultPayload: '{\n  "client_ip": "192.168.88.105",\n  "customer_name": "Budi Santoso",\n  "due_date": "10 Oktober 2026"\n}' },
      { id: "isolir_restore", path: "/api/v1/wisp/isolir/restore", method: "POST", cat: "pppoe", desc: "Pulihkan akses internet pelanggan setelah lunas.", defaultPayload: '{\n  "client_ip": "192.168.88.105"\n}' },
      { id: "isolir_list", path: "/api/v1/wisp/isolir/list", method: "GET", cat: "pppoe", desc: "Daftar seluruh pelanggan yang sedang diisolir saat ini.", defaultPayload: "{}" },

      // --- IP & Routing Tables (cat: ip) ---
      { id: "ip_addr", path: "/api/v1/ip/addresses", method: "GET", cat: "ip", desc: "Daftar alamat IP yang dikonfigurasi pada setiap interface router.", defaultPayload: "{}" },
      { id: "ip_addr_add", path: "/api/v1/ip/address/add", method: "POST", cat: "ip", desc: "Pasang IP Address baru ke interface.", defaultPayload: '{\n  "address": "192.168.50.1/24",\n  "interface": "ether3",\n  "comment": "LAN Subnet 2"\n}' },
      { id: "ip_addr_remove", path: "/api/v1/ip/address/remove", method: "POST", cat: "ip", desc: "Hapus konfigurasi IP Address dari interface.", defaultPayload: '{\n  "id": "*1"\n}' },
      { id: "ip_routes", path: "/api/v1/ip/routes", method: "GET", cat: "ip", desc: "Tabel routing IPv4 lengkap (default gateway, routing dinamik dan statis).", defaultPayload: "{}" },
      { id: "ip_route_add", path: "/api/v1/ip/route/add", method: "POST", cat: "ip", desc: "Tambah static route baru ke gateway target.", defaultPayload: '{\n  "dst_address": "0.0.0.0/0",\n  "gateway": "192.168.1.1",\n  "distance": 1\n}' },
      { id: "ip_route_remove", path: "/api/v1/ip/route/remove", method: "POST", cat: "ip", desc: "Hapus rute dari tabel routing.", defaultPayload: '{\n  "id": "*1"\n}' },
      { id: "ip_arp", path: "/api/v1/ip/arp", method: "GET", cat: "ip", desc: "Tabel pemetaan ARP (IP ke MAC Address) dari seluruh interface router.", defaultPayload: "{}" },
      { id: "ip_arp_add", path: "/api/v1/ip/arp/add", method: "POST", cat: "ip", desc: "Tambah entri static ARP manual.", defaultPayload: '{\n  "address": "192.168.88.99",\n  "mac_address": "AA:BB:CC:DD:EE:FF",\n  "interface": "bridge-lan"\n}' },
      { id: "ip_arp_remove", path: "/api/v1/ip/arp/remove", method: "POST", cat: "ip", desc: "Hapus entri ARP.", defaultPayload: '{\n  "id": "*1"\n}' },
      { id: "ip_pools", path: "/api/v1/ip/pools", method: "GET", cat: "ip", desc: "Daftar IP Pool yang terdaftar untuk DHCP, Hotspot, dan VPN.", defaultPayload: "{}" },
      { id: "ip_pools_utilization", path: "/api/v1/ip/pools/utilization", method: "GET", cat: "ip", desc: "Analisis persentase penggunaan IP pool dan sisa kapasitas alamat yang tersedia.", defaultPayload: "{}" },
      { id: "ip_dns", path: "/api/v1/ip/dns", method: "GET", cat: "ip", desc: "Konfigurasi upstream DNS resolver dan opsi allow-remote-requests.", defaultPayload: "{}" },
      { id: "dns_static", path: "/api/v1/dns/static", method: "GET", cat: "ip", desc: "Daftar entri DNS statis lokal yang terpasang di router.", defaultPayload: "{}" },
      { id: "dns_static_add", path: "/api/v1/dns/static/add", method: "POST", cat: "ip", desc: "Tambah mapping DNS statis baru.", defaultPayload: '{\n  "name": "router.local",\n  "address": "192.168.88.1"\n}' },
      { id: "dns_cache_flush", path: "/api/v1/dns/cache/flush", method: "POST", cat: "ip", desc: "Bersihkan cache DNS router.", defaultPayload: "{}" },
      { id: "dns_adlist", path: "/api/v1/dns/adlist", method: "GET", cat: "ip", desc: "RouterOS v7 native AdList (Blocklist Iklan & Malware tingkat router).", defaultPayload: "{}" },
      { id: "dns_adlist_presets", path: "/api/v1/dns/adlist/presets", method: "GET", cat: "ip", desc: "Katalog preset blocklist resmi aman (HaGeZi, StevenBlack, AdGuard).", defaultPayload: "{}" },
      { id: "dns_adlist_deploy", path: "/api/v1/dns/adlist/deploy-preset", method: "POST", cat: "ip", desc: "Deploy preset AdList pilihan langsung ke router.", defaultPayload: '{\n  "preset": "hagezi_light"\n}' },
      { id: "routing_bgp", path: "/api/v1/routing/bgp/sessions", method: "GET", cat: "ip", desc: "Daftar sesi peering BGP aktif dan status state (Established, Active, Idle).", defaultPayload: "{}" },
      { id: "routing_ospf", path: "/api/v1/routing/ospf/neighbors", method: "GET", cat: "ip", desc: "Daftar tetangga OSPF (OSPF Neighbors) dan status adjacency.", defaultPayload: "{}" },
      { id: "routing_tables", path: "/api/v1/routing/tables", method: "GET", cat: "ip", desc: "Daftar Routing Table yang terdaftar di sistem RouterOS v7.", defaultPayload: "{}" },

      // --- System & Hardware (cat: system) ---
      { id: "sys_resource", path: "/api/v1/system/resource", method: "GET", cat: "system", desc: "Spesifikasi resource CPU, RAM bebas, storage, dan arsitektur router.", defaultPayload: "{}" },
      { id: "sys_routerboard", path: "/api/v1/system/routerboard", method: "GET", cat: "system", desc: "Model board MikroTik, serial number, firmware saat ini dan firmware upgrade.", defaultPayload: "{}" },
      { id: "sys_identity", path: "/api/v1/system/identity", method: "GET", cat: "system", desc: "Nama identity sistem router MikroTik.", defaultPayload: "{}" },
      { id: "sys_identity_set", path: "/api/v1/system/identity/set", method: "POST", cat: "system", desc: "Ubah nama router identity MikroTik.", defaultPayload: '{\n  "name": "ROUTER-UTAMA-NOC"\n}' },
      { id: "sys_clock", path: "/api/v1/system/clock", method: "GET", cat: "system", desc: "Waktu jam, tanggal, dan timezone internal router.", defaultPayload: "{}" },
      { id: "sys_packages", path: "/api/v1/system/packages", method: "GET", cat: "system", desc: "Daftar paket software RouterOS terpasang (wireless, routing, ppp, security).", defaultPayload: "{}" },
      { id: "sys_services", path: "/api/v1/system/services", method: "GET", cat: "system", desc: "Daftar service port router (API, SSH, Winbox, WWW) dan status disable/enable.", defaultPayload: "{}" },
      { id: "sys_service_toggle", path: "/api/v1/system/service/toggle", method: "POST", cat: "system", desc: "Aktifkan atau nonaktifkan service port router.", defaultPayload: '{\n  "name": "telnet",\n  "disabled": true\n}' },
      { id: "sys_ntp", path: "/api/v1/system/ntp", method: "GET", cat: "system", desc: "Status sinkronisasi waktu NTP Client router.", defaultPayload: "{}" },
      { id: "sys_ntp_setup", path: "/api/v1/system/ntp/setup", method: "POST", cat: "system", desc: "Atur server NTP pool Indonesia (id.pool.ntp.org).", defaultPayload: '{\n  "primary_ntp": "162.159.200.1",\n  "secondary_ntp": "162.159.200.123"\n}' },
      { id: "cloud_status", path: "/api/v1/cloud/status", method: "GET", cat: "system", desc: "Status IP Cloud DDNS MikroTik (*.sn.mynetname.net) dan deteksi IP publik aktif.", defaultPayload: "{}" },
      { id: "cloud_set", path: "/api/v1/cloud/set", method: "POST", cat: "system", desc: "Aktifkan atau atur interval IP Cloud DDNS dan sinkronisasi jam RouterOS.", defaultPayload: '{\n  "ddns_enabled": true,\n  "update_time": true\n}' },
      { id: "cloud_force_update", path: "/api/v1/cloud/force-update", method: "POST", cat: "system", desc: "Paksa update rekaman DDNS publik ke server MikroTik mynetname.net.", defaultPayload: "{}" },
      { id: "users_all", path: "/api/v1/users/all", method: "GET", cat: "system", desc: "Daftar akun administrator router MikroTik.", defaultPayload: "{}" },
      { id: "users_create", path: "/api/v1/users/create", method: "POST", cat: "system", desc: "Tambah user administrator baru dengan group wewenang (full, read, write).", defaultPayload: '{\n  "name": "noc_operator",\n  "group": "read",\n  "password": "SecurePassword123"\n}' },
      { id: "scripts_all", path: "/api/v1/scripts/all", method: "GET", cat: "system", desc: "Daftar seluruh script tersimpan di sistem RouterOS.", defaultPayload: "{}" },
      { id: "scripts_run", path: "/api/v1/scripts/run", method: "POST", cat: "system", desc: "Jalankan script RouterOS yang sudah tersimpan.", defaultPayload: '{\n  "name": "auto-backup"\n}' },
      { id: "schedulers_all", path: "/api/v1/schedulers/all", method: "GET", cat: "system", desc: "Daftar cron scheduler otomatis yang sedang aktif.", defaultPayload: "{}" },
      { id: "logs_all", path: "/api/v1/logs/all", method: "GET", cat: "system", desc: "Daftar log aktivitas sistem, login, firewall, dan error router.", defaultPayload: "{}" },
      { id: "raw_command", path: "/api/v1/command", method: "POST", cat: "system", desc: "Universal Raw Command Executor: Kirim perintah sembarang ke RouterOS API secara langsung.", defaultPayload: '{\n  "command": "/system/resource/print"\n}' },
      { id: "raw_batch", path: "/api/v1/batch", method: "POST", cat: "system", desc: "Eksekusi kumpulan perintah RouterOS berurutan dalam satu transaksi.", defaultPayload: '{\n  "commands": ["/system/identity/print", "/system/clock/print"]\n}' },

      // --- Network Topology & Discovery (cat: net) ---
      { id: "topology", path: "/api/v1/network/topology-graph", method: "GET", cat: "net", desc: "Data graf relasi simpul (Nodes & Edges) jaringan berjenjang untuk visualizer topologi.", defaultPayload: "{}" },
      { id: "if_all", path: "/api/v1/interfaces/all", method: "GET", cat: "net", desc: "Daftar seluruh interface jaringan (ether, bridge, wlan, pppoe, vlan) dan statistik trafik.", defaultPayload: "{}" },
      { id: "bridge_all", path: "/api/v1/bridge/all", method: "GET", cat: "net", desc: "Daftar seluruh bridge interface di router.", defaultPayload: "{}" },
      { id: "bridge_ports", path: "/api/v1/bridge/ports", method: "GET", cat: "net", desc: "Daftar port interface fisik yang tergabung dalam bridge.", defaultPayload: "{}" },
      { id: "bridge_vlans", path: "/api/v1/bridge/vlans", method: "GET", cat: "net", desc: "Daftar VLAN pada bridge (Bridge VLAN Filtering).", defaultPayload: "{}" },
      { id: "capsman_radios", path: "/api/v1/capsman/radios", method: "GET", cat: "net", desc: "Daftar seluruh radio Access Point MikroTik (CAPs) yang terhubung ke controller.", defaultPayload: "{}" },
      { id: "capsman_registrations", path: "/api/v1/capsman/registrations", method: "GET", cat: "net", desc: "Daftar seluruh klien wireless (HP/Laptop) yang tersambung di semua AP.", defaultPayload: "{}" },
      { id: "capsman_set_wifi", path: "/api/v1/capsman/set-wifi", method: "POST", cat: "net", desc: "Konfigurasi SSID & Password WiFi terpusat ke seluruh AP MikroTik sekaligus.", defaultPayload: '{\n  "ssid": "HOTSPOT-WARUNG",\n  "passphrase": "password123"\n}' },
      { id: "wireless_ifaces", path: "/api/v1/wireless/interfaces", method: "GET", cat: "net", desc: "Daftar interface kartu WiFi internal router MikroTik.", defaultPayload: "{}" },
      { id: "wireless_registrations", path: "/api/v1/wireless/registrations", method: "GET", cat: "net", desc: "Daftar klien WiFi yang tersambung ke antarmuka wireless router.", defaultPayload: "{}" },
      { id: "neighbors_all", path: "/api/v1/neighbors/all", method: "GET", cat: "net", desc: "Daftar perangkat tetangga yang ditemukan via MNDP / CDP / LLDP discovery.", defaultPayload: "{}" },
      { id: "infra_scan", path: "/api/v1/network/infrastructure/scan", method: "GET", cat: "net", desc: "Deteksi otomatis Access Point pihak ketiga (Totolink, TP-Link, Mercusys, ZTE) di jaringan.", defaultPayload: "{}" },
      { id: "infra_poe_cycle", path: "/api/v1/network/infrastructure/poe-cycle", method: "POST", cat: "net", desc: "Reboot paksa Access Point yang hang lewat pemutusan daya PoE port sementara.", defaultPayload: '{\n  "interface": "ether5",\n  "off_seconds": 3\n}' },
      { id: "infra_ap_tunnel", path: "/api/v1/network/infrastructure/ap-tunnel", method: "POST", cat: "net", desc: "Buka port forwarding sementara untuk akses Web Admin AP pihak ketiga dari luar LAN.", defaultPayload: '{\n  "ap_ip": "192.168.100.3",\n  "external_port": 8083\n}' }
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
        const json = await res.json();
        viewer.textContent = JSON.stringify(json, null, 2);

        if (currentEndpoint.path === '/api/v1/tools/ping' && (json.avg_rtt || json.min_rtt)) {
          latencyBadge.textContent = `HTTP: ${latency}ms | ICMP Avg: ${json.avg_rtt || json.min_rtt}`;
          latencyBadge.title = "HTTP adalah total durasi transmisi batch paket ICMP oleh router. ICMP Ping adalah rata-rata latency jaringan ke target.";
        } else {
          latencyBadge.textContent = `Latency: ${latency} ms`;
        }

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

      // Mendukung response terstruktur (d.system dari /overview) maupun langsung (dari /system/resource)
      const sys = d.system || (d['cpu-load'] !== undefined || d['free-memory'] !== undefined || d['cpu_load_percent'] !== undefined ? d : null);

      // Identitas & Tipe Board Router
      const idName = d.identity || (!sys ? d.name : (d.name && !d['cpu-load'] ? d.name : null));
      if (idName) {
        const idEl = document.getElementById('kpi-identity');
        if (idEl) idEl.textContent = idName;
      }
      const board = d.routerboard?.model || sys?.board_name || d['board-name'] || sys?.['board-name'];
      if (board) {
        const bEl = document.getElementById('kpi-board');
        if (bEl) bEl.textContent = board;
      }
      const rosVer = sys?.version || d['version'] || sys?.['version'] || '';
      const uptime = sys?.uptime || d['uptime'] || sys?.['uptime'] || '';
      if (rosVer || uptime) {
        window.currentRouterOSVersion = rosVer;
        const rosEl = document.getElementById('kpi-ros');
        if (rosEl) rosEl.textContent = `RouterOS ${rosVer} | Uptime: ${uptime}`;
      }

      // CPU Load & Frekuensi
      if (sys) {
        let cpuVal = sys.cpu_load_percent !== undefined ? sys.cpu_load_percent : (sys['cpu-load'] !== undefined ? parseInt(sys['cpu-load'], 10) : sys.cpu_load);
        if (cpuVal !== undefined && !isNaN(cpuVal)) {
          const cpuEl = document.getElementById('kpi-cpu');
          if (cpuEl) cpuEl.textContent = `${cpuVal}%`;
          const barEl = document.getElementById('kpi-cpu-bar');
          if (barEl) barEl.style.width = `${Math.min(100, Math.max(0, cpuVal))}%`;
        }

        const freq = sys.cpu_frequency_mhz || sys['cpu-frequency'] || d.cpu_frequency_mhz;
        if (freq) {
          const freqEl = document.getElementById('kpi-freq');
          const cpuCnt = sys.cpu_count || sys['cpu-count'] || d.cpu_count;
          const cores = cpuCnt ? ` (${cpuCnt} Core)` : '';
          if (freqEl) freqEl.textContent = `${freq} MHz${cores}`;
        }

        // RAM Free, Total, dan Persentase Terpakai
        let freeMb = sys.free_memory_mb !== undefined ? sys.free_memory_mb : (d.free_memory_mb !== undefined ? d.free_memory_mb : undefined);
        if (freeMb === undefined && (sys['free-memory'] || sys.free_memory)) {
          const rawFree = parseInt(sys['free-memory'] || sys.free_memory, 10);
          if (!isNaN(rawFree)) freeMb = Math.round(rawFree / (1024 * 1024));
        }
        if (freeMb !== undefined && !isNaN(freeMb)) {
          const freeEl = document.getElementById('kpi-free-ram');
          if (freeEl) freeEl.textContent = `${freeMb} MB`;
        }

        let totMb = sys.total_memory_mb !== undefined ? sys.total_memory_mb : (d.total_memory_mb !== undefined ? d.total_memory_mb : undefined);
        if (totMb === undefined && (sys['total-memory'] || sys.total_memory)) {
          const rawTot = parseInt(sys['total-memory'] || sys.total_memory, 10);
          if (!isNaN(rawTot)) totMb = Math.round(rawTot / (1024 * 1024));
        }
        if (totMb !== undefined && !isNaN(totMb)) {
          const totEl = document.getElementById('kpi-total-ram');
          if (totEl) totEl.textContent = `${totMb} MB`;
        }

        const memPercentEl = document.getElementById('kpi-mem-percent');
        if (memPercentEl) {
          const rawPct = sys.memory_used_percent || d.memory_used_percent;
          if (rawPct !== undefined && rawPct !== null && rawPct !== '') {
            const formattedPct = typeof rawPct === 'number'
              ? `${rawPct.toFixed(1)}%`
              : (String(rawPct).endsWith('%') ? String(rawPct) : `${rawPct}%`);
            memPercentEl.textContent = `Penggunaan RAM: ${formattedPct}`;
          } else if (totMb && freeMb !== undefined && totMb > 0) {
            const usedPct = (((totMb - freeMb) / totMb) * 100).toFixed(1);
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

    // --- Live WebSocket Real-Time Terminal & Developer SDK Controller ---
    let wsSocket = null;
    let wsPingInterval = null;

    function openWsModal() {
      const modal = document.getElementById('ws-modal');
      if (modal) {
        modal.classList.add('show');
        switchWsModalTab('terminal');
      }
    }

    function closeWsModal() {
      const modal = document.getElementById('ws-modal');
      if (modal) modal.classList.remove('show');
      stopWsStream();
    }

    function onBackdropClick(event) {
      if (event.target.id === 'ws-modal') {
        closeWsModal();
      }
    }

    window.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        closeWsModal();
      }
    });

    function switchWsModalTab(tab) {
      const termPane = document.getElementById('ws-pane-terminal');
      const docsPane = document.getElementById('ws-pane-docs');
      const termBtn = document.getElementById('tab-btn-terminal');
      const docsBtn = document.getElementById('tab-btn-docs');

      if (tab === 'terminal') {
        if (termPane) termPane.style.display = 'block';
        if (docsPane) docsPane.style.display = 'none';
        if (termBtn) termBtn.classList.add('active');
        if (docsBtn) docsBtn.classList.remove('active');
      } else {
        if (termPane) termPane.style.display = 'none';
        if (docsPane) docsPane.style.display = 'block';
        if (termBtn) termBtn.classList.remove('active');
        if (docsBtn) docsBtn.classList.add('active');
        switchWsSnippet('flutter', document.getElementById('ws-sdk-tab-flutter'));
      }
    }

    function switchWsSnippet(lang, btn) {
      document.querySelectorAll('#ws-pane-docs .code-tab-btn').forEach(b => b.classList.remove('active'));
      if (btn) btn.classList.add('active');

      const host = document.getElementById('target-host').value.trim() || '192.168.88.1';
      const port = document.getElementById('target-port').value.trim() || '8728';
      const user = document.getElementById('target-user').value.trim() || 'admin';
      const pass = document.getElementById('target-pass').value || 'password';
      const token = document.getElementById('gw-token').value.trim() || 'change-me-to-a-long-random-string';
      const gwHost = window.location.host || '127.0.0.1:8080';
      const wsUrl = `ws://${gwHost}/ws?token=${token}`;

      const box = document.getElementById('ws-snippet-box');
      if (!box) return;

      if (lang === 'flutter') {
        box.textContent = `// 1. Tambahkan dependency di pubspec.yaml:
//    web_socket_channel: ^2.4.0
import 'dart:convert';
import 'package:web_socket_channel/web_socket_channel.dart';

final ws = WebSocketChannel.connect(
  Uri.parse('${wsUrl}'),
);

// 2. Subscribe ke Live ICMP Ping Stream (<2ms gateway socket duplex)
ws.sink.add(jsonEncode({
  "action": "subscribe_ping",
  "address": "8.8.8.8",
  "interval_ms": 1000,
  "router": {
    "host": "${host}",
    "port": ${port},
    "user": "${user}",
    "password": "${pass}"
  }
}));

// 3. Tangkap frame packet per detik secara instan di UI / Bloc / Riverpod:
ws.stream.listen((message) {
  final frame = jsonDecode(message);
  if (frame['event'] == 'ping_frame') {
    print('Ping \${frame['target']}: seq=\${frame['sequence']} time=\${frame['rtt']}');
  }
});`;
      } else if (lang === 'js') {
        box.textContent = `// Koneksi WebSocket Real-Time (Browser / React / Vue / Next.js / Node.js)
const token = "${token}";
const ws = new WebSocket("${wsUrl}");

ws.onopen = () => {
  // Subscribe ke live throughput bandwidth interface
  ws.send(JSON.stringify({
    action: "subscribe_traffic",
    interface: "ether1",
    router: {
      host: "${host}",
      port: ${port},
      user: "${user}",
      password: "${pass}"
    }
  }));
};

ws.onmessage = (event) => {
  const frame = JSON.parse(event.data);
  if (frame.event === 'traffic_frame') {
    const rxBps = frame.data['rx-bits-per-second'];
    const txBps = frame.data['tx-bits-per-second'];
    console.log(\`RX: \${rxBps} bps | TX: \${txBps} bps\`);
  }
};`;
      } else if (lang === 'python') {
        box.textContent = `# pip install websockets
import asyncio, json, websockets

async def stream_live_telemetry():
    uri = "${wsUrl}"
    async with websockets.connect(uri) as ws:
        # Subscribe ke System Overview Telemetry Live Heartbeat
        await ws.send(json.dumps({
            "action": "subscribe_overview",
            "interval_ms": 2000,
            "router": {
                "host": "${host}",
                "port": ${port},
                "user": "${user}",
                "password": "${pass}"
            }
        }))
        async for msg in ws:
            frame = json.loads(msg)
            if frame.get("event") == "overview_frame":
                data = frame.get("data", {})
                print(f"Router {data.get('identity')} CPU: {data.get('cpu_load_percent')}% RAM Free: {data.get('free_memory_mb')}MB")

asyncio.run(stream_live_telemetry())`;
      }
    }

    function clearWsTerminal() {
      const term = document.getElementById('ws-terminal');
      if (term) term.innerHTML = '<div class="ws-line comment"># Terminal cleared.</div>';
    }

    function onWsStreamTypeChange() {
      const type = document.getElementById('ws-stream-type').value;
      const paramInput = document.getElementById('ws-target-param');
      if (type === 'ping') {
        paramInput.style.display = 'inline-block';
        paramInput.placeholder = 'Target IP / Host (e.g. 8.8.8.8)';
        if (!paramInput.value || paramInput.value === 'ether1') paramInput.value = '8.8.8.8';
      } else if (type === 'traffic') {
        paramInput.style.display = 'inline-block';
        paramInput.placeholder = 'Interface Name (e.g. ether1)';
        if (!paramInput.value || paramInput.value === '8.8.8.8') paramInput.value = 'ether1';
      } else if (type === 'logs') {
        paramInput.style.display = 'inline-block';
        paramInput.placeholder = 'Topic filter (e.g. info, firewall, hotspot)';
        paramInput.value = '';
      } else {
        paramInput.style.display = 'none';
      }
    }

    function appendWsLine(text, type = 'normal') {
      const term = document.getElementById('ws-terminal');
      if (!term) return;
      const line = document.createElement('div');
      line.className = `ws-line ${type}`;
      const now = new Date().toTimeString().split(' ')[0];
      line.textContent = `[${now}] ${text}`;
      term.appendChild(line);
      term.scrollTop = term.scrollHeight;
    }

    function toggleWsStream() {
      if (wsSocket && (wsSocket.readyState === WebSocket.OPEN || wsSocket.readyState === WebSocket.CONNECTING)) {
        stopWsStream();
      } else {
        startWsStream();
      }
    }

    function stopWsStream() {
      if (wsPingInterval) {
        clearInterval(wsPingInterval);
        wsPingInterval = null;
      }
      if (wsSocket) {
        wsSocket.close();
        wsSocket = null;
      }
      const dot = document.getElementById('ws-status-dot');
      const text = document.getElementById('ws-status-text');
      const btn = document.getElementById('ws-connect-btn');
      if (dot) dot.classList.remove('active');
      if (text) text.textContent = 'Disconnected';
      if (btn) {
        btn.innerHTML = '<span>▶ Mulai Stream Real-Time</span>';
        btn.className = 'ws-btn ws-btn-start';
      }
      appendWsLine('WebSocket stream disconnected.', 'comment');
    }

    function startWsStream() {
      const host = document.getElementById('target-host').value.trim();
      const port = document.getElementById('target-port').value.trim();
      const user = document.getElementById('target-user').value.trim();
      const pass = document.getElementById('target-pass').value;
      const token = document.getElementById('gw-token').value.trim();

      const streamType = document.getElementById('ws-stream-type').value;
      const targetParam = document.getElementById('ws-target-param').value.trim();

      const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const wsUrl = `${wsProtocol}//${window.location.host}/ws?token=${encodeURIComponent(token)}`;

      appendWsLine(`Connecting to WebSocket: ${wsUrl}...`, 'comment');
      const dot = document.getElementById('ws-status-dot');
      const text = document.getElementById('ws-status-text');
      const btn = document.getElementById('ws-connect-btn');

      try {
        wsSocket = new WebSocket(wsUrl);

        wsSocket.onopen = () => {
          if (dot) dot.classList.add('active');
          if (text) text.textContent = 'Connected (Live <2ms Socket)';
          if (btn) {
            btn.innerHTML = '<span>⏹ Hentikan Stream</span>';
            btn.className = 'ws-btn ws-btn-stop';
          }
          appendWsLine(`WebSocket Handshake ESTABLISHED (<2ms Rust Gateway)`, 'success');

          const routerObj = {
            host: host || undefined,
            port: port ? parseInt(port, 10) : 8728,
            user: user || undefined,
            password: pass || undefined
          };

          if (streamType === 'ping') {
            const targetHost = targetParam || '8.8.8.8';
            appendWsLine(`Subscribing to live ICMP Ping -> ${targetHost}...`, 'comment');
            wsSocket.send(JSON.stringify({
              action: "subscribe_ping",
              address: targetHost,
              router: routerObj,
              router_host: host,
              router_port: port ? parseInt(port, 10) : 8728,
              router_user: user,
              router_pass: pass
            }));
          } else if (streamType === 'traffic') {
            const iface = targetParam || 'ether1';
            appendWsLine(`Subscribing to live traffic on [${iface}]...`, 'comment');
            wsSocket.send(JSON.stringify({
              action: "subscribe_traffic",
              interface: iface,
              router: routerObj,
              router_host: host,
              router_port: port ? parseInt(port, 10) : 8728,
              router_user: user,
              router_pass: pass
            }));
          } else if (streamType === 'overview') {
            appendWsLine(`Subscribing to live System Overview stream...`, 'comment');
            wsSocket.send(JSON.stringify({
              action: "subscribe_overview",
              router: routerObj,
              router_host: host,
              router_port: port ? parseInt(port, 10) : 8728,
              router_user: user,
              router_pass: pass
            }));
          } else if (streamType === 'logs') {
            appendWsLine(`Listening to live syslog stream...`, 'comment');
            wsSocket.send(JSON.stringify({
              action: "listen_logs",
              params: targetParam ? { topic: targetParam } : {},
              router: routerObj,
              router_host: host,
              router_port: port ? parseInt(port, 10) : 8728,
              router_user: user,
              router_pass: pass
            }));
          }
        };

        wsSocket.onmessage = (event) => {
          try {
            const msg = JSON.parse(event.data);
            if (msg.event === 'connected') {
              appendWsLine(`Gateway greeting: authenticated=${msg.authenticated}`, 'comment');
            } else if (msg.event === 'ping_frame') {
              const rtt = msg.rtt || '0ms';
              const seq = msg.sequence || '1';
              const target = msg.target || targetParam || 'target';
              appendWsLine(`64 bytes from ${target}: seq=${seq} time=${rtt} status=LIVE_REPLY`, 'success');
            } else if (msg.event === 'traffic_frame') {
              const d = msg.data || {};
              const rxBps = parseFloat(d['rx-bits-per-second'] || d.rx_bps || 0);
              const txBps = parseFloat(d['tx-bits-per-second'] || d.tx_bps || 0);
              const rxPps = d['rx-packets-per-second'] || 0;
              const txPps = d['tx-packets-per-second'] || 0;
              const rxFormatted = rxBps > 1000000 ? (rxBps / 1000000).toFixed(2) + ' Mbps' : (rxBps / 1000).toFixed(1) + ' Kbps';
              const txFormatted = txBps > 1000000 ? (txBps / 1000000).toFixed(2) + ' Mbps' : (txBps / 1000).toFixed(1) + ' Kbps';
              appendWsLine(`[TRAFFIC ${msg.interface || 'iface'}] ⬇ RX: ${rxFormatted} (${rxPps} pps) | ⬆ TX: ${txFormatted} (${txPps} pps)`, 'normal');
            } else if (msg.event === 'overview_frame') {
              const d = msg.data || {};
              appendWsLine(`[TELEMETRY] ${d.identity || 'RouterOS'} | CPU: ${d.cpu_load_percent}% | RAM Free: ${d.free_memory_mb}MB (${d.memory_used_percent} used) | Clients: ${d.total_clients_count} | Uptime: ${d.uptime}`, 'normal');
            } else if (msg.event === 'log_entry') {
              const d = msg.data || {};
              appendWsLine(`[SYSLOG] ${d.time || ''} [${d.topics || 'info'}] ${d.message || JSON.stringify(d)}`, 'comment');
            } else if (msg.event === 'command_result') {
              appendWsLine(`[CLI RESULT] Count: ${msg.count} row(s)\n${JSON.stringify(msg.data, null, 2)}`, 'success');
            } else if (msg.event === 'stream_error' || msg.event === 'ping_error' || msg.event === 'command_error' || msg.event === 'error' || msg.event === 'unauthorized') {
              appendWsLine(`[ERROR] ${msg.error || JSON.stringify(msg)}`, 'error');
            } else {
              appendWsLine(JSON.stringify(msg), 'normal');
            }
          } catch (e) {
            appendWsLine(event.data, 'normal');
          }
        };

        wsSocket.onerror = (err) => {
          appendWsLine(`WebSocket error: ${err.message || 'Connection interrupted'}`, 'error');
        };

        wsSocket.onclose = () => {
          stopWsStream();
        };

      } catch (err) {
        appendWsLine(`Gagal membuat WebSocket: ${err.message}`, 'error');
        stopWsStream();
      }
    }

    window.addEventListener('DOMContentLoaded', () => {
      initCreds();
      initSavedProfiles();
      populateDropdown('all');

      const urlParams = new URLSearchParams(window.location.search);
      if (urlParams.get('endpoint')) {
        const targetEp = urlParams.get('endpoint');
        const sel = document.getElementById('ep-select');
        if (sel) {
          sel.value = targetEp;
          onEndpointChange();
        }
      }
      if (urlParams.get('ws_modal') === 'true') {
        openWsModal();
      }

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
        <div class="cat-menu-title">API Categories</div>
        <a href="#sec-fast" class="cat-link"><span>🚀 Fast-Path &amp; Performance</span><span class="cat-count">5</span></a>
        <a href="#sec-ws" class="cat-link"><span>⚡ Real-Time WebSocket</span><span class="cat-count">4</span></a>
        <a href="#sec-security" class="cat-link"><span>🛡️ Security &amp; Diagnostics</span><span class="cat-count">16</span></a>
        <a href="#sec-vouchers" class="cat-link"><span>🎟️ Voucher Engine 2.0</span><span class="cat-count">7</span></a>
        <a href="#sec-notify" class="cat-link"><span>🤖 Bot &amp; Netwatch Automation</span><span class="cat-count">9</span></a>
        <a href="#sec-loadbalance" class="cat-link"><span>🔀 Load Balancing &amp; NAT</span><span class="cat-count">8</span></a>
        <a href="#sec-vpn" class="cat-link"><span>🔐 VPN &amp; WireGuard</span><span class="cat-count">7</span></a>
        <a href="#sec-queues" class="cat-link"><span>📊 QoS &amp; Traffic Shaping</span><span class="cat-count">8</span></a>
        <a href="#sec-tools" class="cat-link"><span>🛠️ NOC Diagnostic Tools</span><span class="cat-count">14</span></a>
        <a href="#sec-maintenance" class="cat-link"><span>💾 Backup &amp; Maintenance</span><span class="cat-count">7</span></a>
        <a href="#sec-hotspot" class="cat-link"><span>🔥 Hotspot Captive Portal</span><span class="cat-count">15</span></a>
        <a href="#sec-dhcp" class="cat-link"><span>💻 DHCP &amp; IPAM</span><span class="cat-count">8</span></a>
        <a href="#sec-pppoe" class="cat-link"><span>🌐 PPPoE &amp; ISP Access</span><span class="cat-count">11</span></a>
        <a href="#sec-ip" class="cat-link"><span>📡 IP &amp; Routing Tables</span><span class="cat-count">14</span></a>
        <a href="#sec-sys" class="cat-link"><span>⚙️ System &amp; Hardware</span><span class="cat-count">13</span></a>
        <a href="#sec-net" class="cat-link"><span>🗺️ Network Topology &amp; Discovery</span><span class="cat-count">5</span></a>
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
        <div class="section-header">🚀 Fast-Path &amp; Performance Optimization</div>

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

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/system/fasttrack/status</div>
            <a href="/?endpoint=fasttrack_status" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Inspeksi status aturan FastTrack Hardware Acceleration RouterOS, packet counter ter-bypass, dan konfirmasi rule forward active.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/system/fasttrack/deploy</div>
            <a href="/?endpoint=fasttrack_deploy" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">1-Klik Deploy Rule FastTrack IPv4 Connection Tracking: Bypass firewall untuk paket established/related, memangkas CPU load hingga 80% saat beban trafik gigabit tinggi.</div>
          <div class="code-preview">POST /api/v1/system/fasttrack/deploy
{}</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/system/cpu-profiler</div>
            <a href="/?endpoint=cpu_profiler" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem;">Uji di Playground</a>
          </div>
          <div class="ep-desc">Pemeriksa profil CPU live &amp; pendeteksi bottleneck proses RouterOS (networking, firewall, profiling) untuk menemukan penyebab router lambat.</div>
        </div>
      </section>

      <!-- Section: Real-Time WebSocket Protocol -->
      <section class="doc-section" id="sec-ws">
        <div class="section-header">⚡ Real-Time WebSocket Streaming Protocol (Sub-2ms Gateway Engine)</div>
        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill" style="background:#dcfce7; border: 1px solid #86efac; color:#15803d; font-weight: 700;">WS / WSS</span></div>
            <div class="ep-path">/ws?token={bearer_token}</div>
            <a href="/?ws_modal=true" class="nav-btn" style="padding: 3px 8px; font-size: 0.72rem; font-weight: 700;">Buka Terminal WebSocket di Console</a>
          </div>
          <div class="ep-desc">
            Koneksi duplex real-time langsung ke Rust Socket Gateway untuk streaming data live tanpa polling HTTP.
            Berbeda dengan endpoint batch HTTP (seperti <code>/api/v1/tools/ping</code> yang menunggu seluruh 4 paket selesai ~3.5 detik),
            koneksi WebSocket mengalirkan respons paket demi paket setiap detik secara instan (&lt;2ms latensi pemrosesan gateway).
          </div>

          <table class="params-table" style="margin-top: 10px;">
            <tr><th>Action</th><th>Parameter Kunci</th><th>Event Balasan</th><th>Deskripsi</th></tr>
            <tr><td><code>subscribe_ping</code></td><td><code>address</code>, <code>interval_ms</code></td><td><code>ping_frame</code></td><td>Streaming paket ICMP ping per detik dari router ke host tujuan secara instan</td></tr>
            <tr><td><code>subscribe_traffic</code></td><td><code>interface</code></td><td><code>traffic_frame</code></td><td>Monitor throughput bitrate RX/TX &amp; packet counter interface real-time</td></tr>
            <tr><td><code>subscribe_overview</code></td><td><code>interval_ms</code></td><td><code>overview_frame</code></td><td>Live heartbeat CPU load, RAM usage, total client hotspot/dhcp/ppp, dan uptime</td></tr>
            <tr><td><code>listen_logs</code></td><td><code>params.topic</code></td><td><code>log_entry</code></td><td>Streaming live RouterOS syslog (firewall alerts, hotspot logins, critical events)</td></tr>
            <tr><td><code>command</code></td><td><code>command</code>, <code>params</code></td><td><code>command_result</code></td><td>Eksekusi duplex RouterOS API command langsung melalui persistent socket</td></tr>
          </table>

          <div style="margin-top: 14px; font-size: 0.8rem; font-weight: 700; color: #334155;">Contoh Integrasi JavaScript (Browser / Node.js):</div>
          <pre class="code-preview">// 1. Handshake & Auth via Query Param
const ws = new WebSocket("ws://127.0.0.1:8080/ws?token=change-me-to-a-long-random-string");

// 2. Subscribe ke Live ICMP Ping (Mendukung kredensial nested atau flat)
ws.onopen = () => {
  ws.send(JSON.stringify({
    action: "subscribe_ping",
    address: "8.8.8.8",
    interval_ms: 1000,
    router: {
      host: "192.168.88.1",
      port: 8728,
      user: "admin",
      password: "router_password"
    }
  }));
};

// 3. Tangkap Event Reply Paket per Detik
ws.onmessage = (event) => {
  const frame = JSON.parse(event.data);
  if (frame.event === 'ping_frame') {
    console.log(`Live Ping reply seq=${frame.sequence}: ${frame.rtt}`);
  }
};</pre>

          <div style="margin-top: 10px; font-size: 0.8rem; font-weight: 700; color: #334155;">Contoh Integrasi Flutter (Dart / web_socket_channel):</div>
          <pre class="code-preview">import 'dart:convert';
import 'package:web_socket_channel/web_socket_channel.dart';

final ws = WebSocketChannel.connect(
  Uri.parse('ws://127.0.0.1:8080/ws?token=change-me-to-a-long-random-string'),
);

ws.sink.add(jsonEncode({
  "action": "subscribe_traffic",
  "interface": "ether1",
  "router": { "host": "192.168.88.1", "port": 8728, "user": "admin", "password": "password" }
}));

ws.stream.listen((message) {
  final frame = jsonDecode(message);
  if (frame['event'] == 'traffic_frame') {
    print('Bandwidth RX: ${frame['data']['rx-bits-per-second']} bps');
  }
});</pre>
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

      <!-- Section: Cloud DDNS, Failover & WISP Isolir -->
      <section class="doc-section" id="sec-wisp-routing">
        <div class="section-header">☁️ IP Cloud DDNS, Recursive Failover &amp; WISP Isolir Web Redirect</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/cloud/status</div>
          </div>
          <div class="ep-desc">Status IP Cloud DDNS resmi MikroTik (*.sn.mynetname.net): IP publik terdeteksi dan status koneksi untuk dynamic IP ISP (Indihome, Biznet).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/cloud/set</div>
          </div>
          <div class="ep-desc">Aktifkan/nonaktifkan DDNS bawaan RouterOS dan sinkronisasi update-time.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/routing/failover/setup</div>
          </div>
          <div class="ep-desc">1-Klik MTCRE Recursive Gateway Failover: setup rute ganda berbasis ping DNS 8.8.8.8 &amp; 1.1.1.1 (scope=10, target-scope=11, check-gateway=ping) tanpa script eksternal.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/wisp/isolir/setup</div>
          </div>
          <div class="ep-desc">Infrastruktur Isolir Web Redirect: NAT dst-nat port 80 ke server halaman invoice/peringatan jatuh tempo dan filter drop HTTPS 443.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/wisp/isolir/isolate</div>
          </div>
          <div class="ep-desc">Isolir pelanggan menunggak: masukkan IP pelanggan ke address-list ISOLIR_LIST dengan catatan nama dan jatuh tempo.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/wisp/isolir/restore</div>
          </div>
          <div class="ep-desc">Pulihkan akses internet pelanggan seketika setelah pembayaran tagihan lunas.</div>
        </div>
      </section>

      <!-- Section: MTCSE Security Suite & Packet Sniffer -->
      <section class="doc-section" id="sec-mtcse-security">
        <div class="section-header">🛡️ MTCSE 3-Step Port Knocking, Hardening &amp; Packet Sniffer</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/security/port-knocking/setup</div>
          </div>
          <div class="ep-desc">3-Step Stealth Port Knocking Wizard: Amankan port Winbox 8291 dan SSH 22. Port tertutup total dari internet publik sampai teknisi melakukan ketukan port rahasia (contoh 7000->8000->9000).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/security/hardening/deploy-full-hardening</div>
          </div>
          <div class="ep-desc">1-Klik Full Hardening Suite: Filter Bogon/Martian IP di WAN, blokir DNS Open-Resolver UDP/53, Raw Firewall Anti-DDoS, dan Auto-Reboot jam 04:00 subuh.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/tools/sniffer/start</div>
          </div>
          <div class="ep-desc">Winbox Packet Sniffer: Rekam paket data ke internal storage router (.pcap) atau streaming TZSP langsung ke software Wireshark di PC.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/tools/graphing/enable</div>
          </div>
          <div class="ep-desc">Aktifkan RouterOS Built-in Web Graphing MRTG (/graphs) untuk memantau grafik historis interface, simple queue, dan CPU tanpa server eksternal.</div>
        </div>
      </section>

      <!-- Section: Mikhmon v4 Hotspot Suite -->
      <section class="doc-section" id="sec-mikhmon-v4">
        <div class="section-header">🎫 Mikhmon v4 Advanced Suite: Lock MAC, 3-Mode Expiry &amp; Trial Controller</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/mikhmon-v4/lock-mac/enable</div>
          </div>
          <div class="ep-desc">Lock User to MAC: Injeksi script on-login otomatis mengunci MAC address perangkat pada login pertama agar voucher tidak bisa dipindahtangankan ke HP lain.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/mikhmon-v4/expiry-mode/setup</div>
          </div>
          <div class="ep-desc">3-Mode Expiry Voucher: Konfigurasi validitas mode Remove (hapus akun), Notice (alihkan ke web peringatan habis), atau Record (catat transaksi kasir ke log).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/mikhmon-v4/trial/setup</div>
          </div>
          <div class="ep-desc">Hotspot Trial Controller: Atur durasi internet gratisan (misal 15 menit per 1 hari) dan batasan profil pengguna uji coba.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/hotspot/mikhmon-v4/trial/reset-cookies</div>
          </div>
          <div class="ep-desc">Reset MAC trial cookies: Hapus riwayat perangkat gratisan agar pengunjung dapat mencoba koneksi kembali jika diizinkan.</div>
        </div>
      </section>

      <!-- Section: Advanced IP, DHCP Relay & Server CRUD -->
      <section class="doc-section" id="sec-dhcp-relay-suite">
        <div class="section-header">📡 DHCP Server CRUD, DHCP Relay &amp; Lease Reservation</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dhcp/server/add</div>
          </div>
          <div class="ep-desc">Tambah DHCP Server baru pada interface (LAN/Bridge/VLAN) dengan address-pool dan lease-time tertentu.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dhcp/server/set</div>
          </div>
          <div class="ep-desc">Update parameter DHCP Server (ubah nama, interface, address-pool, lease-time, status disabled).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dhcp/server/remove</div>
          </div>
          <div class="ep-desc">Hapus instans DHCP Server berdasarkan ID (.id).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dhcp/relays</div>
          </div>
          <div class="ep-desc">Daftar DHCP Relay agent (/ip/dhcp-relay/print) untuk meneruskan request DHCP lokal ke DHCP Server pusat/datacenter.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dhcp/relay/add</div>
          </div>
          <div class="ep-desc">Tambah DHCP Relay agent baru (/ip/dhcp-relay/add) dengan parameter interface, target dhcp-server IP, dan local-address.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dhcp/relay/set</div>
          </div>
          <div class="ep-desc">Update pengaturan DHCP Relay agent (ubah IP DHCP server pusat, delay threshold, atau interface).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/dhcp/relay/remove</div>
          </div>
          <div class="ep-desc">Hapus DHCP Relay agent dari router.</div>
        </div>
      </section>

      <!-- Section: Enterprise IPv6 Suite -->
      <section class="doc-section" id="sec-ipv6-enterprise">
        <div class="section-header">🌐 Enterprise IPv6 Management &amp; Prefix Delegation</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ipv6/route/add</div>
          </div>
          <div class="ep-desc">Tambah IPv6 Static Route (/ipv6/route/add) dengan tujuan dst-address (misal ::/0) dan IPv6 gateway ISP.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ipv6/route/remove</div>
          </div>
          <div class="ep-desc">Hapus IPv6 Route berdasarkan ID.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ipv6/pool/add</div>
          </div>
          <div class="ep-desc">Tambah IPv6 Address Pool (/ipv6/pool/add) dengan alokasi prefix dan prefix-length (contoh 2001:db8::/48 prefix-length=64).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ipv6/pool/remove</div>
          </div>
          <div class="ep-desc">Hapus IPv6 Pool dari router.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ipv6/dhcp-client</div>
          </div>
          <div class="ep-desc">Daftar DHCPv6 Client (/ipv6/dhcp-client/print) peminta Prefix Delegation dari uplink provider.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ipv6/dhcp-client/add</div>
          </div>
          <div class="ep-desc">Tambah DHCPv6 Client pada interface WAN untuk menarik IPv6 Prefix Delegation otomatis dari ISP ke pool lokal.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ipv6/dhcp-client/remove</div>
          </div>
          <div class="ep-desc">Hapus DHCPv6 Client dari interface WAN.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ipv6/dhcp-server</div>
          </div>
          <div class="ep-desc">Daftar DHCPv6 Server (/ipv6/dhcp-server/print) yang melayani alokasi prefix/address ke klien LAN.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ipv6/dhcp-server/add</div>
          </div>
          <div class="ep-desc">Tambah DHCPv6 Server pada interface LAN dengan rujukan address-pool IPv6.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ipv6/dhcp-server/remove</div>
          </div>
          <div class="ep-desc">Hapus DHCPv6 Server dari interface LAN.</div>
        </div>
      </section>

      <!-- Section: Site-to-Site Tunneling (GRE & IPIP) -->
      <section class="doc-section" id="sec-tunneling-suite">
        <div class="section-header">🚇 Site-to-Site Tunneling: GRE &amp; IPIP Tunnels</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/vpn/gre</div>
          </div>
          <div class="ep-desc">Daftar Generic Routing Encapsulation (GRE) tunnels (/interface/gre/print) untuk interkoneksi antar kantor cabang.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/vpn/gre/add</div>
          </div>
          <div class="ep-desc">Buat interface GRE tunnel baru dengan target remote-address dan opsi enkripsi IPsec secret langsung.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/vpn/gre/remove</div>
          </div>
          <div class="ep-desc">Hapus interface GRE tunnel berdasarkan ID.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/vpn/ipip</div>
          </div>
          <div class="ep-desc">Daftar IP-in-IP (IPIP) tunnels (/interface/ipip/print) dengan overhead paket terendah untuk menghubungkan subnet kantor.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/vpn/ipip/add</div>
          </div>
          <div class="ep-desc">Buat interface IPIP tunnel baru dengan remote-address publik dan proteksi IPsec secret.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/vpn/ipip/remove</div>
          </div>
          <div class="ep-desc">Hapus interface IPIP tunnel berdasarkan ID.</div>
        </div>
      </section>

      <!-- Section: Hairpin NAT Loopback -->
      <section class="doc-section" id="sec-hairpin-nat-suite">
        <div class="section-header">🪃 Port Forwarding &amp; Hairpin NAT Loopback</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/firewall/hairpin-nat</div>
          </div>
          <div class="ep-desc">Aturan Mandiri Hairpin NAT Loopback: Mengizinkan komputer/HP dalam LAN membuka Web/CCTV internal menggunakan IP Publik/Domain DDNS tanpa error timeout.</div>
        </div>
      </section>

      <!-- Section: IPv6 Assistant & 1-Click Wizard -->
      <section class="doc-section" id="sec-ipv6-assistant">
        <div class="section-header">🧠 IPv6 Assistant &amp; 1-Click Quick Setup Wizard</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ipv6/explain</div>
          </div>
          <div class="ep-desc">IPv6 Analyzer &amp; Explainer: Masukkan string IPv6 apa saja (misal 2001:db8::1/64 atau fe80::1) dan dapatkan analisis ramah pemula (tipe alamat, lingkup scope, kemampuan routing internet, kapasitas subnet /64, serta contoh perintah RouterOS).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/ipv6/wizard/quick-setup</div>
          </div>
          <div class="ep-desc">1-Klik Setup IPv6 ISP to LAN: Cukup masukkan WAN &amp; LAN interface! Otomatis memasang DHCPv6 Client prefix delegation, IPv6 address LAN, Neighbor Discovery (ND) SLAAC, dan baseline IPv6 firewall hardening tanpa pusing menghitung heksadesimal.</div>
        </div>
      </section>

      <!-- Section: Advanced BGP & Policy Routing Tables -->
      <section class="doc-section" id="sec-advanced-routing">
        <div class="section-header">🗺️ BGP Peering &amp; Policy Routing Rules (PBR)</div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/routing/bgp/peer/add</div>
          </div>
          <div class="ep-desc">Tambah Sesi BGP Peering: Membuat koneksi BGP Carrier Uplink yang otomatis kompatibel dengan RouterOS v7 (/routing/bgp/connection) dan v6 (/routing/bgp/peer).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/routing/tables</div>
          </div>
          <div class="ep-desc">Daftar Tabel Routing FIB: Menampilkan seluruh tabel routing independen (wajib di RouterOS v7 untuk Multi-WAN PBR).</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/routing/table/add</div>
          </div>
          <div class="ep-desc">Tambah Tabel Routing FIB Baru: Mendaftarkan nama tabel baru (misal to-WAN2) dengan flag fib=yes.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-get">GET</span><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/routing/rules</div>
          </div>
          <div class="ep-desc">Daftar Routing Rules (/ip/route/rule): Menampilkan aturan kebijakan routing berdasarkan IP asal, IP tujuan, atau routing-mark.</div>
        </div>

        <div class="ep-doc-card">
          <div class="ep-top">
            <div class="methods-group"><span class="m-pill m-post">POST</span></div>
            <div class="ep-path">/api/v1/routing/rule/add</div>
          </div>
          <div class="ep-desc">Tambah Policy Routing Rule: Memetakan aliran trafik langsung ke tabel routing tertentu tanpa membebani CPU router dengan Mangle conntrack.</div>
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

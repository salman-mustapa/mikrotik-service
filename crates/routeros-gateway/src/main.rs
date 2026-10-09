pub mod error;
pub mod routes;
pub mod state;

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use state::{AppState, Config};

fn ct_eq(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn auth(State(st): State<Arc<AppState>>, req: Request, next: Next) -> Response {
    let ok = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|t| ct_eq(t, &st.token));

    if ok {
        next.run(req).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "success": false, "error": "unauthorized: invalid or missing Bearer token" })),
        )
            .into_response()
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "mikrotik-universal-rust-gateway",
        "version": "0.2.0"
    }))
}

async fn playground() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>MikroTik Universal Rust Gateway - Live Playground</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>
    :root {
      --bg: #0f172a; --card: #1e293b; --border: #334155; --text: #f8fafc;
      --muted: #94a3b8; --primary: #3b82f6; --primary-hover: #2563eb;
      --success: #10b981; --danger: #ef4444; --code-bg: #090d16;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
    body { background: var(--bg); color: var(--text); padding: 2rem 1rem; line-height: 1.5; }
    .container { max-width: 1100px; margin: 0 auto; }
    header { margin-bottom: 2rem; border-bottom: 1px solid var(--border); padding-bottom: 1rem; }
    h1 { font-size: 1.75rem; display: flex; align-items: center; gap: 0.5rem; }
    .badge { background: #1e3a8a; color: #93c5fd; font-size: 0.75rem; padding: 0.2rem 0.6rem; border-radius: 999px; }
    .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1.5rem; }
    @media (max-width: 768px) { .grid { grid-template-columns: 1fr; } }
    .card { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 1.25rem; }
    h2 { font-size: 1.1rem; margin-bottom: 1rem; color: #cbd5e1; }
    .form-group { margin-bottom: 0.85rem; }
    label { display: block; font-size: 0.8rem; font-weight: 600; margin-bottom: 0.25rem; color: var(--muted); }
    input, select, textarea {
      width: 100%; padding: 0.5rem 0.75rem; border-radius: 6px; border: 1px solid var(--border);
      background: var(--code-bg); color: var(--text); font-size: 0.9rem;
    }
    input:focus, textarea:focus { outline: none; border-color: var(--primary); }
    .btn {
      background: var(--primary); color: white; border: none; padding: 0.5rem 1rem;
      border-radius: 6px; font-weight: 600; cursor: pointer; transition: 0.2s; font-size: 0.9rem;
    }
    .btn:hover { background: var(--primary-hover); }
    .btn-row { display: flex; flex-wrap: wrap; gap: 0.5rem; margin-top: 1rem; }
    pre {
      background: var(--code-bg); border: 1px solid var(--border); border-radius: 6px;
      padding: 1rem; overflow-x: auto; font-size: 0.85rem; color: #38bdf8; min-height: 250px; max-height: 480px;
    }
    .live-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--success); display: inline-block; animation: pulse 1.5s infinite; }
    @keyframes pulse { 0% { opacity: 0.4; } 50% { opacity: 1; } 100% { opacity: 0.4; } }
  </style>
</head>
<body>
<div class="container">
    <div style="display: flex; justify-content: space-between; align-items: flex-start; flex-wrap: wrap; gap: 10px;">
      <div>
        <h1><span>🦀 MikroTik Universal Rust Engine</span> <span class="badge">v0.2.0 Active</span></h1>
        <p style="color: var(--muted); font-size: 0.9rem; margin-top: 0.25rem;">
          High-performance sub-millisecond gateway connecting any web, mobile, or backend stack to RouterOS.
        </p>
      </div>
      <div>
        <a href="/topology" target="_blank" style="display: inline-flex; align-items: center; gap: 6px; background: linear-gradient(135deg, #0284c7, #06b6d4); color: white; text-decoration: none; padding: 8px 16px; border-radius: 8px; font-size: 0.85rem; font-weight: 700; box-shadow: 0 4px 12px rgba(6, 182, 212, 0.25);">
          🌐 Buka Visualizer Relasi Topologi &rarr;
        </a>
      </div>
    </div>
  </header>

  <div class="grid">
    <div>
      <div class="card" style="margin-bottom: 1.5rem;">
        <h2>🔐 Connection Target Parameters</h2>
        <div class="form-group">
          <label>Gateway Bearer Token</label>
          <input type="text" id="token" value="change-me-to-a-long-random-string">
        </div>
        <div class="grid" style="grid-template-columns: 2fr 1fr; gap: 0.5rem;">
          <div class="form-group">
            <label>Router Host (IP / Domain)</label>
            <input type="text" id="host" value="ath.vpnbersama.us">
          </div>
          <div class="form-group">
            <label>API Port</label>
            <input type="number" id="port" value="51121">
          </div>
        </div>
        <div class="grid" style="grid-template-columns: 1fr 1fr; gap: 0.5rem;">
          <div class="form-group">
            <label>Router Username</label>
            <input type="text" id="user" value="admin">
          </div>
          <div class="form-group">
            <label>Router Password</label>
            <input type="password" id="password" placeholder="Enter router password">
          </div>
        </div>
      </div>

      <div class="card">
        <h2>⚡ Quick Action Testers</h2>
        <p style="font-size: 0.8rem; color: var(--muted); margin-bottom: 0.75rem;">
          Klik salah satu aksi untuk mengeksekusi langsung ke router melalui persistent pool Rust:
        </p>
        <div class="btn-row">
          <button class="btn" style="background: #10b981;" onclick="execApi('/api/v1/overview', {})">⚡ Fast-Path Overview (Sub-ms)</button>
          <button class="btn" style="background: #0ea5e9;" onclick="execApi('/api/v1/network/connected-devices', {})">🌐 Unified Devices Map (Cross-Layer)</button>
          <button class="btn" style="background: #f59e0b; color: #000;" onclick="execApi('/api/v1/expert/quick-diagnose', {})">🩺 Expert Quick Diagnose</button>
          <button class="btn" style="background: #06b6d4;" onclick="execApi('/api/v1/expert/traffic-matrix', {})">📊 All Ports Traffic Matrix</button>
          <button class="btn" style="background: #e11d48;" onclick="execApi('/api/v1/security/vulnerability-audit', {})">🔎 Deep CVE & Arch Audit</button>
          <button class="btn" style="background: #be123c;" onclick="execApi('/api/v1/security/deploy-antibruteforce', { blacklist_timeout: '7d' })">🛡️ Deploy Anti-Bruteforce</button>
          <button class="btn" style="background: #0284c7;" onclick="execApi('/api/v1/dude/status', {})">📡 The Dude Status</button>
          <button class="btn" style="background: #059669;" onclick="execApi('/api/v1/network/infrastructure/scan', {})">📡 Scan AP & Infra Devices</button>
          <button class="btn" style="background: #2563eb;" onclick="execApi('/api/v1/traffic/preset/game-social-separation', { total_bandwidth: '50M', game_reserved: '10M' })">🎮 Pisah Trafik Game & Sosmed</button>
          <button class="btn" style="background: #d97706;" onclick="execApi('/api/v1/tools/torch', { interface: 'ether1' })">🔦 Live Torch Sniffer (ether1)</button>
          <button class="btn" style="background: #7c3aed;" onclick="execApi('/api/v1/tools/romon/status', {})">🌐 RoMON Status</button>
          <button class="btn" style="background: #0d9488;" onclick="execApi('/api/v1/telegram/list-monitors', {})">✈️ Netwatch Telegram Alerts</button>
          <button class="btn" style="background: #ef4444;" onclick="execApi('/api/v1/security/app-block', { app_type: 'whatsapp' })">🚫 Blokir WhatsApp</button>
          <button class="btn" style="background: #e11d48;" onclick="execApi('/api/v1/security/anti-tethering/enable', { hotspot_interface: 'bridge' })">📵 Anti-Tethering (TTL=1)</button>
          <button class="btn" style="background: #0ea5e9;" onclick="execApi('/api/v1/firewall/port-forward', { dst_port: '8080', to_addresses: '192.168.88.50', to_ports: '80' })">🔀 Port Forwarding (Dst-NAT)</button>
          <button class="btn" style="background: #8b5cf6;" onclick="execApi('/api/v1/hotspot/wizard/setup', { interface: 'ether2', local_address: '192.168.50.1/24', dhcp_pool_range: '192.168.50.10-192.168.50.254', dns_name: 'login.wifi' })">🧙‍♂️ 1-Klik Hotspot Setup</button>
          <button class="btn" style="background: #0284c7;" onclick="execApi('/api/v1/queues/inspect-user', { query: '192.168.88.50' })">🔍 Cek Limit & Speed User</button>
          <button class="btn" style="background: #0d9488;" onclick="execApi('/api/v1/queues/overview-summary', {})">📊 Rekap Bandwidth & Top Downloaders</button>
          <button class="btn" style="background: #6366f1;" onclick="execApi('/api/v1/load-balance/pcc/setup', { lan_interface: 'bridge', wans: [{ interface: 'ether1', gateway: '192.168.1.1', weight: 1 }, { interface: 'ether2', gateway: '192.168.2.1', weight: 1 }] })">⚖️ 1-Klik Multi-WAN PCC Load Balance</button>
          <button class="btn" style="background: #4f46e5;" onclick="execApi('/api/v1/load-balance/status', {})">📈 Status Load Balance</button>
          <button class="btn" style="background: #8b5cf6;" onclick="execApi('/api/v1/hotspot/generate-batch', { qty: 5, prefix: 'V-', profile: 'default' })">🎟️ Generate 5 Vouchers</button>
          <button class="btn" onclick="execApi('/api/v1/hotspot/hosts', {})">Hotspot Hosts (All Devices)</button>
          <button class="btn" onclick="execApi('/api/v1/ip/arp', {})">ARP Table</button>
          <button class="btn" onclick="execApi('/api/v1/system/resource', {})">System Resource</button>
          <button class="btn" onclick="execApi('/api/v1/system/identity', {})">Identity</button>
          <button class="btn" onclick="execApi('/api/v1/interfaces/all', {})">All Interfaces</button>
          <button class="btn" onclick="execApi('/api/v1/ip/addresses', {})">IPv4 Addresses</button>
          <button class="btn" onclick="execApi('/api/v1/ipv6/addresses', {})">IPv6 Addresses</button>
          <button class="btn" onclick="execApi('/api/v1/dhcp/leases', {})">DHCP Leases</button>
          <button class="btn" onclick="execApi('/api/v1/hotspot/users', {})">Hotspot Users</button>
          <button class="btn" onclick="execApi('/api/v1/hotspot/active', {})">Hotspot Online</button>
          <button class="btn" onclick="execApi('/api/v1/ppp/secrets', {})">PPPoE Secrets</button>
          <button class="btn" onclick="execApi('/api/v1/ppp/servers', {})">PPPoE Servers</button>
          <button class="btn" onclick="execApi('/api/v1/wireless/registrations', {})">WiFi Clients</button>
          <button class="btn" onclick="execApi('/api/v1/tools/ping', { address: '8.8.8.8', count: 3 })">Ping Test</button>
          <button class="btn" onclick="execApi('/api/v1/neighbors/all', {})">Neighbors Scan</button>
          <button class="btn" onclick="execApi('/api/v1/system/check-update', {})">Check Updates</button>
        </div>
      </div>
    </div>

    <div>
      <div class="card">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;">
          <h2>📊 Live API Response Output</h2>
          <span id="timing" style="font-size: 0.8rem; color: var(--muted);">Ready</span>
        </div>
        <pre id="output">// Results will appear here in clean JSON...</pre>
      </div>
    </div>
  </div>
</div>

<script>
async function execApi(path, payload) {
  const token = document.getElementById('token').value;
  const host = document.getElementById('host').value;
  const port = parseInt(document.getElementById('port').value) || 8728;
  const user = document.getElementById('user').value;
  const password = document.getElementById('password').value;

  const out = document.getElementById('output');
  const timing = document.getElementById('timing');
  out.textContent = "Executing request via Rust persistent pool...";
  timing.textContent = "Running...";

  const t0 = performance.now();
  try {
    const res = await fetch(path, {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + token,
        'X-Router-Host': host,
        'X-Router-Port': port.toString(),
        'X-Router-User': user,
        'X-Router-Pass': password,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(payload)
    });
    const t1 = performance.now();
    const data = await res.json();
    timing.innerHTML = `<span class="live-dot"></span> Status: ${res.status} | Time: ${(t1 - t0).toFixed(1)} ms`;
    out.textContent = JSON.stringify(data, null, 2);
  } catch (err) {
    timing.textContent = "Failed";
    out.textContent = "Error: " + err.message;
  }
}
</script>
</body>
</html>"#)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let path = std::env::args().nth(1).unwrap_or_else(|| "config.toml".into());
    let cfg: Config = toml::from_str(&std::fs::read_to_string(&path)?)?;
    let listen_addr = cfg.listen.clone();

    let state = Arc::new(AppState::new(cfg));

    // Protected API router with bearer auth
    let protected = routes::build_api_router(state.clone())
        .layer(middleware::from_fn_with_state(state.clone(), auth));

    // Combined router with web playground, health check, visualizer, websocket & permissive CORS
    let app = Router::new()
        .route("/", get(playground))
        .route("/health", get(health))
        .route("/topology", get(routes::visualizer::visualizer_page))
        .route("/visualizer", get(routes::visualizer::visualizer_page))
        .route("/sdk/mikrotik-widget.js", get(routes::visualizer::sdk_script))
        .route("/ws", get(routes::ws::ws_handler))
        .merge(protected)
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    tracing::info!("MikroTik Universal Gateway listening on http://{}", listen_addr);
    axum::serve(listener, app).await?;
    Ok(())
}

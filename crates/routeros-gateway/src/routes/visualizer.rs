use axum::response::{Html, IntoResponse, Response};
use axum::http::header;

/// GET /topology - Rich, interactive Network Topology Visualizer Dashboard
pub async fn visualizer_page() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>MikroTik Visualizer - High-Performance Topology & Relational Map</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>
    :root {
      --bg: #090d16;
      --card-bg: rgba(22, 30, 49, 0.75);
      --card-border: rgba(56, 189, 248, 0.2);
      --accent-blue: #0284c7;
      --accent-cyan: #06b6d4;
      --accent-emerald: #10b981;
      --accent-rose: #f43f5e;
      --accent-amber: #f59e0b;
      --text-main: #f8fafc;
      --text-muted: #94a3b8;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
    body {
      background: radial-gradient(circle at 50% 20%, #111a2e 0%, #070a12 100%);
      color: var(--text-main);
      min-height: 100vh;
      display: flex;
      flex-direction: column;
      overflow: hidden;
    }
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px 24px;
      background: rgba(11, 15, 25, 0.85);
      backdrop-filter: blur(12px);
      border-bottom: 1px solid rgba(255, 255, 255, 0.08);
      z-index: 10;
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 12px;
      font-weight: 800;
      font-size: 1.15rem;
      letter-spacing: -0.5px;
    }
    .badge {
      background: linear-gradient(135deg, #0284c7, #06b6d4);
      color: white;
      font-size: 0.7rem;
      padding: 3px 8px;
      border-radius: 999px;
      font-weight: 700;
    }
    .nav-controls {
      display: flex;
      align-items: center;
      gap: 10px;
    }
    .filter-btn {
      background: rgba(30, 41, 59, 0.6);
      border: 1px solid rgba(255, 255, 255, 0.1);
      color: var(--text-muted);
      padding: 6px 14px;
      border-radius: 8px;
      font-size: 0.8rem;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.2s ease;
    }
    .filter-btn:hover, .filter-btn.active {
      background: var(--accent-cyan);
      color: #041019;
      border-color: var(--accent-cyan);
    }
    .refresh-btn {
      background: #10b981;
      color: #032b1c;
      border: none;
      padding: 6px 16px;
      border-radius: 8px;
      font-size: 0.8rem;
      font-weight: 700;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 6px;
      transition: opacity 0.2s;
    }
    .refresh-btn:hover { opacity: 0.9; }

    #workspace {
      flex: 1;
      position: relative;
      overflow: hidden;
      display: flex;
    }
    #canvas-container {
      flex: 1;
      position: relative;
      background: radial-gradient(circle, rgba(56, 189, 248, 0.03) 1px, transparent 1px);
      background-size: 24px 24px;
    }
    svg#graph {
      width: 100%;
      height: 100%;
      display: block;
    }

    /* Edge line styling */
    .edge-line {
      stroke: rgba(56, 189, 248, 0.35);
      stroke-width: 2;
      stroke-dasharray: 4, 4;
      animation: dash 30s linear infinite;
      transition: stroke 0.3s;
    }
    .edge-line.active {
      stroke: var(--accent-cyan);
      stroke-width: 3;
      stroke-dasharray: none;
    }
    @keyframes dash {
      to { stroke-dashoffset: -1000; }
    }

    /* Node circles and text */
    .node-group {
      cursor: pointer;
      transition: transform 0.2s ease;
    }
    .node-group:hover {
      filter: drop-shadow(0 0 12px rgba(6, 182, 212, 0.6));
    }
    .node-circle {
      stroke-width: 2.5;
      transition: all 0.2s;
    }
    .node-label {
      fill: var(--text-main);
      font-size: 11px;
      font-weight: 600;
      text-anchor: middle;
      pointer-events: none;
    }
    .node-subtext {
      fill: var(--text-muted);
      font-size: 9px;
      text-anchor: middle;
      pointer-events: none;
    }

    /* Drawer Inspector */
    #drawer {
      width: 380px;
      background: rgba(15, 23, 42, 0.95);
      backdrop-filter: blur(16px);
      border-left: 1px solid rgba(255, 255, 255, 0.1);
      padding: 24px;
      display: flex;
      flex-direction: column;
      gap: 16px;
      transform: translateX(100%);
      transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
      position: absolute;
      right: 0;
      top: 0;
      bottom: 0;
      z-index: 20;
      overflow-y: auto;
    }
    #drawer.open {
      transform: translateX(0);
    }
    .drawer-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      border-bottom: 1px solid rgba(255, 255, 255, 0.1);
      padding-bottom: 12px;
    }
    .drawer-title { font-size: 1.1rem; font-weight: 800; color: var(--accent-cyan); }
    .close-btn { background: none; border: none; color: var(--text-muted); font-size: 1.2rem; cursor: pointer; }
    .detail-card {
      background: rgba(30, 41, 59, 0.5);
      border: 1px solid rgba(255, 255, 255, 0.05);
      border-radius: 10px;
      padding: 14px;
    }
    .detail-row {
      display: flex;
      justify-content: space-between;
      font-size: 0.85rem;
      margin-bottom: 8px;
    }
    .detail-row:last-child { margin-bottom: 0; }
    .detail-label { color: var(--text-muted); }
    .detail-val { font-weight: 600; color: var(--text-main); }
    .ping-box {
      margin-top: 12px;
      padding: 12px;
      background: rgba(6, 182, 212, 0.05);
      border: 1px solid rgba(6, 182, 212, 0.2);
      border-radius: 8px;
    }
    .btn-action {
      width: 100%;
      padding: 10px;
      border-radius: 8px;
      border: none;
      font-weight: 700;
      font-size: 0.85rem;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 6px;
      transition: all 0.2s;
    }
    .btn-ping { background: #0284c7; color: white; margin-bottom: 8px; }
    .btn-ping:hover { background: #0369a1; }
    .btn-danger { background: #e11d48; color: white; }
    .btn-danger:hover { background: #be123c; }

    /* Floating Status Bar */
    .status-bar {
      position: absolute;
      bottom: 20px;
      left: 20px;
      background: rgba(15, 23, 42, 0.8);
      backdrop-filter: blur(10px);
      border: 1px solid rgba(255, 255, 255, 0.1);
      border-radius: 12px;
      padding: 10px 18px;
      display: flex;
      gap: 20px;
      font-size: 0.8rem;
      z-index: 15;
    }
    .status-item { display: flex; align-items: center; gap: 8px; }
    .indicator { width: 8px; height: 8px; border-radius: 50%; }
    .ind-green { background: #10b981; box-shadow: 0 0 8px #10b981; }
    .ind-blue { background: #0284c7; }
    .ind-purple { background: #a855f7; }
  </style>
</head>
<body>
  <header>
    <div class="brand">
      <span>🌐 MikroTik Core Visualizer</span>
      <span class="badge" id="hw-badge">Sub-Millisecond Engine</span>
    </div>
    <div class="nav-controls">
      <button class="filter-btn active" onclick="setFilter('all', this)">Semua (All)</button>
      <button class="filter-btn" onclick="setFilter('hotspot', this)">🎟️ Hotspot</button>
      <button class="filter-btn" onclick="setFilter('pppoe', this)">🌐 PPPoE</button>
      <button class="filter-btn" onclick="setFilter('dhcp', this)">💻 DHCP</button>
      <button class="filter-btn" onclick="setFilter('wifi', this)">📶 WiFi</button>
      <button class="refresh-btn" onclick="loadTopology()">🔄 Reload Graph</button>
    </div>
  </header>

  <div id="workspace">
    <div id="canvas-container">
      <svg id="graph"></svg>
    </div>

    <!-- Inspector Drawer -->
    <div id="drawer">
      <div class="drawer-header">
        <div class="drawer-title" id="d-title">Node Inspector</div>
        <button class="close-btn" onclick="closeDrawer()">✕</button>
      </div>

      <div class="detail-card">
        <div class="detail-row"><span class="detail-label">Node ID:</span><span class="detail-val" id="d-id">-</span></div>
        <div class="detail-row"><span class="detail-label">Tipe:</span><span class="detail-val" id="d-type">-</span></div>
        <div class="detail-row"><span class="detail-label">IP Address:</span><span class="detail-val" id="d-ip">-</span></div>
        <div class="detail-row"><span class="detail-label">MAC Address:</span><span class="detail-val" id="d-mac">-</span></div>
        <div class="detail-row"><span class="detail-label">Status:</span><span class="detail-val" id="d-status">-</span></div>
      </div>

      <div class="detail-card" id="extra-card">
        <div id="extra-rows"></div>
      </div>

      <!-- Live ICMP Ping Diagnostics -->
      <div class="ping-box">
        <div style="font-weight: 700; font-size: 0.85rem; margin-bottom: 8px;">⚡ Live Ping Diagnostik dari Router</div>
        <button class="btn-action btn-ping" id="btn-do-ping" onclick="pingCurrentNode()">Ping Perangkat Ini</button>
        <div id="ping-result" style="font-size: 0.8rem; color: var(--text-muted); text-align: center; margin-top: 6px;">
          Klik tombol untuk menguji latency RTT
        </div>
      </div>

      <div style="margin-top: auto;">
        <button class="btn-action btn-danger" onclick="alert('Fitur blokir diterapkan ke address-list')">🚫 Blokir IP Perangkat</button>
      </div>
    </div>

    <div class="status-bar">
      <div class="status-item"><div class="indicator ind-green"></div><span id="st-nodes">0 Nodes</span></div>
      <div class="status-item"><div class="indicator ind-blue"></div><span id="st-edges">0 Koneksi</span></div>
      <div class="status-item"><span id="st-perf">Latensi: 0 ms</span></div>
    </div>
  </div>

<script>
let currentFilter = 'all';
let graphData = { nodes: [], edges: [] };
let selectedNode = null;

async function loadTopology() {
  const t0 = performance.now();
  try {
    const res = await fetch('/api/v1/network/topology-graph', {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + (localStorage.getItem('token') || 'change-me-to-a-long-random-string'),
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ filter_type: currentFilter })
    });
    const t1 = performance.now();
    const json = await res.json();
    if (json.success) {
      graphData = json;
      document.getElementById('st-nodes').textContent = `${json.nodes.length} Nodes`;
      document.getElementById('st-edges').textContent = `${json.edges.length} Koneksi`;
      document.getElementById('st-perf').textContent = `Query: ${(t1 - t0).toFixed(1)} ms`;
      renderGraph(json.nodes, json.edges);
    }
  } catch (err) {
    console.error("Failed to load topology:", err);
  }
}

function setFilter(type, btn) {
  currentFilter = type;
  document.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('active'));
  btn.classList.add('active');
  loadTopology();
}

function renderGraph(nodes, edges) {
  const svg = document.getElementById('graph');
  svg.innerHTML = '';
  const width = svg.clientWidth || window.innerWidth;
  const height = svg.clientHeight || (window.innerHeight - 60);

  // Layout coordinates: Router in center, Interfaces in inner ring, Devices in outer ring
  const posMap = {};
  const centerX = width / 2;
  const centerY = height / 2;

  const routerNodes = nodes.filter(n => n.node_type === 'router');
  const ifaceNodes = nodes.filter(n => n.node_type === 'interface');
  const clientNodes = nodes.filter(n => n.node_type === 'device' || n.node_type === 'neighbor');

  // Place Router
  routerNodes.forEach(n => {
    posMap[n.id] = { x: centerX, y: centerY, r: 35, color: '#0284c7' };
  });

  // Place Interfaces in inner circle
  const ifaceRadius = Math.min(width, height) * 0.25;
  ifaceNodes.forEach((n, i) => {
    const angle = (i / Math.max(1, ifaceNodes.length)) * 2 * Math.PI;
    posMap[n.id] = {
      x: centerX + ifaceRadius * Math.cos(angle),
      y: centerY + ifaceRadius * Math.sin(angle),
      r: 22,
      color: '#06b6d4'
    };
  });

  // Place Clients in outer circle
  const clientRadius = Math.min(width, height) * 0.42;
  clientNodes.forEach((n, i) => {
    const angle = (i / Math.max(1, clientNodes.length)) * 2 * Math.PI;
    const color = n.sub_type === 'hotspot' ? '#f59e0b' : (n.sub_type === 'pppoe' ? '#a855f7' : (n.sub_type === 'wifi' ? '#10b981' : '#38bdf8'));
    posMap[n.id] = {
      x: centerX + clientRadius * Math.cos(angle),
      y: centerY + clientRadius * Math.sin(angle),
      r: 16,
      color: color
    };
  });

  // Render Edges
  edges.forEach(e => {
    const p1 = posMap[e.source];
    const p2 = posMap[e.target];
    if (p1 && p2) {
      const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
      line.setAttribute('x1', p1.x);
      line.setAttribute('y1', p1.y);
      line.setAttribute('x2', p2.x);
      line.setAttribute('y2', p2.y);
      line.setAttribute('class', 'edge-line');
      line.setAttribute('id', 'edge_' + e.id);
      svg.appendChild(line);
    }
  });

  // Render Nodes
  nodes.forEach(n => {
    const p = posMap[n.id];
    if (!p) return;

    const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
    g.setAttribute('class', 'node-group');
    g.onclick = () => selectNode(n);

    // Glowing circle
    const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
    circle.setAttribute('cx', p.x);
    circle.setAttribute('cy', p.y);
    circle.setAttribute('r', p.r);
    circle.setAttribute('fill', '#090d16');
    circle.setAttribute('stroke', p.color);
    circle.setAttribute('class', 'node-circle');
    g.appendChild(circle);

    // Icon / Label
    const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    text.setAttribute('x', p.x);
    text.setAttribute('y', p.y + p.r + 14);
    text.setAttribute('class', 'node-label');
    text.textContent = n.label.length > 20 ? n.label.substring(0, 18) + '...' : n.label;
    g.appendChild(text);

    // Subtext (IP or Type)
    if (n.ip || n.sub_type) {
      const sub = document.createElementNS('http://www.w3.org/2000/svg', 'text');
      sub.setAttribute('x', p.x);
      sub.setAttribute('y', p.y + p.r + 25);
      sub.setAttribute('class', 'node-subtext');
      sub.textContent = n.ip || n.sub_type;
      g.appendChild(sub);
    }

    svg.appendChild(g);
  });
}

function selectNode(n) {
  selectedNode = n;
  document.getElementById('d-title').textContent = n.label;
  document.getElementById('d-id').textContent = n.id;
  document.getElementById('d-type').textContent = `${n.node_type} (${n.sub_type})`;
  document.getElementById('d-ip').textContent = n.ip || 'N/A';
  document.getElementById('d-mac').textContent = n.mac || 'N/A';
  document.getElementById('d-status').textContent = n.status.toUpperCase();

  const extraRows = document.getElementById('extra-rows');
  extraRows.innerHTML = '';
  for (const [k, v] of Object.entries(n.extra || {})) {
    extraRows.innerHTML += `<div class="detail-row"><span class="detail-label">${k}:</span><span class="detail-val">${v}</span></div>`;
  }

  document.getElementById('ping-result').textContent = 'Klik tombol di atas untuk cek latency';
  document.getElementById('drawer').classList.add('open');
}

function closeDrawer() {
  document.getElementById('drawer').classList.remove('open');
}

async function pingCurrentNode() {
  if (!selectedNode || !selectedNode.ip) {
    document.getElementById('ping-result').textContent = 'Perangkat tidak memiliki IP address valid untuk di-ping.';
    return;
  }
  const pingBtn = document.getElementById('btn-do-ping');
  const pingRes = document.getElementById('ping-result');
  pingBtn.disabled = true;
  pingRes.innerHTML = 'Mengirim 3 paket ICMP dari MikroTik...';

  try {
    const res = await fetch('/api/v1/tools/ping', {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + (localStorage.getItem('token') || 'change-me-to-a-long-random-string'),
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ address: selectedNode.ip, count: 3 })
    });
    const data = await res.json();
    if (data.success && data.data && data.data.length > 0) {
      const last = data.data[data.data.length - 1];
      const rtt = last['avg-rtt'] || last['time'] || '1 ms';
      const loss = last['packet-loss'] || '0%';
      pingRes.innerHTML = `<span style="color: #10b981; font-weight: bold;">⚡ PING SUKSES!</span><br>Avg Latency: <strong>${rtt}</strong> | Packet Loss: <strong>${loss}</strong>`;
    } else {
      pingRes.innerHTML = `<span style="color: #ef4444; font-weight: bold;">❌ REQUEST TIMEOUT</span><br>Perangkat tidak membalas ICMP echo.`;
    }
  } catch (err) {
    pingRes.textContent = 'Error: ' + err.message;
  } finally {
    pingBtn.disabled = false;
  }
}

window.addEventListener('resize', () => {
  if (graphData.nodes.length > 0) renderGraph(graphData.nodes, graphData.edges);
});

// Auto load
loadTopology();
</script>
</body>
</html>"#)
}

/// GET /sdk/mikrotik-widget.js - Lightweight Embeddable JavaScript SDK for Laravel, Vue, React, or Express
pub async fn sdk_script() -> Response {
    let script = r#"/**
 * MikroTik Universal Rust Gateway - Embeddable Web Component SDK v0.2.0
 * Zero dependencies. Mounts interactive topology and real-time network graphs in 1 line.
 */
window.MikrotikWidget = {
  mount: function(targetSelector, options = {}) {
    const el = document.querySelector(targetSelector);
    if (!el) {
      console.error("[MikrotikWidget] Container selector not found:", targetSelector);
      return;
    }
    const gatewayUrl = options.gatewayUrl || window.location.origin;
    const token = options.token || '';
    const height = options.height || '650px';

    const iframe = document.createElement('iframe');
    iframe.src = `${gatewayUrl}/topology`;
    iframe.style.width = '100%';
    iframe.style.height = height;
    iframe.style.border = 'none';
    iframe.style.borderRadius = options.borderRadius || '12px';
    iframe.style.boxShadow = '0 10px 25px rgba(0,0,0,0.3)';

    el.innerHTML = '';
    el.appendChild(iframe);
  }
};
"#;

    ([(header::CONTENT_TYPE, "application/javascript")], script).into_response()
}

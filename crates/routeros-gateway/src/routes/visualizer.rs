use axum::response::{Html, IntoResponse, Response};
use axum::http::header;

/// GET /topology - Rich, interactive Network Topology Visualizer with Draggable Nodes & Light Theme
pub async fn visualizer_page() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html lang="id">
<head>
  <meta charset="UTF-8">
  <title>MikroTik Visualizer - Relasi Jaringan &amp; Topologi Interaktif</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500;700&display=swap" rel="stylesheet">
  <style>
    :root {
      --bg: #f8fafc;
      --canvas-bg: #f1f5f9;
      --card-bg: #ffffff;
      --card-border: #e2e8f0;
      --text-main: #0f172a;
      --text-muted: #64748b;
      --accent-indigo: #4f46e5;
      --accent-blue: #0284c7;
      --accent-emerald: #059669;
      --accent-amber: #d97706;
      --accent-rose: #e11d48;
      --accent-purple: #7c3aed;
      --edge-color: #94a3b8;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; font-family: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
    body {
      background: var(--bg);
      color: var(--text-main);
      min-height: 100vh;
      display: flex;
      flex-direction: column;
      overflow: hidden;
      user-select: none;
    }
    code, pre { font-family: 'JetBrains Mono', monospace; }

    /* Clean Header */
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 10px 20px;
      background: var(--card-bg);
      border-bottom: 1px solid var(--card-border);
      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
      z-index: 20;
      gap: 12px;
      flex-wrap: wrap;
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 10px;
      font-weight: 800;
      font-size: 1.05rem;
      color: var(--text-main);
    }
    .badge {
      background: #eff6ff;
      border: 1px solid #bfdbfe;
      color: #1d4ed8;
      font-size: 0.72rem;
      padding: 2px 8px;
      border-radius: 999px;
      font-weight: 700;
    }

    /* Router Target Credentials Bar */
    .creds-inline {
      display: flex;
      align-items: center;
      gap: 6px;
      background: #f8fafc;
      padding: 4px 10px;
      border-radius: 8px;
      border: 1px solid var(--card-border);
    }
    .creds-inline input {
      background: #ffffff;
      border: 1px solid #cbd5e1;
      color: var(--text-main);
      padding: 4px 8px;
      border-radius: 6px;
      font-size: 0.75rem;
      font-family: 'JetBrains Mono', monospace;
      outline: none;
    }
    .creds-inline input:focus { border-color: var(--accent-indigo); }
    .btn-connect {
      background: var(--accent-indigo);
      color: white;
      border: none;
      padding: 5px 12px;
      border-radius: 6px;
      font-size: 0.75rem;
      font-weight: 700;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 4px;
      transition: background 0.15s;
    }
    .btn-connect:hover { background: #4338ca; }

    /* Filter Buttons */
    .nav-controls {
      display: flex;
      align-items: center;
      gap: 6px;
      flex-wrap: wrap;
    }
    .filter-btn {
      background: #ffffff;
      border: 1px solid var(--card-border);
      color: var(--text-muted);
      padding: 5px 12px;
      border-radius: 7px;
      font-size: 0.78rem;
      font-weight: 600;
      cursor: pointer;
      transition: all 0.15s ease;
    }
    .filter-btn:hover { background: #f1f5f9; color: var(--text-main); }
    .filter-btn.active {
      background: var(--accent-indigo);
      color: white;
      border-color: var(--accent-indigo);
      box-shadow: 0 2px 6px rgba(79, 70, 229, 0.25);
    }
    .nav-link-btn {
      background: #f1f5f9;
      border: 1px solid var(--card-border);
      color: var(--text-main);
      padding: 5px 12px;
      border-radius: 7px;
      font-size: 0.78rem;
      font-weight: 600;
      text-decoration: none;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 4px;
    }
    .nav-link-btn:hover { background: #e2e8f0; }

    /* Canvas & Workspace Area */
    #workspace {
      flex: 1;
      height: calc(100vh - 60px);
      max-height: calc(100vh - 60px);
      min-height: 0;
      position: relative;
      overflow: hidden;
      display: flex;
    }
    #canvas-container {
      flex: 1;
      position: relative;
      background-color: var(--canvas-bg);
      background-image: radial-gradient(#cbd5e1 1.2px, transparent 1.2px);
      background-size: 24px 24px;
      cursor: grab;
      min-height: 0;
    }
    #canvas-container:active { cursor: grabbing; }

    svg#graph {
      width: 100%;
      height: 100%;
      display: block;
    }

    /* Edge line styling */
    .edge-line {
      stroke: var(--edge-color);
      stroke-width: 2;
      stroke-dasharray: 5, 5;
      animation: dashflow 25s linear infinite;
    }
    .edge-wan {
      stroke: #0284c7 !important;
      stroke-width: 3.2px !important;
      stroke-dasharray: 7, 4 !important;
      animation: wandashflow 12s linear infinite !important;
    }
    @keyframes dashflow {
      to { stroke-dashoffset: -1000; }
    }
    @keyframes wandashflow {
      to { stroke-dashoffset: -1000; }
    }

    /* Node Styling */
    .node-group {
      cursor: pointer;
      transition: transform 0.1s;
    }
    .node-group:hover circle {
      filter: drop-shadow(0 4px 10px rgba(0, 0, 0, 0.18));
    }
    .node-label {
      font-size: 11px;
      font-weight: 700;
      fill: #0f172a;
      text-anchor: middle;
      pointer-events: none;
      paint-order: stroke fill;
      stroke: #ffffff;
      stroke-width: 3.5px;
      stroke-linejoin: round;
    }
    .node-subtext {
      font-size: 9.5px;
      font-weight: 600;
      fill: #475569;
      text-anchor: middle;
      pointer-events: none;
      paint-order: stroke fill;
      stroke: #ffffff;
      stroke-width: 2.5px;
      stroke-linejoin: round;
    }

    /* Floating Zoom Controls */
    .zoom-toolbar {
      position: absolute;
      bottom: 20px;
      left: 20px;
      background: white;
      border: 1px solid var(--card-border);
      border-radius: 8px;
      padding: 4px;
      display: flex;
      gap: 4px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
      z-index: 10;
    }
    .zoom-btn {
      background: transparent;
      border: none;
      width: 30px;
      height: 30px;
      border-radius: 6px;
      font-size: 0.9rem;
      font-weight: 700;
      color: var(--text-main);
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
    }
    .zoom-btn:hover { background: #f1f5f9; }

    /* Node Inspector Drawer with Smooth Internal Scrolling */
    #drawer {
      width: 380px;
      min-width: 380px;
      max-width: 380px;
      height: 100%;
      max-height: 100%;
      min-height: 0;
      background: var(--card-bg);
      border-left: 1px solid var(--card-border);
      box-shadow: -4px 0 24px rgba(0, 0, 0, 0.08);
      display: none;
      flex-direction: column;
      padding: 18px 18px 40px 18px;
      z-index: 25;
      overflow-y: auto !important;
      overflow-x: hidden;
      box-sizing: border-box;
    }
    #drawer::-webkit-scrollbar {
      width: 6px;
    }
    #drawer::-webkit-scrollbar-track {
      background: #f1f5f9;
      border-radius: 4px;
    }
    #drawer::-webkit-scrollbar-thumb {
      background: #cbd5e1;
      border-radius: 4px;
    }
    #drawer::-webkit-scrollbar-thumb:hover {
      background: #94a3b8;
    }
    .scrollable-device-list::-webkit-scrollbar {
      width: 5px;
    }
    .scrollable-device-list::-webkit-scrollbar-track {
      background: #f1f5f9;
    }
    .scrollable-device-list::-webkit-scrollbar-thumb {
      background: #cbd5e1;
      border-radius: 3px;
    }
    .drawer-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;
      padding-bottom: 12px;
      border-bottom: 1px solid var(--card-border);
    }
    .drawer-title { font-weight: 800; font-size: 1.05rem; color: var(--text-main); }
    .close-btn {
      background: transparent;
      border: none;
      color: var(--text-muted);
      font-size: 1.2rem;
      cursor: pointer;
      padding: 4px;
    }
    .close-btn:hover { color: var(--text-main); }

    .detail-card {
      background: #f8fafc;
      border: 1px solid var(--card-border);
      border-radius: 10px;
      padding: 14px;
      margin-bottom: 14px;
    }
    .detail-row {
      display: flex;
      justify-content: space-between;
      padding: 5px 0;
      font-size: 0.8rem;
      border-bottom: 1px dashed #e2e8f0;
    }
    .detail-row:last-child { border-bottom: none; }
    .detail-label { color: var(--text-muted); font-weight: 600; }
    .detail-val { font-weight: 700; color: var(--text-main); font-family: 'JetBrains Mono', monospace; text-align: right; }

    /* Diagnostic Ping Box */
    .ping-box {
      background: #eff6ff;
      border: 1px solid #bfdbfe;
      border-radius: 10px;
      padding: 14px;
      margin-bottom: 14px;
    }
    .btn-action {
      width: 100%;
      padding: 8px;
      border-radius: 7px;
      font-size: 0.8rem;
      font-weight: 700;
      cursor: pointer;
      border: none;
      transition: all 0.15s;
    }
    .btn-ping { background: var(--accent-blue); color: white; }
    .btn-ping:hover { background: #0369a1; }
    .btn-danger { background: #fee2e2; border: 1px solid #fecaca; color: #dc2626; margin-top: 8px; }
    .btn-danger:hover { background: #fecaca; }

    /* Status Bar */
    .status-bar {
      position: absolute;
      bottom: 20px;
      right: 20px;
      background: white;
      border: 1px solid var(--card-border);
      border-radius: 8px;
      padding: 6px 14px;
      display: flex;
      gap: 16px;
      font-size: 0.75rem;
      font-weight: 600;
      color: var(--text-muted);
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.06);
    }
    .status-item { display: flex; align-items: center; gap: 6px; }
    .dot-online { width: 7px; height: 7px; border-radius: 50%; background: var(--accent-emerald); }

    /* Helper notification banner */
    #notice-box {
      position: absolute;
      top: 20px;
      left: 50%;
      transform: translateX(-50%);
      background: white;
      border: 1px solid var(--card-border);
      color: var(--text-main);
      padding: 8px 18px;
      border-radius: 8px;
      font-size: 0.82rem;
      font-weight: 600;
      box-shadow: 0 6px 20px rgba(0, 0, 0, 0.08);
      display: none;
      z-index: 100;
    }
  </style>
</head>
<body>

  <!-- Clean Enterprise Light Header -->
  <header>
    <div class="brand">
      <span>🌐</span>
      <span>MikroTik Topology Map</span>
      <span class="badge" id="hw-badge">Relasi Jaringan Hierarkis</span>
      <span class="badge" style="background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe;">115+ Endpoints API</span>
    </div>

    <!-- Active Router Target Bar -->
    <div class="creds-inline">
      <span style="font-size: 0.72rem; color: var(--text-muted); font-weight: 700;">Router:</span>
      <input type="text" id="target-host" placeholder="Host IP / Domain" style="width: 130px;" value="192.168.88.1">
      <input type="number" id="target-port" placeholder="Port" style="width: 60px;" value="8728">
      <input type="text" id="target-user" placeholder="User" style="width: 75px;" value="admin">
      <input type="password" id="target-pass" placeholder="Password" style="width: 80px;">
      <button class="btn-connect" onclick="loadTopology()">⚡ Render Graf</button>
    </div>

    <!-- Category Filters & Navigation -->
    <div class="nav-controls">
      <button class="filter-btn active" onclick="setFilter('all', this)">Semua (All)</button>
      <button class="filter-btn" onclick="setFilter('hotspot', this)">🎟️ Hotspot</button>
      <button class="filter-btn" onclick="setFilter('pppoe', this)">🌐 PPPoE</button>
      <button class="filter-btn" onclick="setFilter('dhcp', this)">💻 DHCP</button>
      <button class="filter-btn" onclick="setFilter('wifi', this)">📶 WiFi</button>
      <a href="/" class="nav-link-btn">🎮 Live Console</a>
      <a href="/docs" target="_blank" class="nav-link-btn">📖 API Docs</a>
    </div>
  </header>

  <div id="notice-box">Memuat relasi perangkat dari router...</div>

  <div id="workspace">
    <!-- SVG Canvas with Drag & Zoom -->
    <div id="canvas-container">
      <svg id="graph">
        <g id="viewport-group">
          <g id="edges-layer"></g>
          <g id="nodes-layer"></g>
        </g>
      </svg>

      <!-- Zoom Floating Bar -->
      <div class="zoom-toolbar">
        <button class="zoom-btn" onclick="zoomIn()" title="Perbesar (Zoom In)">+</button>
        <button class="zoom-btn" onclick="zoomOut()" title="Perkecil (Zoom Out)">&minus;</button>
        <button class="zoom-btn" onclick="resetZoom()" title="Reset Tampilan (100%)">↺</button>
      </div>
    </div>

    <!-- Right Inspector Drawer -->
    <div id="drawer">
      <div class="drawer-header">
        <div class="drawer-title" id="d-title">Detail Perangkat</div>
        <button class="close-btn" onclick="closeDrawer()">✕</button>
      </div>

      <div class="detail-card">
        <div class="detail-row"><span class="detail-label">Node ID:</span><span class="detail-val" id="d-id">-</span></div>
        <div class="detail-row"><span class="detail-label">Tipe Node:</span><span class="detail-val" id="d-type">-</span></div>
        <div class="detail-row"><span class="detail-label">IP Address:</span><span class="detail-val" id="d-ip">-</span></div>
        <div class="detail-row"><span class="detail-label">MAC Address:</span><span class="detail-val" id="d-mac">-</span></div>
        <div class="detail-row"><span class="detail-label">Vendor / Brand:</span><span class="detail-val" id="d-vendor">-</span></div>
        <div class="detail-row"><span class="detail-label">Tercolok Pada:</span><span class="detail-val" id="d-parent">-</span></div>
        <div class="detail-row"><span class="detail-label">Status Koneksi:</span><span class="detail-val" id="d-status">-</span></div>
      </div>

      <div class="detail-card" id="extra-card">
        <div id="extra-rows"></div>
      </div>

      <!-- Live ICMP Ping Diagnostics -->
      <div class="ping-box">
        <div style="font-weight: 700; font-size: 0.82rem; margin-bottom: 6px; color: #1e3a8a;">⚡ Live Ping dari Router</div>
        <button class="btn-action btn-ping" id="btn-do-ping" onclick="pingCurrentNode()">Ping Perangkat Ini (ICMP)</button>
        <div id="ping-result" style="font-size: 0.78rem; color: var(--text-muted); text-align: center; margin-top: 6px;">
          Uji latency round-trip time (RTT)
        </div>
      </div>

      <div style="margin-top: 12px; padding-bottom: 24px;">
        <button class="btn-action btn-danger" onclick="blockCurrentNode()">🚫 Blokir IP Perangkat di Firewall</button>
      </div>
    </div>

    <!-- Status Bar -->
    <div class="status-bar">
      <div class="status-item"><div class="dot-online"></div><span id="st-nodes">0 Nodes</span></div>
      <div class="status-item"><span id="st-edges">0 Koneksi Cabang</span></div>
      <div class="status-item"><span id="st-perf">Latency: 0 ms</span></div>
    </div>
  </div>

  <script>
    let currentFilter = 'all';
    let graphData = { nodes: [], edges: [] };
    let selectedNode = null;
    let nodePositions = {}; // id -> { x, y, r, data }

    // Canvas Pan & Zoom State
    let scale = 1.0;
    let panX = 0;
    let panY = 0;
    let isPanning = false;
    let startPanX = 0;
    let startPanY = 0;

    // Node Dragging State
    let draggedNodeId = null;
    let dragStartX = 0;
    let dragStartY = 0;

    function syncCreds() {
      localStorage.setItem('ros_host', document.getElementById('target-host').value);
      localStorage.setItem('ros_port', document.getElementById('target-port').value);
      localStorage.setItem('ros_user', document.getElementById('target-user').value);
      localStorage.setItem('ros_pass', document.getElementById('target-pass').value);
    }

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

      ['target-host', 'target-port', 'target-user', 'target-pass'].forEach(id => {
        document.getElementById(id).addEventListener('input', syncCreds);
      });
    }

    function showNotice(msg, isErr = false) {
      const box = document.getElementById('notice-box');
      box.textContent = msg;
      box.style.display = 'block';
      box.style.color = isErr ? '#dc2626' : '#0f172a';
      if (!isErr) {
        setTimeout(() => { box.style.display = 'none'; }, 2500);
      }
    }

    async function loadTopology() {
      syncCreds();
      const host = document.getElementById('target-host').value.trim();
      const port = parseInt(document.getElementById('target-port').value) || 8728;
      const user = document.getElementById('target-user').value.trim();
      const pass = document.getElementById('target-pass').value;
      const token = localStorage.getItem('ros_token') || 'change-me-to-a-long-random-string';

      showNotice("Menghubungkan ke MikroTik & mengkorelasikan relasi cabang...");
      const t0 = performance.now();

      try {
        const res = await fetch(`/api/v1/network/topology-graph?filter=${currentFilter}`, {
          method: 'GET',
          headers: {
            'Authorization': 'Bearer ' + token,
            'X-Router-Host': host,
            'X-Router-Port': port.toString(),
            'X-Router-User': user,
            'X-Router-Pass': pass,
            'Content-Type': 'application/json'
          }
        });
        const t1 = performance.now();
        const json = await res.json();

        if (json.success && json.nodes) {
          graphData = json;
          document.getElementById('st-nodes').textContent = `${json.nodes.length} Nodes`;
          document.getElementById('st-edges').textContent = `${json.edges.length} Cabang`;
          document.getElementById('st-perf').textContent = `Latency: ${(t1 - t0).toFixed(1)} ms`;
          buildHierarchicalLayout(json.nodes, json.edges);
          document.getElementById('notice-box').style.display = 'none';
        } else {
          showNotice("⚠️ " + (json.error || "Gagal memuat topologi"), true);
        }
      } catch (err) {
        showNotice("❌ Error koneksi: " + err.message, true);
      }
    }

    function setFilter(type, btn) {
      currentFilter = type;
      document.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      loadTopology();
    }

    /* Hierarchical Multi-Tier Branch Layout: Internet -> WAN Port -> Core Router -> Interfaces & APs -> Client Devices */
    function buildHierarchicalLayout(nodes, edges) {
      const svg = document.getElementById('graph');
      const width = svg.clientWidth || window.innerWidth;
      const height = svg.clientHeight || (window.innerHeight - 60);

      const centerX = width / 2;
      const centerY = height / 2;
      nodePositions = {};

      const routerNode = nodes.find(n => n.node_type === 'router') || { id: 'node_router', label: 'Router' };
      const internetNode = nodes.find(n => n.node_type === 'internet');
      const ifaceNodes = nodes.filter(n => n.node_type === 'interface');
      const apNodes = nodes.filter(n => n.node_type === 'ap');
      const deviceNodes = nodes.filter(n => n.node_type === 'device');

      // 1. Center Root Router
      nodePositions[routerNode.id] = {
        x: centerX,
        y: centerY,
        r: 34,
        color: '#2563eb',
        bg: '#eff6ff',
        textColor: '#1e40af',
        icon: '🦀',
        data: routerNode
      };

      // 2. Identify WAN interface (e.g. ether1 or port with sub_type wan)
      const wanNode = ifaceNodes.find(i => i.sub_type === 'wan' || i.id === 'if_ether1' || i.label.toLowerCase().includes('wan'));
      const wanAngle = -Math.PI * 0.55; // Placed at top-left (~100 degrees)

      // 3. Position Interfaces radially around Router Core
      const ifaceRadius = Math.max(185, Math.min(width, height) * 0.24);
      const otherIfaces = ifaceNodes.filter(i => !wanNode || i.id !== wanNode.id);
      const totalOther = Math.max(1, otherIfaces.length);

      if (wanNode) {
        nodePositions[wanNode.id] = {
          x: centerX + ifaceRadius * Math.cos(wanAngle),
          y: centerY + ifaceRadius * Math.sin(wanAngle),
          r: 23,
          angle: wanAngle,
          color: '#0284c7',
          bg: '#e0f2fe',
          textColor: '#0369a1',
          icon: '🔌',
          data: wanNode
        };
      }

      otherIfaces.forEach((ifNode, i) => {
        // Distribute other interfaces around the remaining angular circle
        const angle = -Math.PI * 0.25 + (i / totalOther) * (Math.PI * 1.5);
        const ifX = centerX + ifaceRadius * Math.cos(angle);
        const ifY = centerY + ifaceRadius * Math.sin(angle);

        nodePositions[ifNode.id] = {
          x: ifX,
          y: ifY,
          r: 22,
          angle: angle,
          color: '#0284c7',
          bg: '#f0f9ff',
          textColor: '#0369a1',
          icon: ifNode.sub_type.includes('wlan') ? '📶' : (ifNode.sub_type.includes('bridge') ? '🌉' : '🔌'),
          data: ifNode
        };
      });

      // 4. Position Upstream Internet Node (ISP Cloud feeding into WAN interface)
      if (internetNode) {
        const wanPos = (wanNode && nodePositions[wanNode.id]) || { x: centerX - 180, y: centerY - 150, angle: wanAngle };
        const inetDist = 135;
        const inetAngle = wanPos.angle !== undefined ? wanPos.angle : wanAngle;
        const inetX = wanPos.x + inetDist * Math.cos(inetAngle);
        const inetY = wanPos.y + inetDist * Math.sin(inetAngle);

        nodePositions[internetNode.id] = {
          x: inetX,
          y: inetY,
          r: 28,
          angle: inetAngle,
          color: '#0284c7',
          bg: '#e0f2fe',
          textColor: '#0369a1',
          icon: '🌐',
          data: internetNode
        };
      }

      // 5. Position Access Points outward from their parent interface
      apNodes.forEach(apNode => {
        const parentId = apNode.parent_id || 'node_router';
        const parentPos = nodePositions[parentId] || { x: centerX, y: centerY, angle: 0 };
        const baseAngle = parentPos.angle !== undefined
          ? parentPos.angle
          : Math.atan2(parentPos.y - centerY, parentPos.x - centerX);
        const apDist = 105;
        const apX = parentPos.x + apDist * Math.cos(baseAngle);
        const apY = parentPos.y + apDist * Math.sin(baseAngle);

        nodePositions[apNode.id] = {
          x: apX,
          y: apY,
          r: 21,
          angle: baseAngle,
          color: '#d97706',
          bg: '#fffbeb',
          textColor: '#b45309',
          icon: '📡',
          data: apNode
        };
      });

      // 6. Group Devices by Parent Node (Interface or AP)
      const childrenByParent = {};
      deviceNodes.forEach(dev => {
        const parentId = dev.parent_id || 'node_router';
        if (!childrenByParent[parentId]) childrenByParent[parentId] = [];
        childrenByParent[parentId].push(dev);
      });

      function assignChildNode(child, cx, cy) {
        let color = '#059669';
        let bg = '#ecfdf5';
        let textColor = '#047857';
        let icon = '📱';

        if (child.sub_type === 'hotspot') {
          color = '#7c3aed'; bg = '#f5f3ff'; textColor = '#6d28d9'; icon = '🎟️';
        } else if (child.sub_type === 'pppoe') {
          color = '#2563eb'; bg = '#eff6ff'; textColor = '#1d4ed8'; icon = '🌐';
        } else if (child.sub_type === 'wifi') {
          color = '#d97706'; bg = '#fffbeb'; textColor = '#b45309'; icon = '📶';
        }

        nodePositions[child.id] = {
          x: cx,
          y: cy,
          r: 16,
          color,
          bg,
          textColor,
          icon,
          data: child
        };
      }

      // 7. Multi-Tier Concentric Orbital Placement for Child Devices
      Object.keys(childrenByParent).forEach(parentId => {
        const children = childrenByParent[parentId];
        const parentPos = nodePositions[parentId] || { x: centerX, y: centerY, angle: 0 };
        const outwardAngle = (parentPos.x !== centerX || parentPos.y !== centerY)
          ? Math.atan2(parentPos.y - centerY, parentPos.x - centerX)
          : (parentPos.angle || 0);

        if (children.length <= 5) {
          // Single Ring: up to 5 children with comfortable arc
          const arcSpread = Math.min(Math.PI * 0.75, Math.max(0.4, children.length * 0.35));
          const branchDist = 115;
          children.forEach((child, idx) => {
            const offset = children.length > 1 ? (idx / (children.length - 1) - 0.5) * arcSpread : 0;
            const childAngle = outwardAngle + offset;
            const childX = parentPos.x + branchDist * Math.cos(childAngle);
            const childY = parentPos.y + branchDist * Math.sin(childAngle);
            assignChildNode(child, childX, childY);
          });
        } else {
          // Multi-Tier Concentric Orbital Rings (e.g. 20+ Hotspot hosts on ether3)
          const ringCapacities = [6, 8, 10, 12];
          let assigned = 0;
          let ringIndex = 0;

          while (assigned < children.length) {
            const cap = ringCapacities[ringIndex] || 12;
            const slice = children.slice(assigned, assigned + cap);
            const dist = 115 + (ringIndex * 78);
            const arcSpread = Math.min(Math.PI * 1.15, 0.45 + (slice.length * 0.22));
            const step = slice.length > 1 ? arcSpread / (slice.length - 1) : 0;
            const stagger = (ringIndex % 2 === 1 && slice.length > 1) ? (step * 0.5) : 0;
            const startAngle = outwardAngle - (arcSpread / 2) + stagger;

            slice.forEach((child, idx) => {
              const childAngle = slice.length === 1 ? outwardAngle : startAngle + (idx * step);
              const childX = parentPos.x + dist * Math.cos(childAngle);
              const childY = parentPos.y + dist * Math.sin(childAngle);
              assignChildNode(child, childX, childY);
            });

            assigned += slice.length;
            ringIndex++;
          }
        }
      });

      // 8. Collision Relaxation Pass (40 iterations) to guarantee ZERO overlapping nodes
      const allIds = Object.keys(nodePositions);
      for (let iter = 0; iter < 40; iter++) {
        for (let i = 0; i < allIds.length; i++) {
          for (let j = i + 1; j < allIds.length; j++) {
            const a = nodePositions[allIds[i]];
            const b = nodePositions[allIds[j]];
            const dx = b.x - a.x;
            const dy = b.y - a.y;
            const dist = Math.hypot(dx, dy) || 0.001;
            const minClearance = a.r + b.r + 38; // radius sum + 38px clearance for labels
            if (dist < minClearance) {
              const push = (minClearance - dist) * 0.5;
              const nx = (dx / dist) * push;
              const ny = (dy / dist) * push;
              if (a.data.node_type !== 'router') {
                a.x -= nx;
                a.y -= ny;
              }
              if (b.data.node_type !== 'router') {
                b.x += nx;
                b.y += ny;
              }
            }
          }
        }
      }

      drawSvgGraph();
    }

    function drawSvgGraph() {
      const edgesLayer = document.getElementById('edges-layer');
      const nodesLayer = document.getElementById('nodes-layer');
      edgesLayer.innerHTML = '';
      nodesLayer.innerHTML = '';

      // Draw Edges
      graphData.edges.forEach(edge => {
        const s = nodePositions[edge.source];
        const t = nodePositions[edge.target];
        if (!s || !t) return;

        const isWan = edge.edge_type === 'wan';
        const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
        line.setAttribute('x1', s.x);
        line.setAttribute('y1', s.y);
        line.setAttribute('x2', t.x);
        line.setAttribute('y2', t.y);
        line.setAttribute('class', isWan ? 'edge-line edge-wan' : 'edge-line');
        line.setAttribute('id', `edge-${edge.id}`);
        line.setAttribute('stroke', isWan ? '#0284c7' : (edge.edge_type === 'physical' ? '#64748b' : '#94a3b8'));
        line.setAttribute('stroke-width', isWan ? '3.5' : (edge.edge_type === 'physical' ? '2.5' : '1.8'));
        edgesLayer.appendChild(line);
      });

      // Draw Nodes
      Object.keys(nodePositions).forEach(nodeId => {
        const p = nodePositions[nodeId];
        const n = p.data;

        const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        g.setAttribute('class', 'node-group');
        g.setAttribute('id', `node-${nodeId}`);
        g.setAttribute('transform', `translate(${p.x}, ${p.y})`);

        // Node Circle
        const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
        circle.setAttribute('r', p.r);
        circle.setAttribute('fill', p.bg);
        circle.setAttribute('stroke', p.color);
        circle.setAttribute('stroke-width', '2.5');

        // Node Icon
        const iconText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        iconText.setAttribute('y', p.r * 0.35);
        iconText.setAttribute('text-anchor', 'middle');
        iconText.setAttribute('font-size', p.r * 0.9);
        iconText.textContent = p.icon;

        // Node Label with stroke halo for high contrast
        const labelText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        labelText.setAttribute('class', 'node-label');
        labelText.setAttribute('y', p.r + 14);
        labelText.textContent = n.label.length > 22 ? n.label.substring(0, 20) + '...' : n.label;

        // Subtext (IP or vendor)
        const subText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        subText.setAttribute('class', 'node-subtext');
        subText.setAttribute('y', p.r + 26);
        subText.textContent = n.ip || (n.vendor ? n.vendor : n.sub_type);

        g.appendChild(circle);
        g.appendChild(iconText);
        g.appendChild(labelText);
        g.appendChild(subText);

        // Click to Open Inspector
        g.addEventListener('click', (e) => {
          e.stopPropagation();
          openDrawer(n);
        });

        // Dragging Event Handlers
        g.addEventListener('mousedown', (e) => {
          e.stopPropagation();
          draggedNodeId = nodeId;
          dragStartX = e.clientX;
          dragStartY = e.clientY;
        });

        nodesLayer.appendChild(g);
      });

      updateViewportTransform();
    }

    /* Update All Edge Lines connected to a dragged node in real-time */
    function updateConnectedEdges(nodeId) {
      const pos = nodePositions[nodeId];
      if (!pos) return;

      const nodeElem = document.getElementById(`node-${nodeId}`);
      if (nodeElem) {
        nodeElem.setAttribute('transform', `translate(${pos.x}, ${pos.y})`);
      }

      graphData.edges.forEach(edge => {
        if (edge.source === nodeId || edge.target === nodeId) {
          const line = document.getElementById(`edge-${edge.id}`);
          if (line) {
            const s = nodePositions[edge.source];
            const t = nodePositions[edge.target];
            if (s && t) {
              line.setAttribute('x1', s.x);
              line.setAttribute('y1', s.y);
              line.setAttribute('x2', t.x);
              line.setAttribute('y2', t.y);
            }
          }
        }
      });
    }

    /* Pan & Zoom Transformations */
    function updateViewportTransform() {
      const g = document.getElementById('viewport-group');
      g.setAttribute('transform', `translate(${panX}, ${panY}) scale(${scale})`);
    }

    function zoomIn() { scale = Math.min(3.0, scale * 1.2); updateViewportTransform(); }
    function zoomOut() { scale = Math.max(0.35, scale / 1.2); updateViewportTransform(); }
    function resetZoom() { scale = 1.0; panX = 0; panY = 0; updateViewportTransform(); }

    /* Canvas Mouse Listeners for Pan & Drag */
    const container = document.getElementById('canvas-container');

    container.addEventListener('wheel', (e) => {
      e.preventDefault();
      const zoomFactor = e.deltaY < 0 ? 1.1 : 0.9;
      scale = Math.min(3.0, Math.max(0.35, scale * zoomFactor));
      updateViewportTransform();
    }, { passive: false });

    container.addEventListener('mousedown', (e) => {
      if (draggedNodeId) return;
      isPanning = true;
      startPanX = e.clientX - panX;
      startPanY = e.clientY - panY;
    });

    window.addEventListener('mousemove', (e) => {
      if (draggedNodeId) {
        // Dragging Node
        const dx = (e.clientX - dragStartX) / scale;
        const dy = (e.clientY - dragStartY) / scale;
        dragStartX = e.clientX;
        dragStartY = e.clientY;

        if (nodePositions[draggedNodeId]) {
          nodePositions[draggedNodeId].x += dx;
          nodePositions[draggedNodeId].y += dy;
          updateConnectedEdges(draggedNodeId);
        }
      } else if (isPanning) {
        // Panning Canvas
        panX = e.clientX - startPanX;
        panY = e.clientY - startPanY;
        updateViewportTransform();
      }
    });

    window.addEventListener('mouseup', () => {
      draggedNodeId = null;
      isPanning = false;
    });

    function openDrawerById(nodeId) {
      const node = graphData.nodes.find(n => n.id === nodeId);
      if (node) openDrawer(node);
    }

    /* Node Inspector Drawer with Deduplicated Fields & Scrollable Hierarchy */
    function openDrawer(node) {
      selectedNode = node;
      document.getElementById('d-title').textContent = node.label;
      document.getElementById('d-id').textContent = node.id;
      document.getElementById('d-type').textContent = `${node.node_type} (${node.sub_type})`;
      document.getElementById('d-ip').textContent = node.ip || 'N/A';
      document.getElementById('d-mac').textContent = node.mac || 'N/A';
      document.getElementById('d-vendor').textContent = node.vendor || 'Generic Device';

      let parentLabel = 'Core Router';
      if (node.node_type === 'internet') {
        parentLabel = 'ISP Global Cloud (Upstream)';
      } else if (node.parent_id) {
        parentLabel = node.parent_id.replace('if_', 'Port ').replace('node_', '');
      }
      document.getElementById('d-parent').textContent = parentLabel;
      document.getElementById('d-status').textContent = node.status.toUpperCase();

      // Highlight edges connecting to this node
      document.querySelectorAll('.edge-line').forEach(el => {
        el.style.stroke = '#cbd5e1';
        el.style.strokeWidth = '1.5';
      });
      graphData.edges.forEach(edge => {
        if (edge.source === node.id || edge.target === node.id) {
          const line = document.getElementById(`edge-${edge.id}`);
          if (line) {
            line.style.stroke = edge.edge_type === 'wan' ? '#0284c7' : '#4f46e5';
            line.style.strokeWidth = '3.5';
          }
        }
      });

      // Filter out duplicate fields from extra-card that are already shown above
      const skipKeys = new Set(['id', 'ip', 'mac', 'vendor', 'parent', 'status', 'type', 'web_url', 'device_type', 'port', 'interface', 'identity']);
      const extraDiv = document.getElementById('extra-rows');
      extraDiv.innerHTML = '';
      let hasExtra = false;

      if (node.extra) {
        Object.keys(node.extra).forEach(k => {
          if (!skipKeys.has(k.toLowerCase())) {
            hasExtra = true;
            const row = document.createElement('div');
            row.className = 'detail-row';
            const cleanLabel = k.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
            row.innerHTML = `<span class="detail-label">${cleanLabel}:</span><span class="detail-val">${node.extra[k]}</span>`;
            extraDiv.appendChild(row);
          }
        });
      }

      const extraCard = document.getElementById('extra-card');
      extraCard.style.display = hasExtra ? 'block' : 'none';

      // AP Web Management button
      let webBtnContainer = document.getElementById('ap-web-btn-container');
      if (!webBtnContainer) {
        webBtnContainer = document.createElement('div');
        webBtnContainer.id = 'ap-web-btn-container';
        extraCard.parentNode.insertBefore(webBtnContainer, extraCard.nextSibling);
      }
      webBtnContainer.innerHTML = '';
      if (node.extra && node.extra.web_url) {
        webBtnContainer.innerHTML = `
          <div style="margin-bottom: 14px;">
            <a href="${node.extra.web_url}" target="_blank" style="display:flex; align-items:center; justify-content:center; gap:6px; padding:10px; background:#ecfdf5; border:1px solid #a7f3d0; color:#065f46; font-weight:700; font-size:0.82rem; border-radius:8px; text-decoration:none; box-shadow:0 1px 3px rgba(0,0,0,0.04);">
              <span>🌐</span>
              <span>Buka Web GUI AP (${node.extra.web_url})</span>
            </a>
          </div>`;
      }

      // Scrollable Connected Children list for interface or AP node
      let childSection = document.getElementById('child-devices-section');
      if (!childSection) {
        childSection = document.createElement('div');
        childSection.id = 'child-devices-section';
        const pingBox = document.querySelector('.ping-box');
        pingBox.parentNode.insertBefore(childSection, pingBox);
      }
      childSection.innerHTML = '';

      const children = graphData.nodes.filter(n => n.parent_id === node.id);
      if (children.length > 0) {
        childSection.innerHTML = `
          <div style="margin-bottom: 14px;">
            <div style="font-size: 0.8rem; font-weight: 800; color: #334155; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
              <span>Perangkat Terkoneksi Pada Port Ini:</span>
              <span class="badge" style="background:#f1f5f9; color:#475569; border:1px solid #cbd5e1;">${children.length} Device</span>
            </div>
            <div style="max-height: 220px; overflow-y: auto; border: 1px solid var(--card-border); border-radius: 8px; padding: 6px; background: #f8fafc; display: flex; flex-direction: column; gap: 6px;" class="scrollable-device-list">
              ${children.map(c => `
                <div style="padding: 8px 10px; background: #ffffff; border: 1px solid #e2e8f0; border-radius: 6px; font-size: 0.75rem; cursor: pointer; display: flex; justify-content: space-between; align-items: center; transition: all 0.12s;" onmouseover="this.style.borderColor='#93c5fd';this.style.background='#eff6ff';" onmouseout="this.style.borderColor='#e2e8f0';this.style.background='#ffffff';" onclick="event.stopPropagation(); openDrawerById('${c.id}')">
                  <div>
                    <div style="font-weight: 700; color: #1e293b;">${c.label}</div>
                    <div style="color: #64748b; font-size: 0.7rem; font-family: 'JetBrains Mono', monospace;">${c.ip || 'No IP'} | ${c.mac || 'No MAC'}</div>
                  </div>
                  <span style="font-size: 0.72rem; color: #4f46e5; font-weight: 700;">Lihat →</span>
                </div>
              `).join('')}
            </div>
          </div>`;
      }

      document.getElementById('ping-result').textContent = "Klik tombol di atas untuk uji latency";
      document.getElementById('drawer').style.display = 'flex';
    }

    function closeDrawer() {
      document.getElementById('drawer').style.display = 'none';
      selectedNode = null;
      document.querySelectorAll('.edge-line').forEach(el => {
        el.style.stroke = '';
        el.style.strokeWidth = '';
      });
    }

    async function pingCurrentNode() {
      if (!selectedNode || !selectedNode.ip) {
        alert("Node ini tidak memiliki alamat IP untuk di-ping!");
        return;
      }
      const pingResult = document.getElementById('ping-result');
      pingResult.textContent = `Mengirim 3 ICMP ping ke ${selectedNode.ip}...`;

      const host = document.getElementById('target-host').value;
      const port = document.getElementById('target-port').value;
      const user = document.getElementById('target-user').value;
      const pass = document.getElementById('target-pass').value;
      const token = localStorage.getItem('ros_token') || 'change-me-to-a-long-random-string';

      try {
        const res = await fetch('/api/v1/tools/ping', {
          method: 'POST',
          headers: {
            'Authorization': 'Bearer ' + token,
            'X-Router-Host': host,
            'X-Router-Port': port,
            'X-Router-User': user,
            'X-Router-Pass': pass,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ address: selectedNode.ip, count: 3 })
        });
        const json = await res.json();
        if (json.success) {
          pingResult.innerHTML = `<strong style="color: #059669;">RTT Min/Avg/Max:</strong> ${json.min_rtt || '-'} / <strong>${json.avg_rtt || '-'}</strong> / ${json.max_rtt || '-'}<br>Packet Loss: ${json.packet_loss || 0}%`;
        } else {
          pingResult.textContent = "Ping gagal: " + (json.error || "Host unreachable");
        }
      } catch (e) {
        pingResult.textContent = "Error ping: " + e.message;
      }
    }

    async function blockCurrentNode() {
      if (!selectedNode || !selectedNode.ip) {
        alert("Node ini tidak memiliki IP untuk diblokir!");
        return;
      }
      if (!confirm(`Yakin ingin memblokir IP ${selectedNode.ip} di firewall MikroTik?`)) return;

      const host = document.getElementById('target-host').value;
      const port = document.getElementById('target-port').value;
      const user = document.getElementById('target-user').value;
      const pass = document.getElementById('target-pass').value;
      const token = localStorage.getItem('ros_token') || 'change-me-to-a-long-random-string';

      try {
        const res = await fetch('/api/v1/firewall/block-ip', {
          method: 'POST',
          headers: {
            'Authorization': 'Bearer ' + token,
            'X-Router-Host': host,
            'X-Router-Port': port,
            'X-Router-User': user,
            'X-Router-Pass': pass,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ address: selectedNode.ip, list: "Blacklist" })
        });
        const json = await res.json();
        if (json.success) {
          alert(`IP ${selectedNode.ip} berhasil diblokir dan dimasukkan ke Blacklist!`);
        } else {
          alert("Gagal memblokir: " + json.error);
        }
      } catch (e) {
        alert("Error: " + e.message);
      }
    }

    window.addEventListener('DOMContentLoaded', () => {
      initCreds();
      loadTopology();
    });
  </script>
</body>
</html>"#)
}

/// GET /sdk/mikrotik-widget.js - Embeddable Web Component SDK
pub async fn sdk_script() -> impl IntoResponse {
    let js = r#"/**
 * MikroTik Universal Rust Gateway - Embeddable Web Component SDK v0.2.0
 * Mounts interactive topology and real-time network widgets seamlessly into any web app.
 */
class MikroTikWidget extends HTMLElement {
  connectedCallback() {
    const host = this.getAttribute('gateway') || 'https://ros-gateway.samrifa.com';
    const routerHost = this.getAttribute('router-host') || '';
    const iframe = document.createElement('iframe');
    iframe.src = `${host}/topology?host=${encodeURIComponent(routerHost)}`;
    iframe.style.width = '100%';
    iframe.style.height = this.getAttribute('height') || '500px';
    iframe.style.border = 'none';
    iframe.style.borderRadius = '12px';
    this.appendChild(iframe);
  }
}
customElements.define('mikrotik-topology', MikroTikWidget);
"#;
    Response::builder()
        .header(header::CONTENT_TYPE, "application/javascript")
        .body(js.to_string())
        .unwrap()
}

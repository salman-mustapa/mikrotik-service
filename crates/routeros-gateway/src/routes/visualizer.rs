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

    /* Canvas Area */
    #workspace {
      flex: 1;
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
    @keyframes dashflow {
      to { stroke-dashoffset: -1000; }
    }

    /* Node Styling */
    .node-group {
      cursor: pointer;
      transition: transform 0.1s;
    }
    .node-group:hover circle {
      filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.15));
    }
    .node-label {
      font-size: 11px;
      font-weight: 700;
      fill: #1e293b;
      text-anchor: middle;
      pointer-events: none;
    }
    .node-subtext {
      font-size: 9px;
      font-weight: 600;
      fill: #64748b;
      text-anchor: middle;
      pointer-events: none;
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

    /* Node Inspector Drawer */
    #drawer {
      width: 360px;
      background: var(--card-bg);
      border-left: 1px solid var(--card-border);
      box-shadow: -4px 0 20px rgba(0, 0, 0, 0.06);
      display: none;
      flex-direction: column;
      padding: 20px;
      z-index: 15;
      overflow-y: auto;
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
      <input type="text" id="target-host" placeholder="Host IP / Domain" style="width: 130px;" value="ath.vpnbersama.us">
      <input type="number" id="target-port" placeholder="Port" style="width: 60px;" value="51121">
      <input type="text" id="target-user" placeholder="User" style="width: 75px;" value="salman">
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

      <div style="margin-top: auto;">
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

    /* Hierarchical Branch Layout: Router at Center -> Interfaces radiate out -> Devices branch from their port */
    function buildHierarchicalLayout(nodes, edges) {
      const svg = document.getElementById('graph');
      const width = svg.clientWidth || window.innerWidth;
      const height = svg.clientHeight || (window.innerHeight - 60);

      const centerX = width / 2;
      const centerY = height / 2;
      nodePositions = {};

      const routerNode = nodes.find(n => n.node_type === 'router') || { id: 'node_router', label: 'Router' };
      const ifaceNodes = nodes.filter(n => n.node_type === 'interface');
      const apNodes = nodes.filter(n => n.node_type === 'ap');
      const deviceNodes = nodes.filter(n => n.node_type === 'device');

      // 1. Center Router
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

      // 2. Interfaces radiate out around center
      const ifaceRadius = Math.min(width, height) * 0.22;
      const totalIfaces = Math.max(1, ifaceNodes.length);

      ifaceNodes.forEach((ifNode, i) => {
        const angle = (i / totalIfaces) * 2 * Math.PI - (Math.PI / 2);
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

      // 3. AP nodes (e.g. AP on ether5) radiate slightly further out from interface
      apNodes.forEach(apNode => {
        const parentId = apNode.parent_id || 'node_router';
        const parentPos = nodePositions[parentId] || { x: centerX, y: centerY, angle: 0 };
        const angle = parentPos.angle || 0;
        const apX = parentPos.x + 80 * Math.cos(angle);
        const apY = parentPos.y + 80 * Math.sin(angle);

        nodePositions[apNode.id] = {
          x: apX,
          y: apY,
          r: 20,
          angle: angle,
          color: '#d97706',
          bg: '#fffbeb',
          textColor: '#b45309',
          icon: '📡',
          data: apNode
        };
      });

      // 4. Group Devices by Parent Node (so children branch from their respective port/AP!)
      const childrenByParent = {};
      deviceNodes.forEach(dev => {
        const parentId = dev.parent_id || 'node_router';
        if (!childrenByParent[parentId]) childrenByParent[parentId] = [];
        childrenByParent[parentId].push(dev);
      });

      // Position children outward in arc around their parent node
      Object.keys(childrenByParent).forEach(parentId => {
        const children = childrenByParent[parentId];
        const parentPos = nodePositions[parentId] || { x: centerX, y: centerY, angle: 0 };
        const baseAngle = parentPos.angle !== undefined ? parentPos.angle : 0;
        const branchDist = 125;
        const arcSpread = Math.min(Math.PI * 0.9, children.length * 0.35);

        children.forEach((child, idx) => {
          const offsetAngle = children.length > 1
            ? (idx / (children.length - 1) - 0.5) * arcSpread
            : 0;
          const childAngle = baseAngle + offsetAngle;
          const childX = parentPos.x + branchDist * Math.cos(childAngle);
          const childY = parentPos.y + branchDist * Math.sin(childAngle);

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
            x: childX,
            y: childY,
            r: 16,
            color,
            bg,
            textColor,
            icon,
            data: child
          };
        });
      });

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

        const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
        line.setAttribute('x1', s.x);
        line.setAttribute('y1', s.y);
        line.setAttribute('x2', t.x);
        line.setAttribute('y2', t.y);
        line.setAttribute('class', 'edge-line');
        line.setAttribute('id', `edge-${edge.id}`);
        line.setAttribute('stroke', edge.edge_type === 'physical' ? '#64748b' : '#94a3b8');
        line.setAttribute('stroke-width', edge.edge_type === 'physical' ? '2.5' : '1.8');
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

        // Node Label
        const labelText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        labelText.setAttribute('class', 'node-label');
        labelText.setAttribute('y', p.r + 14);
        labelText.textContent = n.label.length > 20 ? n.label.substring(0, 18) + '...' : n.label;

        // Subtext (IP or type)
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

    /* Node Inspector Drawer */
    function openDrawer(node) {
      selectedNode = node;
      document.getElementById('d-title').textContent = node.label;
      document.getElementById('d-id').textContent = node.id;
      document.getElementById('d-type').textContent = `${node.node_type} (${node.sub_type})`;
      document.getElementById('d-ip').textContent = node.ip || 'N/A';
      document.getElementById('d-mac').textContent = node.mac || 'N/A';
      document.getElementById('d-vendor').textContent = node.vendor || 'Generic';
      document.getElementById('d-parent').textContent = node.parent_id ? node.parent_id.replace('if_', '').replace('node_', '') : 'Core';
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
            line.style.stroke = '#4f46e5';
            line.style.strokeWidth = '3.5';
          }
        }
      });

      const extraDiv = document.getElementById('extra-rows');
      extraDiv.innerHTML = '';
      if (node.extra) {
        Object.keys(node.extra).forEach(k => {
          const row = document.createElement('div');
          row.className = 'detail-row';
          row.innerHTML = `<span class="detail-label">${k}:</span><span class="detail-val">${node.extra[k]}</span>`;
          extraDiv.appendChild(row);
        });
      }

      // If node has web management URL (e.g. Access Point), show direct clickable button
      if (node.extra && node.extra.web_url) {
        const webRow = document.createElement('div');
        webRow.style.marginTop = '10px';
        webRow.innerHTML = `<a href="${node.extra.web_url}" target="_blank" style="display:block; text-align:center; padding:8px; background:#f0fdf4; border:1px solid #bbf7d0; color:#166534; font-weight:700; font-size:0.8rem; border-radius:6px; text-decoration:none;">🔗 Buka Web Management AP (${node.extra.web_url})</a>`;
        extraDiv.appendChild(webRow);
      }

      // If interface or AP node, list child devices connected to this branch!
      const children = graphData.nodes.filter(n => n.parent_id === node.id);
      if (children.length > 0) {
        const childSection = document.createElement('div');
        childSection.style.marginTop = '14px';
        childSection.innerHTML = `<div style="font-size: 0.78rem; font-weight: 800; color: #475569; margin-bottom: 6px;">Perangkat Terkoneksi Pada Port Ini (${children.length}):</div>`;
        const list = document.createElement('div');
        list.style.display = 'flex';
        list.style.flexDirection = 'column';
        list.style.gap = '5px';
        children.forEach(c => {
          const item = document.createElement('div');
          item.style.padding = '6px 10px';
          item.style.background = '#f8fafc';
          item.style.border = '1px solid #e2e8f0';
          item.style.borderRadius = '6px';
          item.style.fontSize = '0.74rem';
          item.style.cursor = 'pointer';
          item.innerHTML = `<strong>${c.label}</strong> <br><span style="color:#64748b; font-size:0.7rem;">IP: ${c.ip || 'N/A'} | MAC: ${c.mac || 'N/A'}</span>`;
          item.onclick = (e) => { e.stopPropagation(); openDrawer(c); };
          list.appendChild(item);
        });
        childSection.appendChild(list);
        extraDiv.appendChild(childSection);
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

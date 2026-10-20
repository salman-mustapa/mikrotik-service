/* Logic of the stand-alone /map page. Kept in its own file so the page CSP can forbid inline scripts. */
(() => {
  'use strict';
  const $ = (id) => document.getElementById(id);
  const stage = $('stage');
  const KEYS = ['layout', 'interval', 'router-id', 'host', 'port', 'user', 'ticket'];   // non-secret: localStorage
  const SECRET = ['pass'];                                                              // secret: sessionStorage only
  const field = (k) => $('f-' + k);

  let activeTab = 'direct'; // 'direct' | 'registered'

  function setupTabs() {
    const tabDirect = $('tab-direct');
    const tabReg = $('tab-registered');
    const panelDirect = $('panel-direct');
    const panelReg = $('panel-registered');
    if (!tabDirect || !tabReg) return;

    tabDirect.addEventListener('click', () => {
      activeTab = 'direct';
      tabDirect.classList.add('active');
      tabReg.classList.remove('active');
      panelDirect.style.display = 'block';
      panelReg.style.display = 'none';
    });

    tabReg.addEventListener('click', () => {
      activeTab = 'registered';
      tabReg.classList.add('active');
      tabDirect.classList.remove('active');
      panelDirect.style.display = 'none';
      panelReg.style.display = 'block';
    });
  }

  function load() {
    for (const k of KEYS) {
      const v = localStorage.getItem('mm.' + k);
      if (v != null && field(k)) field(k).value = v;
    }
    for (const k of SECRET) {
      const v = sessionStorage.getItem('mm.' + k);
      if (v != null && field(k)) field(k).value = v;
    }
    // Default host and user if completely blank
    if (field('host') && !field('host').value) field('host').value = '192.168.88.1';
    if (field('user') && !field('user').value) field('user').value = 'admin';
    if (field('port') && !field('port').value) field('port').value = '8728';
  }

  function save() {
    for (const k of KEYS) {
      if (field(k)) localStorage.setItem('mm.' + k, field(k).value);
    }
    for (const k of SECRET) {
      if (field(k)) sessionStorage.setItem('mm.' + k, field(k).value);
    }
  }

  // #ticket=...&layout=home&router_id=main&mode=edit&demo=1
  function readHash() {
    if (!location.hash.length) return null;
    const p = new URLSearchParams(location.hash.slice(1));
    history.replaceState(null, '', location.pathname + location.search);
    const m = { ticket: 'ticket', layout: 'layout', router_id: 'router-id', host: 'host', port: 'port', user: 'user', pass: 'pass', interval: 'interval' };
    for (const [h, k] of Object.entries(m)) {
      if (p.has(h) && field(k)) field(k).value = p.get(h);
    }
    return { mode: p.get('mode'), demo: p.get('demo') === '1' };
  }

  function mount(demo, mode) {
    stage.replaceChildren();
    const m = document.createElement('mikrotik-map');
    m.setAttribute('base-url', location.origin);
    m.setAttribute('layout-id', (field('layout') && field('layout').value) || 'default');
    m.setAttribute('interval', (field('interval') && field('interval').value) || '2000');
    m.setAttribute('theme', 'light');

    if (demo) {
      m.setAttribute('demo', '');
    } else {
      const ticket = (field('ticket') && field('ticket').value) || '';
      if (ticket) m.setAttribute('ticket', ticket);

      if (activeTab === 'direct' || (field('host') && field('host').value && field('user') && field('user').value)) {
        if (field('host') && field('host').value) m.setAttribute('router-host', field('host').value);
        if (field('user') && field('user').value) m.setAttribute('router-user', field('user').value);
        if (field('port') && field('port').value) m.setAttribute('router-port', field('port').value);
        if (field('pass') && field('pass').value) m.setAttribute('router-pass', field('pass').value);
      } else if (field('router-id') && field('router-id').value) {
        m.setAttribute('router-id', field('router-id').value);
      }
    }

    if (mode === 'edit') m.setAttribute('mode', 'edit');
    stage.append(m);
  }

  let currentEmbedTab = 'widget';

  function getSnippetContent(tab) {
    const id = (field('layout') && field('layout').value) || 'default';
    const host = (field('host') && field('host').value) || '192.168.88.1';
    const port = (field('port') && field('port').value) || '8728';
    const user = (field('user') && field('user').value) || 'admin';
    const pass = (field('pass') && field('pass').value) || 'password';
    const base = location.origin;
    const wsProto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${wsProto}//${location.host}/ws`;

    if (tab === 'widget') {
      return [
        '<!-- ================================================================= -->',
        '<!-- 1. INTEGRASI WEB COMPONENT (HTML, REACT, VUE, SVELTE, FLUTTER WEB) -->',
        '<!-- ================================================================= -->',
        '',
        '<!-- Opsi A: Menggunakan Ephemeral Connection Ticket (Disarankan untuk Produksi) -->',
        '<!-- Password router aman di server backend, frontend hanya menerima ticket RAM -->',
        `<script src="${base}/sdk/mikrotik-map.js"></script>`,
        '<mikrotik-map',
        `  base-url="${base}"`,
        '  ticket="conn_tiket_dari_backend..."',
        `  layout-id="${id}"`,
        '  theme="light"',
        '  interval="2000"',
        '  style="width:100%; height:640px; display:block;">',
        '</mikrotik-map>',
        '',
        '<!-- Opsi B: Testing Internal Langsung (Tanpa Backend Tiket) -->',
        '<mikrotik-map',
        `  base-url="${base}"`,
        `  router-host="${host}"`,
        port !== '8728' ? `  router-port="${port}"` : null,
        `  router-user="${user}"`,
        `  router-pass="${pass}"`,
        `  layout-id="${id}"`,
        '  theme="light"',
        '  style="width:100%; height:640px; display:block;">',
        '</mikrotik-map>',
        '',
        '<!-- Menangkap Event Interaksi & Peringatan di JavaScript/React -->',
        '<script>',
        '  const map = document.querySelector("mikrotik-map");',
        '  map.addEventListener("alert", (e) => {',
        '    console.warn("⚠️ Peringatan Jaringan MikroTik:", e.detail);',
        '  });',
        '  map.addEventListener("select", (e) => {',
        '    console.log("Perangkat dipilih:", e.detail);',
        '  });',
        '</script>'
      ].filter(Boolean).join('\n');
    }

    if (tab === 'rest') {
      return [
        '# =================================================================',
        '# 2. REST API: MENGAMBIL DATA SCENE TOPOLOGI (JSON RAW)',
        '# Gunakan endpoint ini jika Anda membangun UI/Canvas kustom sendiri',
        '# =================================================================',
        '',
        '# A. Menggunakan cURL dengan Kredensial Router di Header:',
        `curl -X GET "${base}/api/v1/map/scene?layout_id=${id}" \\`,
        `  -H "X-Router-Host: ${host}" \\`,
        `  -H "X-Router-Port: ${port}" \\`,
        `  -H "X-Router-User: ${user}" \\`,
        `  -H "X-Router-Pass: ${pass}"`,
        '',
        '# B. Menggunakan cURL dengan Tiket Sesi (Aman untuk Frontend):',
        `curl -X GET "${base}/api/v1/map/scene?layout_id=${id}&ticket=conn_abc123"`,
        '',
        '// C. Contoh Fetch di JavaScript / Node.js / React Native:',
        `const res = await fetch("${base}/api/v1/map/scene?layout_id=${id}", {`,
        '  headers: {',
        '    "X-Connection-Ticket": "conn_abc123",',
        '  }',
        '});',
        'const data = await res.json();',
        '',
        '// Struktur Response Data JSON:',
        '// {',
        '//   "nodes": [',
        '//     {',
        '//       "id": "dev_192_168_88_254",',
        '//       "label": "Galaxy-A50",',
        '//       "display_label": "HP Kasir Utama",',
        '//       "node_type": "device",',
        '//       "ip": "192.168.88.254",',
        '//       "mac": "AA:BB:CC:DD:EE:FF",',
        '//       "vendor": "Samsung Electronics",',
        '//       "status": "ok", // "ok" | "warning" | "critical" | "idle"',
        '//       "position": { "x": 0.45, "y": 0.32, "source": "saved" },',
        '//       "traffic": { "rx_bps": 1250000, "tx_bps": 85000, "rx_formatted": "1.25 MB/s" },',
        '//       "queue_limit": { "warn": false, "crit": false }',
        '//     }',
        '//   ],',
        '//   "edges": [',
        '//     {',
        '//       "id": "edge_wlan1_dev1",',
        '//       "source": "wlan1",',
        '//       "target": "dev_192_168_88_254",',
        '//       "traffic": { "bps": 1335000, "formatted": "1.34 MB/s" },',
        '//       "style": "smooth", // "straight" | "orthogonal" | "smooth"',
        '//       "waypoints": [{ "x": 0.3, "y": 0.35 }]',
        '//     }',
        '//   ],',
        '//   "alerts": [ /* Outage / overload warnings */ ]',
        '// }'
      ].join('\n');
    }

    if (tab === 'ws') {
      return [
        '// =================================================================',
        '// 3. WEBSOCKET REALTIME STREAM: LIVE TRAFIK & STATUS DETIK-DEMI-DETIK',
        '// Mengalirkan pembaruan scene setiap 1-2 detik tanpa polling berulang',
        '// =================================================================',
        '',
        `const ws = new WebSocket("${wsUrl}?ticket=conn_tiket_dari_backend");`,
        '',
        'ws.onopen = () => {',
        '  console.log("Terhubung ke MikroTik Gateway WebSocket");',
        '  // Langganan stream peta jaringan real-time',
        '  ws.send(JSON.stringify({',
        '    action: "subscribe_map",',
        `    layout_id: "${id}",`,
        '    interval_ms: 2000,',
        '    filter_type: "all"',
        '  }));',
        '};',
        '',
        'ws.onmessage = (event) => {',
        '  const msg = JSON.parse(event.data);',
        '  if (msg.event === "map_frame") {',
        '    const scene = msg.data;',
        '    console.log("Frame Peta Realtime:", scene.timestamp);',
        '    console.log("Total Nodes Aktif:", scene.nodes.length);',
        '    console.log("Total Throughput WAN:", scene.wan_rate);',
        '',
        '    // Hubungkan data scene ke engine render Anda (D3.js / Vis.js / Canvas)',
        '    updateMyCustomCanvas(scene);',
        '  }',
        '};',
        '',
        'ws.onerror = (err) => console.error("WS Error:", err);',
        'ws.onclose = () => console.log("Koneksi WS Terputus, coba reconnect...");'
      ].join('\n');
    }

    if (tab === 'custom') {
      return [
        '# =================================================================',
        '# 4. REST API: KUSTOMISASI POSISI, IKON, JALUR KABEL, & DENAH',
        '# Semua kustomisasi tersimpan permanen di gateway per-layout',
        '# =================================================================',
        '',
        '# A. Simpan Posisi / Ikon / Label Kustom Perangkat:',
        `curl -X PUT "${base}/api/v1/map/layouts/${id}/nodes/dev_192_168_88_254" \\`,
        '  -H "Content-Type: application/json" \\',
        '  -H "X-Connection-Ticket: conn_abc123" \\',
        '  -d \'{',
        '    "x": 0.4500,',
        '    "y": 0.3200,',
        '    "label": "Kamera CCTV Parkir",',
        '    "icon": "cctv",',
        '    "parent_override": "sw_lt1",',
        '    "locked": true',
        '  }\'',
        '',
        '# B. Kustomisasi Jalur Kabel (Waypoints / Belokan Jalan / Kapasitas):',
        `curl -X PUT "${base}/api/v1/map/layouts/${id}/paths/edge_sw1_cctv1" \\`,
        '  -H "Content-Type: application/json" \\',
        '  -H "X-Connection-Ticket: conn_abc123" \\',
        '  -d \'{',
        '    "style": "orthogonal",',
        '    "capacity_bps": 100000000,',
        '    "waypoints": [',
        '      { "x": 0.25, "y": 0.20 },',
        '      { "x": 0.45, "y": 0.20 }',
        '    ]',
        '  }\'',
        '',
        '# C. Upload Gambar Denah Kantor / Gedung Sendiri (PNG/JPG/WebP maks 4 MB):',
        `curl -X PUT "${base}/api/v1/map/layouts/${id}/background" \\`,
        '  -H "Content-Type: image/png" \\',
        '  -H "X-Connection-Ticket: conn_abc123" \\',
        '  --data-binary "@denah_kantor_lt1.png"',
        '',
        '# D. Jalankan Auto-Place Bersih Secara Otomatis:',
        `curl -X POST "${base}/api/v1/map/layouts/${id}/auto-place" \\`,
        '  -H "Content-Type: application/json" \\',
        '  -H "X-Connection-Ticket: conn_abc123" \\',
        '  -d \'{"overwrite": false}\''
      ].join('\n');
    }

    return '';
  }

  function setupEmbedTabs() {
    const tabs = [
      { id: 'tab-embed-widget', name: 'widget' },
      { id: 'tab-embed-rest', name: 'rest' },
      { id: 'tab-embed-ws', name: 'ws' },
      { id: 'tab-embed-custom', name: 'custom' },
    ];
    for (const t of tabs) {
      const btn = $(t.id);
      if (!btn) continue;
      btn.addEventListener('click', () => {
        currentEmbedTab = t.name;
        for (const o of tabs) {
          const ob = $(o.id);
          if (ob) ob.classList.toggle('active', o.id === t.id);
        }
        $('embed-code').textContent = getSnippetContent(currentEmbedTab);
      });
    }
  }

  const dlg = $('dlg-connect');
  setupTabs();
  setupEmbedTabs();

  $('btn-connect').addEventListener('click', () => dlg.showModal());
  $('btn-cancel').addEventListener('click', () => dlg.close());
  $('btn-demo').addEventListener('click', () => { dlg.close(); mount(true); });
  $('frm-connect').addEventListener('submit', (e) => {
    e.preventDefault();
    save();
    dlg.close();
    mount(false);
  });
  $('btn-embed').addEventListener('click', () => {
    $('embed-code').textContent = getSnippetContent(currentEmbedTab);
    $('dlg-embed').showModal();
  });
  $('btn-embed-close').addEventListener('click', () => $('dlg-embed').close());
  $('btn-copy').addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText($('embed-code').textContent);
      $('btn-copy').textContent = '✅ Berhasil Disalin!';
    } catch (_) {
      $('btn-copy').textContent = 'Silakan Salin Manual';
    }
    setTimeout(() => { $('btn-copy').textContent = '📋 Salin Kode / Contoh'; }, 1800);
  });

  load();
  const h = readHash();
  if (h && h.demo) {
    mount(true);
  } else if ((field('host') && field('host').value && field('user') && field('user').value) || (field('router-id') && field('router-id').value)) {
    // If we have host + user or router-id saved, auto-mount (demo=false)
    mount(false, h && h.mode);
  } else {
    dlg.showModal();
  }
})();


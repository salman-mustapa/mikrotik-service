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

  function embedSnippet() {
    const id = (field('layout') && field('layout').value) || 'default';
    const host = (field('host') && field('host').value) || '192.168.88.1';
    const port = (field('port') && field('port').value) || '8728';
    const user = (field('user') && field('user').value) || 'admin';
    const pass = (field('pass') && field('pass').value) || 'password';

    const lines = [
      '<!-- ===================================================================== -->',
      '<!-- OPSI 1: INTEGRASI AMAN DENGAN TIKET (DIREKOMENDASIKAN UNTUK PRODUKSI)  -->',
      '<!-- Backend Anda membuat tiket via POST /api/v1/ticket/issue lalu kirim ke frontend -->',
      '<!-- Password router TIDAK PERNAH terekspos di browser / source code client! -->',
      '<!-- ===================================================================== -->',
      `<script src="${location.origin}/sdk/mikrotik-map.js"></` + 'script>',
      '<mikrotik-map',
      `  base-url="${location.origin}"`,
      '  ticket="conn_xyz123abc..."',
      `  layout-id="${id}"`,
      '  theme="light"',
      '  style="height:640px">',
      '</mikrotik-map>',
      '',
      '<!-- ===================================================================== -->',
      '<!-- OPSI 2: TESTING LANGSUNG / INTERNAL DASHBOARD (TANPA BACKEND)          -->',
      '<!-- ===================================================================== -->',
      '<mikrotik-map',
      `  base-url="${location.origin}"`,
      `  router-host="${host}"`,
      port !== '8728' ? `  router-port="${port}"` : null,
      `  router-user="${user}"`,
      `  router-pass="${pass}"`,
      `  layout-id="${id}"`,
      '  theme="light"',
      '  style="height:640px">',
      '</mikrotik-map>',
      '',
      '<!-- PANDUAN INTEGRASI REST API & WEBSOCKET:',
      `  1. Issue Tiket Sesi (Backend):`,
      `     POST ${location.origin}/api/v1/ticket/issue`,
      `     Content-Type: application/json`,
      `     Body: {"host":"${host}", "port":${port}, "user":"${user}", "pass":"..."}`,
      `     Response: {"ticket": "conn_...", "expires_in": 3600}`,
      '',
      `  2. WebSocket Real-time Stream:`,
      `     URL: ws://${location.host}/ws?ticket=conn_...`,
      `     Kirim Pesan: {"action":"subscribe_map", "layout_id":"${id}"}`,
      '',
      `  3. Atribut Tambahan:`,
      `     - mode="edit" : Aktifkan mode penataan posisi alat, tiang/ODP, & belokan kabel`,
      `     - interval="2000" : Frekuensi refresh real-time (ms)`,
      '-->'
    ].filter(Boolean);
    return lines.join('\n');
  }

  const dlg = $('dlg-connect');
  setupTabs();

  $('btn-connect').addEventListener('click', () => dlg.showModal());
  $('btn-cancel').addEventListener('click', () => dlg.close());
  $('btn-demo').addEventListener('click', () => { dlg.close(); mount(true); });
  $('frm-connect').addEventListener('submit', (e) => {
    e.preventDefault();
    save();
    dlg.close();
    mount(false);
  });
  $('btn-embed').addEventListener('click', () => { $('embed-code').textContent = embedSnippet(); $('dlg-embed').showModal(); });
  $('btn-embed-close').addEventListener('click', () => $('dlg-embed').close());
  $('btn-copy').addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText($('embed-code').textContent);
      $('btn-copy').textContent = '✅ Tersalin!';
    } catch (_) {
      $('btn-copy').textContent = 'Salin Manual';
    }
    setTimeout(() => { $('btn-copy').textContent = 'Salin Kode'; }, 1800);
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


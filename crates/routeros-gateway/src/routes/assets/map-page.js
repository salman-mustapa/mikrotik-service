/* Logic of the stand-alone /map page. Kept in its own file so the page CSP can forbid inline scripts. */
(() => {
  'use strict';
  const $ = (id) => document.getElementById(id);
  const stage = $('stage');
  const KEYS = ['layout', 'interval', 'router-id', 'host', 'port', 'user'];   // non-secret: localStorage
  const SECRET = ['token', 'pass'];                                         // secret: sessionStorage only
  const field = (k) => $('f-' + k);

  function load() {
    for (const k of KEYS) { const v = localStorage.getItem('mm.' + k); if (v != null) field(k).value = v; }
    for (const k of SECRET) { const v = sessionStorage.getItem('mm.' + k); if (v != null) field(k).value = v; }
  }
  function save() {
    for (const k of KEYS) localStorage.setItem('mm.' + k, field(k).value);
    for (const k of SECRET) sessionStorage.setItem('mm.' + k, field(k).value);
  }

  // #token=...&layout=home&router_id=main&mode=edit&demo=1  (hash never reaches the server or logs)
  function readHash() {
    if (!location.hash.length) return null;
    const p = new URLSearchParams(location.hash.slice(1));
    history.replaceState(null, '', location.pathname + location.search);
    const m = { token: 'token', layout: 'layout', router_id: 'router-id', host: 'host', port: 'port', user: 'user', pass: 'pass', interval: 'interval' };
    for (const [h, k] of Object.entries(m)) if (p.has(h)) field(k).value = p.get(h);
    return { mode: p.get('mode'), demo: p.get('demo') === '1' };
  }

  function mount(demo, mode) {
    stage.replaceChildren();
    const m = document.createElement('mikrotik-map');
    m.setAttribute('base-url', location.origin);
    m.setAttribute('layout-id', field('layout').value || 'default');
    m.setAttribute('interval', field('interval').value || '2000');
    if (demo) m.setAttribute('demo', '');
    else {
      m.setAttribute('token', field('token').value);
      if (field('host').value && field('user').value) {
        m.setAttribute('router-host', field('host').value);
        m.setAttribute('router-user', field('user').value);
        if (field('port').value) m.setAttribute('router-port', field('port').value);
        if (field('pass').value) m.setAttribute('router-pass', field('pass').value);
      } else if (field('router-id').value) m.setAttribute('router-id', field('router-id').value);
    }
    if (mode === 'edit') m.setAttribute('mode', 'edit');
    stage.append(m);
  }

  function embedSnippet() {
    const id = field('layout').value || 'default';
    const rid = field('router-id').value;
    return [
      `<script src="${location.origin}/sdk/mikrotik-map.js"></` + 'script>',
      '<mikrotik-map',
      `  base-url="${location.origin}"`,
      '  token="YOUR_GATEWAY_TOKEN"',
      `  layout-id="${id}"`,
      rid ? `  router-id="${rid}"` : '  router-id="main"',
      '  style="height:640px"></mikrotik-map>',
      '',
      '<!-- add mode="edit" to let operators drag devices, change icons and draw road paths -->',
    ].join('\n');
  }

  const dlg = $('dlg-connect');
  $('btn-connect').addEventListener('click', () => dlg.showModal());
  $('btn-cancel').addEventListener('click', () => dlg.close());
  $('btn-demo').addEventListener('click', () => { dlg.close(); mount(true); });
  $('frm-connect').addEventListener('submit', (e) => {
    e.preventDefault();
    if (!field('token').value) return;
    save(); dlg.close(); mount(false);
  });
  $('btn-embed').addEventListener('click', () => { $('embed-code').textContent = embedSnippet(); $('dlg-embed').showModal(); });
  $('btn-embed-close').addEventListener('click', () => $('dlg-embed').close());
  $('btn-copy').addEventListener('click', async () => {
    try { await navigator.clipboard.writeText($('embed-code').textContent); $('btn-copy').textContent = 'Copied'; } catch (_) { $('btn-copy').textContent = 'Select & copy manually'; }
    setTimeout(() => { $('btn-copy').textContent = 'Copy'; }, 1800);
  });

  load();
  const h = readHash();
  if (h && h.demo) mount(true);
  else if (field('token').value) mount(false, h && h.mode);
  else dlg.showModal();
})();

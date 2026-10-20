/*!
 * <mikrotik-map> - ready-to-use live network map for the MikroTik Universal Gateway.
 *
 *   <script src="https://GATEWAY/sdk/mikrotik-map.js"></script>
 *   <!-- Mode 1: Tiket Sesi (Aman untuk frontend produksi) -->
 *   <mikrotik-map base-url="https://GATEWAY" ticket="conn_..." layout-id="home" style="height:640px"></mikrotik-map>
 *   <!-- Mode 2: Direct Router (Testing internal) -->
 *   <mikrotik-map base-url="https://GATEWAY" router-host="192.168.88.1" router-user="admin" router-pass="..." layout-id="home" style="height:640px"></mikrotik-map>
 *
 * No framework, no build step, no dependencies. All server data is rendered through
 * textContent / setAttribute (never innerHTML), so a hostile DHCP hostname cannot inject script.
 * Full reference: docs/07-network-map-visualizer.md
 */
(() => {
  'use strict';
  if (customElements.get('mikrotik-map')) return;

  const SCRIPT_ORIGIN = (() => {
    try { return new URL(document.currentScript.src).origin; } catch (_) { return location.origin; }
  })();
  const NS = 'http://www.w3.org/2000/svg';
  const W = 1000;                 // viewBox width; height follows the background aspect ratio
  // Default view = whole canvas plus a margin, so icons and labels of nodes placed near the edge stay visible.
  const homeVb = (h) => ({ x: -90, y: -60, w: W + 180, h: h + 120 });
  const TAG = 'map';
  const COLORS = { ok: '#16a34a', warning: '#d97706', critical: '#dc2626', idle: '#94a3b8', down: '#0284c7', up: '#7c3aed' };
  const NODE_R = { router: 32, internet: 30, ap: 28, odp: 26, tiang: 22, closure: 22, 'unmanaged-switch': 24, olt: 30, ont: 24, interface: 18, device: 22 };
  const MAX_DOTS = 8;
  let uid = 0;

  // 24x24 stroke icons. Ids match GET /api/v1/map/icons.
  const ICONS = {
    internet: 'M7 18a4 4 0 0 1-.5-7.97A5.5 5.5 0 0 1 17 8.5 4.5 4.5 0 0 1 17.5 18z',
    router: 'M4 14h16a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1z M7 16.5h.01 M11 16.5h.01 M8 14V9 M16 14V9 M12 14V6',
    switch: 'M3 8h18v8H3z M6.5 12h.01 M10 12h.01 M13.5 12h.01 M17 12h.01 M6 8V6 M18 8V6',
    port: 'M6 5h12v10h-3v3H9v-3H6z M9 8v3 M12 8v3 M15 8v3',
    'wan-port': 'M6 6h12v10h-3v3H9v-3H6z M9 9v3 M12 9v3 M15 9v3 M12 1.5v2.5 M10.5 3L12 1.5 13.5 3',
    ap: 'M2.5 9.5a14 14 0 0 1 19 0 M5.5 12.8a9.5 9.5 0 0 1 13 0 M8.6 16a5 5 0 0 1 6.8 0 M12 19.5h.01',
    repeater: 'M4 8a11 11 0 0 1 16 0 M7 11.2a6.8 6.8 0 0 1 10 0 M12 15h.01 M7 20h10 M14.5 17.5L17 20l-2.5 2.5',
    server: 'M4 4h16v6H4z M4 14h16v6H4z M7.5 7h.01 M7.5 17h.01 M11 7h6 M11 17h6',
    odp: 'M4 4h16v16H4z M4 9h16 M7 6.5h.01 M10 6.5h.01 M13 6.5h.01 M8 13a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z M16 13a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z M8 18a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z M16 18a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z',
    tiang: 'M12 2v20 M7 6h10 M8 10h8 M9 14h6',
    closure: 'M6 9h12a3 3 0 0 1 3 3v0a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3v0a3 3 0 0 1 3-3z M3 12h-1 M21 12h1 M8 9v6 M16 9v6',
    'unmanaged-switch': 'M3 7h18v10H3z M6 13h2v2H6z M10 13h2v2h-2z M14 13h2v2h-2z M18 13h2v2h-2z M6 9.5h.01 M10 9.5h.01 M14 9.5h.01 M18 9.5h.01',
    olt: 'M3 5h18v14H3z M6 9h12 M6 13h12 M6 16h4 M16 16h2',
    ont: 'M4 8h16v12H4z M17 4v4 M7 4v4 M8 14h.01 M12 14h.01 M16 14h.01',
    phone: 'M8 2h8a1.5 1.5 0 0 1 1.5 1.5v17A1.5 1.5 0 0 1 16 22H8a1.5 1.5 0 0 1-1.5-1.5v-17A1.5 1.5 0 0 1 8 2z M10.5 19h3',
    tablet: 'M5 3h14a1.5 1.5 0 0 1 1.5 1.5v15A1.5 1.5 0 0 1 19 21H5a1.5 1.5 0 0 1-1.5-1.5v-15A1.5 1.5 0 0 1 5 3z M11 18h2',
    laptop: 'M5 5h14v10H5z M2 19h20l-1.5-4h-17z',
    desktop: 'M3 4h18v12H3z M8 20h8 M12 16v4',
    tv: 'M3 7h18v11H3z M8 3l4 4 4-4',
    printer: 'M7 8V3h10v5 M7 17H4v-7h16v7h-3 M7 14h10v7H7z',
    camera: 'M3 8h13v7H3z M16 10.5l5-2.5v7l-5-2.5 M8 15v4 M5 19h6',
    iot: 'M7 7h10v10H7z M10 10h4v4h-4z M9 3v4 M15 3v4 M9 17v4 M15 17v4 M3 9h4 M3 15h4 M17 9h4 M17 15h4',
    'game-console': 'M6 8h12a4 4 0 0 1 4 4v1a3 3 0 0 1-5.2 2L15 13H9l-1.8 2A3 3 0 0 1 2 13v-1a4 4 0 0 1 4-4z M7 10.5v3 M5.5 12h3 M16 11.5h.01 M18 13h.01',
    'hotspot-user': 'M12 12a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7z M5 21a7 7 0 0 1 14 0 M17.5 4a4 4 0 0 1 3.5 0',
    'pppoe-user': 'M12 12a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7z M5 21a7 7 0 0 1 14 0 M19 2v3 M21.5 2v3 M18.5 5h4v1.5a2 2 0 0 1-4 0z',
    unknown: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z M9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.7.4-1 1-1 1.7 M12 17h.01',
  };

  const ATTR_KEYS = {
    'base-url': 'baseUrl', token: 'token', ticket: 'ticket', 'layout-id': 'layoutId', 'router-id': 'routerId', 'router-host': 'routerHost',
    'router-port': 'routerPort', 'router-user': 'routerUser', 'router-pass': 'routerPass', 'tenant-id': 'tenantId',
    mode: 'mode', interval: 'interval', transport: 'transport', filter: 'filter', theme: 'theme', demo: 'demo', editable: 'editable',
  };

  const el = (tag, props, kids) => {
    const e = document.createElement(tag);
    for (const [k, v] of Object.entries(props || {})) {
      if (v === false || v == null) continue;
      if (k === 'class') e.className = v;
      else if (k === 'text') e.textContent = v;
      else if (k.startsWith('on')) e.addEventListener(k.slice(2), v);
      else e.setAttribute(k, v === true ? '' : String(v));
    }
    for (const c of [].concat(kids || [])) if (c != null && c !== false) e.append(c);
    return e;
  };
  const sv = (tag, props, kids) => {
    const e = document.createElementNS(NS, tag);
    for (const [k, v] of Object.entries(props || {})) {
      if (v === false || v == null) continue;
      if (k === 'class') e.setAttribute('class', v);
      else if (k === 'text') e.textContent = v;
      else e.setAttribute(k, String(v));
    }
    for (const c of [].concat(kids || [])) if (c != null && c !== false) e.append(c);
    return e;
  };
  const r4 = (v) => Math.round(v * 10000) / 10000;
  const clamp01 = (v) => Math.min(1, Math.max(0, v));
  const col = (h) => COLORS[h] || COLORS.idle;
  const short = (s, n) => { s = String(s == null ? '' : s); return s.length > n ? s.slice(0, n - 1) + '\u2026' : s; };

  const CSS = `
:host{display:block;position:relative;height:640px;min-height:320px;color:#0f172a;font:13px/1.35 system-ui,-apple-system,"Segoe UI",Roboto,sans-serif;--bg:#f8fafc;--panel:rgba(255,255,255,.94);--line:rgba(203,213,225,.8);--fg:#0f172a;--mut:#64748b;--acc:#0284c7;--node:#ffffff;--node-s:#94a3b8;--shadow:0 4px 18px rgba(15,23,42,.08)}
:host([theme=dark]){--bg:#0b1220;--panel:rgba(15,23,42,.88);--line:rgba(148,163,184,.25);--fg:#e2e8f0;--mut:#94a3b8;--acc:#38bdf8;--node:#0f172a;--node-s:#334155;--shadow:0 4px 20px rgba(0,0,0,.4);color:var(--fg)}
.wrap{position:absolute;inset:0;background:var(--bg);overflow:hidden;border-radius:10px;border:1px solid var(--line);box-shadow:var(--shadow)}
svg.stage{position:absolute;inset:0;width:100%;height:100%;touch-action:none;cursor:grab;user-select:none}
svg.stage.pan{cursor:grabbing}
.bar,.alerts,.panel,.legend,.tip,.banner{position:absolute;background:var(--panel);backdrop-filter:blur(10px);border:1px solid var(--line);border-radius:10px;color:var(--fg);box-shadow:var(--shadow)}
.bar{top:10px;left:10px;right:10px;display:flex;gap:8px;align-items:center;flex-wrap:wrap;padding:7px 12px}
.bar .t{font-weight:700;margin-right:4px;font-size:14px;color:var(--fg)}
.pill{padding:3px 10px;border-radius:99px;font-size:11px;font-weight:700;background:#e2e8f0;color:#334155}
.pill.live{background:#dcfce7;color:#15803d}.pill.stale,.pill.connecting{background:#fef3c7;color:#b45309}.pill.error,.pill.offline{background:#fee2e2;color:#b91c1c}.pill.demo{background:#ede9fe;color:#6d28d9}
.chip{color:var(--mut);font-variant-numeric:tabular-nums;font-size:12px}
.chip b{color:var(--fg)}
.sp{flex:1}
button,select,input{font:inherit;color:var(--fg);background:#ffffff;border:1px solid var(--line);border-radius:7px;padding:5px 10px;outline:none;transition:all .15s ease}
button{cursor:pointer}button:hover{border-color:var(--acc);background:#f1f5f9}button.on{background:var(--acc);color:#ffffff;border-color:var(--acc);font-weight:700}
button.danger:hover{border-color:#ef4444;color:#ef4444}
.alerts{left:10px;bottom:10px;width:min(360px,60%);max-height:32%;overflow:auto;padding:8px 10px}
.alerts:empty{display:none}
.alerts h4,.panel h4{margin:0 0 6px;font-size:11px;letter-spacing:.06em;text-transform:uppercase;color:var(--mut);font-weight:700}
.al{display:flex;gap:7px;align-items:flex-start;padding:4px 6px;cursor:pointer;border-radius:6px;transition:background .12s}.al:hover{background:#f1f5f9}
.dot{flex:none;width:9px;height:9px;border-radius:50%;margin-top:4px}
.panel{top:58px;right:10px;width:280px;max-height:calc(100% - 130px);overflow:auto;padding:12px}
.panel[hidden],.tip[hidden],.banner[hidden]{display:none}
.panel label{display:block;margin:8px 0 3px;color:var(--mut);font-size:11px;font-weight:600}
.panel input[type=text],.panel input[type=number],.panel select{width:100%;box-sizing:border-box}
.panel .row{display:flex;gap:6px;margin-top:8px;flex-wrap:wrap}
.panel .hint{margin-top:8px;color:var(--mut);font-size:11px}
.legend{right:10px;bottom:10px;display:flex;gap:12px;padding:6px 12px;font-size:11px;color:var(--mut);flex-wrap:wrap;max-width:60%}
.legend i{display:inline-block;width:10px;height:10px;border-radius:50%;margin-right:4px;vertical-align:-1px}
.tip{pointer-events:none;max-width:300px;padding:8px 12px;z-index:5;font-size:12px}
.tip .h{font-weight:700;margin-bottom:3px}.tip .m{color:var(--mut)}
.banner{left:50%;transform:translateX(-50%);top:56px;padding:6px 16px;z-index:6;font-weight:600}
.banner.err{background:#fee2e2;border-color:#f87171;color:#b91c1c}.banner.ok{background:#dcfce7;border-color:#86efac;color:#15803d}
.node{cursor:pointer}.node.edit{cursor:move}.node.locked.edit{cursor:not-allowed}
.node text{pointer-events:none;fill:var(--fg);font-size:11px;font-weight:600;text-anchor:middle;paint-order:stroke;stroke:var(--bg);stroke-width:3.5px}
.node text.rate{fill:var(--mut);font-size:9.5px;font-weight:500;font-variant-numeric:tabular-nums;stroke:var(--bg);stroke-width:2.5px}
.node.sel .body{stroke:var(--acc);stroke-width:3}
.halo.pulse{animation:pulse 1.1s ease-in-out infinite}
@keyframes pulse{50%{stroke-opacity:.25}}
.edge .hit{fill:none;stroke:transparent;stroke-width:16;pointer-events:none}
.edge.edit .hit{pointer-events:stroke;cursor:pointer}
.edge .line{fill:none;stroke-linecap:round;stroke-linejoin:round;transition:stroke .4s}
.edge.sel .line{stroke-width:5}
.handle{fill:#fff;stroke:var(--acc);stroke-width:2.5;cursor:move}
.wrap.stale svg.stage, .wrap.offline svg.stage, .wrap.connecting svg.stage{filter:grayscale(.85) opacity(.7);transition:filter .3s ease}
.wrap.stale .halo, .wrap.offline .halo, .wrap.connecting .halo{animation:none !important}
@media (prefers-reduced-motion:reduce){.halo.pulse{animation:none}}
`;

  class MikrotikMap extends HTMLElement {
    static get observedAttributes() { return Object.keys(ATTR_KEYS); }

    constructor() {
      super();
      this._o = {};
      this._scene = null;
      this._edit = false;
      this._nodes = new Map();     // id -> record
      this._edges = new Map();
      this._pos = new Map();       // id -> {x,y} normalised
      this._vb = homeVb(W / 1.7778);
      this._zoomed = false;
      this._bgKey = null;
      this._bgAspect = null;
      this._bgBlob = null;
      this._sel = null;            // {kind:'node'|'edge', id}
      this._layout = null;
      this._alertIds = new Set();
      this._alertSig = '';
      this._status = 'connecting';
      this._lastFrame = 0;
      this._wsFails = 0;
      this._drag = null;
      this._pending = null;
      this._reduce = matchMedia('(prefers-reduced-motion: reduce)').matches;
      this._uid = ++uid;
      this._build();
    }

    // ------------------------------------------------------------ public API
    get config() { return { ...this._o }; }
    set config(o) {
      const c = { ...(o || {}) };
      if (c.router) {
        c.routerHost = c.router.host; c.routerPort = c.router.port; c.routerUser = c.router.user; c.routerPass = c.router.password;
        delete c.router;
      }
      this._o = c;
      if (this.isConnected) this._restart();
    }
    get scene() { return this._scene; }
    refresh() { return this._pollOnce(); }
    fit() { this._zoomed = false; this._fit(); }
    setMode(m) { this._setEdit(m === 'edit'); }
    selectNode(id) { if (this._nodes.has(id)) this._select({ kind: 'node', id }); }

    // ------------------------------------------------------------ lifecycle
    connectedCallback() {
      this._edit = this._get('mode') === 'edit' && this._get('editable') !== 'false';
      this._applyEditClasses();
      this._restart();
      if (this._edit) this._loadLayout().catch(() => {});
      this._raf = requestAnimationFrame((t) => this._frame(t));
      this._watch = setInterval(() => this._watchdog(), 1000);
      document.addEventListener('keydown', (this._kd = (e) => this._onKey(e)));
    }
    disconnectedCallback() {
      this._stop();
      cancelAnimationFrame(this._raf);
      clearInterval(this._watch);
      document.removeEventListener('keydown', this._kd);
      if (this._bgBlob) URL.revokeObjectURL(this._bgBlob);
    }
    attributeChangedCallback(name, a, b) {
      if (a === b || !this.isConnected) return;
      if (name === 'mode') { this._setEdit(b === 'edit'); return; }
      clearTimeout(this._rt);
      this._rt = setTimeout(() => this._restart(), 50);
    }

    // ------------------------------------------------------------ config helpers
    _get(name) {
      const attr = this.getAttribute(name);
      if (attr != null) return attr;
      const v = this._o[ATTR_KEYS[name]];
      return v == null ? null : String(v);
    }
    _cfg() {
      const g = (n, d) => this._get(n) ?? d;
      return {
        base: String(g('base-url', SCRIPT_ORIGIN)).replace(/\/+$/, ''),
        token: g('token', ''),
        ticket: g('ticket', ''),
        layout: g('layout-id', 'default'),
        routerId: g('router-id', ''), host: g('router-host', ''), port: g('router-port', ''),
        user: g('router-user', ''), pass: g('router-pass', ''),
        tenant: g('tenant-id', ''),
        interval: Math.min(60000, Math.max(1000, parseInt(g('interval', '2000'), 10) || 2000)),
        transport: g('transport', 'auto'), filter: g('filter', 'all'),
        demo: this.hasAttribute('demo') || this._o.demo === true || this._o.demo === 'true',
      };
    }
    _routerFields(c) {
      if (c.ticket) return { ticket: c.ticket };
      if (c.host && c.user) return { router_host: c.host, router_port: c.port ? +c.port : undefined, router_user: c.user, router_pass: c.pass || undefined };
      return c.routerId ? { router_id: c.routerId } : {};
    }
    async _api(method, path, body) {
      const c = this._cfg();
      const headers = {};
      if (c.token) headers.Authorization = 'Bearer ' + c.token;
      if (c.ticket) headers['X-Connection-Ticket'] = c.ticket;
      if (c.host && c.user) {
        headers['X-Router-Host'] = c.host; headers['X-Router-User'] = c.user;
        if (c.port) headers['X-Router-Port'] = c.port;
        if (c.pass) headers['X-Router-Pass'] = c.pass;
      } else if (c.routerId) headers['X-Router-Id'] = c.routerId;
      if (c.tenant) headers['X-Tenant-Id'] = c.tenant;
      let payload;
      if (body instanceof Blob) payload = body;
      else if (body !== undefined) { headers['Content-Type'] = 'application/json'; payload = JSON.stringify(body); }
      const res = await fetch(c.base + path, { method, headers, body: payload });
      if (path.endsWith('/background') && method === 'GET' && res.ok) return res;
      let data = null;
      try { data = await res.json(); } catch (_) { /* non-json */ }
      if (!res.ok) {
        const e = new Error((data && data.error) || 'HTTP ' + res.status);
        e.status = res.status; e.code = data && data.code;
        throw e;
      }
      return data;
    }
    _lp(sub) { return '/api/v1/map/layouts/' + encodeURIComponent(this._cfg().layout) + (sub || ''); }

    // ------------------------------------------------------------ DOM skeleton
    _build() {
      const root = this.attachShadow({ mode: 'open' });
      const pid = 'grid' + this._uid;
      this._svg = sv('svg', { class: 'stage', role: 'img', 'aria-label': 'Live network map', viewBox: `0 0 ${W} ${W / 1.7778}` }, [
        sv('defs', {}, sv('pattern', { id: pid, width: 40, height: 40, patternUnits: 'userSpaceOnUse' },
          sv('path', { d: 'M40 0H0V40', fill: 'none', stroke: 'rgba(203,213,225,.55)', 'stroke-width': 1 }))),
      ]);
      this._gridRect = sv('rect', { x: 0, y: 0, width: W, height: W / 1.7778, fill: `url(#${pid})` });
      this._bgImg = sv('image', { x: 0, y: 0, width: W, height: W / 1.7778, preserveAspectRatio: 'none' });
      this._gEdges = sv('g'); this._gHandles = sv('g'); this._gNodes = sv('g');
      this._svg.append(this._gridRect, this._bgImg, this._gEdges, this._gHandles, this._gNodes);

      this._title = el('span', { class: 't', text: 'Network map' });
      this._pill = el('span', { class: 'pill connecting', text: 'connecting' });
      this._wan = el('span', { class: 'chip' });
      this._cnt = el('span', { class: 'chip' });
      this._btnEdit = el('button', { text: 'Edit', title: 'Toggle edit mode', onclick: () => this._setEdit(!this._edit) });
      this._btnFit = el('button', { text: 'Fit', title: 'Reset zoom', onclick: () => this.fit() });
      this._tools = el('span', { class: 'tools' });
      this._bar = el('div', { class: 'bar' }, [this._title, this._pill, this._wan, this._cnt, el('span', { class: 'sp' }), this._tools, this._btnFit, this._btnEdit]);
      this._alerts = el('div', { class: 'alerts' });
      this._panel = el('aside', { class: 'panel', hidden: true });
      this._tip = el('div', { class: 'tip', hidden: true });
      this._banner = el('div', { class: 'banner', hidden: true });
      this._file = el('input', { type: 'file', hidden: true });
      const legend = el('div', { class: 'legend' }, [
        ['ok', 'Connected'], ['warning', 'Warning / high'], ['critical', 'Failed / limit reached'], ['idle', 'Idle'],
      ].map(([k, t]) => el('span', {}, [el('i', { style: 'background:' + COLORS[k] }), t])));
      legend.append(el('span', {}, [el('i', { style: 'background:' + COLORS.down }), 'download']), el('span', {}, [el('i', { style: 'background:' + COLORS.up }), 'upload']));
      root.append(el('style', { text: CSS }), el('div', { class: 'wrap' }, [this._svg, this._bar, this._alerts, this._panel, legend, this._tip, this._banner, this._file]));
      this._wrap = root.querySelector('.wrap');

      this._svg.addEventListener('pointerdown', (e) => this._onBgDown(e));
      this._svg.addEventListener('wheel', (e) => this._onWheel(e), { passive: false });
      this._svg.addEventListener('pointermove', (e) => this._onMove(e));
      this._svg.addEventListener('pointerup', (e) => this._onUp(e));
      this._svg.addEventListener('pointercancel', (e) => this._onUp(e));
      this._svg.addEventListener('pointerleave', () => { this._tip.hidden = true; });
      this._buildTools();
    }

    _buildTools() {
      this._tools.replaceChildren();
      if (!this._edit) return;
      const b = (t, fn, title, cls) => el('button', { class: cls, text: t, title, onclick: fn });
      this._tools.append(
        b('➕ Tambah Alat', () => this._addNodePanel(), 'Pick alat infrastruktur / pasif / ODP / Tiang / CCTV dan letakkan di peta', 'on'),
        b('Auto-place', () => this._autoPlace(false), 'Simpan posisi otomatis alat yang belum ditempatkan'),
        b('Re-layout', () => { if (confirm('Pindahkan semua alat yang tidak dikunci kembali ke tata letak otomatis?')) this._autoPlace(true); }, 'Tata ulang semua posisi'),
        b('🗺️ Peta / GIS', () => this._bgDialog(), 'Ganti latar ke OpenStreetMap GIS, Citra Satelit, atau Denah Gambar'),
        b('Thresholds', () => this._thresholdPanel(), 'Batas peringatan utilisasi link & antrean queue'),
        b('Export', () => this._export(), 'Download layout JSON'),
        b('Import', () => this._import(), 'Load layout JSON'),
      );
    }

    _addNodePanel() {
      const iconTypes = [
        ['odp', '📦 ODP (Kotak Fiber Optic)'],
        ['tiang', '🗼 Tiang FO / Listrik'],
        ['closure', '🧰 Joint Closure FO'],
        ['unmanaged-switch', '🔀 Switch Unmanaged / Hub'],
        ['camera', '📹 CCTV Kamera'],
        ['ap', '📶 Access Point'],
        ['ont', '🌐 ONT / Modem Pelanggan'],
        ['desktop', '💻 Komputer / Server'],
      ];
      const selIcon = el('select', {}, iconTypes.map(([val, txt]) => el('option', { value: val, text: txt })));
      const nameIn = el('input', { type: 'text', placeholder: 'Contoh: ODP-01 Jl. Mawar' });
      const parentSel = el('select', {}, [
        el('option', { value: '', text: '-- Tanpa Induk / Pilih Nanti --' }),
        ...Array.from(this._nodes.values()).map(r => el('option', { value: r.data.id, text: `${r.data.display_label || r.data.label} (${r.data.node_type})` }))
      ]);

      this._panel.hidden = false;
      this._panel.replaceChildren(
        el('h4', { text: '➕ Tambah Alat / Titik Pasif' }),
        el('label', { text: 'Jenis Alat / Ikon' }), selIcon,
        el('label', { text: 'Nama / Label Alat' }), nameIn,
        el('label', { text: 'Terhubung ke (Kabel dari):' }), parentSel,
        el('div', { class: 'row', style: 'margin-top:12px' }, [
          el('button', {
            class: 'on',
            text: '⚡ Letakkan di Peta',
            onclick: () => this._guard(async () => {
              const label = nameIn.value.trim() || selIcon.value.toUpperCase();
              const id = 'custom_' + selIcon.value + '_' + Math.random().toString(36).substring(2, 8);
              const parent = parentSel.value || undefined;
              const cx = (this._vb.x + this._vb.w / 2) / W;
              const cy = (this._vb.y + this._vb.h / 2) / this._H;
              const x = Math.min(0.95, Math.max(0.05, r4(cx)));
              const y = Math.min(0.95, Math.max(0.05, r4(cy)));

              await this._api('PUT', this._lp('/nodes/' + encodeURIComponent(id)), {
                x, y, icon: selIcon.value, label, parent_override: parent
              });
              this._toast(`Alat "${label}" berhasil diletakkan di peta`, 'ok');
              this._reload();
            })
          }),
          el('button', { text: 'Tutup', onclick: () => { this._panel.hidden = true; } })
        ])
      );
    }

    // ------------------------------------------------------------ connection
    _hideAllDots() {
      for (const r of this._edges.values()) {
        r.n.down = 0;
        r.n.up = 0;
        r.sp.down = 0;
        r.sp.up = 0;
        for (const k of ['down', 'up']) {
          for (const d of r.dots[k]) {
            d.c.setAttribute('display', 'none');
          }
        }
      }
    }
    _stop() {
      clearTimeout(this._rc); clearInterval(this._poll);
      if (this._ws) { const w = this._ws; this._ws = null; try { w.close(); } catch (_) { /* ignore */ } }
      this._stopped = true;
      this._hideAllDots();
    }
    _restart() {
      this._stop();
      this._stopped = false;
      this._wsFails = 0; this._wsAuthed = false;
      this._btnEdit.hidden = this._get('editable') === 'false';
      const c = this._cfg();
      if (c.demo) { this._startDemo(); return; }
      this._setStatus('connecting');
      if (c.transport === 'poll') this._startPoll(); else this._startWs();
    }
    _setStatus(s, msg) {
      const changed = s !== this._status;
      this._status = s;
      this._pill.className = 'pill ' + s;
      this._pill.textContent = s;
      this._pill.title = msg || '';
      const isDown = s === 'stale' || s === 'error' || s === 'offline' || s === 'connecting';
      this._wrap.classList.toggle('stale', s === 'stale');
      this._wrap.classList.toggle('offline', s === 'offline' || s === 'error');
      this._wrap.classList.toggle('connecting', s === 'connecting');
      this._wrap.classList.toggle('live', s === 'live');
      if (isDown && !this._cfg().demo) {
        this._hideAllDots();
      }
      if (s === 'error' && msg && msg !== this._lastErr) this._toast(msg, 'err');
      this._lastErr = s === 'error' ? msg : null;
      if (changed) this.dispatchEvent(new CustomEvent('status', { detail: { status: s, message: msg || '' } }));
    }
    _watchdog() {
      if (this._status === 'live' && this._lastFrame && Date.now() - this._lastFrame > Math.max(6000, this._cfg().interval * 3)) {
        this._hideAllDots();
        this._setStatus('stale', 'Koneksi terputus / tidak ada pembaruan data');
      }
    }
    _startWs() {
      const c = this._cfg();
      let ws;
      let wsUrl = c.base.replace(/^http/i, 'ws') + '/ws';
      const q = [];
      if (c.ticket) q.push('ticket=' + encodeURIComponent(c.ticket));
      else if (c.token) q.push('token=' + encodeURIComponent(c.token));
      if (q.length) wsUrl += '?' + q.join('&');

      try { ws = new WebSocket(wsUrl); } catch (e) { this._startPoll(); return; }
      this._ws = ws;
      ws.onopen = () => ws.send(JSON.stringify({ action: 'auth', token: c.token || '', ticket: c.ticket || undefined, tag: 'auth' }));
      ws.onmessage = (ev) => {
        let m; try { m = JSON.parse(ev.data); } catch (_) { return; }
        if (m.event === 'auth_ok') {
          this._wsAuthed = true;
          ws.send(JSON.stringify({
            action: 'subscribe_map', tag: TAG, layout_id: c.layout, interval_ms: c.interval, filter_type: c.filter,
            tenant_id: c.tenant || undefined, ...this._routerFields(c),
          }));
        } else if (m.event === 'auth_error' || m.event === 'unauthorized') {
          this._stopped = true;
          this._hideAllDots();
          this._setStatus('error', 'Otentikasi ditolak. Periksa kredensial router atau tiket koneksi.');
          ws.close();
        } else if (m.event === 'map_frame' && m.tag === TAG) {
          this._wsFails = 0; this._onScene(m.data); this._setStatus('live');
        } else if (m.event === 'stream_error' && m.tag === TAG) {
          this._hideAllDots();
          this._setStatus('error', m.error || 'stream error');
        }
      };
      ws.onclose = () => {
        if (this._ws !== ws || this._stopped) return;
        this._ws = null; this._wsFails++;
        this._hideAllDots();
        if (!this._wsAuthed && this._wsFails >= 3 && c.transport === 'auto') { this._startPoll(); return; }
        this._setStatus(this._lastFrame ? 'stale' : 'connecting');
        this._rc = setTimeout(() => this._startWs(), Math.min(15000, 1000 * 2 ** this._wsFails));
      };
      ws.onerror = () => {
        this._hideAllDots();
      };
    }
    _startPoll() {
      const c = this._cfg();
      this._pollOnce();
      this._poll = setInterval(() => this._pollOnce(), c.interval);
    }
    async _pollOnce() {
      if (this._polling || this._cfg().demo) return;
      this._polling = true;
      try {
        const c = this._cfg();
        const s = await this._api('GET', `/api/v1/map/scene?layout_id=${encodeURIComponent(c.layout)}&filter=${encodeURIComponent(c.filter)}`);
        this._onScene(s); this._setStatus('live');
      } catch (e) {
        this._hideAllDots();
        if (e.status === 401) { this._stop(); this._setStatus('error', 'Otentikasi ditolak (HTTP 401). Periksa IP/Akun router atau tiket koneksi.'); } else this._setStatus('error', e.message);
      } finally { this._polling = false; }
    }

    // ------------------------------------------------------------ scene ingest
    _onScene(scene) {
      if (!scene || !Array.isArray(scene.nodes)) return;
      this._lastFrame = Date.now();
      if (this._drag && this._drag.kind === 'node') { this._pending = scene; return; }
      this._render(scene);
      this.dispatchEvent(new CustomEvent('scene', { detail: scene }));
      const seen = new Set();
      for (const a of scene.alerts || []) {
        seen.add(a.id);
        if (!this._alertIds.has(a.id)) this.dispatchEvent(new CustomEvent('alert', { detail: a }));
      }
      this._alertIds = seen;
    }

    get _H() {
      const ar = this._bgAspect || (this._scene && this._scene.layout && this._scene.layout.background.aspect_ratio) || 16 / 9;
      return W / ar;
    }

    _render(scene) {
      this._scene = scene;
      const H = this._H;
      this._applyBackground(scene.layout.background);
      this._bgImg.setAttribute('opacity', String(scene.layout.background.opacity ?? 1));
      for (const r of [this._gridRect, this._bgImg]) { r.setAttribute('height', H); }
      this._title.textContent = scene.layout.name || scene.layout.id || 'Network map';
      if (!this._zoomed) this._vb = homeVb(H);
      this._applyVb();

      const ids = new Set(scene.nodes.map((n) => n.id));
      const dragId = this._drag && this._drag.kind === 'node' ? this._drag.id : null;
      for (const n of scene.nodes) if (n.id !== dragId) this._pos.set(n.id, { x: n.position.x, y: n.position.y });

      for (const [id, r] of this._nodes) if (!ids.has(id)) { r.g.remove(); this._nodes.delete(id); }
      for (const n of scene.nodes) this._updateNode(n);

      const eids = new Set();
      for (const e of scene.edges) {
        if (!this._pos.has(e.source) || !this._pos.has(e.target)) continue;
        eids.add(e.id); this._updateEdge(e);
      }
      for (const [id, r] of this._edges) if (!eids.has(id)) { r.g.remove(); this._edges.delete(id); }

      const s = scene.summary || {};
      const w = s.wan || {};
      this._wan.replaceChildren('WAN ', el('b', { text: '\u2193 ' + (w.down_human || '0 B/s') }), ' ', el('b', { text: '\u2191 ' + (w.up_human || '0 B/s') }));
      const hl = s.health || {};
      this._cnt.textContent = `${s.nodes || 0} nodes \u00b7 ${hl.critical || 0} critical \u00b7 ${hl.warning || 0} warning`;
      this._renderAlerts(scene.alerts || [], scene.warnings || []);
      if (this._sel) {
        const ok = this._sel.kind === 'node' ? this._nodes.has(this._sel.id) : this._edges.has(this._sel.id);
        if (!ok) this._select(null); else this._markSel();
      }
    }

    // ------------------------------------------------------------ nodes
    _updateNode(n) {
      let r = this._nodes.get(n.id);
      if (!r) {
        const rad = NODE_R[n.node_type] || 22;
        const g = sv('g', { class: 'node', 'data-id': n.id });
        const halo = sv('circle', { class: 'halo', r: rad + 7, fill: 'none', 'stroke-width': 3 });
        const body = sv('circle', { class: 'body', r: rad, fill: 'var(--node)', stroke: 'var(--node-s)', 'stroke-width': 2 });
        const icon = sv('g', { transform: `translate(${-rad * 0.5} ${-rad * 0.5}) scale(${rad / 24})` }, sv('path', { fill: 'none', 'stroke-width': 1.7, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }));
        const label = sv('text', { y: rad + 17 });
        const rate = sv('text', { class: 'rate', y: rad + 30 });
        const badge = sv('g', { transform: `translate(${rad * 0.8} ${-rad * 0.8})` }, [sv('circle', { r: 9, fill: COLORS.critical }), sv('text', { y: 3.5, 'font-size': 10, 'font-weight': 700, fill: '#fff', style: 'fill:#fff;stroke:none' })]);
        const lim = sv('g', { transform: `translate(0 ${-rad - 14})` }, [sv('rect', { x: -22, y: -9, width: 44, height: 16, rx: 8, fill: COLORS.critical }), sv('text', { y: 3, style: 'fill:#fff;stroke:none;font-size:9px;font-weight:700', text: 'LIMIT' })]);
        g.append(halo, body, icon, label, rate, badge, lim);
        g.addEventListener('pointerdown', (e) => this._onNodeDown(e, n.id));
        g.addEventListener('pointerenter', (e) => this._showTip(e, n.id));
        g.addEventListener('pointermove', (e) => this._moveTip(e));
        g.addEventListener('pointerleave', () => { this._tip.hidden = true; });
        this._gNodes.append(g);
        r = { g, halo, body, icon: icon.firstChild, label, rate, badge, lim, rad, data: n };
        this._nodes.set(n.id, r);
      }
      r.data = n;
      const p = this._pos.get(n.id);
      r.g.setAttribute('transform', `translate(${p.x * W} ${p.y * this._H})`);
      const c = col(n.health);
      r.halo.setAttribute('stroke', c);
      r.halo.setAttribute('stroke-opacity', n.health === 'idle' ? '.45' : '.9');
      r.halo.setAttribute('class', 'halo' + (n.health === 'critical' && !this._reduce ? ' pulse' : ''));
      r.icon.setAttribute('d', ICONS[n.icon && n.icon.id] || ICONS.unknown);
      r.icon.setAttribute('stroke', c);
      r.label.textContent = short(n.display_label || n.label, n.node_type === 'interface' ? 14 : 24);
      const t = n.traffic;
      r.rate.textContent = t ? `\u2193 ${t.down_human}  \u2191 ${t.up_human}` : (n.health === 'idle' ? n.status : '');
      const crit = (n.subtree && n.subtree.critical) || 0, warn = (n.subtree && n.subtree.warning) || 0;
      r.badge.style.display = crit + warn > 0 ? '' : 'none';
      if (crit + warn > 0) {
        r.badge.firstChild.setAttribute('fill', crit ? COLORS.critical : COLORS.warning);
        r.badge.lastChild.textContent = String(crit || warn);
      }
      const ls = n.queue && n.queue.limit_state;
      r.lim.style.display = ls === 'reached' || ls === 'near' ? '' : 'none';
      if (ls) { r.lim.firstChild.setAttribute('fill', ls === 'reached' ? COLORS.critical : COLORS.warning); r.lim.lastChild.textContent = ls === 'reached' ? 'LIMIT' : 'NEAR'; }
      r.g.classList.toggle('edit', this._edit);
      r.g.classList.toggle('locked', !!n.locked);
    }

    // ------------------------------------------------------------ edges
    _pathD(pts, style) {
      if (pts.length < 2) return '';
      const f = (v) => Math.round(v * 10) / 10;
      let d = `M${f(pts[0].x)} ${f(pts[0].y)}`;
      if (style === 'orthogonal') {
        for (let i = 1; i < pts.length; i++) d += ` L${f(pts[i].x)} ${f(pts[i - 1].y)} L${f(pts[i].x)} ${f(pts[i].y)}`;
      } else if (style === 'smooth' && pts.length > 2) {
        for (let i = 0; i < pts.length - 1; i++) {
          const p0 = pts[i - 1] || pts[i], p1 = pts[i], p2 = pts[i + 1], p3 = pts[i + 2] || p2;
          d += ` C${f(p1.x + (p2.x - p0.x) / 6)} ${f(p1.y + (p2.y - p0.y) / 6)} ${f(p2.x - (p3.x - p1.x) / 6)} ${f(p2.y - (p3.y - p1.y) / 6)} ${f(p2.x)} ${f(p2.y)}`;
        }
      } else for (let i = 1; i < pts.length; i++) d += ` L${f(pts[i].x)} ${f(pts[i].y)}`;
      return d;
    }
    _edgePts(r) {
      const H = this._H, a = this._pos.get(r.data.source), b = this._pos.get(r.data.target);
      return [a, ...r.wps, b].map((p) => ({ x: p.x * W, y: p.y * H }));
    }
    _geom(r) {
      r.pts = this._edgePts(r);
      const d = this._pathD(r.pts, r.style);
      r.hit.setAttribute('d', d); r.line.setAttribute('d', d); r.path.setAttribute('d', d);
      r.len = r.path.getTotalLength ? r.path.getTotalLength() : 0;
    }
    _updateEdge(e) {
      let r = this._edges.get(e.id);
      if (!r) {
        const g = sv('g', { class: 'edge', 'data-id': e.id });
        const hit = sv('path', { class: 'hit' });
        const line = sv('path', { class: 'line', 'stroke-width': 3, 'stroke-opacity': 0.85 });
        const path = sv('path', { fill: 'none', stroke: 'none' });
        const pg = sv('g', { 'pointer-events': 'none' });
        const dots = { down: [], up: [] };
        for (const k of ['down', 'up']) for (let i = 0; i < MAX_DOTS; i++) {
          const c = sv('circle', { r: 3.5, fill: k === 'down' ? COLORS.down : COLORS.up, stroke: '#ffffff', 'stroke-width': 1.2, display: 'none' });
          pg.append(c); dots[k].push({ c, off: (i / MAX_DOTS) * 400 });
        }
        g.append(hit, line, path, pg);
        hit.addEventListener('pointerdown', (ev) => { if (this._edit) { ev.stopPropagation(); this._select({ kind: 'edge', id: e.id }); } });
        hit.addEventListener('dblclick', (ev) => this._addWaypoint(ev, e.id));
        this._gEdges.append(g);
        r = { id: e.id, g, hit, line, path, pg, dots, len: 0, pts: [], wps: [], style: 'straight', n: { down: 0, up: 0 }, sp: { down: 0, up: 0 }, rev: false };
        this._edges.set(e.id, r);
      }
      r.data = e;
      if (!(this._drag && this._drag.kind === 'wp' && this._drag.id === e.id)) r.wps = (e.path.waypoints || []).map((p) => ({ x: p.x, y: p.y }));
      r.style = e.path.style || 'straight';
      this._geom(r);

      const srcData = this._nodes.get(e.source)?.data;
      const tgtData = this._nodes.get(e.target)?.data;
      const isDisconnected = e.health === 'critical' || srcData?.status === 'offline' || tgtData?.status === 'offline';
      const isInactive = isDisconnected || e.health === 'idle' || (this._status !== 'live' && !this._cfg().demo);

      const c = isDisconnected ? COLORS.critical : col(e.health);
      r.line.setAttribute('stroke', c);
      if (isDisconnected) {
        r.line.setAttribute('stroke-dasharray', '6 5');
        r.line.setAttribute('stroke-opacity', '0.55');
      } else {
        r.line.setAttribute('stroke-dasharray', e.edge_type === 'relay' || e.edge_type === 'logical' ? '9 6' : '');
        r.line.setAttribute('stroke-opacity', '0.85');
      }
      r.g.classList.toggle('edit', this._edit);
      const f = e.flow || {};
      r.rev = f.down_direction === 'reverse';
      for (const k of ['down', 'up']) {
        if (isInactive) {
          r.n[k] = 0;
          r.sp[k] = 0;
          r.dots[k].forEach((d) => d.c.setAttribute('display', 'none'));
        } else {
          const inten = f[k + '_intensity'] || 0;
          r.n[k] = inten > 0 ? Math.min(MAX_DOTS, 1 + Math.round(inten * (MAX_DOTS - 1))) : 0;
          r.sp[k] = 30 + 230 * inten;
          r.dots[k].forEach((d, i) => d.c.setAttribute('display', i < r.n[k] && !this._reduce ? '' : 'none'));
        }
      }
    }
    _frame(ts) {
      this._raf = requestAnimationFrame((t) => this._frame(t));
      if (document.hidden || this._reduce) { this._last = ts; return; }
      if (this._status !== 'live' && !this._cfg().demo) {
        this._last = ts;
        return;
      }
      const dt = Math.min(0.1, ((ts - (this._last || ts)) / 1000) || 0);
      this._last = ts;
      for (const r of this._edges.values()) {
        if (!r.len) continue;
        for (const k of ['down', 'up']) {
          const n = r.n[k];
          if (!n) continue;
          const forward = k === 'down' ? !r.rev : r.rev;
          for (let i = 0; i < n; i++) {
            const d = r.dots[k][i];
            d.off = (d.off + r.sp[k] * dt) % r.len;
            const p = r.path.getPointAtLength(forward ? d.off : r.len - d.off);
            d.c.setAttribute('cx', p.x.toFixed(1)); d.c.setAttribute('cy', p.y.toFixed(1));
          }
        }
      }
    }

    // ------------------------------------------------------------ background
    async _applyBackground(bg) {
      const key = `${bg.kind}|${bg.url || ''}|${bg.has_upload}`;
      if (key === this._bgKey) return;
      this._bgKey = key;
      if (bg.kind === 'url' && bg.url && /^https?:\/\//i.test(bg.url)) return this._setBgSource(bg.url);
      if (bg.kind === 'upload' && bg.has_upload) {
        try {
          const res = await this._api('GET', this._lp('/background'));
          const blob = await res.blob();
          if (this._bgBlob) URL.revokeObjectURL(this._bgBlob);
          this._bgBlob = URL.createObjectURL(blob);
          return this._setBgSource(this._bgBlob);
        } catch (e) { this._toast('Background: ' + e.message, 'err'); }
      }
      this._bgImg.removeAttribute('href'); this._bgAspect = null;
    }
    _setBgSource(url) {
      const img = new Image();
      img.referrerPolicy = 'no-referrer';
      img.onload = () => {
        this._bgAspect = img.naturalWidth / img.naturalHeight;
        this._bgImg.setAttribute('href', url);
        if (this._scene) this._render(this._scene);
      };
      img.onerror = () => this._toast('Background image could not be loaded', 'err');
      img.src = url;
    }

    // ------------------------------------------------------------ viewport
    _applyVb() { const v = this._vb; this._svg.setAttribute('viewBox', `${v.x} ${v.y} ${v.w} ${v.h}`); }
    _fit() { this._vb = homeVb(this._H); this._applyVb(); }
    _pt(ev) {
      const m = this._svg.getScreenCTM();
      if (!m) return { x: 0, y: 0 };
      const p = this._svg.createSVGPoint(); p.x = ev.clientX; p.y = ev.clientY;
      const q = p.matrixTransform(m.inverse());
      return { x: q.x, y: q.y };
    }
    _onWheel(e) {
      e.preventDefault();
      const p = this._pt(e), k = e.deltaY < 0 ? 0.85 : 1.18, v = this._vb;
      const nw = Math.min(W * 2.5, Math.max(W / 8, v.w * k)), s = nw / v.w;
      this._vb = { x: p.x - (p.x - v.x) * s, y: p.y - (p.y - v.y) * s, w: nw, h: v.h * s };
      this._zoomed = true; this._applyVb();
    }
    _onBgDown(e) {
      if (e.target.closest && (e.target.closest('.node') || e.target.closest('.edge') || e.target.closest('.handle'))) return;
      if (this._sel) this._select(null);
      this._drag = { kind: 'pan', x: e.clientX, y: e.clientY, vb: { ...this._vb } };
      this._svg.setPointerCapture(e.pointerId); this._svg.classList.add('pan');
    }
    _onMove(e) {
      const d = this._drag; if (!d) return;
      if (d.kind === 'pan') {
        const m = this._svg.getScreenCTM(); if (!m) return;
        this._vb = { ...d.vb, x: d.vb.x - (e.clientX - d.x) / m.a, y: d.vb.y - (e.clientY - d.y) / m.d };
        this._zoomed = true; this._applyVb();
      } else if (d.kind === 'node') {
        if (d.noMove) return;
        const p = this._pt(e);
        if (Math.hypot(e.clientX - d.cx, e.clientY - d.cy) > 3) d.moved = true;
        if (!d.moved) return;
        const pos = { x: r4(clamp01(p.x / W)), y: r4(clamp01(p.y / this._H)) };
        this._pos.set(d.id, pos); d.pos = pos;
        const r = this._nodes.get(d.id);
        r.g.setAttribute('transform', `translate(${pos.x * W} ${pos.y * this._H})`);
        for (const er of this._edges.values()) if (er.data.source === d.id || er.data.target === d.id) this._geom(er);
        this._drawHandles();
      } else if (d.kind === 'wp') {
        const p = this._pt(e), r = this._edges.get(d.id);
        r.wps[d.i] = { x: r4(clamp01(p.x / W)), y: r4(clamp01(p.y / this._H)) };
        d.moved = true; this._geom(r); this._drawHandles();
      }
    }
    _onUp(e) {
      const d = this._drag; if (!d) return;
      this._drag = null; this._svg.classList.remove('pan');
      try { this._svg.releasePointerCapture(e.pointerId); } catch (_) { /* ignore */ }
      if (d.kind === 'node') {
        if (!d.moved) this._clickNode(d.id);
        else if (this._edit) this._saveNode(d.id, { x: d.pos.x, y: d.pos.y });
      } else if (d.kind === 'wp' && d.moved) this._saveEdge(this._edges.get(d.id));
      if (this._pending) { const s = this._pending; this._pending = null; this._onScene(s); }
    }

    // ------------------------------------------------------------ interaction
    _onNodeDown(e, id) {
      e.stopPropagation();
      const r = this._nodes.get(id);
      // Dragging only in edit mode and only for unlocked nodes; otherwise a press is a plain click.
      this._drag = { kind: 'node', id, cx: e.clientX, cy: e.clientY, moved: false, noMove: !this._edit || !!r.data.locked };
      this._svg.setPointerCapture(e.pointerId);
    }
    _clickNode(id) {
      const r = this._nodes.get(id);
      this.dispatchEvent(new CustomEvent('node-select', { detail: r.data }));
      this._select({ kind: 'node', id });
    }
    _select(s) {
      this._sel = s;
      this._markSel(); this._drawHandles(); this._renderPanel();
    }
    _markSel() {
      for (const [id, r] of this._nodes) r.g.classList.toggle('sel', !!this._sel && this._sel.kind === 'node' && this._sel.id === id);
      for (const [id, r] of this._edges) r.g.classList.toggle('sel', !!this._sel && this._sel.kind === 'edge' && this._sel.id === id);
    }
    _showTip(e, id) {
      const r = this._nodes.get(id); if (!r || this._drag) return;
      const n = r.data, t = n.traffic, q = n.queue;
      const rows = [el('div', { class: 'h', text: n.display_label || n.label })];
      if (n.display_label && n.display_label !== n.label) rows.push(el('div', { class: 'm', text: 'router name: ' + n.label }));
      rows.push(el('div', { class: 'm', text: `${n.node_type}/${n.sub_type} \u00b7 ${n.status} \u00b7 icon ${n.icon.id} (${n.icon.source})` }));
      const meta = [n.ip, n.mac, n.vendor, n.interface && 'if ' + n.interface].filter(Boolean).join(' \u00b7 ');
      if (meta) rows.push(el('div', { text: meta }));
      if (t) {
        rows.push(el('div', { text: `\u2193 ${t.down_human} (${t.down_bits_human})  \u2191 ${t.up_human} (${t.up_bits_human})` }));
        if (t.pps_in != null) rows.push(el('div', { class: 'm', text: `packets/s in ${t.pps_in} \u00b7 out ${t.pps_out}` }));
        if (t.total_down_human) rows.push(el('div', { class: 'm', text: `total \u2193 ${t.total_down_human}  \u2191 ${t.total_up_human}` }));
        if (t.capacity_bps) rows.push(el('div', { class: 'm', text: `capacity ${(t.capacity_bps / 1e6).toFixed(0)} Mbps (${t.capacity_source})` }));
      }
      if (q) rows.push(el('div', { text: `queue ${q.name}: max \u2193 ${q.max_down_human} \u2191 ${q.max_up_human} \u00b7 ${q.limit_state}` }));
      for (const x of n.reasons || []) rows.push(el('div', { style: 'color:' + col(x.severity), text: '\u25cf ' + x.message }));
      this._tip.replaceChildren(...rows); this._tip.hidden = false; this._moveTip(e);
    }
    _moveTip(e) {
      const b = this._wrap.getBoundingClientRect();
      let x = e.clientX - b.left + 14, y = e.clientY - b.top + 14;
      if (x + 310 > b.width) x = e.clientX - b.left - 314;
      if (y + this._tip.offsetHeight + 8 > b.height) y = Math.max(8, b.height - this._tip.offsetHeight - 8);
      this._tip.style.left = Math.max(4, x) + 'px'; this._tip.style.top = y + 'px';
    }
    _onKey(e) {
      if (e.key === 'Escape') this._select(null);
      if ((e.key === 'Delete' || e.key === 'Backspace') && this._hwp != null && this._sel && this._sel.kind === 'edge' && this._edit && !/INPUT|SELECT|TEXTAREA/.test(e.composedPath()[0].tagName || '')) {
        this._removeWaypoint(this._sel.id, this._hwp);
      }
    }
    _renderAlerts(alerts, warnings) {
      const sig = JSON.stringify([alerts.map((a) => a.id + a.message), warnings]);
      if (sig === this._alertSig) return;
      this._alertSig = sig;
      const list = [...alerts].sort((a, b) => (a.severity === b.severity ? 0 : a.severity === 'critical' ? -1 : 1));
      const kids = list.length || warnings.length ? [el('h4', { text: `Alerts (${list.length})` })] : [];
      for (const a of list.slice(0, 40)) {
        kids.push(el('div', { class: 'al', onclick: () => this._focusNode(a.node_id) }, [el('span', { class: 'dot', style: 'background:' + col(a.severity) }), el('span', { text: `${short(a.label, 28)}: ${a.message}` })]));
      }
      for (const w of warnings) kids.push(el('div', { class: 'al' }, [el('span', { class: 'dot', style: 'background:' + COLORS.idle }), el('span', { text: w })]));
      this._alerts.replaceChildren(...kids);
    }
    _focusNode(id) {
      const p = this._pos.get(id); if (!p) return;
      const v = this._vb, nw = Math.min(v.w, W * 0.5), s = nw / v.w;
      this._vb = { x: p.x * W - nw / 2, y: p.y * this._H - (v.h * s) / 2, w: nw, h: v.h * s };
      this._zoomed = true; this._applyVb(); this._select({ kind: 'node', id });
    }

    // ------------------------------------------------------------ edit mode
    _setEdit(on) {
      if (on && this._get('editable') === 'false') return;
      if (on && this._cfg().demo) { this._toast('Demo mode is read-only', 'err'); return; }
      this._edit = !!on;
      this._btnEdit.classList.toggle('on', this._edit);
      this._btnEdit.textContent = this._edit ? 'Done' : 'Edit';
      this._applyEditClasses(); this._buildTools();
      if (this._edit) this._loadLayout().catch(() => {});
      for (const r of this._nodes.values()) r.g.classList.toggle('edit', this._edit);
      for (const r of this._edges.values()) r.g.classList.toggle('edit', this._edit);
      this._select(null);
    }
    _applyEditClasses() {
      this._btnEdit.classList.toggle('on', this._edit);
      this._btnEdit.textContent = this._edit ? 'Done' : 'Edit';
      this._btnEdit.hidden = this._get('editable') === 'false';
      this._buildTools();
    }
    async _loadLayout() {
      try { this._layout = (await this._api('GET', this._lp())).layout; } catch (e) {
        if (e.status === 404) this._layout = { nodes: {}, paths: {}, version: 0 }; else throw e;
      }
      return this._layout;
    }
    _toast(msg, kind) {
      this._banner.textContent = msg; this._banner.className = 'banner ' + (kind || '');
      this._banner.hidden = false; clearTimeout(this._bt);
      this._bt = setTimeout(() => { this._banner.hidden = true; }, kind === 'err' ? 6000 : 3000);
    }
    async _guard(fn) {
      try { return await fn(); } catch (e) {
        this._toast(e.status === 409 ? 'Layout changed elsewhere - reloaded. Try again.' : e.message, 'err');
        if (e.status === 409) this._loadLayout().catch(() => {});
      }
    }
    _saveNode(id, patch) {
      const p = this._pos.get(id);
      return this._guard(async () => {
        const res = await this._api('PUT', this._lp('/nodes/' + encodeURIComponent(id)), { x: p.x, y: p.y, ...patch });
        if (this._layout) { this._layout.nodes[id] = res.node; this._layout.version = res.version; }
        this._toast('Saved', 'ok'); this._reload();
      });
    }
    _saveEdge(r, extra) {
      return this._guard(async () => {
        const prev = (this._layout && this._layout.paths && this._layout.paths[r.id]) || {};
        const body = { waypoints: r.wps, style: r.style, capacity_bps: prev.capacity_bps ?? null, ...extra };
        const res = await this._api('PUT', this._lp('/paths/' + encodeURIComponent(r.id)), body);
        if (this._layout) { this._layout.paths[r.id] = res.path; this._layout.version = res.version; }
        this._toast('Saved', 'ok'); this._reload();
      });
    }
    _reload() { return this._pollOnceScene(); }
    async _pollOnceScene() {
      try { const c = this._cfg(); this._onScene(await this._api('GET', `/api/v1/map/scene?layout_id=${encodeURIComponent(c.layout)}&filter=${encodeURIComponent(c.filter)}`)); } catch (_) { /* next frame will refresh */ }
    }
    _addWaypoint(ev, id) {
      if (!this._edit) return;
      const r = this._edges.get(id), p = this._pt(ev);
      let best = 0, bd = Infinity;
      for (let i = 0; i < r.pts.length - 1; i++) {
        const a = r.pts[i], b = r.pts[i + 1], dx = b.x - a.x, dy = b.y - a.y, l2 = dx * dx + dy * dy || 1;
        const t = Math.max(0, Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / l2));
        const d = Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
        if (d < bd) { bd = d; best = i; }
      }
      r.wps.splice(best, 0, { x: r4(clamp01(p.x / W)), y: r4(clamp01(p.y / this._H)) });
      this._geom(r); this._select({ kind: 'edge', id }); this._saveEdge(r);
    }
    _removeWaypoint(id, i) {
      const r = this._edges.get(id); if (!r || !r.wps[i]) return;
      r.wps.splice(i, 1); this._hwp = null; this._geom(r); this._drawHandles(); this._saveEdge(r);
    }
    _drawHandles() {
      this._gHandles.replaceChildren();
      if (!this._edit || !this._sel || this._sel.kind !== 'edge') return;
      const r = this._edges.get(this._sel.id); if (!r) return;
      r.wps.forEach((w, i) => {
        const c = sv('circle', { class: 'handle', cx: w.x * W, cy: w.y * this._H, r: 7 });
        c.addEventListener('pointerdown', (e) => {
          e.stopPropagation();
          if (e.altKey) { this._removeWaypoint(r.id, i); return; }
          this._hwp = i; this._drag = { kind: 'wp', id: r.id, i, moved: false }; this._svg.setPointerCapture(e.pointerId);
        });
        c.addEventListener('dblclick', () => this._removeWaypoint(r.id, i));
        this._gHandles.append(c);
      });
    }
    _autoPlace(overwrite) {
      return this._guard(async () => {
        const c = this._cfg();
        const res = await this._api('POST', this._lp('/auto-place'), { overwrite, ...(c.host && c.user ? { router: { host: c.host, port: c.port ? +c.port : undefined, user: c.user, password: c.pass } } : { router_id: c.routerId || undefined }), filter_type: c.filter });
        this._toast(`Placed ${res.placed} node(s)`, 'ok'); this._loadLayout().catch(() => {}); this._reload();
      });
    }

    // ------------------------------------------------------------ inspector panel
    _renderPanel() {
      const s = this._sel;
      if (!s || !this._edit) { this._panel.hidden = true; this._panel.replaceChildren(); return; }
      this._panel.hidden = false;
      if (s.kind === 'node') this._nodePanel(this._nodes.get(s.id).data); else this._edgePanel(this._edges.get(s.id));
    }
    _nodePanel(n) {
      const label = el('input', { type: 'text', maxlength: 80, value: n.icon && n.display_label !== n.label ? n.display_label : '', placeholder: n.label });
      const icon = el('select', {}, [el('option', { value: '', text: `Auto-detect (${n.icon.auto_id})` })]);
      for (const id of Object.keys(ICONS)) icon.append(el('option', { value: id, text: id, selected: n.icon.source === 'custom' && n.icon.id === id }));
      const parent = el('select', {}, [
        el('option', { value: '', text: 'Auto (as reported by router)' }),
        el('option', { value: 'none', text: '❌ Terputus / Tanpa Kabel' })
      ]);
      for (const m of this._nodes.values()) {
        const d = m.data;
        if (d.id !== n.id && ['ap', 'router', 'interface', 'infrastructure'].includes(d.node_type)) {
          parent.append(el('option', { value: d.id, text: short(d.display_label || d.label, 34), selected: n.parent_source === 'override' && n.parent_id === d.id }));
        }
      }
      if (n.parent_source === 'override' && (!n.parent_id || n.parent_id === 'none')) {
        parent.value = 'none';
      }
      const lock = el('input', { type: 'checkbox', checked: n.locked });
      const save = () => this._saveNode(n.id, { label: label.value.trim(), icon: icon.value, parent_override: parent.value, locked: lock.checked });

      const actionsRow = el('div', { class: 'row', style: 'margin-top:12px;padding-top:10px;border-top:1px dashed var(--line)' });
      if (n.ip) {
        actionsRow.append(el('button', {
          class: 'danger',
          text: '🚫 Blokir IP di Firewall',
          title: 'Drop IP perangkat ini di Firewall Filter MikroTik',
          onclick: () => this._blockFirewallIp(n.ip, n.display_label || n.label)
        }));
      }
      actionsRow.append(el('button', {
        text: '✂️ Putus Relasi Kabel',
        title: 'Putuskan koneksi kabel ke induk',
        onclick: () => this._guard(async () => {
          const p = this._pos.get(n.id) || { x: 0.5, y: 0.5 };
          await this._api('PUT', this._lp('/nodes/' + encodeURIComponent(n.id)), {
            x: p.x, y: p.y, parent_override: 'none'
          });
          this._toast('Relasi kabel diputuskan', 'ok');
          this._reload();
        })
      }));
      if (n.id.startsWith('custom_') || n.id.startsWith('manual_')) {
        actionsRow.append(el('button', {
          class: 'danger',
          text: '🗑️ Hapus Titik Ini',
          title: 'Hapus alat kustom ini dari peta',
          onclick: () => this._guard(async () => {
            if (!confirm(`Hapus alat "${n.display_label || n.label}" dari peta?`)) return;
            await this._api('DELETE', this._lp('/nodes/' + encodeURIComponent(n.id)));
            this._select(null);
            this._toast('Alat berhasil dihapus', 'ok');
            this._reload();
          })
        }));
      }

      this._panel.replaceChildren(
        el('h4', { text: 'Perangkat / Titik' }),
        el('div', { text: n.display_label || n.label, style: 'font-weight:700;font-size:14px;color:var(--fg)' }),
        el('div', { class: 'hint', text: [n.ip, n.mac, n.vendor].filter(Boolean).join(' \u00b7 ') }),
        el('label', { text: 'Nama Tampilan (Display Name)' }), label,
        el('label', { text: 'Jenis Ikon' }), icon,
        el('label', { text: 'Terhubung ke Induk (Relay / ODP / Switch):' }), parent,
        el('label', {}, [lock, ' Kunci posisi (cegah geser)']),
        el('div', { class: 'row' }, [
          el('button', { class: 'on', text: 'Simpan', onclick: save }),
          el('button', { class: 'danger', text: 'Reset Posisi', onclick: () => this._guard(async () => { await this._api('DELETE', this._lp('/nodes/' + encodeURIComponent(n.id))); this._reload(); this._toast('Kembali ke posisi otomatis', 'ok'); }) }),
        ]),
        actionsRow,
        el('div', { class: 'hint', text: 'Geser titik pada peta untuk memindahkannya. Perubahan posisi tersimpan otomatis.' }),
      );
    }
    _edgePanel(r) {
      const style = el('select', {}, ['straight', 'orthogonal', 'smooth'].map((v) => el('option', { value: v, text: v, selected: v === r.style })));
      const cap = (this._layout && this._layout.paths && this._layout.paths[r.id] && this._layout.paths[r.id].capacity_bps) || null;
      const capIn = el('input', { type: 'number', min: 0, step: 'any', value: cap ? cap / 1e6 : '', placeholder: 'Mbps (empty = auto)' });
      this._panel.replaceChildren(
        el('h4', { text: 'Jalur Kabel' }), el('div', { text: `${r.data.source} \u2192 ${r.data.target}` }),
        el('label', { text: 'Gaya Garis Kabel' }), style,
        el('label', { text: 'Kapasitas Link (Mbps) - untuk ambang batas limit' }), capIn,
        el('div', { class: 'row' }, [
          el('button', { class: 'on', text: 'Simpan', onclick: () => { r.style = style.value; const v = parseFloat(capIn.value); this._saveEdge(r, { capacity_bps: v > 0 ? Math.round(v * 1e6) : null }); } }),
          el('button', { class: 'danger', text: 'Hapus Waypoint', onclick: () => { r.wps = []; this._geom(r); this._drawHandles(); this._saveEdge(r); } }),
        ]),
        el('div', { class: 'hint', text: 'Double-click garis kabel untuk menambah titik belok (tikungan jalan, atap). Geser titik putih untuk membelokkan.' }),
      );
    }
    _thresholdPanel() {
      this._guard(async () => {
        const l = await this._loadLayout();
        const t = (this._scene && this._scene.thresholds) || { warn_pct: 80, crit_pct: 95, high_pps: 50000 };
        const num = (v, ph) => el('input', { type: 'number', min: 0, step: 'any', value: v == null ? '' : v, placeholder: ph });
        const w = num(t.warn_pct), c = num(t.crit_pct), p = num(t.high_pps), b = num(t.high_bps ? t.high_bps / 1e6 : '', 'Mbps (optional)');
        this._panel.hidden = false;
        this._panel.replaceChildren(
          el('h4', { text: 'Batas Peringatan (Thresholds)' }),
          el('label', { text: 'Peringatan kuning (% batas antrean / link)' }), w,
          el('label', { text: 'Batas kritis merah (% limit reached)' }), c,
          el('label', { text: 'Beban paket tinggi (packets/detik)' }), p,
          el('label', { text: 'Trafik throughput tinggi (Mbps)' }), b,
          el('div', { class: 'row' }, [el('button', { class: 'on', text: 'Simpan Ambang', onclick: () => this._guard(async () => {
            const body = { ...l, thresholds: { warn_pct: +w.value, crit_pct: +c.value, high_pps: Math.round(+p.value), high_bps: +b.value > 0 ? Math.round(+b.value * 1e6) : null } };
            const res = await this._api('PUT', this._lp(), body);
            this._layout = res.layout; this._toast('Ambang batas berhasil disimpan', 'ok'); this._reload();
          }) })]),
        );
      });
    }
    _bgDialog() {
      const url = el('input', { type: 'text', placeholder: 'https://.../map.png' });
      this._panel.hidden = false;
      this._panel.replaceChildren(
        el('h4', { text: '🗺️ Peta Latar (GIS / Denah)' }),
        el('label', { text: 'Preset Peta Geografis Dunia (GIS):' }),
        el('div', { class: 'row' }, [
          el('button', {
            class: 'on',
            text: '🗺️ OpenStreetMap',
            title: 'Gunakan peta jalanan OpenStreetMap dunia',
            onclick: () => this._guard(async () => {
              const l = await this._loadLayout();
              const osmUrl = 'https://tile.openstreetmap.org/15/26359/16664.png';
              const res = await this._api('PUT', this._lp(), { ...l, background: { ...(l.background || {}), kind: 'url', url: osmUrl } });
              this._layout = res.layout; this._bgKey = null; this._reload();
              this._toast('Latar OpenStreetMap terpasang', 'ok');
            })
          }),
          el('button', {
            text: '🛰️ Satelit ESRI',
            title: 'Gunakan citra satelit bumi ESRI World Imagery',
            onclick: () => this._guard(async () => {
              const l = await this._loadLayout();
              const esriUrl = 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/15/16664/26359';
              const res = await this._api('PUT', this._lp(), { ...l, background: { ...(l.background || {}), kind: 'url', url: esriUrl } });
              this._layout = res.layout; this._bgKey = null; this._reload();
              this._toast('Latar Satelit ESRI terpasang', 'ok');
            })
          }),
        ]),
        el('label', { text: 'Upload Gambar Denah / Citra Wilayah Sendiri:' }),
        el('div', { class: 'row' }, [el('button', { text: '📁 Upload File Gambar\u2026', onclick: () => this._pickFile('image/png,image/jpeg,image/webp', async (f) => {
          if (f.size > 4 * 1024 * 1024) { this._toast('Ukuran gambar melebihi 4 MB', 'err'); return; }
          await this._guard(async () => { await this._api('PUT', this._lp('/background'), f); this._bgKey = null; this._toast('Gambar denah berhasil diupload', 'ok'); this._reload(); });
        }) })]),
        el('label', { text: 'Atau masukkan URL gambar peta kustom (http/https):' }), url,
        el('div', { class: 'row' }, [
          el('button', { text: 'Terapkan URL', onclick: () => this._guard(async () => {
            const l = await this._loadLayout();
            const res = await this._api('PUT', this._lp(), { ...l, background: { ...(l.background || {}), kind: 'url', url: url.value.trim() } });
            this._layout = res.layout; this._bgKey = null; this._reload();
          }) }),
          el('button', { class: 'danger', text: 'Hapus Latar', onclick: () => this._guard(async () => { await this._api('DELETE', this._lp('/background')); const l = await this._loadLayout(); if (l.background && l.background.kind === 'url') { await this._api('PUT', this._lp(), { ...l, background: { ...l.background, kind: 'none', url: null } }); } this._bgKey = null; this._reload(); }) }),
        ]),
        el('div', { class: 'hint', text: 'Format PNG, JPEG, atau WebP (maks 4 MB). Koordinat tersimpan otomatis sebagai persentase denah/peta.' }),
      );
    }
    async _blockFirewallIp(ip, label) {
      if (!confirm(`Yakin ingin memblokir akses internet untuk perangkat "${label}" (IP: ${ip}) di firewall MikroTik?`)) return;
      try {
        const c = this._cfg();
        const headers = { 'Content-Type': 'application/json' };
        if (c.token) headers.Authorization = 'Bearer ' + c.token;
        if (c.ticket) headers['X-Connection-Ticket'] = c.ticket;
        if (c.host && c.user) {
          headers['X-Router-Host'] = c.host; headers['X-Router-User'] = c.user;
          if (c.port) headers['X-Router-Port'] = c.port;
          if (c.pass) headers['X-Router-Pass'] = c.pass;
        } else if (c.routerId) headers['X-Router-Id'] = c.routerId;
        const res = await fetch(c.base + '/api/v1/firewall/block-ip', {
          method: 'POST',
          headers,
          body: JSON.stringify({ address: ip, list: 'Blacklist' })
        });
        const json = await res.json();
        if (json.success) {
          this._toast(`IP ${ip} berhasil diblokir di firewall!`, 'ok');
        } else {
          this._toast(`Gagal memblokir: ${json.error || 'Unknown error'}`, 'err');
        }
      } catch (e) {
        this._toast(`Gagal memblokir: ${e.message}`, 'err');
      }
    }
    _pickFile(accept, cb) {
      this._file.value = ''; this._file.accept = accept;
      this._file.onchange = () => { const f = this._file.files[0]; if (f) cb(f); };
      this._file.click();
    }
    _export() {
      this._guard(async () => {
        const l = await this._loadLayout();
        const a = document.createElement('a');
        a.href = URL.createObjectURL(new Blob([JSON.stringify(l, null, 2)], { type: 'application/json' }));
        a.download = (this._cfg().layout || 'layout') + '.json'; a.click(); setTimeout(() => URL.revokeObjectURL(a.href), 2000);
      });
    }
    _import() {
      this._pickFile('application/json,.json', async (f) => {
        await this._guard(async () => {
          const body = JSON.parse(await f.text());
          delete body.version; delete body.id; delete body.updated_at;
          if (body.background && body.background.kind === 'upload') body.background = { ...body.background, kind: 'none' };
          const res = await this._api('PUT', this._lp(), body);
          this._layout = res.layout; this._toast('Layout imported', 'ok'); this._reload();
        });
      });
    }

    // ------------------------------------------------------------ demo (no gateway needed)
    _startDemo() {
      this._setStatus('demo');
      this._btnEdit.hidden = true;
      const tick = () => { this._lastFrame = Date.now(); this._onScene(demoScene(Date.now() / 1000)); };
      tick(); this._poll = setInterval(tick, 1000);
    }
  }

  // ---------------------------------------------------------------- demo scene
  function human(bps) {
    const b = Math.max(0, bps) / 8;
    return b >= 1048576 ? (b / 1048576).toFixed(2) + ' MB/s' : b >= 1024 ? (b / 1024).toFixed(1) + ' KB/s' : Math.round(b) + ' B/s';
  }
  function demoScene(t) {
    const wob = (a, p) => 0.55 + 0.45 * Math.sin(t / a + p);
    const nodes = [], edges = [];
    const traffic = (down, up, extra) => ({
      source: 'demo', down_bps: down, up_bps: up, down_human: human(down), up_human: human(up),
      down_bits_human: (down / 1e6).toFixed(2) + ' Mbps', up_bits_human: (up / 1e6).toFixed(2) + ' Mbps',
      pps_in: Math.round(down / 12000), pps_out: Math.round(up / 12000), capacity_bps: null, capacity_source: 'unknown', util_pct: { down: null, up: null }, ...extra,
    });
    const N = (id, label, nt, st, x, y, icon, health, parent, tr, over) => nodes.push({
      id, label, display_label: label, node_type: nt, sub_type: st, ip: null, mac: null, vendor: null, interface: null, parent_id: parent, status: health === 'critical' ? 'limit' : 'online',
      position: { x, y, source: 'saved' }, icon: { id: icon, source: 'auto', auto_id: icon }, health, reasons: [], traffic: tr || null, queue: null, locked: false, subtree: { total: 0, warning: 0, critical: 0 }, extra: {}, ...over,
    });
    const E = (id, s, tg, health, down, up, pts, style, type, rev) => edges.push({
      id, source: s, target: tg, edge_type: type || 'physical', speed: null, health,
      path: { waypoints: pts || [], style: style || 'straight' }, traffic: null,
      flow: { down_intensity: Math.min(1, Math.log10(1 + down) / 9), up_intensity: Math.min(1, Math.log10(1 + up) / 9), down_direction: rev ? 'reverse' : 'forward' },
    });
    const wd = 9e6 * wob(3, 0), wu = 1.2e6 * wob(4, 1);
    const apd = 4e6 * wob(2, 2), apu = 6e5 * wob(3, 3);
    N('node_internet', 'Internet (ISP)', 'internet', 'wan_cloud', 0.05, 0.5, 'internet', 'ok', null, traffic(wd, wu));
    N('if_ether1', 'ether1 (WAN)', 'interface', 'wan', 0.16, 0.5, 'wan-port', 'ok', 'node_router', traffic(wd, wu));
    N('node_router', 'MikroTik hAP', 'router', 'core', 0.29, 0.5, 'router', 'ok', null, traffic(wd, wu));
    N('if_ether5', 'ether5', 'interface', 'ether', 0.43, 0.3, 'port', 'ok', 'node_router', traffic(apd + 2e6, apu));
    N('ap1', 'AP Living room', 'ap', 'ap', 0.56, 0.2, 'ap', 'ok', 'if_ether5', traffic(apd, apu));
    N('ap2', 'AP Garage (relay)', 'ap', 'ap', 0.82, 0.16, 'repeater', 'critical', 'ap1', traffic(3e6, 4e5));
    N('phone', 'Galaxy-S23', 'device', 'dhcp', 0.5, 0.42, 'phone', 'ok', 'ap1', traffic(2e6 * wob(2, 4), 2e5 * wob(2, 5)));
    N('laptop', 'ThinkPad', 'device', 'dhcp', 0.64, 0.4, 'laptop', 'warning', 'ap1', traffic(8.4e6 * wob(5, 6), 6e5), {
      queue: { name: 'laptop', max_down_human: '10.00 Mbps', max_up_human: '2.00 Mbps', limit_state: 'near' },
      reasons: [{ kind: 'limit_near', severity: 'warning', message: 'queue download at 84% of its limit' }],
    });
    N('cctv', 'Hikvision-Gate', 'device', 'dhcp', 0.92, 0.34, 'camera', 'critical', 'ap2', traffic(4e6, 2.4e6), {
      queue: { name: 'cctv', max_down_human: '4.00 Mbps', max_up_human: '2.00 Mbps', limit_state: 'reached' },
      reasons: [{ kind: 'limit_reached', severity: 'critical', message: 'queue upload at 120% of its limit - limit reached' }],
    });
    N('tv', 'LivingRoom-TV', 'device', 'dhcp', 0.76, 0.38, 'tv', 'ok', 'ap2', traffic(5e6 * wob(4, 7), 1e5));
    N('if_ether3', 'ether3', 'interface', 'ether', 0.43, 0.72, 'port', 'ok', 'node_router', traffic(1e6, 3e5));
    N('pc', 'DESKTOP-OFFICE', 'device', 'dhcp', 0.58, 0.78, 'desktop', 'ok', 'if_ether3', traffic(1e6 * wob(3, 8), 3e5));
    N('user1', 'hotspot-user', 'device', 'hotspot', 0.74, 0.88, 'hotspot-user', 'idle', 'if_ether3', null, { status: 'idle' });
    const sub = { ap1: [3, 1, 2], ap2: [2, 0, 1] };
    for (const n of nodes) if (sub[n.id]) n.subtree = { total: sub[n.id][0], warning: sub[n.id][1], critical: sub[n.id][2] };
    E('edge_internet_wan', 'node_internet', 'if_ether1', 'ok', wd, wu);
    E('edge_router_ether1', 'node_router', 'if_ether1', 'ok', wd, wu, [], 'straight', 'wan', true);
    E('edge_router_ether5', 'node_router', 'if_ether5', 'ok', apd + 2e6, apu, [{ x: 0.36, y: 0.3 }], 'orthogonal');
    E('edge_if5_ap1', 'if_ether5', 'ap1', 'ok', apd, apu, [{ x: 0.43, y: 0.2 }]);
    E('edge_ap1_ap2', 'ap1', 'ap2', 'ok', 3e6, 4e5, [{ x: 0.64, y: 0.08 }, { x: 0.74, y: 0.08 }], 'smooth', 'relay');
    E('edge_ap1_phone', 'ap1', 'phone', 'ok', 2e6, 2e5);
    E('edge_ap1_laptop', 'ap1', 'laptop', 'warning', 8.4e6, 6e5);
    E('edge_ap2_cctv', 'ap2', 'cctv', 'critical', 4e6, 2.4e6, [{ x: 0.92, y: 0.16 }], 'orthogonal');
    E('edge_ap2_tv', 'ap2', 'tv', 'ok', 5e6 * wob(4, 7), 1e5);
    E('edge_router_ether3', 'node_router', 'if_ether3', 'ok', 1e6, 3e5, [{ x: 0.36, y: 0.72 }], 'orthogonal');
    E('edge_if3_pc', 'if_ether3', 'pc', 'ok', 1e6, 3e5);
    E('edge_if3_user1', 'if_ether3', 'user1', 'idle', 0, 0);
    const alerts = [
      { id: 'cctv:limit_reached', severity: 'critical', kind: 'limit_reached', node_id: 'cctv', label: 'Hikvision-Gate', message: 'queue upload at 120% of its limit - limit reached' },
      { id: 'laptop:limit_near', severity: 'warning', kind: 'limit_near', node_id: 'laptop', label: 'ThinkPad', message: 'queue download at 84% of its limit' },
    ];
    return {
      success: true, rate_ready: true, nodes, edges, alerts, warnings: [],
      layout: { id: 'demo', name: 'Demo house (sample data)', version: 1, background: { kind: 'none', url: null, aspect_ratio: 16 / 9, opacity: 1, has_upload: false } },
      thresholds: { warn_pct: 80, crit_pct: 95, high_pps: 50000, high_bps: null },
      summary: { nodes: nodes.length, edges: edges.length, health: { ok: 8, idle: 1, warning: 1, critical: 2 }, alerts: { critical: 1, warning: 1 }, wan: { down_human: human(wd), up_human: human(wu) } },
    };
  }

  customElements.define('mikrotik-map', MikrotikMap);
  window.MikrotikMap = MikrotikMap;
})();

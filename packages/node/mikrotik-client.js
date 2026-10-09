/**
 * MikroTik Universal Rust Engine - Node.js / Express Client SDK
 * 
 * Helper client untuk Node.js / Express layaknya plugin npm native.
 */
class MikrotikClient {
  constructor(options = {}) {
    this.gatewayUrl = options.gatewayUrl || process.env.MIKROTIK_GATEWAY_URL || 'http://127.0.0.1:8080';
    this.token = options.token || process.env.MIKROTIK_GATEWAY_TOKEN || 'change-me-to-a-long-random-string';
    this.router = {
      host: options.host || process.env.MIKROTIK_ROUTER_HOST || '192.168.88.1',
      port: (options.port || process.env.MIKROTIK_ROUTER_PORT || 8728).toString(),
      user: options.user || process.env.MIKROTIK_ROUTER_USER || 'admin',
      password: options.password || process.env.MIKROTIK_ROUTER_PASS || ''
    };
  }

  async post(endpoint, payload = {}) {
    const res = await fetch(`${this.gatewayUrl}${endpoint}`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.token}`,
        'X-Router-Host': this.router.host,
        'X-Router-Port': this.router.port,
        'X-Router-User': this.router.user,
        'X-Router-Pass': this.router.password,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(payload)
    });

    if (!res.ok) {
      const errText = await res.text();
      throw new Error(`MikroTik Gateway Error [${res.status}]: ${errText}`);
    }

    return await res.json();
  }

  // ⚡ Fast-Path Overview
  getOverview() {
    return this.post('/api/v1/overview');
  }

  getConnectedDevices() {
    return this.post('/api/v1/network/connected-devices');
  }

  // 🔍 Queue & Bandwidth Inspection
  inspectUser(queryIpOrUser) {
    return this.post('/api/v1/queues/inspect-user', { query: queryIpOrUser });
  }

  getBandwidthSummary() {
    return this.post('/api/v1/queues/overview-summary');
  }

  // ⚖️ PCC Load Balancing
  setupLoadBalance(lanInterface, wans, matcher = 'both-addresses') {
    return this.post('/api/v1/load-balance/pcc/setup', {
      lan_interface: lanInterface,
      wans,
      matcher,
      auto_failover: true,
      add_masquerade: true
    });
  }

  getLoadBalanceStatus() {
    return this.post('/api/v1/load-balance/status');
  }

  // 🚫 Security & Protection
  enableAntiTethering(hotspotInterface = 'bridge') {
    return this.post('/api/v1/security/anti-tethering/enable', {
      hotspot_interface: hotspotInterface,
      enforce_shared_users_one: true
    });
  }

  blockApp(appType, customDomain = null) {
    return this.post('/api/v1/security/app-block', { app_type: appType, custom_domain: customDomain });
  }

  // 🧙‍♂️ Hotspot & Vouchers
  generateVouchers(qty, prefix = 'V-', profile = 'default') {
    return this.post('/api/v1/hotspot/generate-batch', { qty, prefix, profile });
  }

  kickHotspotUser(activeId) {
    return this.post('/api/v1/hotspot/kick', { id: activeId });
  }
}

module.exports = MikrotikClient;

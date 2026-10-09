import { ref } from 'vue';

/**
 * MikroTik Universal Rust Engine - Vue 3 / Nuxt Composable
 * 
 * Composable siap pakai untuk Vue layaknya plugin UI native.
 */
export function useMikrotik(config = {}) {
  const gatewayUrl = config.gatewayUrl || 'http://127.0.0.1:8080';
  const token = config.token || 'change-me-to-a-long-random-string';
  const router = config.router || {
    host: '192.168.88.1',
    port: '8728',
    user: 'admin',
    password: ''
  };

  const loading = ref(false);
  const error = ref(null);

  async function apiPost(endpoint, payload = {}) {
    loading.value = true;
    error.value = null;
    try {
      const res = await fetch(`${gatewayUrl}${endpoint}`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'X-Router-Host': router.host,
          'X-Router-Port': (router.port || 8728).toString(),
          'X-Router-User': router.user,
          'X-Router-Pass': router.password || '',
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(payload)
      });
      const data = await res.json();
      if (!res.ok) {
        throw new Error(data.error || 'Request failed');
      }
      return data;
    } catch (err) {
      error.value = err.message;
      throw err;
    } finally {
      loading.value = false;
    }
  }

  return {
    loading,
    error,
    // Methods
    getOverview: () => apiPost('/api/v1/overview'),
    getConnectedDevices: () => apiPost('/api/v1/network/connected-devices'),
    inspectUser: (query) => apiPost('/api/v1/queues/inspect-user', { query }),
    getBandwidthSummary: () => apiPost('/api/v1/queues/overview-summary'),
    generateVouchers: (qty, prefix, profile) => apiPost('/api/v1/hotspot/generate-batch', { qty, prefix, profile }),
    kickUser: (id) => apiPost('/api/v1/hotspot/kick', { id }),
    ping: (address, count = 3) => apiPost('/api/v1/tools/ping', { address, count })
  };
}

# Panduan Integrasi Multi-Stack (Laravel, Node.js, Vue 3)

Gateway Rust menyediakan interface HTTP/JSON standar dan SSE (Server-Sent Events), sehingga developer di framework apa pun tidak perlu library khusus MikroTik. Cukup HTTP client bawaan masing-masing framework.

---

## 1. Integrasi dengan Laravel (PHP)

Di Laravel, Anda cukup memanfaatkan `Http` client bawaan (`Guzzle`):

### Service Helper: `app/Services/MikrotikService.php`
```php
<?php

namespace App\Services;

use Illuminate\Support\Facades\Http;
use Exception;

class MikrotikService
{
    protected string $baseUrl;
    protected string $token;

    public function __construct()
    {
        $this->baseUrl = config('services.mikrotik.gateway_url', 'http://127.0.0.1:8080');
        $this->token = config('services.mikrotik.token', 'change-me-to-a-long-random-string');
    }

    /**
     * Eksekusi perintah command ke router tertentu
     */
    public function command(string $routerId, string $command, array $args = []): array
    {
        $response = Http::withToken($this->token)
            ->post("{$this->baseUrl}/routers/{$routerId}/command", [
                'command' => $command,
                'args' => (object) $args,
            ]);

        if ($response->failed()) {
            throw new Exception("MikroTik Error: " . $response->json('error', 'Unknown Error'));
        }

        return $response->json();
    }

    // Contoh: Ambil daftar Hotspot User
    public function getHotspotUsers(string $routerId = 'main'): array
    {
        return $this->command($routerId, '/ip/hotspot/user/print');
    }

    // Contoh: Generate Voucher Hotspot Baru
    public function addHotspotUser(string $routerId, string $username, string $password, string $profile): array
    {
        return $this->command($routerId, '/ip/hotspot/user/add', [
            'name' => $username,
            'password' => $password,
            'profile' => $profile,
        ]);
    }
}
```

### Penggunaan di Controller Laravel:
```php
public function index(MikrotikService $mt)
{
    $users = $mt->getHotspotUsers('main');
    return response()->json([
        'total' => count($users),
        'data' => $users,
    ]);
}
```

---

## 2. Integrasi dengan Node.js / TypeScript / Express

Anda dapat menggunakan `fetch` (native di Node 18+) atau `axios`:

### Helper Client: `mikrotikClient.ts`
```typescript
const GATEWAY_URL = process.env.MIKROTIK_GATEWAY_URL || "http://127.0.0.1:8080";
const GATEWAY_TOKEN = process.env.MIKROTIK_GATEWAY_TOKEN || "change-me-to-a-long-random-string";

export async function runCommand<T = any>(
  routerId: string,
  command: string,
  args: Record<string, any> = {}
): Promise<T[]> {
  const res = await fetch(`${GATEWAY_URL}/routers/${routerId}/command`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${GATEWAY_TOKEN}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ command, args }),
  });

  const body = await res.json();
  if (!res.ok) {
    throw new Error(`[MikroTik Gateway Error] ${body.error || res.statusText}`);
  }

  return body as T[];
}

// Contoh Penggunaan:
async function getInterfaces() {
  const ifaces = await runCommand("main", "/interface/print", {
    ".proplist": ".id,name,type,running",
  });
  console.log("Daftar Interface:", ifaces);
}
```

---

## 3. Integrasi Frontend Real-Time dengan Vue 3 (Chart / Live Traffic)

Karena Gateway Rust menyediakan endpoint Server-Sent Events (SSE), Vue 3 dapat menerima streaming data traffic per detik secara langsung menggunakan API bawaan browser:

### Komponen Vue: `LiveTraffic.vue`
```vue
<template>
  <div class="traffic-card">
    <h3>Live Traffic: {{ interfaceName }}</h3>
    <div class="stats">
      <p>Download (Rx): <strong>{{ formatBps(rxBps) }}</strong></p>
      <p>Upload (Tx): <strong>{{ formatBps(txBps) }}</strong></p>
    </div>
    <button @click="toggleMonitoring">
      {{ isStreaming ? 'Stop Streaming' : 'Start Streaming' }}
    </button>
  </div>
</template>

<script setup>
import { ref, onUnmounted } from 'vue';

const interfaceName = ref('ether1');
const rxBps = ref(0);
const txBps = ref(0);
const isStreaming = ref(false);
let eventSource = null;

function formatBps(bits) {
  const n = Number(bits || 0);
  if (n > 1000000) return (n / 1000000).toFixed(2) + ' Mbps';
  if (n > 1000) return (n / 1000).toFixed(2) + ' Kbps';
  return n + ' bps';
}

function startMonitoring() {
  const url = `http://127.0.0.1:8080/routers/main/listen?command=/interface/monitor-traffic&interface=${interfaceName.value}`;
  
  // Membuka koneksi SSE ke Gateway Rust
  eventSource = new EventSource(url);
  isStreaming.value = true;

  eventSource.onmessage = (event) => {
    const data = JSON.parse(event.data);
    rxBps.value = data['rx-bits-per-second'] || 0;
    txBps.value = data['tx-bits-per-second'] || 0;
  };

  eventSource.onerror = (err) => {
    console.error("SSE stream error", err);
    stopMonitoring();
  };
}

function stopMonitoring() {
  if (eventSource) {
    // Menutup koneksi di sisi browser.
    // Core Rust otomatis mendeteksi socket disconnect dan mengirim /cancel ke router!
    eventSource.close();
    eventSource = null;
  }
  isStreaming.value = false;
}

function toggleMonitoring() {
  if (isStreaming.value) stopMonitoring();
  else startMonitoring();
}

onUnmounted(() => {
  stopMonitoring();
});
</script>

<style scoped>
.traffic-card {
  padding: 1.5rem;
  border-radius: 8px;
  background: #1e1e24;
  color: #fff;
  font-family: sans-serif;
}
.stats {
  display: flex;
  gap: 2rem;
  margin: 1rem 0;
}
</style>
```

> **Catatan**: Jika SSE dipanggil langsung dari browser lintas domain, tambahkan CORS layer pada Axum di `routeros-gateway`, atau arahkan melalui reverse proxy (Nginx / Caddy / Laravel API proxy).

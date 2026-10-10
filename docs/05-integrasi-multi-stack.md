# Panduan Integrasi Multi-Stack (Web, Mobile, & Backend)

Karena Core Engine dibangun menggunakan **Rust** dan mengekspos **Universal REST & SSE API**, developer dengan stack bahasa apa pun (JavaScript, Python, Dart/Flutter, PHP, Go) dapat berinteraksi dengan MikroTik menggunakan variabel dan skema JSON yang seragam.

---

## 1. Integrasi Frontend / Node.js (JavaScript & TypeScript)

```typescript
// Konfigurasi Variabel Router
const config = {
  gatewayUrl: "http://127.0.0.1:8080",
  token: "change-me-to-a-long-random-string",
  router: {
    host: "192.168.88.1",
    port: 8728,
    user: "admin",
    password: "password_anda"
  }
};

// 1. Ambil Resource Router (CPU / RAM / Uptime)
async function getSystemResource() {
  const res = await fetch(`${config.gatewayUrl}/api/v1/system/resource`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${config.token}`,
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ router: config.router })
  });
  const data = await res.json();
  console.log("Resource:", data.data);
}

// 2. Buat Voucher Hotspot Baru
async function createVoucher(username: string, pass: string, profile: string) {
  const res = await fetch(`${config.gatewayUrl}/api/v1/hotspot/user/create`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${config.token}`,
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      router: config.router,
      name: username,
      password: pass,
      profile: profile,
      timelimit: "3h"
    })
  });
  return await res.json();
}
```

---

## 2. Integrasi Mobile Flutter / Dart (Android & iOS)

```dart
import 'dart:convert';
import 'package:http/http.dart' as http;

class MikrotikApiService {
  final String gatewayUrl = 'http://10.0.2.2:8080'; // emulator localhost
  final String token = 'change-me-to-a-long-random-string';
  
  final Map<String, dynamic> router = {
    'host': '192.168.88.1',
    'port': 8728,
    'user': 'admin',
    'password': 'password_anda',
  };

  Future<Map<String, dynamic>> getHotspotUsers() async {
    final response = await http.post(
      Uri.parse('$gatewayUrl/api/v1/hotspot/users'),
      headers: {
        'Authorization': 'Bearer $token',
        'Content-Type': 'application/json',
      },
      body: jsonEncode({'router': router}),
    );

    if (response.statusCode == 200) {
      return jsonDecode(response.body);
    } else {
      throw Exception('Gagal memuat user: ${response.body}');
    }
  }

  Future<void> kickUser(String activeId) async {
    await http.post(
      Uri.parse('$gatewayUrl/api/v1/hotspot/kick'),
      headers: {
        'Authorization': 'Bearer $token',
        'Content-Type': 'application/json',
      },
      body: jsonEncode({
        'router': router,
        'id': activeId,
      }),
    );
  }
}
```

---

## 3. Integrasi Python (FastAPI / Django / Flask / AI Script)

```python
import requests

GATEWAY_URL = "http://127.0.0.1:8080"
TOKEN = "change-me-to-a-long-random-string"

HEADERS = {
    "Authorization": f"Bearer {TOKEN}",
    "Content-Type": "application/json"
}

ROUTER = {
    "host": "192.168.88.1",
    "port": 8728,
    "user": "admin",
    "password": "password_anda"
}

# 1. Cek User Hotspot yang Sedang Online
def get_active_hotspot():
    res = requests.post(
        f"{GATEWAY_URL}/api/v1/hotspot/active",
        headers=HEADERS,
        json={"router": ROUTER}
    )
    return res.json().get("data", [])

# 2. Blokir IP Otomatis jika Terdeteksi Brute Force
def block_ip(ip_address: str):
    res = requests.post(
        f"{GATEWAY_URL}/api/v1/firewall/block-ip",
        headers=HEADERS,
        json={
            "router": ROUTER,
            "address": ip_address,
            "list": "BLACKLIST_AI",
            "timeout": "1d"
        }
    )
    return res.json()
```

---

## 4. Integrasi PHP / Laravel

```php
<?php

use Illuminate\Support\Facades\Http;

$router = [
    'host'     => '192.168.88.1',
    'port'     => 8728,
    'user'     => 'admin',
    'password' => 'password_anda',
];

$gatewayUrl = 'http://127.0.0.1:8080';
$token = 'change-me-to-a-long-random-string';

// Ambil Daftar IPv6
$response = Http::withToken($token)
    ->post("{$gatewayUrl}/api/v1/ipv6/addresses", [
        'router' => $router,
    ]);

$ipv6List = $response->json('data');

// Tambah Voucher Hotspot
Http::withToken($token)->post("{$gatewayUrl}/api/v1/hotspot/user/create", [
    'router'    => $router,
    'name'      => 'member_vip',
    'password'  => '123456',
    'profile'   => 'default',
    'timelimit' => '2h',
]);
```

---

## 5. Integrasi Go (Golang)

```go
package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
)

func main() {
	payload := map[string]interface{}{
		"router": map[string]interface{}{
			"host":     "192.168.88.1",
			"port":     8728,
			"user":     "admin",
			"password": "password_anda",
		},
	}
	body, _ := json.Marshal(payload)

	req, _ := http.NewRequest("POST", "http://127.0.0.1:8080/api/v1/system/resource", bytes.NewBuffer(body))
	req.Header.Set("Authorization", "Bearer change-me-to-a-long-random-string")
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		panic(err)
	}
	defer resp.Body.Close()
	fmt.Println("Status:", resp.Status)
}
```

---

## 6. Live Streaming Traffic di Browser / Mobile (SSE)

Browser Web atau aplikasi Mobile cukup menghubungkan ke URL endpoint stream SSE:

```javascript
const url = "http://127.0.0.1:8080/api/v1/interfaces/stream?host=192.168.88.1&port=8728&user=admin&password=password_anda&interface=ether1";

const eventSource = new EventSource(url);

eventSource.onmessage = (event) => {
  const stats = JSON.parse(event.data);
  console.log("Download (Rx bps):", stats["rx-bits-per-second"]);
  console.log("Upload (Tx bps):", stats["tx-bits-per-second"]);
};

// Menutup stream:
// eventSource.close(); -> Rust Gateway otomatis mengirim sinyal /cancel ke MikroTik!
```

---

## 7. Integrasi Vue 3 / Nuxt 3 (WebSocket Real-Time Dashboard)

Contoh komponen Vue 3 reactive yang memanfaatkan **Full-Duplex WebSocket** (`/ws`) untuk memantau traffic live tanpa HTTP polling, serta mengambil Fast-Path Overview dalam 2ms:

```vue
<template>
  <div class="dashboard">
    <h1>MikroTik Realtime Monitor</h1>
    <div class="stats-card">
      <p>CPU Load: {{ overview.cpu_load }}%</p>
      <p>Free Memory: {{ overview.free_memory_mb }} MB</p>
      <p>Hotspot Online: {{ overview.hotspot_active_count }}</p>
    </div>

    <div class="traffic-box">
      <h3>Live Traffic (ether1):</h3>
      <p>⬇️ Rx: {{ (rxBps / 1024 / 1024).toFixed(2) }} Mbps</p>
      <p>⬆️ Tx: {{ (txBps / 1024 / 1024).toFixed(2) }} Mbps</p>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';

const rxBps = ref(0);
const txBps = ref(0);
const overview = ref({});
let socket = null;

const routerTarget = {
  host: "192.168.88.1",
  port: 8728,
  user: "admin",
  password: "password_anda"
};

onMounted(() => {
  // 1. Hubungkan ke Full-Duplex WebSocket Rust Gateway
  socket = new WebSocket("ws://127.0.0.1:8080/ws?token=change-me-to-a-long-random-string");

  socket.onopen = () => {
    // Tarik snapshot overview pertama kali
    socket.send(JSON.stringify({
      action: "overview",
      router: routerTarget,
      tag: "init-overview"
    }));

    // Subscribe live traffic streaming interface
    socket.send(JSON.stringify({
      action: "subscribe_traffic",
      router: routerTarget,
      interface: "ether1",
      tag: "eth1-stream"
    }));
  };

  socket.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    if (msg.event === "traffic_frame") {
      rxBps.value = parseInt(msg.data["rx-bits-per-second"] || 0);
      txBps.value = parseInt(msg.data["tx-bits-per-second"] || 0);
    } else if (msg.event === "overview_result") {
      overview.value = msg.data;
    }
  };
});

onUnmounted(() => {
  if (socket) {
    socket.send(JSON.stringify({ action: "unsubscribe", tag: "eth1-stream" }));
    socket.close();
  }
});
</script>
```

---

## 8. Integrasi Express.js / Node.js (Full-Duplex Proxy Daemon)

```javascript
import express from 'express';
import WebSocket from 'ws';

const app = express();
app.use(express.json());

const GATEWAY_HTTP = 'http://127.0.0.1:8080';
const GATEWAY_WS = 'ws://127.0.0.1:8080/ws?token=change-me-to-a-long-random-string';
const GATEWAY_TOKEN = 'change-me-to-a-long-random-string';

// Route: Fast Overview Snapshot
app.get('/api/router/overview', async (req, res) => {
  try {
    const response = await fetch(`${GATEWAY_HTTP}/api/v1/overview`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${GATEWAY_TOKEN}`,
        'X-Router-Host': '192.168.88.1',
        'X-Router-Port': '8728',
        'X-Router-User': 'admin',
        'X-Router-Pass': 'password_anda',
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({})
    });
    const data = await response.json();
    res.json(data);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// Route: Stream Log Realtime via WebSocket ke Node.js
const wsClient = new WebSocket(GATEWAY_WS);
wsClient.on('open', () => {
  console.log("Terhubung ke Rust Gateway WebSocket");
  wsClient.send(JSON.stringify({
    action: "listen_logs",
    router: { host: "192.168.88.1", port: 8728, user: "admin", password: "password_anda" },
    tag: "log-daemon",
    params: { topic: "critical" }
  }));
});

wsClient.on('message', (data) => {
  const log = JSON.parse(data);
  if (log.event === "log_entry") {
    console.warn("⚠️ MikroTik Critical Alert:", log.data.message);
  }
});

app.listen(3000, () => console.log("Express running on port 3000"));
```

---

## 9. Integrasi Laravel 11 (Otomatisasi Isolir Tagihan Pelanggan PPPoE)

Pada aplikasi Billing ISP / RT-RW Net berbasis Laravel, isolir pelanggan yang menunggak dapat dilakukan dalam 1 pemanggilan API bersih:

```php
namespace App\Services;

use Illuminate\Support\Facades\Http;

class MikrotikBillingService
{
    protected string $gateway = 'http://127.0.0.1:8080';
    protected string $token = 'change-me-to-a-long-random-string';

    protected array $router = [
        'host'     => '192.168.88.1',
        'port'     => 8728,
        'user'     => 'admin',
        'password' => 'password_anda',
    ];

    /**
     * Isolir pelanggan PPPoE yang menunggak
     * Mengubah profile secret ke 'ISOLIR' dan otomatis menendang (kick) sesi online-nya.
     */
    public function isolateOverdueCustomer(string $username): array
    {
        $response = Http::withToken($this->token)
            ->post("{$this->gateway}/api/v1/ppp/customer/isolate", [
                'router'          => $this->router,
                'username'        => $username,
                'isolate_profile' => 'ISOLIR',
                'comment'         => 'Menunggak Tagihan Bulan Ini',
            ]);

        return $response->json();
    }

    /**
     * Buka kembali isolir setelah pelanggan membayar lunas
     */
    public function restorePaidCustomer(string $username, string $profile = 'Paket-20Mbps'): array
    {
        $response = Http::withToken($this->token)
            ->post("{$this->gateway}/api/v1/ppp/customer/restore", [
                'router'         => $this->router,
                'username'       => $username,
                'active_profile' => $profile,
                'comment'        => 'Lunas',
            ]);

        return $response->json();
    }
}
```


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
    host: "ath.vpnbersama.us",
    port: 51121,
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
    'host': 'ath.vpnbersama.us',
    'port': 51121,
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
    "host": "ath.vpnbersama.us",
    "port": 51121,
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
    'host'     => 'ath.vpnbersama.us',
    'port'     => 51121,
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
			"host":     "ath.vpnbersama.us",
			"port":     51121,
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
const url = "http://127.0.0.1:8080/api/v1/interfaces/stream?host=ath.vpnbersama.us&port=51121&user=admin&password=password_anda&interface=ether1";

const eventSource = new EventSource(url);

eventSource.onmessage = (event) => {
  const stats = JSON.parse(event.data);
  console.log("Download (Rx bps):", stats["rx-bits-per-second"]);
  console.log("Upload (Tx bps):", stats["tx-bits-per-second"]);
};

// Menutup stream:
// eventSource.close(); -> Rust Gateway otomatis mengirim sinyal /cancel ke MikroTik!
```

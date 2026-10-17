# Spesifikasi Universal Gateway API (REST, WebSocket & SSE)

Gateway daemon (`routeros-gateway`) berjalan sebagai HTTP REST, WebSocket & SSE service berkinerja tinggi pada port yang dikonfigurasi (default: `http://127.0.0.1:8080`).

---

## 1. Autentikasi Gateway

Semua endpoint dilindungi oleh autentikasi Bearer Token atau Query Token (kecuali `/health`, `/topology`, `/map`, `/sdk/*`, dan portal dokumentasi `/docs` & `/` yang hanya menyajikan halaman/skrip statis tanpa data jaringan):

### Cara A: Header RFC Standar
```http
Authorization: Bearer <API_TOKEN_ANDA>
```

### Cara B: URL Query Parameter
```http
GET /api/v1/overview?token=<API_TOKEN_ANDA>
```

Untuk WebSocket (`/ws`), autentikasi dapat dilewatkan via URL parameter `?token=<API_TOKEN_ANDA>` atau frame JSON pertama: `{"action": "auth", "token": "..."}`.

### Peran token: Admin vs Viewer
| Token | Env / config | Boleh |
|---|---|---|
| **Admin** | `GATEWAY_TOKEN` / `api_token` | Semua endpoint (termasuk `/api/v1/command`) |
| **Viewer** (opsional) | `GATEWAY_VIEWER_TOKEN` / `viewer_token` (min. 16 karakter, tidak boleh sama dengan admin) | Hanya `GET` `/api/v1/map/scene`, `/icons`, `/layouts[/…]` dan WebSocket `subscribe_map`/`unsubscribe`. Selain itu → `403 FORBIDDEN`. Header `X-Router-*` dari viewer diabaikan (hanya `router_id` terdaftar). |

Gunakan viewer token untuk frontend/embed agar admin token tidak pernah sampai ke browser pengguna akhir. Greeting WebSocket `auth_ok` menyertakan field `role`.

---

## 2. Fleksibilitas Transport & Penentuan Router Target

Engine mendukung **Dual HTTP Transport (GET & POST)** di seluruh 115+ endpoint enterprise. Target router MikroTik dapat dispesifikasikan melalui salah satu mekanisme berikut:

### Opsi 1: HTTP Headers (Format Bersih untuk Microservice & Mobile)
```http
X-Router-Host: 192.168.88.1
X-Router-Port: 8728
X-Router-User: admin
X-Router-Pass: secretpassword
```

### Opsi 2: URL Query Parameters (Direkomendasikan untuk NOC Triage, cURL & Browser)
Sistem secara otomatis menghidrasi parameter router dari query string pada request **GET** maupun **POST**:
```bash
# Contoh cURL GET langsung
curl "http://127.0.0.1:8080/api/v1/overview?host=192.168.88.1&port=8728&user=admin&pass=secretpassword&token=mytoken"

# Contoh cURL POST dengan query params
curl -X POST "http://127.0.0.1:8080/api/v1/queues/inspect-user?query=192.168.88.50&host=192.168.88.1&port=8728&user=admin&pass=secretpassword&token=mytoken"
```

### Opsi 3: JSON Body (Standar REST POST)
```json
{
  "router": {
    "host": "192.168.88.1",
    "port": 8728,
    "user": "admin",
    "password": "secretpassword"
  }
}
```

### Opsi 4: Pre-configured Router ID / Default Fallback
Jika tidak ada kredensial yang disertakan, gateway otomatis menggunakan profil router default dari `config.toml`:
```json
{
  "router_id": "main"
}
```

---

## 3. Fast-Path Overview Snapshot (`GET` / `POST` `/api/v1/overview`)

Endpoint agregasi tercepat untuk dashboard NOC, monitoring Mikhmon, dan ISP Billing.
Mengirimkan 6 perintah MikroTik secara simultan (`tokio::join!`) melalui socket persisten yang sama, mengembalikan ringkasan lengkap dalam **< 1.5 milidetik**:

### Response Format:
```json
{
  "success": true,
  "execution_time_ms": 1.2,
  "data": {
    "identity": "MikroTik-Main-GW",
    "system": {
      "cpu_load_percent": 3,
      "free_memory_mb": 218,
      "total_memory_mb": 256,
      "memory_used_percent": "14.8%",
      "uptime": "2w3d14h20m10s",
      "version": "7.15.2 (stable)",
      "board_name": "RB4011iGS+5HacQ2HnD",
      "architecture": "arm",
      "cpu_count": "4",
      "cpu_frequency_mhz": "1400"
    },
    "routerboard": {
      "model": "RB4011iGS+5HacQ2HnD",
      "serial_number": "HE608F7XXXX",
      "current_firmware": "7.15.2",
      "upgrade_firmware": "7.15.2"
    },
    "hotspot": {
      "active_total": 42,
      "active_recent": [ ... ]
    },
    "ppp": {
      "active_total": 18,
      "active_recent": [ ... ]
    },
    "interfaces": {
      "total": 11,
      "running": 6,
      "list": [ ... ]
    },
    "health_status": {
      "cpu": "healthy",
      "memory_status": "healthy"
    }
  }
}
```

---

## 4. Full-Duplex WebSocket Engine (`GET /ws`)

Untuk aplikasi modern yang membutuhkan koneksi dua arah tanpa overhead polling HTTP.

### Inisialisasi Koneksi
```javascript
const ws = new WebSocket("ws://127.0.0.1:8080/ws?token=API_TOKEN_ANDA");
```

### Action 1: Subscribe Real-Time Traffic Monitor
Kirim dari client:
```json
{
  "action": "subscribe_traffic",
  "router": { "host": "192.168.88.1", "port": 8728, "user": "admin", "password": "" },
  "interface": "ether1",
  "tag": "traffic-eth1"
}
```
Gateway streaming data per detik:
```json
{
  "event": "traffic_frame",
  "tag": "traffic-eth1",
  "interface": "ether1",
  "data": {
    "rx-bits-per-second": "128450",
    "tx-bits-per-second": "45210",
    "rx-packets-per-second": "145",
    "tx-packets-per-second": "68"
  }
}
```

### Action 2: Unsubscribe Traffic
```json
{
  "action": "unsubscribe",
  "tag": "traffic-eth1"
}
```

### Action 3: Eksekusi Command Instan via WebSocket
Kirim dari client:
```json
{
  "action": "command",
  "router": { "host": "192.168.88.1", "port": 8728, "user": "admin", "password": "" },
  "command": "/ip/address/print",
  "tag": "req-99"
}
```
Gateway mengembalikan hasil:
```json
{
  "event": "command_result",
  "tag": "req-99",
  "success": true,
  "count": 2,
  "data": [
    { ".id": "*1", "address": "192.168.88.1/24", "interface": "bridge" }
  ]
}
```

### Action 4: Subscribe Live Overview Telemetry (Continuous Dashboard Stream)
Streaming telemetri CPU, RAM, Uptime, Klien Hotspot, PPPoE, dan DHCP leases setiap interval detik (ideal untuk Dashboard React/Vue):
```json
{
  "action": "subscribe_overview",
  "interval_ms": 2000,
  "tag": "dashboard-telemetry"
}
```
Gateway streaming frame berkala:
```json
{
  "event": "overview_frame",
  "tag": "dashboard-telemetry",
  "timestamp": 1773294800000,
  "data": {
    "identity": "MikroTik-Main",
    "cpu_load": "4",
    "cpu_frequency_mhz": "880",
    "free_memory": "195420160",
    "total_memory": "268435456",
    "uptime": "3w1d04:22:10",
    "hotspot_active_count": 28,
    "ppp_active_count": 14,
    "dhcp_lease_count": 52,
    "total_clients_count": 94
  }
}
```

### Action 5: Subscribe Live Ping Matrix
Streaming hasil ping real-time per detik (mirip utilitas Ping di Winbox):
```json
{
  "action": "subscribe_ping",
  "address": "8.8.8.8",
  "interval_ms": 1000,
  "tag": "ping-google"
}
```
Gateway mengalirkan balasan ICMP:
```json
{
  "event": "ping_frame",
  "tag": "ping-google",
  "target": "8.8.8.8",
  "sequence": 1,
  "rtt": "12ms",
  "data": { "sent": "1", "received": "1", "time": "12ms", "status": "echo reply" }
}
```

### Action 6: Listen RouterOS Logs Real-time
Streaming log RouterOS saat kejadian berlangsung (`/log/print follow-only`):
```json
{
  "action": "listen_logs",
  "params": { "topic": "hotspot,info" },
  "tag": "hotspot-logs"
}
```

### Action 7: Subscribe Network Map (`subscribe_map`)
Stream *scene* peta (node, edge, status, rate, alert) — dipakai oleh `<mikrotik-map>`. Interval dibatasi 1000–60000 ms; masuk hitungan maksimal 10 stream per koneksi:
```json
{ "action": "subscribe_map", "layout_id": "lantai-1", "router_id": "core", "interval_ms": 3000, "tag": "map" }
```
Server mengirim `{"event":"map_frame", ...scene}`; bila router gagal: `{"event":"stream_error","retrying":true,...}` dan stream tetap mencoba lagi. Batalkan dengan `{"action":"unsubscribe","tag":"map"}`. Lihat [07](07-network-map-visualizer.md).

### 💻 Contoh Implementasi WebSocket untuk Developer (Frontend & Mobile)

#### 1. JavaScript / TypeScript (Vue 3 / React / Next.js):
```javascript
const socket = new WebSocket("ws://127.0.0.1:8080/ws?token=change-me-to-a-long-random-string");

socket.onopen = () => {
  console.log("WebSocket terhubung!");
  // Subscribe live overview dashboard
  socket.send(JSON.stringify({
    action: "subscribe_overview",
    interval_ms: 2000,
    tag: "main-dashboard"
  }));
};

socket.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.event === "overview_frame") {
    console.log("CPU Load:", msg.data.cpu_load + "%");
    console.log("Klien Aktif:", msg.data.total_clients_count);
  }
};
```

#### 2. Flutter / Dart (Mobile Application):
```dart
import 'dart:convert';
import 'package:web_socket_channel/web_socket_channel.dart';

final channel = WebSocketChannel.connect(
  Uri.parse('ws://10.0.2.2:8080/ws?token=change-me-to-a-long-random-string'),
);

void startMonitoring() {
  channel.sink.add(jsonEncode({
    'action': 'subscribe_traffic',
    'interface': 'ether1',
    'tag': 'live-traffic',
  }));

  channel.stream.listen((message) {
    final frame = jsonDecode(message);
    if (frame['event'] == 'traffic_frame') {
      print("RX: ${frame['data']['rx-bits-per-second']} bps");
    }
  });
}
```


---

## 5. Format Error Terstandarisasi

| HTTP Status | Kode Error | Keterangan | Contoh Body |
|---|---|---|---|
| `400 Bad Request` | - | Argumen salah / format command tidak valid | `{"success": false, "error": "command must start with '/'"}` |
| `401 Unauthorized` | - | Token Bearer salah atau tidak ada | `{"success": false, "error": "unauthorized: invalid or missing Bearer token"}` |
| `404 Not Found` | - | Router ID tidak ditemukan di `config.toml` | `{"success": false, "error": "router 'x' not found in configuration"}` |
| `422 Unprocessable` | `ROUTER_TRAP` | MikroTik menolak perintah (`!trap`) | `{"success": false, "error": "already have such item", "code": "ROUTER_TRAP"}` |
| `502 Bad Gateway` | `AUTH_FAILED` | Login ke router gagal (User/Pass salah) | `{"success": false, "error": "router authentication failed: invalid username", "code": "AUTH_FAILED"}` |
| `502 Bad Gateway` | `CONNECTION_LOST` | Socket ke MikroTik putus/unreachable | `{"success": false, "error": "router connection dropped", "code": "CONNECTION_LOST"}` |
| `504 Gateway Timeout` | `TIMEOUT_GUARD` | Router hang / tidak merespons dalam 15s | `{"success": false, "error": "command execution timed out after 15s anti-hang guard"}` |

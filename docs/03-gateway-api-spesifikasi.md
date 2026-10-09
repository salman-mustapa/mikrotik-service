# Spesifikasi Universal Gateway API (REST, WebSocket & SSE)

Gateway daemon (`routeros-gateway`) berjalan sebagai HTTP REST, WebSocket & SSE service berkinerja tinggi pada port yang dikonfigurasi (default: `http://127.0.0.1:8080`).

---

## 1. Autentikasi Gateway

Semua endpoint dilindungi oleh Bearer Token (kecuali `/health` dan `/` playground):
```http
Authorization: Bearer <API_TOKEN_ANDA>
```
Nilai token dikonfigurasi pada file `config.toml` (field `api_token`).

Untuk WebSocket, autentikasi dapat dilewatkan via URL parameter `?token=<API_TOKEN_ANDA>` atau frame JSON pertama: `{"action": "auth", "token": "..."}`.

---

## 2. Penentuan Router Target

Setiap request dapat menentukan router MikroTik target melalui salah satu cara berikut:

### Opsi 1: HTTP Headers (Paling Bersih untuk REST & Mobile)
```http
X-Router-Host: ath.vpnbersama.us
X-Router-Port: 51121
X-Router-User: admin
X-Router-Pass: secretpassword
```

### Opsi 2: JSON Body
```json
{
  "router": {
    "host": "ath.vpnbersama.us",
    "port": 51121,
    "user": "admin",
    "password": "secretpassword"
  }
}
```

### Opsi 3: Pre-configured Router ID
```json
{
  "router_id": "main"
}
```

---

## 3. Fast-Path Overview Snapshot (`POST /api/v1/overview`)

Endpoint agregasi tercepat untuk dashboard monitoring, Mikhmon, dan ISP Billing.
Mengirimkan 6 perintah MikroTik secara simultan (`tokio::join!`) melalui socket persisten yang sama, mengembalikan ringkasan lengkap dalam **1-2 milidetik**:

### Response Format:
```json
{
  "success": true,
  "execution_time_ms": 2,
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

### Action 4: Instant Fast Overview via WebSocket
```json
{
  "action": "overview",
  "router": { "host": "192.168.88.1", "port": 8728, "user": "admin", "password": "" },
  "tag": "ov-1"
}
```

---

## 5. Batch Hotspot Voucher Generation (`POST /api/v1/hotspot/generate-batch`)

Membuat 10 hingga 1000 voucher hotspot secara bersamaan dalam satu request terpipelinisasi.

### Request Body:
```json
{
  "router": { "host": "192.168.88.1", "port": 8728, "user": "admin", "password": "" },
  "qty": 50,
  "prefix": "VIP-",
  "length": 6,
  "profile": "1-Jam-3k",
  "timelimit": "1h",
  "comment": "Batch-Oktober-01"
}
```

### Response:
```json
{
  "success": true,
  "total_requested": 50,
  "total_created": 50,
  "vouchers": [
    { "username": "VIP-a8k3n2", "password": "VIP-a8k3n2", "profile": "1-Jam-3k", "timelimit": "1h" },
    { "username": "VIP-m9p4x1", "password": "VIP-m9p4x1", "profile": "1-Jam-3k", "timelimit": "1h" }
  ]
}
```

---

## 6. Format Error Terstandarisasi

| HTTP Status | Kode Error | Keterangan | Contoh Body |
|---|---|---|---|
| `400 Bad Request` | - | Argumen salah / format command tidak valid | `{"success": false, "error": "command must start with '/'"}` |
| `401 Unauthorized` | - | Token Bearer salah atau tidak ada | `{"success": false, "error": "unauthorized: invalid or missing Bearer token"}` |
| `404 Not Found` | - | Router ID tidak ditemukan di `config.toml` | `{"success": false, "error": "router 'x' not found in configuration"}` |
| `422 Unprocessable` | `ROUTER_TRAP` | MikroTik menolak perintah (`!trap`) | `{"success": false, "error": "already have such item", "code": "ROUTER_TRAP"}` |
| `502 Bad Gateway` | `AUTH_FAILED` | Login ke router gagal (User/Pass salah) | `{"success": false, "error": "router authentication failed: invalid username", "code": "AUTH_FAILED"}` |
| `502 Bad Gateway` | `CONNECTION_LOST` | Socket ke MikroTik putus/unreachable | `{"success": false, "error": "router connection dropped", "code": "CONNECTION_LOST"}` |

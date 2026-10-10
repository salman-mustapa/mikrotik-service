# Spesifikasi Universal Gateway API (REST, WebSocket & SSE)

Gateway daemon (`routeros-gateway`) berjalan sebagai HTTP REST, WebSocket & SSE service berkinerja tinggi pada port yang dikonfigurasi (default: `http://127.0.0.1:8080`).

---

## 1. Autentikasi Gateway

Semua endpoint dilindungi oleh autentikasi Bearer Token atau Query Token (kecuali `/health`, `/topology`, dan portal dokumentasi `/docs` & `/`):

### Cara A: Header RFC Standar
```http
Authorization: Bearer <API_TOKEN_ANDA>
```

### Cara B: URL Query Parameter
```http
GET /api/v1/overview?token=<API_TOKEN_ANDA>
```

Untuk WebSocket (`/ws`), autentikasi dapat dilewatkan via URL parameter `?token=<API_TOKEN_ANDA>` atau frame JSON pertama: `{"action": "auth", "token": "..."}`.

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

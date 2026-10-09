# Spesifikasi Universal Gateway API (REST & SSE)

Gateway daemon (`routeros-gateway`) berjalan sebagai HTTP REST & SSE service lokal/private pada port yang dikonfigurasi (default: `http://127.0.0.1:8080`).

---

## 1. Autentikasi Gateway

Semua endpoint dilindungi oleh Bearer Token (kecuali `/health`):
```http
Authorization: Bearer <API_TOKEN_ANDA>
```
Nilai token dikonfigurasi pada file `config.toml` (field `api_token`).

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

## 3. Struktur Format Respons Standar

### Respons Sukses (`200 OK`)
```json
{
  "success": true,
  "count": 2,
  "data": [
    {
      ".id": "*1",
      "address": "192.168.88.1/24",
      "interface": "bridge",
      "network": "192.168.88.0"
    }
  ]
}
```

### Format Error Terstandarisasi
| HTTP Status | Kode Error | Keterangan | Contoh Body |
|---|---|---|---|
| `400 Bad Request` | - | Argumen salah / format command tidak valid | `{"success": false, "error": "command must start with '/'"}` |
| `401 Unauthorized` | - | Token Bearer salah atau tidak ada | `{"success": false, "error": "unauthorized: invalid or missing Bearer token"}` |
| `404 Not Found` | - | Router ID tidak ditemukan di `config.toml` | `{"success": false, "error": "router 'x' not found in configuration"}` |
| `422 Unprocessable` | `ROUTER_TRAP` | MikroTik menolak perintah (`!trap`) | `{"success": false, "error": "already have such item", "code": "ROUTER_TRAP"}` |
| `502 Bad Gateway` | `AUTH_FAILED` | Login ke router gagal (User/Pass salah) | `{"success": false, "error": "router authentication failed: invalid username", "code": "AUTH_FAILED"}` |
| `502 Bad Gateway` | `CONNECTION_LOST` | Socket ke MikroTik putus/unreachable | `{"success": false, "error": "router connection dropped", "code": "CONNECTION_LOST"}` |

---

## 4. Pola Realtime Traffic Streaming (SSE)

Untuk pemantauan traffic live per detik:
```http
GET /api/v1/interfaces/stream?host=ath.vpnbersama.us&port=51121&user=admin&password=secret&interface=ether1
Accept: text/event-stream
```

### Data Stream yang Diterima Klien:
```
data: {"rx-bits-per-second":"15280","tx-bits-per-second":"4210","rx-packets-per-second":"14","tx-packets-per-second":"7"}

data: {"rx-bits-per-second":"18400","tx-bits-per-second":"5100","rx-packets-per-second":"17","tx-packets-per-second":"9"}
```

Saat tab browser atau koneksi HTTP ditutup oleh klien, Rust Gateway secara otomatis mendeteksi pemutusan koneksi dan mengirim sinyal `/cancel` ke router MikroTik. Tidak ada kebocoran proses di router!

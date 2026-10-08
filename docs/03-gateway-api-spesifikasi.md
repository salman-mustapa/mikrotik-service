# Spesifikasi Gateway API (HTTP & SSE)

Gateway daemon (`routeros-gateway`) berjalan sebagai REST & SSE server lokal/private pada port yang dikonfigurasi (default: `http://127.0.0.1:8080`).

---

## 1. Autentikasi

Semua endpoint dilindungi oleh Bearer Token (kecuali `/health`):

```http
Authorization: Bearer <API_TOKEN_ANDA>
```
Nilai token dikonfigurasi pada file `config.toml` (field `api_token`).

---

## 2. Endpoint Overview

| Method | Path | Tipe | Fungsi |
|---|---|---|---|
| `GET` | `/health` | REST | Health check status daemon |
| `POST` | `/routers/:id/command` | REST | Eksekusi perintah (CRUD / print / set / add) |
| `GET` | `/routers/:id/listen` | SSE | Streaming data real-time (monitor-traffic, logs) |

---

## 3. Detail Endpoint

### A. Health Check
- **Path**: `GET /health`
- **Auth**: Tidak butuh
- **Response**: `200 OK`
```json
{
  "status": "ok"
}
```

---

### B. Eksekusi Perintah (Command Execution)
- **Path**: `POST /routers/:id/command`
- **Path Param**: `:id` — ID router yang terdaftar di `config.toml` (contoh: `main`)
- **Headers**:
  ```http
  Authorization: Bearer change-me-to-a-long-random-string
  Content-Type: application/json
  ```

#### Request Body:
```json
{
  "command": "/ip/address/print",
  "args": {
    "?disabled": "no",
    ".proplist": ".id,address,network,interface"
  }
}
```

#### Aturan pada `args`:
- Key diawali tanda `?` -> Query filter MikroTik (contoh: `"?disabled": "no"`).
- Key diawali tanda `.` -> API control attributes (contoh: `".proplist": "name,type"`).
- Key biasa -> Parameter perintah MikroTik (contoh: `"address": "10.0.0.1/24"`, `"interface": "ether1"`).

#### Response Sukses (`200 OK`):
Mengembalikan list data object (`!re` records):
```json
[
  {
    ".id": "*1",
    "address": "192.168.88.1/24",
    "network": "192.168.88.0",
    "interface": "bridge"
  }
]
```

#### Contoh: Menambah Hotspot User (Add Item):
```json
{
  "command": "/ip/hotspot/user/add",
  "args": {
    "name": "user123",
    "password": "secretpassword",
    "profile": "default",
    "limit-uptime": "1h"
  }
}
```
*Response*: `200 OK` dengan array kosong `[]` (karena command add hanya mengembalikan `!done` dengan atribut `=ret=*id`).

---

### C. Streaming Real-Time (Server-Sent Events / SSE)
- **Path**: `GET /routers/:id/listen`
- **Headers**:
  ```http
  Authorization: Bearer change-me-to-a-long-random-string
  Accept: text/event-stream
  ```
- **Query Parameters**:
  - `command`: Nama perintah streaming (wajib diawali `/`, misal: `/interface/monitor-traffic`)
  - Parameter lain langsung disertakan sebagai query parameter (misal: `interface=ether1`).

#### Contoh Request URL:
```
http://127.0.0.1:8080/routers/main/listen?command=/interface/monitor-traffic&interface=ether1
```

#### Format Data Stream (SSE):
```
data: {"rx-bits-per-second":"15280","tx-bits-per-second":"4210","rx-packets-per-second":"14","tx-packets-per-second":"7"}

data: {"rx-bits-per-second":"18400","tx-bits-per-second":"5100","rx-packets-per-second":"17","tx-packets-per-second":"9"}
```

*Saat koneksi HTTP ditutup oleh klien (misal browser ditutup), Rust gateway otomatis mengirimkan `/cancel` ke MikroTik sehingga proses background di router langsung berhenti.*

---

## 4. Format Error Response

| HTTP Status | Keterangan | Contoh Body |
|---|---|---|
| `400 Bad Request` | Command tidak diawali `/` atau argumen salah | `{"error": "command must start with '/'"}` |
| `401 Unauthorized` | Token Bearer salah atau tidak ada | `{"error": "unauthorized"}` |
| `404 Not Found` | ID Router belum didaftarkan di `config.toml` | `{"error": "unknown router"}` |
| `422 Unprocessable` | Router MikroTik menolak perintah (`!trap`) | `{"error": "no such item", "category": null}` |
| `502 Bad Gateway` | Gagal konek ke MikroTik / password router salah | `{"error": "router login failed: invalid user name or password"}` |

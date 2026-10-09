# MikroTik Universal Rust Core & API Gateway Engine

Service perantara berkinerja tinggi (*High-Performance Universal Gateway Service*) yang ditulis dengan **Rust** (`tokio` async). Proyek ini bertindak sebagai **jembatan universal terpusat** antara router MikroTik RouterOS dengan seluruh aplikasi klien (**Web React/Vue/Angular, Mobile Flutter/React Native/Swift/Kotlin, Backend Laravel/PHP, Node.js, Python, Go, Java**) tanpa batasan bahasa pemrograman.

---

## 🏛️ Arsitektur Sistem Universal

<p align="center">
  <img src="assets/architecture-diagram.svg" width="100%" alt="MikroTik Universal Rust Gateway Architecture" />
</p>

### Diagram Alur Komunikasi

```mermaid
graph TB
    subgraph Clients ["Aplikasi Pengembang (Stack Bebas)"]
        W["Web Apps: React / Vue / Svelte"]
        M["Mobile Apps: Flutter / React Native"]
        B["Backend: Laravel / Node.js / Python / Go"]
    end

    subgraph RustEngine ["MikroTik Universal Rust Gateway (Port 8080)"]
        AUTH["Bearer Token Auth"]
        WS["WebSocket Engine (/ws)"]
        FAST["Fast-Path Overview (/api/v1/overview)"]
        REST["Universal REST API Engine"]
        POOL["Dynamic Connection Pool (Zero-Churn TCP)"]
        MUX["Wire Protocol Parser & .tag Multiplexer"]
    end

    subgraph Hardware ["Perangkat Router MikroTik"]
        R1["Router Lokal (Port 8728)"]
        R2["Router Remote VPN (Port 51121)"]
    end

    W -->|"HTTP JSON / WebSocket"| RustEngine
    M -->|"HTTP JSON / WebSocket"| RustEngine
    B -->|"HTTP POST JSON"| RustEngine

    AUTH --> REST
    AUTH --> FAST
    REST --> POOL
    FAST --> POOL
    WS --> POOL
    POOL --> MUX
    MUX <-->|"Persistent Multiplexed Stream"| R1
    MUX <-->|"Persistent Multiplexed Stream"| R2
```

---

## ⚡ Mengapa Menggunakan Rust Core Gateway Ini?

<p align="center">
  <img src="assets/tag-multiplexing-flow.svg" width="100%" alt="Tag Multiplexing vs Naive Single-Socket Churn" />
</p>

1. **Universal Across Any Stack**: Tidak ada ketergantungan library khusus bahasa tertentu. Seluruh klien di stack mana pun cukup mengirimkan request HTTP standar dengan parameter yang konsisten:
   `host`, `port`, `user`, `password` (melalui JSON body atau HTTP headers `X-Router-*`).
2. **Kecepatan Sub-Millisecond (< 1-3 ms)**: Tidak ada lagi overhead koneksi lambat di mana klien harus buka-tutup socket TCP dan kalkulasi MD5 setiap kali memuat halaman. Rust menjaga koneksi TCP tetap hidup (*warm persistent pool*).
3. **Fast-Path Aggregated Snapshot (`/api/v1/overview`)**: Menggabungkan 6 query router (CPU, RAM, Identity, RouterBOARD, Hotspot Online, PPPoE Online, Interface Link) secara paralel simultan menggunakan `.tag` multiplexer via `tokio::join!`. Dashboard NOC & billing termuat instan dalam 2ms tanpa 6x round-trip terpisah.
4. **Full-Duplex WebSocket Engine (`/ws`)**: Streaming real-time monitoring traffic interface dan eksekusi command dua arah tanpa overhead polling HTTP.
5. **Batch Voucher Generator (`/api/v1/hotspot/generate-batch`)**: Pembuatan ratusan voucher hotspot sekaligus dalam hitungan milidetik secara asinkron.
6. **Aman untuk Router & Klien**: Mengurangi beban CPU MikroTik hingga ~85% sehingga voucher pelanggan dan pengguna online tidak terputus. Kredensial router juga tidak perlu terekspos langsung ke browser pengguna.

---

## 📑 Daftar Isi Dokumentasi

Dokumentasi teknis lengkap tersedia di folder [`docs/`](docs/):

1. **[01. Arsitektur & Alur Kerja Sistem](docs/01-arsitektur-dan-alur.md)**
   - Perbandingan arsitektur socket konvensional vs Rust Gateway.
   - Cara kerja multiplexing tag `.tag` pada satu koneksi TCP.
2. **[02. Protokol Biner RouterOS (Port 8728)](docs/02-protokol-routeros-binary.md)**
   - Format biner length-prefix (1–5 byte), Words, dan Sentences (`!re`, `!done`, `!trap`).
   - Alur autentikasi otomatis: RouterOS v6 (MD5 challenge) & RouterOS v7 (plain).
3. **[03. Spesifikasi Gateway REST, WebSocket & SSE API](docs/03-gateway-api-spesifikasi.md)**
   - Mekanisme passing kredensial (Headers vs Body vs ID).
   - Protokol frame WebSocket bidirectional (`/ws`).
   - Format response terstandarisasi dan error handling trap.
4. **[04. Referensi Lengkap Universal API](docs/04-universal-api-reference.md)**
   - Seluruh daftar endpoint modular: Overview, System, IPv4, IPv6, DHCP, Hotspot & Batch Voucher, PPP, Firewall, Queues, Interfaces, Tools, dan WebSocket.
5. **[05. Panduan Integrasi Multi-Stack](docs/05-integrasi-multi-stack.md)**
   - Contoh kode integrasi siap pakai untuk JavaScript/TypeScript, Mobile Flutter/Dart, Python, PHP/Laravel, dan Go.
6. **[06. Katalog Lengkap Command MikroTik](docs/06-katalog-command-mikrotik.md)**
   - Kamus referensi seluruh path command MikroTik RouterOS API beserta filter query dan atributnya.

---

## 🚀 API Collections untuk Pengujian

Koleksi pengujian siap pakai tersedia di folder [`collections/`](collections/):
- **[`collections/mikrotik-gateway.http`](collections/mikrotik-gateway.http)**: Pengujian instan di VS Code (REST Client) atau JetBrains HTTP Client.
- **[`collections/mikrotik_gateway.postman_collection.json`](collections/mikrotik_gateway.postman_collection.json)**: Siap di-import langsung ke Postman, Insomnia, atau Bruno.

---

## 🛠️ Menjalankan Service di Komputer Anda

### 1. Jalankan Gateway
```powershell
# Menggunakan config default
cargo run -p routeros-gateway -- config.toml
```
Server akan aktif di `http://127.0.0.1:8080`.
Buka browser Anda ke `http://127.0.0.1:8080/` untuk mengakses **Interactive Live Playground**.

### 2. Contoh Fast-Path Overview
```bash
curl -X POST http://127.0.0.1:8080/api/v1/overview \
  -H "Authorization: Bearer change-me-to-a-long-random-string" \
  -H "X-Router-Host: ath.vpnbersama.us" \
  -H "X-Router-Port: 51121" \
  -H "X-Router-User: admin" \
  -H "X-Router-Pass: secret" \
  -H "Content-Type: application/json" \
  -d '{}'
```

### 3. Contoh WebSocket Full-Duplex (/ws)
```javascript
const ws = new WebSocket("ws://127.0.0.1:8080/ws?token=change-me-to-a-long-random-string");

ws.onopen = () => {
  // Subscribe live interface traffic
  ws.send(JSON.stringify({
    action: "subscribe_traffic",
    router: { host: "ath.vpnbersama.us", port: 51121, user: "admin", password: "secret" },
    interface: "ether1",
    tag: "traffic-eth1"
  }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  console.log("Live stream frame:", msg);
};
```

# MikroTik Universal Rust Core & API Gateway Engine

Service perantara berkinerja tinggi (*High-Performance Universal Gateway Service*) yang ditulis dengan **Rust** (`tokio` async). Proyek ini bertindak sebagai **jembatan universal terpusat** antara router MikroTik RouterOS dengan seluruh aplikasi klien (**Web, Mobile Flutter/React Native, Backend Laravel, Node.js, Python, Go**) tanpa batasan bahasa pemrograman.

---

## ⚡ Mengapa Menggunakan Rust Core Gateway Ini?

```mermaid
graph TB
    subgraph Clients ["Aplikasi Pengembang (Stack Bebas)"]
        W[Web Apps: React / Vue / Angular]
        M[Mobile Apps: Flutter / React Native / Swift / Kotlin]
        B[Backend Services: Laravel / Node.js / Python / Go / Java]
    end

    subgraph RustEngine ["MikroTik Universal Rust Gateway (Port 8080)"]
        API[Universal REST & SSE Router]
        POOL[Dynamic Persistent Connection Pool<br/>(Sub-millisecond latency & zero handshake churn)]
        MUX[Wire Protocol Parser & Tag Multiplexer]
    end

    subgraph Hardware ["Perangkat Router MikroTik"]
        R1["Router Lokal (Port 8728)"]
        R2["Router Remote VPN (Port 51121)"]
    end

    W -->|"HTTP JSON / Headers"| API
    M -->|"HTTP JSON / Headers"| API
    B -->|"HTTP JSON / Headers"| API

    API --> POOL
    POOL --> MUX
    MUX <===>|"Persistent Multiplexed Socket"| R1
    MUX <===>|"Persistent Multiplexed Socket"| R2
```

1. **Universal across Any Stack**: Tidak ada ketergantungan library khusus bahasa tertentu. Seluruh klien di stack mana pun cukup mengirimkan request HTTP standar dengan variabel yang sama:
   `host`, `port`, `user`, `password` (melalui JSON body atau HTTP headers `X-Router-*`).
2. **Kecepatan Setara Winbox / Port 80**: Tidak ada lagi overhead koneksi lambat di mana klien harus buka-tutup socket TCP dan kalkulasi MD5 setiap kali memuat halaman. Rust menjaga koneksi TCP tetap hidup (*warm persistent pool*).
3. **Aman untuk Router & Klien**: Mengurangi beban CPU MikroTik secara signifikan sehingga voucher dan pengguna online tidak terganggu. Kredensial router juga tidak perlu terekspos langsung ke browser pengguna.
4. **Cakupan Fitur Lengkap**: Meliputi IPv4, IPv6, DHCP, Hotspot & Voucher, PPPoE, Firewall & Keamanan (IP Blocking), Simple Queues, Interfaces, dan Realtime Traffic Monitoring via SSE (Server-Sent Events).

---

## 📑 Daftar Isi Dokumentasi

Dokumentasi teknis lengkap tersedia di folder [`docs/`](file:///d:/MyPorto/mikrotik/docs):

1. **[01. Arsitektur & Alur Kerja Sistem](file:///d:/MyPorto/mikrotik/docs/01-arsitektur-dan-alur.md)**
   - Perbandingan arsitektur socket konvensional vs Rust Gateway.
   - Diagram alur Mermaid: Cara kerja multiplexing tag `.tag` pada satu koneksi TCP.
2. **[02. Protokol Biner RouterOS (Port 8728)](file:///d:/MyPorto/mikrotik/docs/02-protokol-routeros-binary.md)**
   - Format biner length-prefix (1–5 byte), Words, dan Sentences (`!re`, `!done`, `!trap`).
   - Alur autentikasi otomatis: RouterOS v6 (MD5 challenge) & RouterOS v7 (plain).
3. **[03. Spesifikasi Gateway REST & SSE API](file:///d:/MyPorto/mikrotik/docs/03-gateway-api-spesifikasi.md)**
   - Mekanisme passing kredensial (Headers vs Body vs ID).
   - Format error standar HTTP (400, 401, 404, 422 trap, 502).
4. **[04. Referensi Lengkap Universal API](file:///d:/MyPorto/mikrotik/docs/04-universal-api-reference.md)**
   - Seluruh daftar endpoint modular: System, IPv4, IPv6, DHCP, Hotspot, PPP, Firewall, Queues, Interfaces, dan Raw Command.
5. **[05. Panduan Integrasi Multi-Stack](file:///d:/MyPorto/mikrotik/docs/05-integrasi-multi-stack.md)**
   - Contoh kode siap pakai untuk JavaScript/TypeScript, Mobile Flutter/Dart, Python, PHP, dan Go.
6. **[06. Katalog Lengkap Command MikroTik](file:///d:/MyPorto/mikrotik/docs/06-katalog-command-mikrotik.md)**
   - Kamus referensi seluruh path command MikroTik RouterOS API beserta filter query dan atributnya.

---

## 🚀 API Collections untuk Pengujian

Koleksi pengujian siap pakai tersedia di folder [`collections/`](file:///d:/MyPorto/mikrotik/collections):
- **[`collections/mikrotik-gateway.http`](file:///d:/MyPorto/mikrotik/collections/mikrotik-gateway.http)**: Pengujian instan di VS Code (REST Client) atau JetBrains HTTP Client.
- **[`collections/mikrotik_gateway.postman_collection.json`](file:///d:/MyPorto/mikrotik/collections/mikrotik_gateway.postman_collection.json)**: Siap di-import langsung ke Postman, Insomnia, atau Bruno.

---

## 🛠️ Menjalankan Service di Komputer Anda

### 1. Pasang Rust Toolchain (Jika belum ada)
Buka PowerShell dan jalankan:
```powershell
winget install Rustlang.Rustup
```
Setelah selesai, restart terminal Anda.

### 2. Salin Konfigurasi & Jalankan Gateway
```powershell
Copy-Item config.example.toml config.toml
cargo run -p routeros-gateway -- config.toml
```
Server akan aktif di `http://127.0.0.1:8080`.

### 3. Menguji Perintah CLI Direct (Opsional)
```powershell
cargo run -p routeros-core --example poc -- ath.vpnbersama.us:51121 admin "password123"
```
CLI ini akan login langsung ke router target, membaca resource, dan streaming live traffic selama 5 detik lalu mengirimkan `/cancel`.

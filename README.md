<p align="center">
  <img src="assets/animated-banner.svg" width="100%" alt="MikroTik Universal Rust Engine Banner" />
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Language-Rust%202021-f97316?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" /></a>
  <a href="https://www.docker.com/"><img src="https://img.shields.io/badge/Docker-Ready%20%3C25MB-0284c7?style=for-the-badge&logo=docker&logoColor=white" alt="Docker" /></a>
  <a href="https://mikrotik.com/"><img src="https://img.shields.io/badge/RouterOS-v6.49%20%2B%20v7.x-3b82f6?style=for-the-badge&logo=mikrotik&logoColor=white" alt="RouterOS" /></a>
  <a href="docs/04-universal-api-reference.md"><img src="https://img.shields.io/badge/API%20Endpoints-140%2B%20Enterprise-10b981?style=for-the-badge&logo=fastapi&logoColor=white" alt="Endpoints" /></a>
  <a href="#-arsitektur-dan-alur-kerja"><img src="https://img.shields.io/badge/Latency-Sub--ms%20%3C1.5ms-8b5cf6?style=for-the-badge&logo=speedtest&logoColor=white" alt="Sub-Millisecond" /></a>
</p>

---

## 🌟 Ikhtisar (Overview)

**MikroTik Universal Rust Gateway Engine** adalah service orkestrasi jaringan tingkat perusahaan (*Enterprise NOC Gateway & BRAS Middleware*) yang dibangun murni menggunakan bahasa **Rust** asinkron (`tokio`, `axum`).

Service ini dirancang dari perspektif **Network Operations Center (NOC) Engineer** & **ISP Network Architect** untuk mengatasi kelemahan fundamental integrasi RouterOS konvensional. Gateway ini berfungsi sebagai **jembatan universal berperforma tinggi** antara armada perangkat keras MikroTik RouterOS (dari perangkat kelas CPE/WISP seperti *hAP lite, hAP mini, RB750Gr3* hingga core router *CCR, Cloud Hosted Router (CHR)*, dan *x86*) dengan **seluruh ekosistem aplikasi modern** tanpa batasan bahasa pemrograman:
* 🌐 **Web Frontends**: Vue.js, React, Next.js, Nuxt, Svelte, Angular.
* 📱 **Mobile Apps**: Flutter (Dart), React Native, Kotlin, Swift.
* ⚡ **Backend Stacks**: Laravel (PHP), Express / Nest.js (Node.js), Go, Python (FastAPI/Django), Java (Spring).

### 🔗 Portal Langsung & Akses Cepat (Production)
* 📖 **Dokumentasi API & Interactive Explorer**: [`https://ros-gateway.samrifa.com/docs`](https://ros-gateway.samrifa.com/docs)
* 🎮 **Live Web Management Portal**: [`https://ros-gateway.samrifa.com/`](https://ros-gateway.samrifa.com/)
* 🌐 **Interactive Topology & Relational Visualizer**: [`https://ros-gateway.samrifa.com/topology`](https://ros-gateway.samrifa.com/topology)
* 🩺 **Healthcheck Service**: [`https://ros-gateway.samrifa.com/health`](https://ros-gateway.samrifa.com/health)
* 📋 **OpenAPI Specification (JSON)**: [`https://ros-gateway.samrifa.com/api/v1/spec`](https://ros-gateway.samrifa.com/api/v1/spec)

---

## 💡 Mengapa Menggunakan Engine Rust Ini? (Perspektif NOC Engineer)

<p align="center">
  <img src="assets/tag-multiplexing-flow.svg" width="100%" alt="Tag Multiplexing vs Naive Single-Socket Churn" />
</p>

Sebagai tim NOC atau operator ISP/WISP, masalah terbesar pada integrasi API RouterOS konvensional (misal library PHP `routeros-api.php` atau library Node.js) adalah **Socket Churn** yang menyebabkan CPU RouterOS melonjak 100% dan mengunci sistem (*freeze/hang*).

| Fitur Konvensional (Library PHP/Node.js) | 🦀 MikroTik Universal Rust Engine |
|---|---|
| **Koneksi Soket**: Buka-tutup soket TCP baru setiap request (*high socket churn*). | **Persistent Multiplexed Pool**: Soket TCP tetap hangat (*warm*), query berjalan simultan lewat penanda atomik `.tag`. |
| **Beban CPU Router**: CPU RouterOS sering melonjak 100% dan hang saat banyak query. | **Zero CPU Freeze**: Dilengkapi pelindung **15-Second Anti-Hang Guard** dan FastTrack connection bypass. |
| **Kecepatan Respons**: 200 ms – 1.5 detik per request. | **Sub-Milidetik**: Rata-rata respons **< 1.5 milidetik** berkat protokol biner tingkat rendah (*raw wire format*). |
| **Metode HTTP**: Terikat pada library spesifik atau POST dengan JSON kaku. | **Dual HTTP GET & POST**: Semua 140+ endpoint mendukung query parameter langsung (`?host=..&token=..`), cURL one-liner, dan browser direct call. |
| **Ketergantungan Stack**: Terikat library bahasa tertentu. | **Universal REST & WebSocket**: Semua stack cukup memanggil REST JSON atau WebSocket `/ws`. |
| **Kerapian Aturan Winbox**: Script otomatis sering mengotori firewall tanpa jejak. | **Standar Komentar Otomatis**: Semua aturan otomatis ditandai rapi (misal `[PCC-LoadBalance]`, `[App-Blocker]`, `[Anti-Tethering]`). |

---

## 🏛️ Arsitektur & Alur Kerja

<p align="center">
  <img src="assets/architecture-diagram.svg" width="100%" alt="MikroTik Universal Rust Gateway Architecture" />
</p>

```mermaid
graph TB
    subgraph Stacks ["Aplikasi Klien & NOC Operations"]
        W["Web Frontends: Vue / React / Nuxt"]
        M["Mobile Apps: Flutter / React Native"]
        B["Backend Microservices: Laravel / Go / Python"]
        NOC["NOC Scripts: cURL / Browser GET / Webhooks"]
    end

    subgraph RustEngine ["Rust Universal Gateway Core (Port 8080)"]
        AUTH["Bearer Token & Query Auth Middleware"]
        WS["Full-Duplex WebSocket (/ws)"]
        REST["115+ Enterprise Endpoints (Dual GET & POST)"]
        POOL["Async Multiplexed Connection Pool (Arc-Mutex)"]
        GUARD["15s Anti-Hang Timeout Guard"]
    end

    subgraph Hardware ["Perangkat RouterOS (v6.x & v7.x)"]
        R1["Local Hardware (Port 8728: CCR / RB4011 / hEX / hAP)"]
        R2["Remote Branch / VPN (Port 51121)"]
        R3["Cloud Hosted Router (Port 8729: CHR / x86)"]
    end

    W -->|"HTTP JSON / WS"| RustEngine
    M -->|"HTTP JSON / WS"| RustEngine
    B -->|"HTTP POST JSON"| RustEngine
    NOC -->|"HTTP GET & POST + Query Params"| RustEngine

    AUTH --> REST
    AUTH --> WS
    REST --> POOL
    WS --> POOL
    POOL --> GUARD
    GUARD <-->|"Binary Wire Protocol (.tag)"| R1
    GUARD <-->|"Binary Wire Protocol (.tag)"| R2
    GUARD <-->|"Binary Wire Protocol (.tag)"| R3
```

---

## 🎯 Solusi & Fitur Kelas Enterprise untuk NOC & ISP

### 1. 👥 BRAS / BNG & Manajemen Pelanggan PPPoE (`/api/v1/ppp/*`)
* **Live Session Telemetry**: Monitoring seluruh subscriber PPPoE aktif, IP address, uptime, dan bytes in/out secara real-time.
* **Instant Disconnect / Kick**: Memutus sesi pelanggan bermasalah atau menunggak dalam 1 panggilan API (< 10ms) agar segera melakukan dial-up ulang.
* **Profile & Secret Provisioning**: Tambah, ubah, dan hapus secret pelanggan dan profil bandwidth tanpa perlu membuka Winbox.

### 2. 🎫 Carrier Hotspot & Voucher Security (`/api/v1/hotspot/*`, `/api/v1/security/*`)
* **Voucher Lifecycle**: Pantau user aktif, durasi waktu tersisa, dan volume kuota.
* **Anti-Tethering & Anti-WiFi Sharing**: Injeksi otomatis aturan Mangle `action=change-ttl new-ttl=set:1` dan `shared-users=1` untuk memblokir tethering via QR Code, Bluetooth, atau USB.
* **1-Klik Hotspot Wizard**: Membangun topologi hotspot lengkap dalam < 50ms (IP Pool, DHCP Server, Hotspot Profile, Walled Garden, dan NAT Masquerade).

### 3. 🔍 QoS, Queue Tree & Bandwidth Traffic Engineering (`/api/v1/queues/*`, `/api/v1/traffic/*`)
* **Deep Queue Inspection**: Inspeksi limit max (`5M/10M`), kecepatan live upload & download, rasio utilisasi antrean, dan deteksi paket drop.
* **Throttling Alert**: Otomatis mendeteksi status pelanggan: `THROTTLED (Merah di Winbox)`, `ACTIVE`, atau `IDLE`.
* **Hierarchical PCQ & Queue Tree**: Skema pembagian bandwidth adil berbasis Per-Connection Queuing (PCQ) untuk mencegah keluhan pelanggan akibat pemakaian rakus.
* **Pisah Trafik Game vs Sosmed**: Paket game (Mobile Legends, PUBG, Valorant) diprioritaskan di Queue prioritas 1, sementara video streaming dialihkan ke antrean reguler.

### 4. ⚖️ 1-Klik Multi-WAN PCC Load Balancing Wizard (`/api/v1/load-balance/*`)
* Mengotomatisasi konfigurasi rumit **Per Connection Classifier (PCC)** Multi-WAN dalam 1 panggilan API (< 30ms).
* Mendukung rasio bobot tidak seimbang (misal ISP 1 50M vs ISP 2 100M dengan bobot `1:2`), otomatis failover ping `check-gateway`, dan NAT Masquerade rapi berlabel `[PCC-LoadBalance]`.

### 5. 🌐 Korelasi Topologi L2/L3 & Deteksi Access Point (`/topology`, `/api/v1/network/infrastructure/scan`)
* **Deteksi Pintar Access Point (AP)**: Mengidentifikasi perangkat AP (TP-Link, Ubiquiti, Ruijie Reyee, Tenda, Totolink, Mercusys) yang terhubung pada port fisik (seperti `ether5` dengan IP `192.168.100.3`).
* **Branching Hierarkis 3-Tier**: Seluruh client wireless/voucher otomatis menginduk di bawah node Access Point masing-masing, bukan langsung menempel di interface port.
* **Tombol Akses Langsung Web Admin AP**: Tautan langsung ke `http://192.168.100.3` dari drawer inspektur node untuk konfigurasi SSID dan password tanpa repot.
* **Interactive Canvas**: Visualisasi graf berbasis canvas interaktif yang **dapat di-drag/geser, di-zoom, dan diklik** dengan **Live ICMP Ping dari Router**.

### 6. 🚫 1-Klik App & Content Blocker (`/api/v1/security/app-block`)
* Blokir instan untuk aplikasi dan konten: **WhatsApp, TikTok, YouTube, Judi Online, Torrent/P2P, atau Domain Custom**.

### 7. 🚀 FastTrack CPU Accelerator & Profiler (`/api/v1/system/*`)
* Memangkas beban CPU hingga 80% pada router entry-level (hAP lite/mini/RB750Gr3) dengan membypass connection tracking untuk paket established/related.

### 8. 📊 Native Prometheus Exporter (`GET /metrics`)
* Endpoint standar **Prometheus text exposition format (version 0.0.4)** untuk scraping langsung dari Grafana / Prometheus / VictoriaMetrics.
* Menyajikan gauge CPU, free RAM, free storage, uptime, jumlah subscriber PPPoE aktif, voucher Hotspot aktif, serta counter RX/TX byte & packet per interface **tanpa perlu mengaktifkan SNMP daemon di router**.

### 9. 🩺 Heuristic "Doctor MikroTik" Assistant (`/api/v1/doctor/diagnose`)
* **Audit Otomatis 6 Pilar**: Memeriksa CPU Load, Free RAM, Latensi & Packet Loss Uplink WAN, Throttling pada Simple Queue, FastTrack acceleration, dan status interface LAN.
* Mengembalikan **Skor Kesehatan (0-100)** beserta rekomendasi perbaikan berbasis bahasa manusia dalam Bahasa Indonesia dan English.

### 10. 🗺️ Carrier BGP & OSPF Dynamic Routing Peering (`/api/v1/routing/*`)
* **BGP Sessions**: Status peering BGP ISP (*Established / Active / Idle*, AS number, prefix count yang diterima, uptime) kompatibel RouterOS v6 dan v7.
* **OSPF Neighbors**: Monitoring OSPF Neighbor Adjacency (*Full, 2-Way, Init*, Designated Router, Interface link).
* **Route Table Breakdown**: Rekapitulasi rute aktif berdasarkan protokol (*Connected, Static, BGP, OSPF*).

### 11. 🎯 Multi-Target SLA Ping Matrix (`/api/v1/tools/multi-ping`)
* Mengirim ping sekaligus ke gateway ISP lokal dan DNS publik global (Cloudflare 1.1.1.1, Google 8.8.8.8, OpenDNS) secara otomatis.
* Mengalkulasi packet loss %, RTT min/avg/max, estimasi jitter, dan status SLA (`EXCELLENT`, `HEALTHY`, `DEGRADED`, `DOWN`).

### 12. 🛡️ Native RouterOS v7 AdList DNS Blocker (`/api/v1/dns/adlist/*`)
* Pengelolaan DNS AdList bawaan MikroTik (RouterOS v7) untuk blokir iklan dan pelacak seluruh jaringan setara Pi-hole tanpa perangkat keras tambahan.
* Dilengkapi preset resmi terverifikasi: HaGeZi Multi Light, StevenBlack Hosts, Threat Intelligence TIF, dan AdGuard DNS.

### 13. 🔍 Conntrack Top Talkers & DDoS Mitigation (`/api/v1/firewall/connections/*`)
* Inspeksi tabel *Connection Tracking* router dan analisis otomatis 15 host penghabis sesi koneksi (*Top Talkers*) untuk melacak penyalahgunaan BitTorrent, malware zombie, atau SYN flood.
* Opsi *Flush Connections* instan untuk memutus paksa sesi koneksi dari IP tertentu.

### 14. 🚨 Rogue DHCP Server Watchdog (`/api/v1/dhcp/alerts`)
* Pemasangan alarm deteksi server DHCP ilegal/liar pada interface LAN untuk mencegah insiden tabrakan IP akibat pelanggan salah mencolokkan port LAN router rumahan.

### 15. 🎫 Auto-Login QR Code Hotspot Vouchers (`/api/v1/hotspot/voucher-template/render`)
* Template pencetakan voucher kartu (Grid A4) dan struk thermal kasir (58mm) dilengkapi **QR Code Auto-Login** (`http://{dns}/login?username={user}&password={pass}`). Pelanggan cukup scan kamera smartphone langsung terhubung tanpa mengetik manual.

### 16. 📡 CAPsMAN Centralized Wireless AP Provisioning (`/api/v1/capsman/*`)
* Mengontrol dan mengubah **SSID & Password WiFi terpusat** untuk seluruh Access Point MikroTik (cAP ax, wAP ac, dsb) secara serentak tanpa perlu login ke masing-masing AP.
* Monitoring live seluruh klien wireless (HP/laptop) yang tersambung di tiap AP beserta sinyal dBm dan rate kecepatan.

### 17. 💾 Binary Restore & Script Importer (`/api/v1/backup/restore` & `/import`)
* **Restore Backup Otomatis**: Memulihkan konfigurasi sistem dari file binary `.backup` dan reboot otomatis via API.
* **Script Importer (.rsc)**: Mengimpor dan mengeksekusi skrip konfigurasi RouterOS untuk migrasi atau deployment armada baru.

### 18. ⚡ PoE Remote Hard Reboot (`/api/v1/network/infrastructure/poe-cycle`)
* Mematikan daya PoE port fisik sementara (3 detik) lalu menyalakannya kembali untuk **me-reboot fisik Access Point (TP-Link, Ubiquiti, Ruijie) yang hang/freeze** di tiang tower atau plafon tanpa perlu memanjat atau mencabut kabel LAN.

### 19. 🌐 Third-Party AP Web GUI Tunnel (`/api/v1/network/infrastructure/ap-tunnel`)
* Membuka tunnel port forwarding NAT sementara agar administrator jaringan dapat mengakses dan mengubah konfigurasi Web Admin Access Point pihak ketiga (`192.168.100.3:80`) dari luar jaringan lokal (remote access).

---

## ⚡ Fleksibilitas Pemanggilan: Dual Transport (GET & POST)

Setiap endpoint pada gateway dapat diakses menggunakan **HTTP POST** (standar enterprise) maupun **HTTP GET** (praktis untuk NOC dan browser), dengan fleksibilitas penentuan kredensial:

### Opsi A: Headers (Direkomendasikan untuk Microservice & Mobile)
```http
POST /api/v1/overview HTTP/1.1
Host: 127.0.0.1:8080
Authorization: Bearer <GATEWAY_TOKEN>
X-Router-Host: 192.168.88.1
X-Router-Port: 8728
X-Router-User: admin
X-Router-Pass: secret123
Content-Type: application/json

{}
```

### Opsi B: URL Query Parameter (Direkomendasikan untuk NOC Triage, cURL & Browser)
Buka langsung di browser atau terminal tanpa perlu menyusun header kustom:
```bash
# cURL GET dengan parameter URL
curl "http://127.0.0.1:8080/api/v1/overview?host=192.168.88.1&port=8728&user=admin&pass=secret123&token=change-me"

# cURL POST dengan parameter URL
curl -X POST "http://127.0.0.1:8080/api/v1/queues/inspect-user?query=192.168.88.50&host=192.168.88.1&port=8728&user=admin&pass=secret123&token=change-me"
```

---

## 🐳 Panduan Instalasi Cepat dengan Docker (1-Command)

Service telah dibungkus rapi dalam image Docker multi-stage ultra-ringan (**< 25 MB**) berbasis Alpine Linux:

### 1. Jalankan Menggunakan Docker Compose (Direkomendasikan)
```bash
docker compose up -d
```

### 2. Atau Jalankan Langsung dengan Docker Run
```bash
docker run -d \
  --name mikrotik-universal-gateway \
  --restart unless-stopped \
  -p 8080:8080 \
  -e GATEWAY_LISTEN=0.0.0.0:8080 \
  -e GATEWAY_TOKEN=change-me-to-a-long-random-string \
  -e RUST_LOG=info \
  salmanmustapa/mikrotik-universal-gateway:latest
```

Server langsung aktif di `http://localhost:8080`:
* 📖 **API Docs Explorer**: `http://localhost:8080/docs`
* 🎮 **Live Management Portal**: `http://localhost:8080/`
* 🌐 **Topology Visualizer**: `http://localhost:8080/topology`
* 🩺 **Health Check**: `http://localhost:8080/health`

---

## 💻 Panduan Menjalankan Secara Manual (Tanpa Docker)

```powershell
# Clone repositori
git clone https://github.com/salman-mustapa/mikrotik-service.git
cd mikrotik-service

# Jalankan gateway dengan config.toml
cargo run -p routeros-gateway -- config.toml
```

---

## 🔌 Contoh Integrasi Multi-Stack

### 1. Laravel (PHP)
```php
use Illuminate\Support\Facades\Http;

$response = Http::withToken('change-me-to-a-long-random-string')
    ->withHeaders([
        'X-Router-Host' => '192.168.88.1',
        'X-Router-Port' => '8728',
        'X-Router-User' => 'admin',
        'X-Router-Pass' => 'password123',
    ])
    ->post('http://127.0.0.1:8080/api/v1/queues/inspect-user', [
        'query' => '192.168.88.50'
    ]);

$userData = $response->json();
echo "Kecepatan Saat Ini: " . $userData['current_speed']['download_formatted'];
```

### 2. Vue 3 / Nuxt / Express (JavaScript / TypeScript)
```javascript
const res = await fetch('http://127.0.0.1:8080/api/v1/overview', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer change-me-to-a-long-random-string',
    'X-Router-Host': '192.168.88.1',
    'X-Router-Port': '8728',
    'X-Router-User': 'admin',
    'X-Router-Pass': 'password123',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({})
});

const overview = await res.json();
console.log("Router CPU Load:", overview.system.cpu_load);
```

### 3. Mobile Flutter / Dart
```dart
import 'dart:convert';
import 'package:http/http.dart' as http;

Future<void> fetchRouterOverview() async {
  final res = await http.post(
    Uri.parse('http://10.0.2.2:8080/api/v1/overview'),
    headers: {
      'Authorization': 'Bearer change-me-to-a-long-random-string',
      'X-Router-Host': '192.168.88.1',
      'X-Router-Port': '8728',
      'X-Router-User': 'admin',
      'X-Router-Pass': 'password123',
      'Content-Type': 'application/json',
    },
    body: jsonEncode({}),
  );
  print(jsonDecode(res.body));
}
```

---

## 📑 Dokumentasi Teknis Lengkap

Folder [`docs/`](docs/) memuat rincian arsitektur dan kamus API:
* **[01. Arsitektur & Alur Kerja Sistem](docs/01-arsitektur-dan-alur.md)**
* **[02. Protokol Biner RouterOS (Port 8728)](docs/02-protokol-routeros-binary.md)**
* **[03. Spesifikasi Gateway REST, WebSocket & SSE](docs/03-gateway-api-spesifikasi.md)**
* **[04. Referensi Lengkap Universal API (115+ Enterprise Endpoints)](docs/04-universal-api-reference.md)**
* **[05. Panduan Integrasi Multi-Stack (Laravel, Vue, Express, Flutter, Go)](docs/05-integrasi-multi-stack.md)**
* **[06. Katalog Lengkap Command MikroTik](docs/06-katalog-command-mikrotik.md)**

---

## 🧪 Koleksi Pengujian Siap Pakai

* **[`collections/mikrotik-gateway.http`](collections/mikrotik-gateway.http)**: Skenario pengujian instan di VS Code REST Client atau JetBrains HTTP Client.
* **[`collections/mikrotik_gateway.postman_collection.json`](collections/mikrotik_gateway.postman_collection.json)**: Siap di-import ke Postman, Insomnia, atau Bruno.

---

## 📄 Lisensi
Proyek ini dilisensikan di bawah lisensi **MIT License**.
Dikembangkan untuk komunitas Network Operations Center (NOC) dan Software Developers modern.

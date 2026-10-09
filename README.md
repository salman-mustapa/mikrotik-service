<p align="center">
  <img src="assets/animated-banner.svg" width="100%" alt="MikroTik Universal Rust Engine Banner" />
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Language-Rust%202021-f97316?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" /></a>
  <a href="https://www.docker.com/"><img src="https://img.shields.io/badge/Docker-Ready%20%3C25MB-0284c7?style=for-the-badge&logo=docker&logoColor=white" alt="Docker" /></a>
  <a href="https://mikrotik.com/"><img src="https://img.shields.io/badge/RouterOS-v6.49%20%2B%20v7.x-3b82f6?style=for-the-badge&logo=mikrotik&logoColor=white" alt="RouterOS" /></a>
  <a href="docs/04-universal-api-reference.md"><img src="https://img.shields.io/badge/API%20Endpoints-192%20Active-10b981?style=for-the-badge&logo=fastapi&logoColor=white" alt="Endpoints" /></a>
  <a href="#-arsitektur-dan-alur-kerja"><img src="https://img.shields.io/badge/Latency-Sub--ms%20%3C1.5ms-8b5cf6?style=for-the-badge&logo=speedtest&logoColor=white" alt="Sub-Millisecond" /></a>
</p>

---

## 🌟 Ikhtisar (Overview)

**MikroTik Universal Rust Gateway Engine** adalah service perantara jaringan tingkat perusahaan (*Enterprise NOC Gateway*) yang dibangun murni menggunakan **Rust** asinkron (`tokio`, `axum`).

Service ini dirancang sebagai **jembatan universal berperforma tinggi** antara perangkat keras MikroTik RouterOS (dari seri hemat daya seperti *hAP lite, hAP mini, RB750Gr3* hingga *CCR, Cloud Hosted Router (CHR)* dan *x86*) dengan **seluruh ekosistem aplikasi pengembang** tanpa batasan bahasa pemrograman:
* 🌐 **Web Frontend**: Vue.js, React, Next.js, Nuxt, Svelte, Angular.
* 📱 **Mobile Apps**: Flutter (Dart), React Native, Kotlin, Swift.
* ⚡ **Backend Stacks**: Laravel (PHP), Express / Nest.js (Node.js), Go, Python (FastAPI/Django), Java (Spring).

---

## 💡 Mengapa Menggunakan Engine Rust Ini?

<p align="center">
  <img src="assets/tag-multiplexing-flow.svg" width="100%" alt="Tag Multiplexing vs Naive Single-Socket Churn" />
</p>

| Fitur Konvensional (Library PHP/Node.js) | 🦀 MikroTik Universal Rust Engine |
|---|---|
| **Koneksi Soket**: Buka-tutup soket TCP baru setiap request (*high socket churn*). | **Persistent Multiplexed Pool**: Soket TCP tetap hangat (*warm*), query berjalan simultan lewat penanda `.tag`. |
| **Beban CPU Router**: CPU RouterOS sering melonjak 100% dan hang/freeze saat banyak query. | **Zero CPU Freeze**: Dilengkapi pelindung **15-Second Timeout Guard** dan FastTrack connection bypass. |
| **Kecepatan Respons**: 200 ms – 1.5 detik per request. | **Sub-Milidetik**: Rata-rata respons **< 1.5 milidetik** berkat protokol biner tingkat rendah (*raw wire format*). |
| **Ketergantungan Stack**: Terikat library spesifik (misal `routeros-api.php`). | **Universal REST & WebSocket**: Semua stack cukup memanggil REST JSON atau WebSocket `/ws`. |
| **Kerapian Aturan Winbox**: Script otomatis sering mengotori firewall tanpa jejak. | **Standar Komentar Otomatis**: Semua aturan otomatis ditandai rapi (misal `[PCC-LoadBalance]`, `[App-Blocker]`). |

---

## 🏛️ Arsitektur & Alur Kerja

<p align="center">
  <img src="assets/architecture-diagram.svg" width="100%" alt="MikroTik Universal Rust Gateway Architecture" />
</p>

```mermaid
graph TB
    subgraph Stacks ["Aplikasi Klien (Bebas Stack)"]
        W["Web: Vue / React / Nuxt"]
        M["Mobile: Flutter / React Native"]
        B["Backend: Laravel / Express / Go / Python"]
    end

    subgraph RustEngine ["Rust Universal Gateway Core (Port 8080)"]
        AUTH["Bearer Token Middleware"]
        WS["Full-Duplex WebSocket (/ws)"]
        REST["192 Modular REST API Endpoints"]
        POOL["Async Multiplexed Connection Pool"]
        GUARD["15s Anti-Hang Timeout Guard"]
    end

    subgraph Hardware ["Perangkat RouterOS (v6.x & v7.x)"]
        R1["Router Lokal (Port 8728)"]
        R2["Router Remote VPN (Port 51121)"]
        R3["Cloud Hosted Router / CCR"]
    end

    W -->|"HTTP JSON / WS"| RustEngine
    M -->|"HTTP JSON / WS"| RustEngine
    B -->|"HTTP POST JSON"| RustEngine

    AUTH --> REST
    AUTH --> WS
    REST --> POOL
    WS --> POOL
    POOL --> GUARD
    GUARD <-->|"Binary Wire Protocol"| R1
    GUARD <-->|"Binary Wire Protocol"| R2
    GUARD <-->|"Binary Wire Protocol"| R3
```

---

## 🎯 Matriks Fitur Unggulan

### 1. 📵 Anti-Tethering & Anti-WiFi Sharing (`/api/v1/security/anti-tethering/*`)
* Menyuntikkan aturan Mangle `action=change-ttl new-ttl=set:1` dan memaksa `shared-users=1`.
* **Memblokir** upaya berbagi internet voucher melalui **QR Code WiFi Sharing, Bluetooth Tethering, maupun USB Tethering**.

### 2. ⚖️ 1-Klik Multi-WAN PCC Load Balancing Wizard (`/api/v1/load-balance/*`)
* Mengotomatisasi konfigurasi rumit **Per Connection Classifier (PCC)** Multi-WAN dalam 1 panggilan API (< 30ms).
* Mendukung rasio bobot tidak seimbang (misal ISP1 50M vs ISP2 100M dengan bobot `1:2`), otomatis failover ping `check-gateway`, dan NAT Masquerade.
* Endpoint pemantau live balance ratio (`/api/v1/load-balance/status`) dan tombol reset aman (`/api/v1/load-balance/remove`).

### 3. 🔍 Inspeksi Mendalam Queue & Kecepatan User (`/api/v1/queues/*`)
* Mengetahui limit max (`5M/10M`), kecepatan real-time upload & download saat ini, persentase utilisasi antrean, kuota total transfer, dan packet drops.
* Menampilkan status antrean langsung: `THROTTLED (Merah di Winbox)`, `ACTIVE`, atau `IDLE`.
* Rekapitulasi eksekutif NOC untuk seluruh router: Total bandwidth dialokasikan vs konsumsi real-time dan **Top 5 Downloaders Terberat**.

### 4. 🚫 1-Klik App & Content Blocker (`/api/v1/security/app-block`)
* Blokir instan untuk aplikasi dan konten: **WhatsApp, TikTok, YouTube, Judi Online, Torrent/P2P, atau Domain Custom**.

### 5. 🧙‍♂️ 1-Klik Complete Hotspot Template Wizard (`/api/v1/hotspot/wizard/setup`)
* Membangun infrastruktur Hotspot lengkap dari nol dalam < 50ms: IP Address, Pool, DHCP Server, DHCP Network, Profil Hotspot, User Admin, Profil Anti-Tethering, dan Masquerade.

### 6. 🔀 Dst-NAT Port Forwarding Wizard (`/api/v1/firewall/port-forward`)
* Sekali klik membuat aturan Dst-NAT port forwarding (CCTV, Web Server, Billing) sekaligus membuka firewall filter forward accept dengan komentar `[Dst-NAT]`.

### 7. 🚀 FastTrack CPU Accelerator & Profiler (`/api/v1/system/*`)
* Memangkas beban CPU hingga 80% pada router kecil (hAP lite/mini/RB750Gr3) dengan membypass connection tracking untuk paket established/related.

### 8. 📡 Access Point & Infrastructure Detector (`/api/v1/network/infrastructure/scan`)
* Mendeteksi AP (Ubiquiti, TP-Link, Ruijie, Tenda), Switch, dan kamera di balik bridge/Hotspot serta fitur **Auto-Bypass AP** ke IP-Binding agar admin bisa remote AP tanpa login voucher.

### 9. ✈️ Telegram Bot & Automated Netwatch (`/api/v1/telegram/*`)
* Monitoring status UP/DOWN link ISP dan AP lokal, notifikasi otomatis terkirim ke bot Telegram via `/tool/fetch`.

### 10. 🎮 Pisah Trafik Game vs Sosmed (`/api/v1/traffic/preset/game-social-separation`)
* Paket Game Online (Mobile Legends, PUBG, Free Fire, Valorant) diprioritaskan di Jalur 1, sementara streaming dan medsos dialihkan ke Prioritas 8.

### 11. 🌐 Visualizer Topologi Interaktif + Embeddable SDK (`/topology` & `/sdk/mikrotik-widget.js`)
* Dashboard visual graf relasi topologi real-time bergaya Obsidian Dark Mode. Dilengkapi fitur **Live ICMP Ping** langsung dari router ke target IP. Dapat di-embed ke Laravel/Vue dengan 1 baris JavaScript.

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

Server langsung aktif di `http://localhost:8080`.
* Buka **Live Web Playground**: `http://localhost:8080/`
* Buka **Visualizer Topologi**: `http://localhost:8080/topology`
* Cek **Health Check**: `http://localhost:8080/health`

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
* **[04. Referensi Lengkap Universal API (192 Endpoints)](docs/04-universal-api-reference.md)**
* **[05. Panduan Integrasi Multi-Stack (Laravel, Vue, Express, Flutter, Go)](docs/05-integrasi-multi-stack.md)**
* **[06. Katalog Lengkap Command MikroTik](docs/06-katalog-command-mikrotik.md)**

---

## 🧪 Koleksi Pengujian Siap Pakai

* **[`collections/mikrotik-gateway.http`](collections/mikrotik-gateway.http)**: 49 skenario pengujian instan di VS Code REST Client atau JetBrains HTTP Client.
* **[`collections/mikrotik_gateway.postman_collection.json`](collections/mikrotik_gateway.postman_collection.json)**: Siap di-import ke Postman, Insomnia, atau Bruno.

---

## 📄 Lisensi
Proyek ini dilisensikan di bawah lisensi **MIT License**.
Dikembangkan untuk komunitas Network Operations Center (NOC) dan Software Developers modern.

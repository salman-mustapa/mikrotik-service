# MikroTik Rust Core & API Gateway

Proyek ini adalah core interaksi MikroTik RouterOS berkinerja tinggi yang ditulis menggunakan **Rust** (`tokio` async). Proyek ini dirancang agar developer di berbagai teknologi (**Laravel, Node.js, Python, Go, Vue, React**) dapat berkomunikasi dengan router MikroTik secara cepat, aman, dan tanpa beban koneksi berulang (*zero connection churn*).

---

## 📑 Daftar Isi Dokumentasi

Semua dokumentasi lengkap tersimpan di folder [`docs/`](file:///d:/MyPorto/mikrotik/docs):

1. **[01. Arsitektur & Alur Kerja Sistem](file:///d:/MyPorto/mikrotik/docs/01-arsitektur-dan-alur.md)**
   - Perbandingan arsitektur konvensional (PHP/Mikhmon) vs Rust Gateway.
   - Diagram alur Mermaid: Cara kerja persistent connection pool & multiplexing tag `.tag`.
   - Mengapa satu socket bisa melayani ratusan request paralel tanpa tabrakan.

2. **[02. Protokol Biner RouterOS (Port 8728)](file:///d:/MyPorto/mikrotik/docs/02-protokol-routeros-binary.md)**
   - Format biner *Length-prefix* (1-5 byte).
   - Struktur *Word* (`=`, `?`, `.`) dan *Sentence* (`!re`, `!done`, `!trap`, `!fatal`).
   - Diagram login otomatis: Kompatibilitas RouterOS v6 (MD5 challenge) & RouterOS v7.

3. **[03. Spesifikasi Gateway REST & SSE API](file:///d:/MyPorto/mikrotik/docs/03-gateway-api-spesifikasi.md)**
   - Format endpoint `POST /routers/:id/command` & `GET /routers/:id/listen`.
   - Header Bearer Token, aturan passing arguments, dan status error code.

4. **[04. Panduan Integrasi Multi-Stack](file:///d:/MyPorto/mikrotik/docs/04-panduan-integrasi-multi-stack.md)**
   - Contoh implementasi Service di **Laravel (PHP)**.
   - Contoh helper client di **Node.js / TypeScript**.
   - Contoh widget live monitoring traffic di **Vue 3** menggunakan `EventSource` (SSE).

5. **[05. Panduan Lengkap Laravel Service & Controller](file:///d:/MyPorto/mikrotik/docs/05-panduan-lengkap-laravel.md)**
   - Class Service lengkap `MikrotikService.php` siap copy-paste.
   - Method lengkap: Hotspot Voucher, Kick User, PPPoE, DHCP Leases, Simple Queue, Firewall Block, dan System Resource.
   - Contoh implementasi Controller Dashboard & Hotspot.

6. **[06. Katalog Lengkap Command MikroTik](file:///d:/MyPorto/mikrotik/docs/06-katalog-command-mikrotik.md)**
   - Kamus referensi semua command MikroTik RouterOS API beserta filter query dan atributnya.

---

## 🚀 API Collections untuk Pengujian

Semua file collection pengujian sudah siap di folder [`collections/`](file:///d:/MyPorto/mikrotik/collections):
- **[`collections/mikrotik-gateway.http`](file:///d:/MyPorto/mikrotik/collections/mikrotik-gateway.http)**: Dapat langsung dijalankan di VS Code (ekstensi REST Client) atau JetBrains HTTP Client.
- **[`collections/mikrotik_gateway.postman_collection.json`](file:///d:/MyPorto/mikrotik/collections/mikrotik_gateway.postman_collection.json)**: Siap di-import langsung ke Postman, Insomnia, atau Bruno.

---

## 🛠️ Persiapan Lingkungan (Setup Rust di Windows)

Sebelum menjalankan core Rust ini di laptop/PC Anda:

### 1. Pasang Rust Toolchain
Buka PowerShell dan jalankan:
```powershell
winget install Rustlang.Rustup
```
*Atau unduh installer `rustup-init.exe` langsung dari [rustup.rs](https://rustup.rs).*

Setelah selesai, restart terminal Anda lalu pastikan instalasi berhasil:
```powershell
rustc --version
cargo --version
```

### 2. Konfigurasi Gateway
Salin file konfigurasi contoh:
```powershell
Copy-Item config.example.toml config.toml
```
Sesuaikan IP router, username, password, dan token autentikasi pada `config.toml`.

### 3. Menjalankan Gateway
```powershell
# Jalankan HTTP Gateway Daemon
cargo run -p routeros-gateway -- config.toml
```
Server akan aktif di `http://127.0.0.1:8080`.

### 4. Menguji Core CLI Langsung (PoC)
Jika ingin menguji koneksi langsung tanpa melalui HTTP gateway:
```powershell
cargo run -p routeros-core --example poc -- 192.168.88.1:8728 admin "password123"
```
CLI ini akan login, membaca resource router, lalu streaming traffic `ether1` selama 5 detik kemudian otomatis mengirim sinyal `/cancel`.

# Changelog

Format mengikuti [Keep a Changelog](https://keepachangelog.com/id-ID/1.1.0/).

## [Unreleased] – 2026-10-17

### Ditambahkan — Network Map siap pakai
- **Diverifikasi** dengan unit test (31), biner asli (`curl` + WebSocket) dan Chrome headless terhadap RouterOS tiruan; belum diuji pada router fisik.
- Nama bawaan di peta dibersihkan (tanpa emoji/awalan/IP; IP tetap di field `ip`), ikon AP/repeater terdeteksi dari hostname, tampilan "Fit" diberi margin agar node di tepi tidak terpotong.
- Halaman `/map` dan web component `<mikrotik-map>` (`/sdk/mikrotik-map.js`) yang bisa di-embed di aplikasi developer.
- Penempatan alat di titik tertentu pada denah/satelit (background PNG/JPEG/WebP), drag & drop, kunci posisi, auto-place.
- Ikon alat terdeteksi otomatis dan dapat diubah manual; override label, parent, dan ikon tersimpan per layout.
- Jalur kabel dengan waypoint (mengikuti jalan / melintasi atap) dan animasi paket mengikuti jalur.
- Status warna: hijau = terhubung, merah = gagal / limit queue tercapai, kuning = warning / trafik tinggi.
- Download/upload realtime dalam KB/s–MB/s dari selisih counter interface.
- Deteksi limit queue per titik/user dan relay/estafet (AP → AP 2 → klien).
- REST `/api/v1/map/*` (scene, icons, layouts, nodes, paths, auto-place, background) dan aksi WebSocket `subscribe_map` (event `map_frame`).
- Penyimpanan layout persisten per-tenant (`GATEWAY_DATA_DIR`, default `./data`; Docker volume `/data`) dengan penulisan atomik dan batas ukuran/jumlah.
- **Viewer token** read-only (`GATEWAY_VIEWER_TOKEN`, min. 16 karakter, berbeda dari admin token): hanya GET map + WebSocket `subscribe_map`; header kredensial router dari viewer diabaikan.
- Dokumentasi: `docs/00-outline-dokumentasi.md` (outline & peta dokumentasi) dan `docs/07-network-map-visualizer.md`.

### Diubah
- `/topology` tidak lagi mengarang perangkat (AP ether5 dan gateway 192.168.1.1 palsu dihapus); hanya data nyata dari router.
- Dockerfile/docker-compose: volume `gateway-data` dan `GATEWAY_DATA_DIR=/data`; `.gitignore` mengabaikan `/data/`.
- Auth middleware berbasis peran (Admin / Viewer); `/ws` melakukan autentikasi sendiri dan membatasi aksi viewer.
- Error API baru: `409 Conflict`, `413 Payload Too Large`, `415 Unsupported Media Type`, `403 Forbidden`.

### Keamanan
- SVG ditolak sebagai background (mencegah script injection); tipe gambar divalidasi lewat magic bytes.
- Widget tidak memakai `innerHTML`; seluruh teks dinamis lewat `textContent`; halaman `/map` kompatibel CSP (tanpa inline script).
- Token deep link (`#token=`) dibersihkan dari URL dan disimpan di `sessionStorage`.
- Isolasi penyimpanan per-tenant (hash tenant sebagai nama direktori).

## 2026-10-17 — commit `e49da2b`
### Keamanan
- Patch temuan kritis, penguatan autentikasi, dan penambahan pipeline CI (clippy, test, build).

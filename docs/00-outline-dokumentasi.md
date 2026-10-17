# 00 — Outline & Peta Dokumentasi

> Dokumen induk: daftar seluruh dokumentasi proyek, siapa yang perlu membaca apa, dan **status** tiap bab.
> Dokumen ini harus selalu diperbarui bersama perubahan fitur (lihat checklist di bagian paling bawah).

**Terakhir diperbarui:** 2026-10-17 · **Versi dokumentasi:** 0.3.0 (Network Map)

---

## 1. Peta pembaca

| Anda adalah… | Mulai dari | Lanjut ke |
|---|---|---|
| Developer front-end / mobile yang ingin **menampilkan peta jaringan** | [07 — Network Map](07-network-map-visualizer.md) §2 *Quick start* | §6 Atribut widget, §9 Kode warna |
| Developer backend yang ingin memanggil API | [03 — Gateway API](03-gateway-api-spesifikasi.md) | [04 — Universal API Reference](04-universal-api-reference.md) |
| Integrator multi-stack (Laravel, Node, Go, Flutter, dll.) | [05 — Integrasi Multi-Stack](05-integrasi-multi-stack.md) | [04 — Panduan integrasi](04-panduan-integrasi-multi-stack.md) |
| Network engineer / NOC | [05 — Entity Relationship](05-network-entity-relationships-guide.md) | [06 — Katalog command](06-katalog-command-mikrotik.md) |
| DevOps / security reviewer | [07 — Network Map](07-network-map-visualizer.md) §12 *Keamanan* | [01 — Arsitektur](01-arsitektur-dan-alur.md) |
| Kontributor | [01 — Arsitektur](01-arsitektur-dan-alur.md) | [../CHANGELOG.md](../CHANGELOG.md) |

## 2. Daftar dokumen

| No | File | Isi | Status |
|---|---|---|---|
| 00 | `00-outline-dokumentasi.md` | Dokumen ini | ✅ terkini |
| 01 | [`01-arsitektur-dan-alur.md`](01-arsitektur-dan-alur.md) | Arsitektur crate, alur request, pooling koneksi | ✅ (ditambah: bagian Network Map & `data_dir`) |
| 02 | [`02-protokol-routeros-binary.md`](02-protokol-routeros-binary.md) | Protokol API biner RouterOS | ✅ |
| 03 | [`03-gateway-api-spesifikasi.md`](03-gateway-api-spesifikasi.md) | Kontrak HTTP umum, auth, error | ✅ (ditambah: peran `viewer`) |
| 04a | [`04-panduan-integrasi-multi-stack.md`](04-panduan-integrasi-multi-stack.md) | Panduan ringkas integrasi | ✅ |
| 04b | [`04-universal-api-reference.md`](04-universal-api-reference.md) | Referensi seluruh endpoint | ✅ (ditambah: `/api/v1/map/*`) |
| 05a | [`05-integrasi-multi-stack.md`](05-integrasi-multi-stack.md) | Contoh kode per stack | ✅ |
| 05b | [`05-network-entity-relationships-guide.md`](05-network-entity-relationships-guide.md) | Hubungan entitas jaringan | ✅ |
| 06 | [`06-katalog-command-mikrotik.md`](06-katalog-command-mikrotik.md) | Katalog command RouterOS | ✅ |
| **07** | [`07-network-map-visualizer.md`](07-network-map-visualizer.md) | **Peta jaringan siap pakai** (widget, REST, WebSocket, editor, keamanan) | ✅ **baru** |

## 3. Outline bab 07 — Network Map (konsep → implementasi)

Urutan ini sengaja dari “apa yang developer lihat” ke “bagaimana ia bekerja”.

1. **Gambaran & tujuan** — developer cukup memasang satu tag; operator menempatkan alat di peta.
2. **Quick start** — 3 skenario: halaman `/map`, tag `<mikrotik-map>`, mode demo tanpa router.
3. **Konsep inti** — *layout* (yang disimpan) vs *scene* (yang dihitung live); koordinat ternormalisasi 0–1.
4. **Alur data** — router → topologi → counter → laju (KB/s, MB/s) → kesehatan → scene → widget.
5. **Menempatkan alat** — drag, auto-place, kunci posisi, ikon otomatis & ganti manual.
6. **Atribut, properti, event widget** — referensi lengkap.
7. **Jalur kabel** — waypoint mengikuti jalan, melintasi atap, gaya garis (straight/orthogonal/smooth).
8. **Estafet / relay** — AP → AP2 → client, `parent_override`, deteksi loop.
9. **Kode warna & aturan status** — hijau/kuning/merah/abu, animasi paket, ambang.
10. **Batas queue (limit)** — deteksi dari `/queue/simple`, badge `LIMIT`, titik merah.
11. **Trafik realtime** — satuan B/s · KB/s · MB/s, download/upload, paket per detik, total byte.
12. **Keamanan** — token `viewer` read-only, XSS, upload gambar, CSP, isolasi tenant, batas ukuran.
13. **Referensi REST** — tiap endpoint + contoh `curl` + kode error.
14. **Referensi WebSocket** — `subscribe_map`, `map_frame`, retry.
15. **Skema scene** — field per field.
16. **Deploy & operasional** — `data_dir`, volume Docker, backup, batas.
17. **Troubleshooting & FAQ**.
18. **Keterbatasan yang diketahui** (jujur, dengan rencana mitigasi).

## 4. Konvensi dokumentasi

- Bahasa narasi: Indonesia. Kode, nama field, dan pesan error: Inggris (sama seperti respons API).
- Semua contoh `curl` memakai `GW=http://127.0.0.1:8080` dan `TOKEN=...`.
- Setiap endpoint dicantumkan: metode, path, peran minimum (`admin`/`viewer`), body, respons, error.
- Angka batas (ukuran, jumlah) **diambil langsung dari kode**; jika berubah, ubah di dokumen ini dan di bab 07 §16.

## 5. Checklist pembaruan (jalankan di akhir setiap perubahan fitur)

- [x] `CHANGELOG.md` — entri versi + tanggal.
- [x] `README.md` — ringkasan fitur & tautan portal (`/map`).
- [x] `docs/07-network-map-visualizer.md` — angka batas, field scene, contoh respons.
- [x] `docs/04-universal-api-reference.md` — endpoint baru/berubah (§AAA).
- [x] `docs/01` & `docs/03` — klaim "tanpa penyimpanan disk" dikoreksi; peran viewer & `subscribe_map` ditambahkan.
- [x] `docs/00-outline-dokumentasi.md` — kolom **Status** & tanggal di atas.
- [x] `config.example.toml` & `docker-compose.yml` — opsi/volume baru.
- [x] Playground & navigasi portal (`routes/docs.rs`) — endpoint map + tautan `/map`.

## 6. Catatan verifikasi (apa yang benar-benar sudah diuji)

| Area | Cara uji | Hasil |
|---|---|---|
| Logika scene, status, limit queue, relay, ikon, label | Unit test (`cargo test --workspace`) | ✅ lulus |
| Auth/role (401/403), validasi input, batas ukuran (413), tolak SVG/HTML (415), path traversal (400), konflik versi (409) | Biner asli + `curl` | ✅ sesuai |
| WebSocket `auth` / `forbidden` untuk viewer / `subscribe_map` | Biner asli + klien WebSocket | ✅ sesuai |
| Render widget (demo) & data live | Chrome headless + RouterOS **tiruan** (protokol API biner) | ✅ tampil benar |
| Router MikroTik **fisik** | — | ⚠️ **belum** diuji; field yang tidak ada di firmware tertentu tampil kosong, bukan dikarang |
| Interaksi drag/waypoint di browser | Ditinjau lewat kode + screenshot mode edit | ⚠️ uji manual disarankan sebelum produksi |

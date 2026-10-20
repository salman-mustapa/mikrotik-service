# 07 — Network Map: peta jaringan siap pakai

Peta jaringan **live** yang bisa dipasang developer hanya dengan satu tag HTML. Operator menaruh alat
di titik tertentu pada denah rumah / citra satelit; peta menampilkan ikon alat (terdeteksi otomatis,
bisa diganti), status koneksi, animasi paket yang berjalan di sepanjang kabel (bisa berbelok mengikuti jalan
atau melintasi atap), batas queue, dan kecepatan download/upload dalam B/s · KB/s · MB/s secara realtime.

> Peta ini **bukan** pengganti `/topology` (visualizer graf otomatis). `/topology` menggambar graf apa adanya;
> **`/map` adalah peta yang bisa Anda tata** dan tersimpan permanen di gateway.

- [1. Gambaran](#1-gambaran)
- [2. Quick start](#2-quick-start)
- [3. Konsep inti: layout vs scene](#3-konsep-inti-layout-vs-scene)
- [4. Alur data](#4-alur-data)
- [5. Menempatkan alat](#5-menempatkan-alat)
- [6. Atribut, properti & event widget](#6-atribut-properti--event-widget)
- [7. Jalur kabel (jalan, atap)](#7-jalur-kabel-jalan-atap)
- [8. Estafet / relay](#8-estafet--relay-ap--ap2--client)
- [9. Kode warna & aturan status](#9-kode-warna--aturan-status)
- [10. Batas queue (limit)](#10-batas-queue-limit)
- [11. Trafik realtime](#11-trafik-realtime)
- [12. Keamanan](#12-keamanan)
- [13. Referensi REST](#13-referensi-rest)
- [14. Referensi WebSocket](#14-referensi-websocket)
- [15. Skema scene](#15-skema-scene)
- [16. Deploy & operasional](#16-deploy--operasional)
- [17. Troubleshooting & FAQ](#17-troubleshooting--faq)
- [18. Keterbatasan yang diketahui](#18-keterbatasan-yang-diketahui)

---

## 1. Gambaran

| Kebutuhan | Cara penyelesaiannya |
|---|---|
| “Developer tinggal pakai” | Satu `<script>` + satu tag `<mikrotik-map>`; tanpa framework, tanpa build, tanpa dependensi. |
| Ikon alat otomatis, tetap bisa diubah | Deteksi dari tipe node + vendor/hostname (§5). Override per alat, tersimpan di layout. |
| Status & warna | Hijau = tersambung, merah = gagal / limit tercapai, kuning = peringatan, abu = idle (§9). |
| Animasi paket berjalan | Titik cyan (download) dan ungu (upload) bergerak di sepanjang garis; jumlah & kecepatan mengikuti trafik. |
| Paket tinggi / peringatan | `high_pps`, `high_traffic`, `drops`, `limit_near` → kuning; `limit_reached`, `link_down` → merah. |
| Batas queue terdeteksi | Dari `/queue/simple`: badge `LIMIT`/`NEAR` di titik, garis klien menjadi merah (§10). |
| Download/upload realtime | `↓ 1.17 MB/s  ↑ 128.0 KB/s` di tiap titik, rincian bit/paket/total di tooltip (§11). |
| Peta denah / satelit | Upload PNG/JPEG/WebP (≤ 4 MB) atau URL gambar. Posisi disimpan sebagai persen → ganti gambar tanpa kehilangan tata letak. |
| Kabel mengikuti jalan / atap | Waypoint per link + gaya garis `straight`/`orthogonal`/`smooth` (§7). |
| Estafet AP → AP2 → client | `parent_override` + edge bertipe `relay`; loop dideteksi & ditolak (§8). |

## 2. Quick start

### A. Halaman siap pakai `/map`

Buka `http://GATEWAY:8080/map`, isi **token** dan **router id** (atau host + user + password), lalu **Connect**.
Tidak punya router? Klik **Try with sample data**.

Deep link (hash tidak pernah dikirim ke server maupun tercatat di log):

```
http://GATEWAY:8080/map#token=XXXX&layout=rumah&router_id=main&mode=edit
http://GATEWAY:8080/map#demo=1
```

### B. Tag `<mikrotik-map>` di aplikasi Anda

```html
<script src="https://GATEWAY/sdk/mikrotik-map.js"></script>

<mikrotik-map
  base-url="https://GATEWAY"
  token="VIEWER_TOKEN"
  layout-id="rumah"
  router-id="main"
  style="height:640px">
</mikrotik-map>
```

Itu saja. Untuk membiarkan operator menata peta, tambahkan `mode="edit"` dan gunakan **token admin** (lihat §12:
jangan menaruh token admin di front-end publik).

React / Vue / Flutter WebView memakai tag yang sama. Contoh React (token diberikan lewat properti, bukan atribut DOM):

```jsx
useEffect(() => {
  ref.current.config = { baseUrl: GW, token: VIEWER_TOKEN, layoutId: 'rumah', routerId: 'main' };
  const onAlert = (e) => console.warn('ALERT', e.detail.message);
  ref.current.addEventListener('alert', onAlert);
  return () => ref.current.removeEventListener('alert', onAlert);
}, []);
return <mikrotik-map ref={ref} style={{ height: 640 }} />;
```

### C. Mode demo (tanpa router, tanpa token)

```html
<script src="https://GATEWAY/sdk/mikrotik-map.js"></script>
<mikrotik-map demo style="height:520px"></mikrotik-map>
```

Menampilkan rumah contoh: relay AP→AP, CCTV yang mencapai limit (merah), laptop mendekati limit (kuning),
perangkat idle (abu), jalur kabel berbelok & melintasi atap.

### D. Panduan Lengkap Integrasi Developer Eksternal (Custom App / Mobile / Dashboard)

Peta jaringan ini **didesain penuh untuk digunakan oleh developer di luar project ini**, baik dengan menyematkan komponen web siap pakai maupun mengonsumsi data mentah (JSON scene) untuk dirender di canvas kustom Anda sendiri (misal React, Vue, Next.js, Flutter, D3.js, Cytoscape, atau Vis.js).

#### 1. Alur Autentikasi Aman untuk Frontend (Ephemeral Connection Ticket)
Agar password router tidak pernah terekspos di browser client:
1. Backend Anda memanggil gateway untuk membuat tiket sesi:
   ```bash
   curl -X POST "http://GATEWAY:8080/api/v1/ticket/issue" \
     -H "Content-Type: application/json" \
     -d '{
       "host": "192.168.88.1",
       "port": 8728,
       "user": "admin",
       "pass": "password_router",
       "ttl_secs": 3600
     }'
   # Response: {"success": true, "ticket": "conn_xyz123...", "expires_in_secs": 3600}
   ```
2. Frontend Anda (React / Vue / Mobile) cukup menggunakan `ticket` tersebut:
   ```html
   <script src="http://GATEWAY:8080/sdk/mikrotik-map.js"></script>
   <mikrotik-map base-url="http://GATEWAY:8080" ticket="conn_xyz123..." layout-id="default" style="height:640px"></mikrotik-map>
   ```

#### 2. Konsumsi REST API Langsung (Untuk Render di Canvas / Mobile Anda Sendiri)
Jika developer ingin membuat visualisasi sendiri tanpa Web Component bawaan:
- **Endpoint**: `GET /api/v1/map/scene?layout_id=default`
- **Headers / Query**:
  - `X-Connection-Ticket: conn_xyz123...` (atau `?ticket=...`)
  - Atau kredensial langsung: `X-Router-Host`, `X-Router-User`, `X-Router-Pass`
- **Data yang Dikembalikan (JSON)**:
  - `nodes`: Array perangkat (`id`, `label`, `ip`, `mac`, `vendor`, `position: {x, y}`, `status`, `traffic: {rx_bps, tx_bps}`, `queue_limit`).
  - `edges`: Array jalur kabel (`source`, `target`, `style`, `waypoints: [{x, y}]`, `traffic: {bps}`).
  - `alerts`: Array peringatan aktif (link putus, limit queue tercapai).
  - `summary`: Ringkasan total bandwidth WAN, node sehat/kritis.

#### 3. Realtime Streaming via WebSocket (ws://GATEWAY/ws)
Untuk pembaruan realtime detik-demi-detik tanpa overhead polling HTTP berulang:
```javascript
const ws = new WebSocket('ws://GATEWAY:8080/ws?ticket=conn_xyz123...');
ws.onopen = () => {
  ws.send(JSON.stringify({
    action: "subscribe_map",
    layout_id: "default",
    interval_ms: 2000 // Frekuensi frame realtime (1000 - 60000 ms)
  }));
};
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.event === "map_frame") {
    const scene = msg.data;
    // Hubungkan data scene ke engine visualizer Anda (D3, Vis.js, Pixi, dll)
    renderMyCustomTopology(scene.nodes, scene.edges, scene.alerts);
  }
};
```

#### 4. Kustomisasi Posisi & Kabel Programmatik via REST
Developer dapat mengedit posisi atau jalur kabel lewat API:
- Ubah posisi atau ikon node: `PUT /api/v1/map/layouts/:id/nodes/:node_id`
- Ubah belokan kabel (waypoint): `PUT /api/v1/map/layouts/:id/paths/:edge_id`
- Upload gambar denah kantor: `PUT /api/v1/map/layouts/:id/background` (PNG/JPG maks 4 MB)
- Auto-place bersih: `POST /api/v1/map/layouts/:id/auto-place`

---

## 3. Konsep inti: layout vs scene

| | **Layout** | **Scene** |
|---|---|---|
| Apa | Hasil kerja operator | Hasil hitungan live |
| Isi | posisi, ikon, nama, kunci, parent override, waypoint, kapasitas, ambang, background | status, laju, queue, alert, warna, intensitas animasi |
| Disimpan | Ya — file JSON di `data_dir` | Tidak — dihitung tiap tick |
| Diubah via | `PUT/DELETE /api/v1/map/layouts/...` atau editor di widget | `GET /api/v1/map/scene` atau WebSocket `subscribe_map` |

Koordinat selalu **ternormalisasi 0..1** terhadap gambar latar (`x` kiri→kanan, `y` atas→bawah). Karena itu layout tetap
sah walau gambar latar diganti ukurannya.

Satu router bisa punya banyak layout (`lantai-1`, `lantai-2`, `satelit`). `layout-id` = `[a-z0-9_-]{1,64}`.
Layout yang belum ada tetap bisa dipakai: semua node memakai posisi **auto**, dan layout dibuat saat pertama kali Anda
menempatkan satu alat.

## 4. Alur data

```
RouterOS ──API biner──► gateway
   │  /interface/print (rx/tx byte, paket, drop, running)
   │  /queue/simple/print (max-limit, rate, bytes, dropped)
   │  topologi (DHCP, hotspot, PPP, neighbor, bridge, dst.)
   ▼
build_topology()  ─┐
fetch_counters()  ─┼─► compose_scene() ──► scene JSON ──► REST / WebSocket ──► <mikrotik-map>
layout (disk)     ─┘     (fungsi murni, diuji unit)
```

- **Laju = selisih counter** antara dua sampel, dibagi selisih waktu. Sampel pertama belum punya laju
  (`rate_ready:false`; widget menampilkan 0 sampai sampel kedua). Counter yang mundur (router reboot) menghasilkan **0**, bukan angka negatif/raksasa.
- Dua konsumen yang meminta dalam jarak < 500 ms memakai ulang laju sebelumnya (tidak membagi dengan waktu ≈ 0).
- Struktur topologi di-*cache* 20 detik pada koneksi WebSocket; counter & queue dibaca ulang tiap tick (default 2 detik, 1–60 detik).
- Tidak ada data karangan: perangkat/gateway yang tidak ditemukan router **tidak** dibuat-buat. Kapasitas yang tidak diketahui
  ditandai `unknown`, bukan ditebak.

## 5. Menempatkan alat

Aktifkan **Edit** (atau `mode="edit"`) lalu:

| Aksi | Cara |
|---|---|
| Pindah alat | Drag titik. Tersimpan otomatis begitu dilepas. |
| Kunci posisi | Inspector → *Lock position* (alat terkunci tidak bisa di-drag). |
| Ganti nama tampilan | Inspector → *Display name* (kosong = nama dari router). |
| Ganti ikon | Inspector → *Icon*. “Auto-detect” mengembalikan ikon otomatis. |
| Isi posisi awal | Toolbar → **Auto-place** (hanya alat yang belum ditempatkan) atau **Re-layout** (semua yang tidak terkunci). |
| Kembalikan ke otomatis | Inspector → *Reset position*. |
| Latar peta | Toolbar → **Background** → upload gambar atau URL. |
| Ambang peringatan | Toolbar → **Thresholds**. |
| Cadangan / pindah server | Toolbar → **Export** / **Import** (JSON). |

### Ikon otomatis

Urutan keputusan (kecocokan pertama menang):

1. Tipe node: `internet`→`internet`; `router`→`router`; `interface` WAN→`wan-port`, bridge→`switch`, lainnya→`port`; `ap`→`ap` (jika induknya juga AP→`repeater`).
2. Perangkat klien, dicocokkan dari **hostname + vendor (OUI) + username**:

| Ikon | Kata kunci contoh |
|---|---|
| `camera` | hikvision, dahua, ezviz, reolink, cctv, ipcam, cam, nvr, dvr |
| `game-console` | playstation, xbox, nintendo, ps4, ps5 |
| `printer` | printer, laserjet, epson, brother, canon |
| `tv` | bravia, webos, tizen, roku, chromecast, tv |
| `tablet` | ipad, tablet, tab |
| `laptop` | macbook, thinkpad, laptop, notebook, ideapad, vivobook, zenbook, surface |
| `desktop` | desktop, pc, workstation |
| `server` | synology, qnap, proxmox, vmware, server |
| `iot` | espressif, esp32, esp8266, tuya, shelly, sonoff, yeelight, smartplug |
| `phone` | iphone, android, galaxy, samsung, xiaomi, redmi, oppo, vivo, huawei, realme, pixel, apple |

3. Jika tidak ada yang cocok: sub-tipe `pppoe`→`pppoe-user`, `hotspot`→`hotspot-user`, selainnya `unknown`.

Deteksi adalah **heuristik** — karena itu selalu bisa di-override. Daftar ikon resmi: `GET /api/v1/map/icons`
(`internet, router, switch, port, wan-port, ap, repeater, server, phone, tablet, laptop, desktop, tv, printer, camera, iot, game-console, hotspot-user, pppoe-user, unknown`).

## 6. Atribut, properti & event widget

### Atribut

| Atribut | Default | Keterangan |
|---|---|---|
| `base-url` | origin file script | Alamat gateway. |
| `token` | — (wajib) | Bearer token. Gunakan token **viewer** untuk tampilan. |
| `layout-id` | `default` | Layout yang dipakai. |
| `router-id` | — | Id router dari `config.toml`. |
| `router-host` `router-port` `router-user` `router-pass` | — | Router langsung (hanya token admin). |
| `tenant-id` | — | Konteks tenant (sama dengan header `X-Tenant-Id`). |
| `mode` | `view` | `view` atau `edit`. |
| `editable` | `true` | `false` menyembunyikan tombol Edit. |
| `interval` | `2000` | Periode refresh ms (1000–60000). |
| `transport` | `auto` | `auto` = WebSocket lalu jatuh ke polling bila WebSocket gagal 3× berturut-turut; `ws`; `poll`. |
| `filter` | `all` | `all`, `hotspot`, `pppoe`, `dhcp`, `wifi`, `infra`. |
| `theme` | `dark` | `dark` / `light`. |
| `demo` | — | Atribut boolean: data contoh, tanpa jaringan. |

Alternatif atribut (menghindari token di DOM): `el.config = { baseUrl, token, layoutId, routerId, router:{host,port,user,password}, tenantId, mode, interval, transport, filter, theme }`.

### Metode & properti

`el.scene` (scene terakhir) · `el.refresh()` · `el.fit()` · `el.setMode('edit'|'view')` · `el.selectNode(id)` · `el.config` (get/set).

### Event (`CustomEvent`, data di `e.detail`)

| Event | `detail` | Kapan |
|---|---|---|
| `scene` | scene lengkap (§15) | Tiap frame |
| `alert` | `{id, severity, kind, node_id, label, message}` | **Sekali** saat alert baru muncul |
| `node-select` | node | Klik pada titik |
| `status` | `{status, message}` | `connecting` · `live` · `stale` · `error` · `demo` (hanya saat berubah) |

Contoh: kirim notifikasi push saat limit tercapai.

```js
map.addEventListener('alert', (e) => {
  if (e.detail.kind === 'limit_reached') notify(`${e.detail.label}: ${e.detail.message}`);
});
```

Interaksi: roda mouse = zoom, drag latar = geser, **Esc** = batal pilih, klik pada daftar *Alerts* = zoom ke alat tersebut.
Status `stale` (kelabu) muncul bila tidak ada frame selama > 3× interval (min. 6 detik) — peta tidak berpura-pura “live”.

## 7. Jalur kabel (jalan, atap)

Setiap link (edge) boleh punya **waypoint**: titik belok antara dua alat. Dengan waypoint, kabel mengikuti
tikungan jalan, melewati atap, atau menghindari bangunan; animasi paket mengikuti garis yang sama.

Di mode edit: klik link → **double-click pada garis** untuk menambah waypoint → drag *handle* putih →
**Alt+klik** atau double-click handle untuk menghapus → *Clear waypoints* untuk mengosongkan.

Gaya garis (`style`):

| Gaya | Perilaku |
|---|---|
| `straight` | Garis lurus antar titik (default). |
| `orthogonal` | Siku-siku (horizontal lalu vertikal) — cocok untuk jalur kabel di pinggir jalan. |
| `smooth` | Kurva halus (Catmull-Rom) melewati semua waypoint. |

Via API: `PUT /api/v1/map/layouts/{id}/paths/{edge_id}`

```json
{ "waypoints": [{"x":0.43,"y":0.20},{"x":0.64,"y":0.08}], "style": "smooth", "capacity_bps": 100000000 }
```

Maksimum 64 waypoint per link. `edge_id` diambil dari `scene.edges[].id`.

## 8. Estafet / relay (AP → AP2 → client)

Topologi otomatis menaruh AP di bawah port router. Untuk relay (mis. AP1 memancarkan ke AP2 yang melayani banyak klien)
atur **parent override** pada AP2 (Inspector → *Connected to*) atau lewat API:

```bash
curl -X PUT "$GW/api/v1/map/layouts/rumah/nodes/ap_ether5_AA" \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"x":0.82,"y":0.16,"parent_override":"ap_ether5_BB"}'
```

Hasilnya:

- node AP2 berinduk AP1 (`parent_source:"override"`), ikonnya menjadi `repeater`;
- edge lama ke router dihapus, edge baru `edge_{induk}_{anak}` bertipe **`relay`** (garis putus-putus);
- trafik AP = jumlah trafik seluruh klien turunannya; `subtree` menghitung berapa turunan yang `warning`/`critical`
  (badge angka merah/kuning di AP);
- berlaku berantai tanpa batas (AP→AP→AP→klien) dan klien bisa dipindah ke AP tertentu dengan cara sama;
- **loop ditolak**: override yang membentuk siklus diabaikan dan muncul di `scene.warnings`.

## 9. Kode warna & aturan status

| Warna | `health` | Arti |
|---|---|---|
| 🟢 hijau `#22c55e` | `ok` | Tersambung dan dalam batas normal. |
| 🟡 kuning `#f59e0b` | `warning` | Mendekati limit, paket/throughput tinggi, ada paket drop. |
| 🔴 merah `#ef4444` (berdenyut) | `critical` | Link mati / WAN putus / limit tercapai. |
| ⚪ abu `#64748b` | `idle` | Port tanpa link & tanpa anak, atau klien tidak aktif. |

Alasan (`reasons[].kind`) dan tingkatnya:

| kind | Level | Pemicu |
|---|---|---|
| `link_down` | critical | Interface mati dan merupakan WAN atau punya perangkat di bawahnya. |
| `wan_down` | critical | Status ISP bukan `online`. |
| `limit_reached` | critical | Utilisasi ≥ `crit_pct` (default 95%) dari limit queue / kapasitas link. |
| `limit_near` | warning | Utilisasi ≥ `warn_pct` (default 80%). |
| `high_pps` | warning | Paket/detik ≥ `high_pps` (default 50.000). |
| `high_traffic` | warning | Throughput ≥ `high_bps` (opsional). |
| `drops` | warning | Ada paket di-drop sejak sampel sebelumnya. |

Aturan pewarnaan garis:

- Garis mengikuti kesehatan **node tujuan**.
- Klien yang terkena limit mewarnai **cabangnya sendiri**, bukan kabel utama atau AP-nya (AP hanya menampilkan badge jumlah).
- Interface/WAN yang kritis mewarnai garisnya sendiri dan semua garis di jalurnya.

Animasi: titik **cyan = download**, **ungu = upload**. Jumlah (1–8) dan kecepatan titik naik dengan intensitas
`log10(1+bps)/9`. Arah titik mengikuti arah data: pada kabel WAN, download mengalir **menuju router**
(`flow.down_direction:"reverse"` pada edge router→WAN). Pengguna dengan `prefers-reduced-motion` tidak melihat titik bergerak (garis & angka tetap).

## 10. Batas queue (limit)

Untuk setiap klien, gateway mencari **simple queue paling spesifik** yang menarget IP-nya (`/32` mengalahkan `/24`).
Dari situ:

- `queue.max_down_bps / max_up_bps` dan versi manusia (`10.00 Mbps`),
- `queue.down_util_pct / up_util_pct` = laju saat ini ÷ `max-limit`,
- `queue.limit_state`: `ok` · `near` (≥ warn) · **`reached`** (≥ crit) · `unlimited` (queue tanpa max-limit),
- `dropped_down / dropped_up`.

Di peta: badge **LIMIT** (merah) / **NEAR** (kuning) di atas titik, halo merah berdenyut, garis ke klien merah,
dan entri di daftar *Alerts*. Queue yang `disabled` diabaikan. Klien tanpa queue memakai selisih counter
(`bytes_in/bytes_out`, mis. hotspot) sebagai laju, tanpa limit.

## 11. Trafik realtime

Setiap node/edge membawa `traffic`:

```json
{
  "source": "interface_counter",
  "down_bps": 9437184, "up_bps": 1048576,
  "down_human": "1.12 MB/s", "up_human": "128.0 KB/s",
  "down_bits_human": "9.44 Mbps", "up_bits_human": "1.05 Mbps",
  "pps_in": 820, "pps_out": 311,
  "capacity_bps": 1000000000, "capacity_source": "assumed",
  "util_pct": { "down": 0.9, "up": 0.1 },
  "total_down_human": "4.20 GB", "total_up_human": "612.00 MB"
}
```

- Satuan `*_human` adalah **byte** per detik (B/s · KB/s · MB/s · GB/s, basis 1024); `*_bits_human` adalah bit (Kbps/Mbps/Gbps, basis 1000), sesuai kebiasaan paket internet.
- **Arah**: pada port LAN, *download* = router → klien (TX router). Pada WAN, *download* = RX router.
- `source`: `interface_counter` · `queue` · `client_bytes_delta` · `client_aggregate` (jumlah klien di bawah AP) · `wan_interface_counter`.
- `capacity_source`: `configured` (Anda isi lewat path), `queue_max_limit`, `assumed` (**hanya** port `ether` biasa dianggap 1 Gbps), `unknown` (tidak ada persen/alert limit — tidak ditebak).
  Atur kapasitas link nyata (mis. uplink 100 Mbps) agar alert limit akurat: `PUT .../paths/{edge}` dengan `capacity_bps`.

## 12. Keamanan

### Dua peran token

| Token | Kemampuan | Dipakai di |
|---|---|---|
| `api_token` (**admin**) | Seluruh API gateway, termasuk `/api/v1/command` (perintah RouterOS apa pun) dan edit peta. | Backend / server tepercaya. |
| `viewer_token` (**read-only**) | Hanya `GET /api/v1/map/scene`, `/icons`, `/layouts`, `/layouts/{id}`, `/layouts/{id}/background` dan `subscribe_map` di `/ws`. **Tidak** bisa menulis, menjalankan command, memilih router lewat host/password, atau membuka stream lain. | **Browser / aplikasi mobile / dashboard.** |

Aktifkan: `viewer_token = "…"` di `config.toml` atau `GATEWAY_VIEWER_TOKEN=…`. Wajib ≥ 16 karakter dan berbeda dari `api_token`
(jika tidak, diabaikan dengan peringatan saat start). Penolakan memberi `403 FORBIDDEN`.

> ⚠️ **Jangan pernah menaruh `api_token` di JavaScript publik.** Token admin = kendali penuh router.
> Untuk menata peta (edit) gunakan halaman `/map` atau widget `mode="edit"` **hanya** pada jaringan/operator tepercaya,
> atau lewat backend Anda yang meneruskan permintaan.

### Perlindungan lain

| Ancaman | Mitigasi |
|---|---|
| XSS dari data router (hostname DHCP, comment, nama hotspot) | Widget **tidak memakai `innerHTML`**; semua teks lewat `textContent`/`setAttribute`; id ikon divalidasi ke daftar putih. |
| Upload berbahaya | Hanya PNG/JPEG/WebP, dideteksi dari **magic bytes** (bukan Content-Type klien), maks 4 MB; **SVG ditolak**. Disajikan dengan `nosniff` + `CSP: default-src 'none'; sandbox`. |
| URL latar berbahaya | Hanya `http://`/`https://`, tanpa spasi/kutip/`<`/`>`, ≤ 2048 karakter (`javascript:` ditolak). Gambar dimuat dengan `referrerpolicy=no-referrer`. |
| Path traversal | `layout_id` `[a-z0-9_-]{1,64}`; kunci node/edge `[A-Za-z0-9_\-:.]{1,128}`; tenant harus 16 hex; semua divalidasi sebelum menyentuh disk. |
| Data korup / race | Tulis atomik (file sementara + rename) di bawah mutex; read-modify-write di bawah lock; `version` untuk optimistic concurrency (`409` bila basi). |
| Input angka jahat | Semua angka harus *finite* dan dalam rentang (posisi 0–1, kapasitas 1–1e12, aspect 0.2–5, opacity 0–1, `warn < crit ≤ 200`). |
| Penyalahgunaan sumber daya | Maks 3000 node, 4000 path, 64 waypoint/path, 100 layout per tenant, 4 MB gambar, 10 stream per WebSocket, cache laju 64 router. |
| Isolasi tenant | Layout disimpan di `data_dir/<hash-tenant>/…`; hash dari token + `X-Tenant-Id`. |
| Token di URL | Widget mengirim token via header `Authorization` (REST) atau pesan `auth` (WebSocket), **bukan** query string. |
| Halaman `/map` | CSP `script-src 'self'` (tanpa inline script), `frame-ancestors 'self'`, `Referrer-Policy: no-referrer`. Token/password hanya di `sessionStorage` tab itu. |

Sisa risiko yang perlu Anda kelola: lihat §18.

## 13. Referensi REST

Semua di bawah `Authorization: Bearer <token>`. `GW` = alamat gateway. Peran: **V** = viewer boleh, **A** = admin saja.

| Metode | Path | Peran | Fungsi |
|---|---|---|---|
| GET/POST | `/api/v1/map/scene` | V (GET) | Scene gabungan untuk widget. |
| GET | `/api/v1/map/icons` | V | Daftar ikon. |
| GET | `/api/v1/map/layouts` | V | Daftar layout milik tenant. |
| GET | `/api/v1/map/layouts/{id}` | V | Satu layout. |
| PUT | `/api/v1/map/layouts/{id}` | A | Buat / ganti seluruh layout (import). |
| DELETE | `/api/v1/map/layouts/{id}` | A | Hapus layout + gambar. |
| PUT | `/api/v1/map/layouts/{id}/nodes/{node_id}` | A | Tempatkan / ubah ikon, nama, kunci, parent. |
| DELETE | `/api/v1/map/layouts/{id}/nodes/{node_id}` | A | Lupakan penempatan (kembali auto). |
| PUT | `/api/v1/map/layouts/{id}/paths/{edge_id}` | A | Waypoint, gaya, kapasitas satu link. |
| DELETE | `/api/v1/map/layouts/{id}/paths/{edge_id}` | A | Hapus pengaturan jalur. |
| POST | `/api/v1/map/layouts/{id}/auto-place` | A | Simpan posisi otomatis. |
| PUT | `/api/v1/map/layouts/{id}/background` | A | Upload gambar (body = bytes mentah). |
| GET | `/api/v1/map/layouts/{id}/background` | V | Ambil gambar. |
| DELETE | `/api/v1/map/layouts/{id}/background` | A | Hapus gambar. |
| GET | `/map`, `/sdk/mikrotik-map.js`, `/sdk/map-page.js` | publik | Halaman & widget (tanpa data). |

Memilih router: header `X-Router-Id`, atau `X-Router-Host/Port/User/Pass` (admin), atau query `?router_id=`/`?host=`…, atau field body.

### Ambil scene

```bash
GW=http://127.0.0.1:8080; TOKEN=...
curl -s "$GW/api/v1/map/scene?layout_id=rumah" -H "Authorization: Bearer $TOKEN" -H "X-Router-Id: main"
```

> Panggilan **pertama** mengembalikan `rate_ready:false` (belum ada pembanding). Panggil lagi ≥ 1 detik kemudian untuk laju — WebSocket melakukannya otomatis.

### Tempatkan alat

```bash
curl -X PUT "$GW/api/v1/map/layouts/rumah/nodes/if_ether5" \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"x":0.43,"y":0.30,"icon":"switch","label":"Switch ruang tamu","locked":true}'
# → {"success":true,"node":{...},"version":3}
```

Field `NodePatch`: `x`, `y` (wajib saat pertama kali), `icon` (`""` = auto), `label` (`""` = nama router), `locked`, `parent_override` (`""` = hapus).

### Import / ekspor layout

```bash
curl -s "$GW/api/v1/map/layouts/rumah" -H "Authorization: Bearer $TOKEN" > rumah.json
curl -X PUT "$GW/api/v1/map/layouts/rumah-copy" -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' -d @rumah.json
```

`background.kind:"upload"` tidak bisa diset lewat JSON — gambar harus di-upload (menjaga konsistensi file).
Kirim `version` yang sama dengan server untuk perlindungan *lost update*; `0` = tanpa pemeriksaan.

### Upload gambar latar

```bash
curl -X PUT "$GW/api/v1/map/layouts/rumah/background" -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: image/png' --data-binary @denah.png
# → {"success":true,"bytes":183422,"content_type":"image/png","version":4}
```

### Kode error

| HTTP | `code` | Penyebab |
|---|---|---|
| 400 | — | Validasi gagal (id, angka di luar rentang, ikon tidak dikenal, …); pesan menyebut field-nya. |
| 401 | — | Token salah/tidak ada. |
| 403 | `FORBIDDEN` | Token viewer dipakai untuk aksi tulis / endpoint lain. |
| 404 | `NOT_FOUND` | Layout / gambar tidak ada. |
| 409 | `VERSION_CONFLICT` | `version` basi, atau batas 100 layout tercapai. |
| 413 | `PAYLOAD_TOO_LARGE` | Gambar > 4 MB. |
| 415 | `UNSUPPORTED_MEDIA` | Bukan PNG/JPEG/WebP (termasuk SVG). |
| 500 | `INTERNAL_ERROR` | Gagal baca/tulis disk (detail hanya di log gateway). |
| 503 | `CONNECTION_LOST` | Router tidak terjangkau. |

## 14. Referensi WebSocket

Endpoint `ws(s)://GW/ws`. Alur widget:

```text
→ {"action":"auth","token":"VIEWER_TOKEN"}
← {"event":"auth_ok","role":"viewer"}
→ {"action":"subscribe_map","tag":"map","layout_id":"rumah","interval_ms":2000,"filter_type":"all","router_id":"main"}
← {"event":"map_frame","tag":"map","timestamp":1760000000000,"data":{ …scene… }}     (tiap interval)
← {"event":"stream_error","tag":"map","retrying":true,"error":"…"}                    (stream tetap hidup)
→ {"action":"unsubscribe","tag":"map"}
```

- `interval_ms` dijepit 1000–60000. Maks 10 stream per koneksi. Subscribe ulang dengan `tag` yang sama menggantikan yang lama.
- Router boot ulang / putus → `stream_error` dengan `retrying:true`, lalu tersambung otomatis; stream **tidak** berhenti.
- Viewer yang mengirim action lain menerima `{"event":"forbidden"}`. Field `router_host/user/pass` diabaikan untuk viewer.
- Token boleh juga lewat `?token=` pada URL upgrade, tetapi **pesan `auth` lebih aman** (URL sering masuk log proxy).

## 15. Skema scene

```jsonc
{
  "success": true, "generated_at": "2026-10-17T14:00:00.000Z", "rate_ready": true,
  "layout": { "id":"rumah","name":"Rumah","exists":true,"version":7,"updated_at":"…",
              "background":{"kind":"upload|url|none","url":null,"aspect_ratio":1.7778,"opacity":1,"has_upload":true},
              "placed_nodes": 12 },
  "thresholds": { "warn_pct":80, "crit_pct":95, "high_pps":50000, "high_bps":null },
  "summary": { "nodes":14,"edges":14,
               "health":{"ok":10,"idle":1,"warning":2,"critical":1},
               "alerts":{"critical":1,"warning":2},
               "wan":{"down_bps":9437184,"up_bps":1048576,"down_human":"1.12 MB/s","up_human":"128.0 KB/s"} },
  "alerts":   [ {"id":"client_dhcp_1:limit_reached","severity":"critical","kind":"limit_reached","node_id":"client_dhcp_1","label":"Hikvision","message":"queue upload at 120% of its limit - limit reached"} ],
  "warnings": [ "parent_override 'x' of 'y' would create a loop (ignored)" ],
  "nodes": [ {
      "id":"client_dhcp_1","label":"Hikvision-Gate","display_label":"CCTV gerbang",
      "node_type":"device","sub_type":"dhcp","ip":"192.168.100.20","mac":"AA:BB:…","vendor":"Hikvision","interface":null,
      "parent_id":"ap_ether5_AA","parent_source":"auto|override","status":"bound",
      "position":{"x":0.92,"y":0.34,"source":"saved|auto"},
      "icon":{"id":"camera","source":"auto|custom","auto_id":"camera"},
      "health":"ok|warning|critical|idle",
      "reasons":[{"kind":"limit_reached","severity":"critical","message":"…"}],
      "traffic":{ /* §11 atau null */ },
      "queue":{"name":"cctv","max_down_bps":4000000,"max_up_bps":2000000,"limit_state":"reached","down_util_pct":98.0,"up_util_pct":120.0,"dropped_down":0,"dropped_up":0},
      "locked":false,"direct_children":0,"subtree":{"total":0,"warning":0,"critical":0},"extra":{}
  } ],
  "edges": [ {
      "id":"edge_ap_ether5_AA_client_dhcp_1","source":"ap_ether5_AA","target":"client_dhcp_1",
      "edge_type":"physical|wan|logical|wireless|relay","speed":null,"health":"ok",
      "path":{"waypoints":[{"x":0.9,"y":0.2}],"polyline":[…],"style":"straight","length":0.31,"source":"saved|auto"},
      "traffic":{ /* §11 atau null */ },
      "flow":{"down_intensity":0.71,"up_intensity":0.43,"down_direction":"forward|reverse"}
  } ]
}
```

`label` = nama mentah dari topologi router (bisa berisi emoji/awalan/IP). `display_label` = nama yang ditampilkan: nama kustom
operator bila ada, jika tidak `label` yang dibersihkan (tanpa emoji, awalan `AP:`/`Host:`/`WiFi:`/`PPPoE:` dan akhiran `(IP/MAC)`).
IP & MAC tetap tersedia di field `ip`/`mac`. Ikon otomatis memakai hostname/vendor (mis. `…-AP-…` → `ap`, `…extender` → `repeater`).

`extra` berisi metadata dari topologi (mis. `host_name`, `web_url`). **Anggap seluruh string dari router sebagai tidak tepercaya** —
jika Anda membangun UI sendiri, tampilkan sebagai teks, bukan HTML.

## 16. Deploy & operasional

| Item | Nilai |
|---|---|
| Direktori data | `data_dir` di `config.toml` atau `GATEWAY_DATA_DIR` (default `./data`) |
| Struktur | `<data_dir>/<hash-tenant>/<layout>.json` dan `<layout>.bg` |
| Docker | `docker-compose.yml` memasang volume `gateway-data:/data` dan `GATEWAY_DATA_DIR=/data` — tanpa volume, layout hilang saat container dibuat ulang. |
| Backup | Salin `data_dir`, atau **Export** per layout. |
| Reverse proxy | Teruskan `Upgrade`/`Connection` untuk `/ws`. Beri `client_max_body_size 5m` agar upload 4 MB lolos. |
| HTTPS | Gunakan TLS di depan gateway; widget otomatis memakai `wss://` bila `base-url` `https://`. |
| Batas | 3000 node · 4000 path · 64 waypoint/path · 100 layout/tenant · 4 MB gambar · 10 stream/WebSocket |

Contoh Nginx:

```nginx
location /ws { proxy_pass http://127.0.0.1:8080; proxy_http_version 1.1;
  proxy_set_header Upgrade $http_upgrade; proxy_set_header Connection "upgrade"; proxy_read_timeout 1h; }
location /     { proxy_pass http://127.0.0.1:8080; client_max_body_size 5m; }
```

## 17. Troubleshooting & FAQ

| Gejala | Penyebab & solusi |
|---|---|
| Pill “error: Invalid gateway token” | Token salah. Pakai `api_token` atau `viewer_token`. |
| Peta kosong, pill `live` | Router tidak mengembalikan perangkat. Cek `GET /api/v1/network/topology-graph` dengan header yang sama. |
| Angka laju selalu 0 | Sampel pertama. Tunggu 1 interval. Jika terus 0: interface benar-benar idle, atau counter tidak naik. |
| Semua titik bertumpuk/di tepi | Belum ditempatkan. Klik **Auto-place**, lalu geser. |
| Latar tidak muncul | Cek `layout.background`. Upload butuh token admin; URL harus http(s) & mengizinkan hotlink. |
| `403 FORBIDDEN` saat edit | Anda memakai token viewer. Edit butuh token admin. |
| `409` saat menyimpan | Layout diubah operator lain. Muat ulang lalu ulangi. |
| WebSocket putus-putus di balik proxy | Tambah `Upgrade` header & `proxy_read_timeout` (§16). Widget otomatis jatuh ke polling. |
| Limit tidak terdeteksi | Klien tidak punya `/queue/simple` dengan `target` yang memuat IP-nya, atau queue `disabled`. |
| Persen kapasitas tidak muncul di WAN | `capacity_source:"unknown"`. Isi `capacity_bps` pada path `edge_router_<iface>`. |

**Perlu klik “Mulai stream”?** Tidak. Widget membuka WebSocket sendiri saat dipasang.

**Apakah posisi ikut berubah bila router mengganti IP klien?** Posisi terikat pada **id node** (mis. `client_dhcp_<MAC>`), bukan IP.

## 18. Keterbatasan yang diketahui

Ditulis terbuka agar tidak ada kejutan:

1. **Tidak ada login per-pengguna.** Hanya dua token bersama (admin & viewer). Pembatasan per-layout/per-pengguna belum ada → letakkan di belakang proxy auth Anda bila perlu.
2. **Token viewer terlihat oleh pemilik browser.** Ia hanya bisa *membaca peta*, tetapi peta memuat IP/MAC/hostname perangkat. Anggap sebagai data internal.
3. **Satu `data_dir` = satu instance gateway.** Tidak ada penguncian lintas-instance; jangan menjalankan dua gateway pada direktori yang sama.
4. **Gambar latar dimuat ulang** hanya saat jenis/URL berubah atau setelah diubah dari widget ini; perubahan gambar oleh operator lain muncul setelah halaman dimuat ulang.
5. **Deteksi ikon bersifat heuristik** (hostname/vendor). Perangkat berhostname generik jadi `unknown` sampai diganti manual.
6. **Kapasitas link non-ethernet tidak diketahui** (SFP, wireless, PPPoE) sampai Anda mengisinya; alert limit untuk link itu tidak aktif.
7. **Laju klien tanpa queue** hanya tersedia bila router mengekspos counter byte (mis. hotspot). DHCP-only tanpa queue tampil tanpa angka laju (tidak diisi nilai karangan).
8. **Skala:** diuji logika-nya untuk ribuan node, tetapi render SVG nyaman hingga ± beberapa ratus node; untuk ISP besar gunakan filter (`filter=hotspot` dll.) atau beberapa layout.
9. **Halaman `/map` tidak bisa di-iframe dari domain lain** (gateway mengirim `X-Frame-Options: SAMEORIGIN`). Embed dengan web component `<mikrotik-map>` (tag `<script>`), bukan iframe; atau letakkan gateway & aplikasi pada origin yang sama lewat reverse proxy.

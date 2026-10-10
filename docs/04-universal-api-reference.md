# Referensi Lengkap Universal API Gateway (120+ Enterprise Endpoints)

Gateway MikroTik Rust mengekspos endpoint REST dan SSE universal berkinerja tinggi. Seluruh 120+ endpoint enterprise mendukung **Dual HTTP Transport (GET & POST)** dan dapat dijalankan oleh stack bahasa apa pun (Web, Mobile, Backend) maupun tool diagnostik NOC (cURL, browser) dengan format JSON standar.

---

## 1. Mekanisme Kredensial Router Target

Setiap request ke endpoint `/api/v1/*` dapat menentukan router target melalui **salah satu dari 4 cara**:

### Opsi A: Melalui HTTP Headers (Direkomendasikan untuk REST & Mobile)
```http
Authorization: Bearer <API_TOKEN>
X-Router-Host: 192.168.88.1
X-Router-Port: 8728
X-Router-User: admin
X-Router-Pass: password123
Content-Type: application/json
```

### Opsi B: Melalui URL Query Parameters (Direkomendasikan untuk NOC Triage, cURL & Browser)
Dapat dipanggil via **GET** maupun **POST** secara langsung:
```bash
curl "http://127.0.0.1:8080/api/v1/overview?host=192.168.88.1&port=8728&user=admin&pass=password123&token=mytoken"
```

### Opsi C: Melalui JSON Body
```json
{
  "router": {
    "host": "192.168.88.1",
    "port": 8728,
    "user": "admin",
    "password": "password123"
  }
}
```

### Opsi D: Melalui Router ID (Yang sudah terdaftar di `config.toml`)
```json
{
  "router_id": "main"
}
```

---

## 2. Katalog Lengkap Seluruh Modul Endpoint

> [!NOTE]
> Seluruh endpoint di bawah ini mendukung metode **`GET`** dan **`POST`**. Pada metode `GET`, parameter query string seperti `?host=..&token=..` akan otomatis dihidrasi oleh gateway engine.

### ⚡ 0. Fast-Path Overview Snapshot (Sub-Millisecond Aggregation)
Endpoint ini menggabungkan 6 query router terpisah menjadi 1 eksekusi simultan (`tokio::join!`) di single persistent socket:
| Method | Endpoint | Kegunaan | Payload / Query |
|---|---|---|---|
| `GET` / `POST` | `/api/v1/overview` | Aggregated Snapshot: Identity, System Resource, RouterBOARD, Hotspot Active, PPP Active, Interface states | `{}` atau query params |

---

### 🌐 0B. Unified Connected Devices Map (Cross-Layer Device Correlator)
Menjawab kebutuhan engineer yang membedakan **DHCP Leases, Hotspot Hosts, ARP Table, dan Wireless Clients**:
Endpoint ini menarik 6 tabel perangkat sekaligus dan mengorelasikannya berdasarkan **MAC address & IP** secara instan:
| Method | Endpoint | Kegunaan | Payload |
|---|---|---|---|
| `POST` | `/api/v1/network/connected-devices` | Unified Map: Menggabungkan DHCP + Hotspot Hosts + Active + Bindings + ARP + WiFi Registration | `{"search": "", "filter_status": "all"}` |

---

### A. System & Upgrade (`/api/v1/system/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/system/resource` | CPU, RAM, Uptime, Versi OS, Arch | `{}` |
| `POST` | `/api/v1/system/routerboard` | Serial Number, Model, Firmware | `{}` |
| `POST` | `/api/v1/system/identity` | Nama hostname router | `{}` |
| `POST` | `/api/v1/system/identity/set` | Mengubah hostname router | `{"name": "Router-Baru"}` |
| `POST` | `/api/v1/system/check-update` | Cek ketersediaan update RouterOS | `{}` |
| `POST` | `/api/v1/system/install-update` | Unduh & instal upgrade RouterOS | `{}` |
| `POST` | `/api/v1/system/reboot` | Reboot router | `{}` |

---

### B. IPv4, ARP & Routing (`/api/v1/ip/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/ip/addresses` | Daftar IP Address | `{"filter": {"?disabled": "no"}}` |
| `POST` | `/api/v1/ip/address/add` | Tambah IP Address | `{"address": "192.168.10.1/24", "interface": "ether2", "comment": "LAN"}` |
| `POST` | `/api/v1/ip/address/remove` | Hapus IP Address | `{"id": "*1"}` |
| `POST` | `/api/v1/ip/routes` | Tabel routing IPv4 | `{}` |
| `POST` | `/api/v1/ip/dns` | Konfigurasi DNS Server | `{}` |
| `POST` | `/api/v1/ip/pools` | Daftar IP Pool | `{}` |
| `POST` | `/api/v1/ip/arp` | **Daftar entri ARP (IP-to-MAC mapping)** | `{}` |
| `POST` | `/api/v1/ip/arp/add` | Tambah entri ARP statis | `{"address": "192.168.88.50", "mac_address": "AA:BB:CC:DD:EE:FF", "interface": "bridge"}` |
| `POST` | `/api/v1/ip/arp/remove` | Hapus entri ARP | `{"id": "*1"}` |

---

### C. Static DNS & Cache (`/api/v1/dns/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/dns/static` | Daftar static DNS record | `{}` |
| `POST` | `/api/v1/dns/static/add` | Tambah static DNS baru (domain intranet/hotspot) | `{"name": "login.wifi", "address": "192.168.88.1"}` |
| `POST` | `/api/v1/dns/static/remove` | Hapus static DNS record | `{"id": "*1"}` |
| `POST` | `/api/v1/dns/cache/flush` | Bersihkan cache DNS router | `{}` |

---

### D. IPv6 Management (`/api/v1/ipv6/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/ipv6/addresses` | Daftar IPv6 Address | `{}` |
| `POST` | `/api/v1/ipv6/address/add` | Tambah IPv6 Address | `{"address": "2001:db8::1/64", "interface": "ether1", "advertise": true}` |
| `POST` | `/api/v1/ipv6/address/remove`| Hapus IPv6 Address | `{"id": "*1"}` |
| `POST` | `/api/v1/ipv6/routes` | Tabel routing IPv6 | `{}` |
| `POST` | `/api/v1/ipv6/pools` | IPv6 Pool | `{}` |
| `POST` | `/api/v1/ipv6/nd` | Neighbor Discovery (ND) | `{}` |

---

### E. DHCP Server & Leases (`/api/v1/dhcp/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/dhcp/servers` | Daftar server DHCP | `{}` |
| `POST` | `/api/v1/dhcp/leases` | Klien yang mendapat IP DHCP | `{}` |
| `POST` | `/api/v1/dhcp/lease/make-static`| Kunci IP dinamis jadi IP statis | `{"id": "*2"}` |
| `POST` | `/api/v1/dhcp/lease/remove` | Hapus lease klien | `{"id": "*2"}` |

---

### F. Hotspot, Vouchers & Hosts (`/api/v1/hotspot/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/hotspot/users` | Daftar seluruh voucher/user | `{}` |
| `POST` | `/api/v1/hotspot/user/create` | Buat 1 voucher baru | `{"name": "vc10", "password": "pass", "profile": "default", "timelimit": "3h"}` |
| `POST` | `/api/v1/hotspot/generate-batch` | **Buat 10-1000 voucher massal (Pipelined)** | `{"qty": 50, "prefix": "VIP-", "profile": "default", "timelimit": "1h"}` |
| `POST` | `/api/v1/hotspot/user/remove` | Hapus voucher/user | `{"id": "*5"}` |
| `POST` | `/api/v1/hotspot/active` | Daftar user yang **sedang online** | `{}` |
| `POST` | `/api/v1/hotspot/kick` | **Kick** (putuskan koneksi) user online | `{"id": "*3"}` |
| `POST` | `/api/v1/hotspot/hosts` | **Daftar seluruh perangkat di subnet Hotspot (Authorized & Pending)** | `{}` |
| `POST` | `/api/v1/hotspot/host/remove` | Hapus entri host hotspot | `{"id": "*1"}` |
| `POST` | `/api/v1/hotspot/host/bind` | **Quick Bypass / Bind Host MAC (CCTV, Printer, TV)** | `{"mac_address": "AA:BB:..", "binding_type": "bypassed"}` |
| `POST` | `/api/v1/hotspot/profiles` | Profil paket voucher | `{}` |
| `POST` | `/api/v1/hotspot/ip-bindings`| Bypass MAC Address (tanpa login voucher) | `{}` |

---

### G. Wireless & WiFi / CAPsMAN (`/api/v1/wireless/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/wireless/interfaces` | Daftar interface wireless/wifi | `{}` |
| `POST` | `/api/v1/wireless/registrations` | **WiFi Clients** (Sinyal dBm, CCQ, Tx/Rx rate, Uptime) | `{}` |
| `POST` | `/api/v1/wireless/security-profiles` | Profil keamanan WiFi WPA2/WPA3 | `{}` |
| `POST` | `/api/v1/wireless/access-list` | Filter MAC address access list | `{}` |

---

### H. PPP & PPPoE ISP Management (`/api/v1/ppp/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/ppp/secrets` | Daftar akun pelanggan PPPoE | `{}` |
| `POST` | `/api/v1/ppp/secret/create` | Tambah akun PPPoE baru | `{"name": "user1", "password": "123", "service": "pppoe", "profile": "default"}` |
| `POST` | `/api/v1/ppp/secret/set` | Ubah password/profile/status akun PPPoE | `{"id": "*1", "profile": "Paket-30M", "disabled": false}` |
| `POST` | `/api/v1/ppp/secret/remove` | Hapus akun PPPoE | `{"id": "*1"}` |
| `POST` | `/api/v1/ppp/customer/isolate` | **Isolir Pelanggan (Ubah profile ke ISOLIR & Kick session aktif)** | `{"username": "user1", "isolate_profile": "ISOLIR"}` |
| `POST` | `/api/v1/ppp/customer/restore` | **Buka Isolir Pelanggan (Restore ke profil aktif & aktifkan kembali)** | `{"username": "user1", "active_profile": "Paket-20M"}` |
| `POST` | `/api/v1/ppp/active` | Pelanggan PPPoE yang sedang online | `{}` |
| `POST` | `/api/v1/ppp/disconnect` | Putuskan koneksi pelanggan aktif | `{"id": "*4"}` |
| `POST` | `/api/v1/ppp/servers` | Daftar server PPPoE per interface | `{}` |
| `POST` | `/api/v1/ppp/server/create` | Buat server PPPoE baru pada interface | `{"service_name": "pppoe-lan", "interface": "ether2"}` |
| `POST` | `/api/v1/ppp/profiles` | Profil paket PPPoE | `{}` |
| `POST` | `/api/v1/ppp/profile/create` | Buat profil paket PPPoE (rate limit, pool) | `{"name": "Paket-50M", "rate_limit": "50M/50M"}` |
| `POST` | `/api/v1/ppp/profiles` | Daftar profil paket PPPoE | `{}` |

---

### I. WireGuard VPN (RouterOS v7) (`/api/v1/wireguard/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/wireguard/interfaces` | Daftar interface WireGuard | `{}` |
| `POST` | `/api/v1/wireguard/peers` | Daftar peer WireGuard | `{}` |
| `POST` | `/api/v1/wireguard/peer/add` | Tambah peer WireGuard | `{"interface": "wg0", "public_key": "...", "allowed_address": "10.0.0.2/32"}` |
| `POST` | `/api/v1/wireguard/peer/remove` | Hapus peer WireGuard | `{"id": "*1"}` |

---

### J. Firewall & Keamanan (`/api/v1/firewall/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/firewall/filters` | Aturan firewall filter | `{}` |
| `POST` | `/api/v1/firewall/nat` | Aturan NAT (Port forwarding/masquerade) | `{}` |
| `POST` | `/api/v1/firewall/address-lists`| Daftar IP Address List | `{}` |
| `POST` | `/api/v1/firewall/block-ip` | **Blokir IP** (masukkan ke address list) | `{"address": "1.2.3.4", "list": "BLACKLIST", "timeout": "7d"}` |
| `POST` | `/api/v1/firewall/unblock-ip` | Hapus IP dari daftar blokir | `{"id": "*1"}` |

---

### K. Simple Queues (`/api/v1/queues/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/queues/simple` | Daftar aturan limit bandwidth | `{}` |
| `POST` | `/api/v1/queues/simple/add` | Tambah aturan limit kecepatan | `{"name": "Limit-1", "target": "192.168.1.50/32", "max_limit": "2M/10M"}` |
| `POST` | `/api/v1/queues/simple/set-limit`| Ubah limit kecepatan | `{"id": "*1", "max_limit": "5M/20M"}` |
| `POST` | `/api/v1/queues/simple/remove` | Hapus aturan limit | `{"id": "*1"}` |

---

### L. Interfaces & Realtime Traffic Streaming (`/api/v1/interfaces/*`)
| Method | Endpoint | Kegunaan |
|---|---|---|
| `POST` | `/api/v1/interfaces/all` | Daftar semua interface ethernet & bridge |
| `POST` | `/api/v1/interfaces/sample-traffic` | Snapshot kecepatan traffic saat ini (`{"interface": "ether1"}`) |
| `GET` | `/api/v1/interfaces/stream` | **SSE Live Stream Traffic** (Query params: `host`, `port`, `user`, `password`, `interface=ether1`) |

---

### M. Network Tools & Diagnostik (`/api/v1/tools/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/tools/ping` | Ping ke IP target dari router | `{"address": "8.8.8.8", "count": 4}` |
| `POST` | `/api/v1/tools/traceroute` | Traceroute dari router | `{"address": "1.1.1.1"}` |
| `POST` | `/api/v1/tools/profile` | **CPU Profiler** (melihat apa yang menghabiskan CPU) | `{}` |
| `POST` | `/api/v1/tools/netwatch` | Monitoring status UP/DOWN host otomatis | `{}` |
| `POST` | `/api/v1/tools/bandwidth-test` | Tes kecepatan bandwidth antar MikroTik | `{"address": "192.168.1.2", "direction": "both"}` |

---

### N. Neighbor Discovery & Winbox Scan (`/api/v1/neighbors/*`)
| Method | Endpoint | Kegunaan |
|---|---|---|
| `POST` | `/api/v1/neighbors/all` | Mendeteksi semua router/switch sekitar via CDP/MNDP/LLDP |

---

### O. Scripting & Schedulers Cron (`/api/v1/scripts/*` & `/api/v1/schedulers/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/scripts/all` | Daftar skrip sistem | `{}` |
| `POST` | `/api/v1/scripts/run` | Jalankan skrip RouterOS | `{"number_or_name": "backup-script"}` |
| `POST` | `/api/v1/scripts/add` | Tambah skrip baru | `{"name": "test", "source": ":put hello"}` |
| `POST` | `/api/v1/schedulers/all` | Daftar scheduler cron otomatis | `{}` |
| `POST` | `/api/v1/schedulers/add` | Tambah scheduler cron | `{"name": "daily-reset", "on_event": "...", "interval": "1d"}` |

---

### P. Backup, Export & Storage Files (`/api/v1/backup/*` & `/api/v1/files/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/backup/create` | Buat file backup `.backup` di router | `{"name": "backup-auto"}` |
| `POST` | `/api/v1/backup/export` | Ekspor konfigurasi skrip `.rsc` | `{"file": "config.rsc"}` |
| `POST` | `/api/v1/files/all` | Daftar file di storage MikroTik | `{}` |
| `POST` | `/api/v1/files/remove` | Hapus file di storage | `{"id": "*1"}` |

---

### Q. Router Administrator Users (`/api/v1/users/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/users/all` | Daftar akun admin router | `{}` |
| `POST` | `/api/v1/users/create` | Tambah akun admin baru | `{"name": "noc", "password": "...", "group": "full"}` |
| `POST` | `/api/v1/users/remove` | Hapus akun admin | `{"id": "*1"}` |
| `POST` | `/api/v1/users/groups` | Daftar grup permission | `{}` |

---

### R. Batch Pipeline Execution (`POST /api/v1/batch`)
Mengeksekusi banyak perintah sekaligus dalam 1 kali HTTP request secara paralel:
```json
{
  "router": {
    "host": "ath.vpnbersama.us",
    "port": 51121,
    "user": "admin",
    "password": "secretpassword"
  },
  "commands": [
    { "command": "/ip/hotspot/user/add", "args": { "name": "user01", "password": "123", "profile": "default" } },
    { "command": "/ip/hotspot/user/add", "args": { "name": "user02", "password": "123", "profile": "default" } },
    { "command": "/ip/hotspot/user/add", "args": { "name": "user03", "password": "123", "profile": "default" } }
  ]
}
```

---

### S. Universal Raw Command Runner (`POST /api/v1/command`)
Mengeksekusi perintah RouterOS arbitrary apa pun secara bebas tanpa batas:
```json
{
  "router": {
    "host": "ath.vpnbersama.us",
    "port": 51121,
    "user": "admin",
    "password": "secretpassword"
  },
  "command": "/tool/ping",
  "args": {
    "address": "8.8.8.8",
    "count": "4"
  }
}
```

---

### T. Filterisasi Log & Live Log Streaming (`/api/v1/logs/*`)
Sistem pencarian log bertenaga tinggi dengan filter topik, severity level, pencarian teks, dan live streaming (SSE/WebSocket):
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/logs/all` | Filter & ambil entri log terbaru | `{"topics": "hotspot,info", "search": "login", "severity": "error", "limit": 100}` |
| `GET` | `/api/v1/logs/stream` | **SSE Live Log Stream** (Realtime per detik) | Query params: `host`, `port`, `user`, `password`, `topics=hotspot` |

Payload contoh `POST /api/v1/logs/all`:
```json
{
  "router_id": "main",
  "topics": "hotspot,info",
  "search": "logged in",
  "limit": 50
}
```

---

### U. RouterOS Package, Upgrade & Service Hardening (`/api/v1/system/*`)
Manajemen update OS mikrofonik dan pemadaman service berbahaya (hardening):
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/system/package/download` | Download file update RouterOS di latar belakang | `{}` |
| `POST` | `/api/v1/system/package/channel` | Ganti update channel (stable, testing, development) | `{"channel": "stable"}` |
| `POST` | `/api/v1/system/packages` | Daftar semua package terinstal (wireless, routing, dll.) | `{}` |
| `POST` | `/api/v1/system/services` | Daftar service router (`api`, `winbox`, `ssh`, `telnet`, `ftp`) | `{}` |
| `POST` | `/api/v1/system/service/toggle` | **Matikan/Nyalakan Service** (Security Hardening) | `{"name": "telnet", "disabled": true}` |

---

### V. Printable Voucher Template Engine (`POST /api/v1/hotspot/voucher-template/render`)
Merender voucher Hotspot langsung menjadi format dokumen HTML/CSS siap cetak (Mikhmon style):
| Format Preset | Deskripsi | Ukuran Kertas |
|---|---|---|
| `thermal_58mm` | Struk Kasir Gulung | Lebar 58mm POS Printer |
| `thermal_80mm` | Struk Kasir Lebar | Lebar 80mm POS Printer |
| `grid_a4` | Lembar Kartu A4 (12 kartu per halaman) | Standar A4 Office Printer |
| `custom` | Custom HTML template dengan placeholder | Fleksibel |

Tersedia placeholder otomatis: `{{username}}`, `{{password}}`, `{{price}}`, `{{timelimit}}`, `{{datalimit}}`, `{{dns_name}}`, `{{hotspot_name}}`.

Contoh Request:
```json
{
  "format": "thermal_58mm",
  "hotspot_name": "WIFI WARUNG KOPI",
  "dns_name": "wifi.warung.net",
  "vouchers": [
    { "username": "VIP-9481", "password": "123", "timelimit": "5 Jam", "price": "Rp 5.000" },
    { "username": "VIP-9482", "password": "456", "timelimit": "24 Jam", "price": "Rp 15.000" }
  ]
}
```
Response mengembalikan field `html` yang siap langsung di-pipe ke iframe browser, WebView Android/iOS, atau cetak Bluetooth Thermal ESC/POS.

---

### W. User Manager v6 & v7 Dual Compatibility (`/api/v1/user-manager/*`)
Mendukung RADIUS User Manager RouterOS v7 (`/user-manager/*`) dan otomatis fallback ke RouterOS v6 (`/tool/user-manager/*`):
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/user-manager/users` | Daftar user RADIUS User Manager | `{}` |
| `POST` | `/api/v1/user-manager/user/create` | Buat akun User Manager baru | `{"name": "user01", "password": "123", "group": "default"}` |
| `POST` | `/api/v1/user-manager/user/remove` | Hapus akun User Manager | `{"id": "*1"}` |
| `POST` | `/api/v1/user-manager/sessions` | Sesi aktif user RADIUS User Manager | `{}` |
| `POST` | `/api/v1/user-manager/profiles` | Profil paket User Manager | `{}` |

---

### X. Bridge & VLAN Management (`/api/v1/bridge/*`)
Manajemen interface Bridge, penugasan port ethernet ke Bridge, serta Trunking VLAN:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/bridge/all` | Daftar seluruh bridge interface | `{}` |
| `POST` | `/api/v1/bridge/add` | Tambah bridge baru | `{"name": "br-hotspot", "vlan_filtering": true}` |
| `POST` | `/api/v1/bridge/remove` | Hapus bridge | `{"id": "*1"}` |
| `POST` | `/api/v1/bridge/ports` | Daftar port yang tergabung ke bridge | `{}` |
| `POST` | `/api/v1/bridge/port/add` | Masukkan interface ethernet ke bridge | `{"bridge": "bridge1", "interface": "ether3", "pvid": "100"}` |
| `POST` | `/api/v1/bridge/port/remove` | Keluarkan interface dari bridge | `{"id": "*1"}` |
| `POST` | `/api/v1/bridge/vlans` | Daftar VLAN filtering pada bridge | `{}` |
| `POST` | `/api/v1/bridge/vlan/add` | Konfigurasi Tagged/Untagged VLAN | `{"bridge": "bridge1", "vlan_ids": "100", "tagged": "ether1", "untagged": "ether2"}` |

---

### Y. VPN Servers & EoIP Tunnels (`/api/v1/vpn/*`)
Konfigurasi VPN gateway server dan tunneling layer-2:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/vpn/sstp` | Cek status SSTP Server | `{}` |
| `POST` | `/api/v1/vpn/sstp/set` | Aktifkan/konfigurasi SSTP VPN Server | `{"enabled": true, "port": "443", "default_profile": "default-encryption"}` |
| `POST` | `/api/v1/vpn/l2tp` | Cek status L2TP/IPsec Server | `{}` |
| `POST` | `/api/v1/vpn/l2tp/set` | Aktifkan/konfigurasi L2TP Server | `{"enabled": true, "use_ipsec": "yes", "ipsec_secret": "mySecretKey"}` |
| `POST` | `/api/v1/vpn/ovpn` | Cek status OpenVPN Server | `{}` |
| `POST` | `/api/v1/vpn/ovpn/set` | Aktifkan/konfigurasi OpenVPN Server | `{"enabled": true, "port": "1194", "mode": "ip"}` |
| `POST` | `/api/v1/vpn/eoip` | Daftar tunnel Ethernet-over-IP (EoIP) | `{}` |
| `POST` | `/api/v1/vpn/eoip/add` | Buat tunnel EoIP layer-2 baru | `{"name": "eoip-branch", "remote_address": "203.0.113.5", "tunnel_id": "10"}` |
| `POST` | `/api/v1/vpn/eoip/remove` | Hapus tunnel EoIP | `{"id": "*1"}` |

---

### Z. 🚀 Expert Network Shorthand Macros (`/api/v1/expert/*`)
Macro ultra-cepat yang dirancang khusus untuk Network Engineer profesional, menggabungkan puluhan perintah diagnostik rumit ke dalam 1 respon instan:

#### 1. Quick Diagnose (`POST /api/v1/expert/quick-diagnose`)
Melakukan audit kesehatan router secara lengkap: status WAN, default gateway, ping test ke ISP Gateway & Internet (8.8.8.8), CPU health, dan resource check dalam hitungan milidetik.
```json
{
  "router_id": "main",
  "ping_targets": ["8.8.8.8", "1.1.1.1"]
}
```

#### 2. Traffic Matrix (`POST /api/v1/expert/traffic-matrix`)
Mengambil matrix kecepatan seluruh interface router secara bersamaan (Tx bps, Rx bps, packets/sec, status link, and errors):
```json
{
  "router_id": "main"
}
```

#### 3. Security Audit (`POST /api/v1/expert/security-audit`)
Pemeriksaan otomatis celah keamanan MikroTik:
- Mendeteksi service berbahaya yang masih aktif (`telnet`, `ftp`, `api` port standar 8728).
- Memeriksa apakah user `admin` masih menggunakan password default / kosong.
- Memeriksa apakah DNS allow-remote-requests terbuka ke publik tanpa firewall drop.
- Menghitung skor keamanan (contoh: `score: 85/100`) dan memberikan daftar rekomendasi tindakan.
```json
{
  "router_id": "main"
}
```

---

### AA. ⚡ WebSocket Real-Time Event Hub (`/ws`)
Koneksi duplex WebSocket berkecepatan tinggi untuk streaming metrik tanpa polling HTTP overhead:

```javascript
const ws = new WebSocket('ws://127.0.0.1:8080/ws');

// Kirim perintah listen traffic interface
ws.send(JSON.stringify({
  action: "monitor_traffic",
  router: { host: "192.168.88.1", user: "admin", password: "123" },
  tag: "wan-monitor",
  params: { interface: "ether1" }
}));

// Atau kirim perintah live logs stream
ws.send(JSON.stringify({
  action: "listen_logs",
  router: { host: "192.168.88.1", user: "admin", password: "123" },
  tag: "log-stream",
  params: { topic: "critical" }
}));

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log("Realtime packet:", data);
};
```

---

### BB. 🛡️ Security, CVE Vulnerability Audit & Anti-Bruteforce (`/api/v1/security/*`)
Mendeteksi kerentanan versi RouterOS secara mendalam (CVE database), mendeteksi celah servis, dan memasang aturan pertahanan anti-bruteforce multi-stage otomatis:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/security/vulnerability-audit` | **Deep CVE Scan**: Audit versi OS (v6/v7), arsitektur CPU, cek celah CVE-2018-14847, Chimay-Red, CVE-2023-30799, CVE-2024-54772, CVE-2026-16347, open DNS resolver, akun admin default | `{}` |
| `POST` | `/api/v1/security/deploy-antibruteforce` | **Deploy Pertahanan Anti-Bruteforce**: Suntik aturan multi-stage connection-rate limiting untuk WinBox (8291), API (8728), SSH (22) dan otomatis blacklist IP penyerang | `{"protect_winbox": true, "protect_ssh": true, "protect_api": true, "blacklist_timeout": "7d"}` |

Contoh Output `/api/v1/security/vulnerability-audit`:
```json
{
  "success": true,
  "router_fingerprint": {
    "version": "6.49.6",
    "architecture": "mipsbe",
    "board_name": "RB951Ui-2HnD",
    "free_storage_mb": 42
  },
  "security_score": 60,
  "threat_level": "VULNERABLE",
  "detected_cves": [
    {
      "cve_id": "CVE-2023-30799",
      "title": "WinBox Privilege Escalation to Super-Admin",
      "severity": "HIGH",
      "affected_versions": "RouterOS v6.49.7 and prior / v7.0 through v7.9",
      "remediation": "Upgrade immediately to RouterOS >= 6.49.8 (v6) or >= 7.10 (v7)"
    }
  ],
  "service_vulnerabilities": [
    "Telnet aktif: Protokol tanpa enkripsi, rentan terhadap sniffing kredensial di jaringan LAN/WISP."
  ],
  "architecture_advisories": []
}
```

---

### CC. 📡 The Dude Network Monitor & Database Maintenance (`/api/v1/dude/*`)
Mengelola server The Dude di MikroTik, memantau node/perangkat jaringan, dan menangani masalah kerusakan database SQLite The Dude:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/dude/status` | Cek status server The Dude (`enabled`, path database, operational status) | `{}` |
| `POST` | `/api/v1/dude/toggle` | **Matikan/Nyalakan The Dude**: Disarankan mematikan Dude sebelum backup/vacuum agar database tidak corrupt | `{"enabled": false}` |
| `POST` | `/api/v1/dude/export-db` | **Backup Database Dude**: Ekspor database SQLite ke file `.db` | `{"backup_file": "dude_backup_2026.db"}` |
| `POST` | `/api/v1/dude/import-db` | **Restore Database Dude**: Impor file backup database | `{"backup_file": "dude_backup_2026.db"}` |
| `POST` | `/api/v1/dude/vacuum` | **Vacuum / Compact Database**: Mengurangi fragmentasi disk dan mengecilkan ukuran `dude.db` | `{"file": "dude.db"}` |
| `POST` | `/api/v1/dude/devices` | Ambil daftar seluruh router & server yang sedang dimonitor oleh The Dude | `{}` |

---

### DD. 🔄 Disaster Recovery, Netinstall & Architecture Package Resolver (`/api/v1/maintenance/*`)
Alat tanggap bencana untuk pemulihan router mati, hard reset, dan resolusi paket CPU arsitektur MikroTik:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/maintenance/netinstall-prep` | **Etherboot / Netinstall Mode**: Mengatur RouterBOARD boot-device ke `try-ethernet-once-then-nand`. Pada reboot berikutnya router otomatis masuk mode Netinstall tanpa perlu menekan tombol reset fisik! | `{}` |
| `POST` | `/api/v1/maintenance/pre-upgrade-snapshot` | **Snapshot Ganda Pra-Upgrade**: Membuat file binary `.backup` dan text `.rsc` dengan timestamp sebelum upgrade berisiko | `{}` |
| `POST` | `/api/v1/maintenance/architecture-package-url` | **Arch Package CDN Resolver**: Mendeteksi CPU arsitektur router (`arm`, `arm64`, `tile`, `mipsbe`, `mmips`, `smips`, `x86`, `chr`, `powerpc`) dan membuat URL download resmi langsung dari CDN MikroTik | `{"target_version": "7.15.3"}` |
| `POST` | `/api/v1/maintenance/reset-configuration` | **Factory Reset Terkontrol**: Eksekusi reset RouterOS dengan opsi simpan user atau tanpa default konfigurasi | `{"keep_users": true, "no_defaults": true, "skip_backup": false}` |

---

### EE. 🌐 Interactive Topology Graph, Live Visualizer & Embeddable Web SDK

Menjawab kebutuhan menampilkan **relasi jaringan interaktif, modern, dan responsif** langsung di browser atau di-embed ke project web apa pun (Laravel, Vue, Express, React):

| Method | Endpoint / URL | Kegunaan | Deskripsi |
|---|---|---|---|
| `POST` | `/api/v1/network/topology-graph` | **Relational Graph JSON API** | Mengembalikan struktur nodes (Router, Interface, Switch/Neighbor, Device) dan edges relasi (koneksi fisik, VLAN, Hotspot, PPPoE, WiFi) beserta metrik perangkat |
| `GET` | `/topology` | **Interactive Visualizer Dashboard** | Fullscreen UI modern bertema Obsidian Dark Mode dengan interaksi klik node, animasi aliran data SVG, dan panel inspeksi |
| `GET` | `/sdk/mikrotik-widget.js` | **Embeddable JavaScript SDK** | Script SDK ultra-ringan tanpa dependensi untuk memasang visualizer topologi ke dalam tag `<div>` Laravel Blade atau Vue dalam 1 baris kode |

#### 1. Cara Pasang di Laravel Blade (Hanya 2 Baris)
```html
<div id="mikrotik-network-map" style="width: 100%; height: 600px;"></div>

<!-- Pasang SDK MikroTik Rust Gateway -->
<script src="http://127.0.0.1:8080/sdk/mikrotik-widget.js"></script>
<script>
  MikrotikWidget.mount('#mikrotik-network-map', {
    gatewayUrl: 'http://127.0.0.1:8080',
    height: '600px',
    borderRadius: '16px'
  });
</script>
```

#### 2. Cara Pasang di Vue 3 / Nuxt
```html
<template>
  <div class="card shadow-lg rounded-2xl overflow-hidden">
    <iframe 
      :src="visualizerUrl" 
      class="w-full h-[650px] border-none"
    />
  </div>
</template>

<script setup>
const visualizerUrl = 'http://127.0.0.1:8080/topology';
</script>
```

#### 3. Fitur Interaktif pada Visualizer:
1. **Interactive Node Click**: Mengklik node client atau router akan memunculkan Inspector Drawer di sisi kanan secara instan.
2. **⚡ Live ICMP Ping Diagnostics**: Tombol "Ping Perangkat Ini" di drawer langsung mengirimkan 3 paket ping dari MikroTik ke IP perangkat tersebut dan menampilkan latensi RTT real-time (`0.8 ms`, `packet loss: 0%`).
3. **Penyaringan Kategori Dinamis**: Tombol filter instan untuk `Semua`, `Hotspot`, `PPPoE`, `DHCP`, dan `WiFi`.
4. **Anti-Hang Protection Engine**: Seluruh pembacaan data berjalan di atas persistent pool biner Rust dengan penjaga batas waktu (*15-second timeout guard*), sehingga RouterOS tidak akan pernah mengalami CPU lock 100% atau freeze meskipun diakses oleh banyak klien secara bersamaan.

---

### FF. 📡 Access Point & Infrastructure Device Detector (`/api/v1/network/infrastructure/*`)
Mendeteksi perangkat infrastruktur seperti Access Point (Ubiquiti, TP-Link, Ruijie, Tenda), Switch, dan IP Camera yang di-bridge di jaringan Hotspot/PPPoE:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/network/infrastructure/scan` | **Pindai Semua AP & Switch**: Mengorelasikan MNDP/LLDP Neighbors, Hotspot Hosts bridge mode, ARP table, dan signature vendor OUI MAC address | `{}` |
| `POST` | `/api/v1/network/infrastructure/auto-bypass-ap` | **Auto-Bypass AP di Hotspot**: Memasukkan MAC AP ke `/ip/hotspot/ip-binding` (`type=bypassed`) agar admin bisa remote Web GUI AP tanpa login voucher | `{"mac_address": "EC:08:6B:11:22:33", "comment": "[Auto-AP] Ruijie Lantai 2"}` |

---

### GG. ✈️ Telegram Bot & Automated Netwatch Alerting (`/api/v1/telegram/*`)
Integrasi notifikasi pesan instan Telegram langsung dari router MikroTik:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/telegram/send-message` | **Kirim Pesan Telegram Instan**: Mengeksekusi `/tool/fetch` di MikroTik untuk mengirim pesan ke Telegram bot | `{"bot_token": "...", "chat_id": "...", "message": "Halo dari Router!"}` |
| `POST` | `/api/v1/telegram/setup-netwatch` | **Pasang Netwatch Otomatis**: Memasang pemantauan UP/DOWN IP tertentu dengan skrip notifikasi Telegram otomatis | `{"host": "192.168.88.2", "bot_token": "...", "chat_id": "...", "device_name": "AP-Utama"}` |
| `POST` | `/api/v1/telegram/list-monitors` | Daftar monitor Netwatch yang sedang aktif | `{}` |
| `POST` | `/api/v1/telegram/remove-monitor` | Hapus monitor Netwatch | `{"id": "*1"}` |

---

### HH. 🎮 Traffic Shaper: Mangle, Queue Tree & Pisah Trafik (`/api/v1/traffic/*`)
Manajemen rekayasa trafik tingkat lanjut, penandaan paket (packet marking), dan pemisahan prioritas:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/traffic/mangle/rules` | Daftar aturan firewall mangle | `{}` |
| `POST` | `/api/v1/traffic/mangle/add` | Tambah aturan mangle (`mark-connection`, `mark-packet`) | `{"chain": "prerouting", "action": "mark-packet", "new_packet_mark": "pkt_game"}` |
| `POST` | `/api/v1/traffic/queue-tree` | Daftar aturan hierarkis Queue Tree | `{}` |
| `POST` | `/api/v1/traffic/queue-tree/add` | Tambah Queue Tree | `{"name": "Games", "parent": "global", "packet_mark": "pkt_game", "priority": 1}` |
| `POST` | `/api/v1/traffic/preset/game-social-separation` | **Deploy Pisah Trafik Game & Sosmed Otomatis**: Menjamin latency game online (Mobile Legends, PUBG, Valorant, dll.) tetap stabil dan memisahkan kuota streaming | `{"total_bandwidth": "50M", "game_reserved": "10M"}` |

---

### II. 🧰 NOC Pro Diagnostic Suite (`/api/v1/tools/*` & `/api/v1/system/*`)
Perangkat diagnostik lengkap untuk Network Operations Center (NOC):
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/tools/torch` | **Live Torch Sniffer**: Memantau aliran bandwidth per IP/port secara real-time pada interface | `{"interface": "ether1", "port": "443"}` |
| `POST` | `/api/v1/tools/ip-scan` | **IP Scanner**: Memindai perangkat aktif pada subnet IP tertentu dari router | `{"interface": "ether2", "address_range": "192.168.88.0/24"}` |
| `POST` | `/api/v1/tools/romon/status` | Cek status RoMON (Router Management Overlay Network) | `{}` |
| `POST` | `/api/v1/tools/romon/set` | Aktifkan/nonaktifkan RoMON | `{"enabled": true}` |
| `POST` | `/api/v1/tools/romon/discover` | Temukan router tetangga dalam jaringan overlay RoMON | `{}` |
| `POST` | `/api/v1/system/watchdog` | Konfigurasi hardware watchdog (reboot otomatis jika ISP putus) | `{"watchdog_timer": true, "ping_address": "8.8.8.8"}` |
| `POST` | `/api/v1/system/ntp` | Sinkronisasi waktu jam NTP Client (Penting untuk log & voucher) | `{"enabled": true, "servers": "0.pool.ntp.org", "time_zone_name": "Asia/Jakarta"}` |
| `POST` | `/api/v1/ip/dhcp-client/all` | Daftar DHCP Client WAN | `{}` |
| `POST` | `/api/v1/ip/dhcp-client/add` | Tambah DHCP Client pada interface WAN | `{"interface": "ether1", "add_default_route": true}` |
| `POST` | `/api/v1/scripts/terminal-exec` | **Terminal Paste Script Runner**: Menjalankan batch script panjang arbitrary hasil copas terminal Winbox secara atomik | `{"script": "/ip firewall filter add chain=input action=accept..."}` |

---

### JJ. 🚫 App & Content Blocker (`/api/v1/security/app-*`)
Pemblokiran aplikasi dan konten berbahaya/mengganggu sekali klik dengan penandaan komentar otomatis di Winbox (`[App-Blocker]`):
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/security/app-block` | **Blokir Aplikasi Sekali Klik**: Pilihan `whatsapp`, `tiktok`, `youtube`, `judi_online`, `torrent_p2p`, atau `custom` | `{"app_type": "whatsapp"}` atau `{"app_type": "custom", "custom_domain": "roblox.com"}` |
| `POST` | `/api/v1/security/app-unblock` | **Buka Blokir Aplikasi**: Menghapus aturan filter firewall aplikasi terkait secara otomatis | `{"app_type": "whatsapp"}` |
| `POST` | `/api/v1/security/blocked-apps` | Daftar seluruh aturan pemblokiran aplikasi yang sedang aktif | `{}` |

---

### KK. 📵 Anti-Tethering & Anti-WiFi Sharing Protection (`/api/v1/security/anti-tethering/*`)
Mencegah pengguna Hotspot/Voucher berbagi koneksi internet menggunakan **WiFi QR Code Sharing, Bluetooth Tethering, atau USB Tethering**:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/security/anti-tethering/enable` | **Aktifkan Proteksi Anti-Tethering**: Menyuntikkan aturan Mangle `change-ttl new-ttl=set:1` pada interface Hotspot dan memaksa profil user `shared-users=1` | `{"hotspot_interface": "bridge-hotspot", "enforce_shared_users_one": true}` |
| `POST` | `/api/v1/security/anti-tethering/disable` | Nonaktifkan proteksi anti-tethering | `{}` |
| `POST` | `/api/v1/security/anti-tethering/status` | Cek status aktifnya proteksi anti-tethering | `{}` |

*Penjelasan Mekanisme Teknis: Nilai TTL paket data yang keluar menuju HP pengguna dipaksa menjadi 1. Jika HP tersebut mencoba mem-forward internet ke HP lain (tethering/QR share), OS HP akan mengurangi TTL menjadi 0, sehingga paket data otomatis dibuang (dropped) dan internet di HP kedua tidak jalan sama sekali.*

---

### LL. 🔀 NAT & Port Forwarding Wizard (`/api/v1/firewall/port-forward/*`)
Membuka akses perangkat lokal (CCTV, Web Server, Game Server, Billing) ke internet publik:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/firewall/port-forward` | **Port Forwarding Sekali Klik**: Otomatis membuat aturan Dst-NAT dan membuka Firewall Filter forward accept | `{"dst_port": "8000", "to_addresses": "192.168.88.50", "to_ports": "80", "protocol": "tcp"}` |
| `POST` | `/api/v1/firewall/port-forward/list` | Daftar seluruh aturan Dst-NAT port forwarding aktif | `{}` |
| `POST` | `/api/v1/firewall/port-forward/remove`| Hapus aturan port forwarding | `{"id": "*1"}` |
| `POST` | `/api/v1/firewall/srcnat/masquerade` | Tambah aturan dasar Src-NAT Masquerade untuk WAN | `{"out_interface": "ether1"}` |

---

### MM. 🧙‍♂️ 1-Klik Complete Hotspot Setup Template Wizard (`POST /api/v1/hotspot/wizard/setup`)
Membangun infrastruktur Hotspot lengkap dari nol dalam 1 kali eksekusi atomik (< 50 milidetik). Semua aturan otomatis diberi penanda komentar rapi `[Wizard]`:
```json
{
  "interface": "ether2",
  "local_address": "192.168.50.1/24",
  "dhcp_pool_range": "192.168.50.10-192.50.254",
  "dns_name": "wifi.kafe.net",
  "hotspot_name": "KAFE-WIFI",
  "admin_user": "admin",
  "admin_password": "mypassword123"
}
```
*Langkah yang otomatis dijalankan: Assign IP Gateway ➔ IP Pool ➔ DHCP Server & Network ➔ Hotspot Profile ➔ Hotspot Server Instance ➔ User Profile (Anti-Share) ➔ Admin User ➔ Src-NAT Masquerade.*

---

### NN. 🚀 Resource Hog Detective & FastTrack CPU Accelerator (`/api/v1/system/*`)
Dirancang khusus agar router dengan spesifikasi kecil (seperti hAP mini, hAP lite, RB750Gr3) tidak mengalami lonjakan CPU 100% atau freeze:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/system/cpu-profiler` | **CPU & RAM Hog Detective**: Menjalankan `/tool/profile` untuk mendeteksi proses apa yang memakan CPU (`networking`, `firewall`, `dns`, `queues`) dan memberikan diagnosa penyebab | `{}` |
| `POST` | `/api/v1/system/fasttrack/deploy` | **Pasang FastTrack Connection**: Menurunkan beban CPU hingga 80% pada router kecil dengan membypass connection tracking untuk paket established/related | `{}` |

---

### OO. 🏊 Pool, DHCP Network, Profil Hotspot & Routing Statis Lengkap
Melengkapi operasi CRUD penuh pada level infrastruktur IP RouterOS:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/ip/pool/add` | Tambah IP Pool baru | `{"name": "pool-tamu", "ranges": "192.168.99.10-192.168.99.200", "comment": "Pool Tamu"}` |
| `POST` | `/api/v1/ip/pool/remove` | Hapus IP Pool | `{"id": "*1"}` |
| `POST` | `/api/v1/dhcp/networks` | Daftar konfigurasi Network DHCP Server | `{}` |
| `POST` | `/api/v1/dhcp/network/add` | Tambah Network DHCP Server | `{"address": "192.168.99.0/24", "gateway": "192.168.99.1", "dns_server": "8.8.8.8"}` |
| `POST` | `/api/v1/dhcp/network/remove` | Hapus Network DHCP Server | `{"id": "*1"}` |
| `POST` | `/api/v1/hotspot/profile/add` | Tambah Profil Pengguna Hotspot (Limit & Kuota) | `{"name": "paket-5mbps", "rate_limit": "5M/5M", "shared_users": "1"}` |
| `POST` | `/api/v1/hotspot/profile/remove`| Hapus Profil Hotspot | `{"id": "*1"}` |
| `POST` | `/api/v1/ip/route/add` | Tambah Static IP Route (Failover / Gateway) | `{"dst_address": "0.0.0.0/0", "gateway": "192.168.1.1", "check_gateway": "ping"}` |
| `POST` | `/api/v1/ip/route/remove` | Hapus Static Route | `{"id": "*1"}` |

---

### PP. 🔍 Queue & User Bandwidth Live Inspection (`/api/v1/queues/*`)
Menjawab kebutuhan administrator untuk mengetahui rincian limit kecepatan user, penggunaan kuota real-time, dan status bottleneck:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/queues/inspect-user` | **Inspeksi Lengkap Limit & Kecepatan User**: Mengetahui limit max (`5M/10M`), kecepatan real-time upload & download saat ini, persentase utilisasi antrean, kuota total transfer, packet drops, dan status antrean (`THROTTLED` / `ACTIVE` / `IDLE`) | `{"query": "192.168.88.50"}` atau `{"query": "salman"}` |
| `POST` | `/api/v1/queues/overview-summary` | **Executive NOC Bandwidth Dashboard**: Menghitung total bandwidth yang dialokasikan router vs konsumsi real-time, top 5 user downloader terberat, dan daftar antrean yang sedang bottleneck | `{}` |
| `POST` | `/api/v1/queues/test-limit` | **Uji Validasi Limit Kecepatan**: Memverifikasi apakah target IP dibatasi dengan benar sesuai batas max-limit yang ditetapkan | `{"target_ip": "192.168.88.50"}` |

---

### QQ. ⚖️ 1-Klik Multi-WAN PCC Load Balancing Wizard (`/api/v1/load-balance/*`)
Mengotomatisasi kerumitan konfigurasi Per Connection Classifier (PCC) 2-WAN atau Multi-WAN menjadi 1 kali panggilan API (< 30 milidetik):
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/load-balance/pcc/setup` | **Wizard PCC Multi-WAN Sekali Klik**: Otomatis membuat aturan Mangle hashing, routing mark, fallback routing distance, check-gateway ping failover, dan NAT masquerade. Mendukung rasio bobot tidak seimbang (misal 50M vs 100M dengan weight 1:2) | `{"lan_interface": "bridge", "wans": [{"interface": "ether1", "gateway": "192.168.1.1", "weight": 1}, {"interface": "ether2", "gateway": "192.168.2.1", "weight": 1}], "matcher": "both-addresses", "auto_failover": true}` |
| `POST` | `/api/v1/load-balance/status` | **Monitor Keseimbangan Trafik Multi-WAN**: Membaca packet & byte counters pada setiap jalur PCC dan interface WAN untuk memastikan beban terdistribusi seimbang | `{}` |
| `POST` | `/api/v1/load-balance/remove` | **Hapus Konfigurasi Load Balancing**: Menghapus seluruh aturan `[PCC-LoadBalance]` dan mengembalikan router ke mode single WAN secara aman tanpa menyentuh aturan lain | `{}` |

---

### RR. 📊 Native Prometheus Telemetry Exporter (`GET /metrics`)
Standar industri untuk scraping otomatis tanpa perlu mengaktifkan daemon SNMP di MikroTik:
| Method | Endpoint | Kegunaan | Parameter Query / Headers |
|---|---|---|---|
| `GET` | `/metrics` | **Prometheus Standard Text Exposition (v0.0.4)**: Mengekspos metrik live CPU, RAM, free storage, uptime, PPPoE online count, Hotspot active count, dan RX/TX byte/packet/drop counter per interface untuk Grafana / Prometheus | `?host=192.168.88.1&token=...` atau header Bearer |

---

### SS. 🩺 Heuristic "Doctor MikroTik" Network Diagnostic Assistant (`/api/v1/doctor/diagnose`)
Melakukan audit komprehensif terhadap 6 pilar kesehatan router dan jaringan dalam 1 kali eksekusi:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `GET` / `POST` | `/api/v1/doctor/diagnose` | **Audit Kesehatan Jaringan Cerdas**: Menguji beban CPU, sisa RAM, latensi & packet loss uplink ISP ke 1.1.1.1, antrean bandwidth yang mengalami throttling, status akselerasi FastTrack, dan drop packet LAN/AP. Mengembalikan Skor Kesehatan (0-100) dan rekomendasi perbaikan dalam Bahasa Indonesia & English | `{}` |

---

### TT. 🗺️ Carrier BGP & OSPF Dynamic Routing Peering Telemetry (`/api/v1/routing/*`)
Monitoring routing dinamis untuk router core ISP, backbone BGP, dan datacenter (kompatibel RouterOS v6 & v7):
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `GET` / `POST` | `/api/v1/routing/bgp/sessions` | **Carrier BGP Peering Sessions**: Status peering BGP (*Established, Active, Idle*), AS Number, prefix count yang diterima, dan uptime sesi | `{}` |
| `GET` / `POST` | `/api/v1/routing/ospf/neighbors` | **OSPF Neighbor Adjacencies**: Status neighbor (*Full, 2-Way, Init*), router-id, interface link, dan priority | `{}` |
| `GET` / `POST` | `/api/v1/routing/routes` | **Rekapitulasi Tabel Routing**: Rangkuman rute aktif beserta rincian protokol (*Connected, Static, BGP, OSPF*) dan jarak administratif | `{}` |

---

### UU. 🎯 Multi-Target SLA Ping Matrix (`/api/v1/tools/multi-ping`)
Menguji matriks konektivitas uplink WAN dan DNS publik sekaligus dalam 1 kali panggilan API:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `GET` / `POST` | `/api/v1/tools/multi-ping` | **SLA Ping Matrix Otomatis**: Ping paralel/sekuensial ke Default Gateway dan benchmark publik (`1.1.1.1`, `8.8.8.8`, `208.67.222.222`). Menghasilkan persentase packet loss, RTT min/avg/max, jitter, dan status SLA (`ALL_ONLINE_OPTIMAL`, `PARTIAL_OUTAGE_DEGRADED`, `CRITICAL_LINK_DOWN`) | `{"targets": ["1.1.1.1", "8.8.8.8", "208.67.222.222"], "count": 3}` |

---

### VV. 🛡️ Native RouterOS v7 AdList DNS Blocker (`/api/v1/dns/adlist/*`)
Memanfaatkan fitur native AdList RouterOS v7.12+ untuk memblokir jutaan domain iklan dan malware setara Pi-hole tanpa hardware eksternal:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `GET` / `POST` | `/api/v1/dns/adlist` | **Daftar Sumber AdList**: Melihat blocklist eksternal yang sedang aktif di-load oleh DNS cache MikroTik | `{}` |
| `POST` | `/api/v1/dns/adlist/add` | **Tambah URL Blocklist**: Mendaftarkan URL file host/adblock eksternal | `{"url": "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/light.txt", "ssl_verify": false}` |
| `POST` | `/api/v1/dns/adlist/remove` | **Hapus Sumber AdList**: Menghapus blocklist dari MikroTik | `{"id": "*1"}` |
| `GET` | `/api/v1/dns/adlist/presets` | **Katalog Blocklist Aman**: Menampilkan daftar preset terkurasi (HaGeZi Light, StevenBlack, Anti-Malware TIF, AdGuard) | `{}` |
| `POST` | `/api/v1/dns/adlist/deploy-preset` | **1-Klik Deploy Blocklist**: Menerapkan preset blocklist langsung ke router | `{"preset": "hagezi-light"}` |

---

### WW. 🔍 Conntrack Top Talkers & DDoS Session Killer (`/api/v1/firewall/connections/*`)
Menginspeksi tabel *Connection Tracking* router untuk mitigasi darurat saat jaringan padat atau router mengalami lonjakan CPU:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `GET` / `POST` | `/api/v1/firewall/connections` | **Tabel Conntrack Aktif**: Melihat daftar koneksi TCP/UDP/ICMP aktif, IP asal, IP tujuan, timeout, dan flag akselerasi | `{"limit": 100, "protocol": "tcp"}` |
| `GET` / `POST` | `/api/v1/firewall/connections/top-talkers` | **Deteksi 15 Host Pembunuh Sesi (Top Talkers)**: Mengagregasikan seluruh koneksi aktif berdasarkan IP sumber untuk mengungkap mesin yang menjalankan BitTorrent, botnet, atau DDoS flood | `{}` |
| `POST` | `/api/v1/firewall/connections/flush` | **Flush / Putus Sesi Tertentu**: Memutuskan paksa seluruh koneksi dari IP target atau protokol tertentu | `{"src_address": "192.168.1.50"}` |

---

### XX. 🚨 Rogue DHCP Server Watchdog (`/api/v1/dhcp/alerts` & `/alert/add`)
Mencegah insiden downtime massal di jaringan RT-RW Net atau kos-kosan akibat pelanggan salah colok port LAN router:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `GET` / `POST` | `/api/v1/dhcp/alerts` | **Daftar Alarm Rogue DHCP**: Melihat status monitoring server DHCP liar di setiap interface | `{}` |
| `POST` | `/api/v1/dhcp/alert/add` | **Pasang Pengawas Rogue DHCP**: Menugaskan router untuk memicu alert saat terdeteksi broadcast DHCP server tidak dikenal di port LAN | `{"interface": "ether5", "valid_server": "192.168.100.1"}` |
| `POST` | `/api/v1/dhcp/alert/remove` | **Hapus Pengawas Rogue DHCP**: Menghapus aturan alert | `{"id": "*1"}` |

---

### YY. 🎫 Auto-Login QR Code Hotspot Vouchers (`/api/v1/hotspot/voucher-template/render`)
Mempermudah pengguna akhir login ke jaringan Hotspot tanpa perlu mengetik manual username dan password:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/hotspot/voucher-template/render` | **Render Voucher Siap Cetak (A4 Grid & Thermal 58mm)**: Menghasilkan dokumen cetak HTML lengkap dengan **QR Code Auto-Login** (`http://{dns}/login?username={user}&password={pass}`). Pengguna cukup memindai dengan kamera ponsel untuk langsung login | `{"template_type": "grid", "title": "WIFI WARUNG", "dns_name": "hotspot.net", "vouchers": [...]}` |

---

### ZZ. 📡 CAPsMAN Centralized Wireless AP Management (`/api/v1/capsman/*`)
Mengontrol armada Access Point MikroTik (cAP ax, wAP ac, hAP) secara terpusat dari satu router controller:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `GET` / `POST` | `/api/v1/capsman/radios` | **Daftar Radio AP Terhubung**: Melihat seluruh radio fisik AP yang terhubung ke controller | `{}` |
| `GET` / `POST` | `/api/v1/capsman/interfaces` | **Interface Virtual AP**: Daftar interface radio yang sedang aktif di-manage | `{}` |
| `GET` / `POST` | `/api/v1/capsman/registrations` | **Klien Wireless Terhubung**: Melihat seluruh HP/laptop yang terhubung di tiap AP, sinyal dBm, uptime, dan bitrate | `{}` |
| `POST` | `/api/v1/capsman/set-wifi` | **Ubah SSID & Password Terpusat**: Otomatis membuat/memperbarui security profile, master configuration, provisioning rule, dan mem-push SSID baru ke seluruh AP sekaligus | `{"ssid": "HOTSPOT-WARUNG", "passphrase": "password123", "country": "indonesia"}` |
| `POST` | `/api/v1/capsman/provision` | **Paksa Reprovisioning**: Memicu AP untuk mengambil ulang konfigurasi dari controller | `{"radio_mac": "XX:XX:XX:XX:XX:XX"}` |

---

### AAA. 💾 Binary Restore, Script Importer & File Reader (`/api/v1/backup/*`)
Pemulihan penuh router dari file cadangan dan eksekusi skrip:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/backup/restore` | **Restore File Binary (.backup)**: Memerintahkan RouterOS me-load file backup dan melakukan reboot otomatis | `{"name": "backup-auto.backup", "password": "optional-pass"}` |
| `POST` | `/api/v1/backup/import` | **Import Skrip Konfigurasi (.rsc)**: Mengimpor dan mengeksekusi skrip konfigurasi RouterOS | `{"file": "config.rsc", "verbose": false}` |
| `POST` | `/api/v1/files/read` | **Baca Isi File Teks**: Membaca isi file teks skrip atau log yang tersimpan di storage flash router | `{"file": "config.rsc"}` |

---

### BBB. ⚡ PoE Remote Hard Reboot (`/api/v1/network/infrastructure/poe-cycle`)
Memulihkan Access Point fisik yang membeku/hang di tiang tower tanpa perlu mendatangi lokasi:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/network/infrastructure/poe-cycle` | **PoE Power Cycle Reboot**: Mematikan daya PoE pada port fisik (misal `ether5`) selama 3 detik lalu menyalakannya kembali untuk me-restart fisik AP yang terhubung | `{"interface": "ether5", "off_seconds": 3}` |

---

### CCC. 🌐 Third-Party AP Web GUI Remote Tunnel (`/api/v1/network/infrastructure/ap-tunnel`)
Membuka akses remote ke Web Admin Access Point pihak ketiga (TP-Link, Ruijie, Tenda) dari luar LAN:
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/network/infrastructure/ap-tunnel` | **Buka Port Forwarding Sementara**: Membuat aturan NAT dst-nat sementara pada router agar Web Admin AP (`192.168.100.3:80`) dapat dibuka melalui port eksternal router (misal `8083`) | `{"ap_ip": "192.168.100.3", "ap_port": 80, "external_port": 8083}` |
| `POST` | `/api/v1/network/infrastructure/ap-tunnel/remove` | **Tutup Port Forwarding**: Menghapus aturan NAT port forward setelah selesai konfigurasi | `{"ap_ip": "192.168.100.3"}` |




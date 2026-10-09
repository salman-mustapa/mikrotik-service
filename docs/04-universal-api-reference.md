# Referensi Lengkap Universal API Gateway (Semua Fitur MikroTik)

Gateway MikroTik Rust mengekspos endpoint REST dan SSE universal. Semua request dapat dijalankan oleh stack bahasa apa pun (Web, Mobile, Backend) dengan format JSON standar.

---

## 1. Mekanisme Kredensial Router Target

Setiap request ke endpoint `/api/v1/*` dapat menentukan router target melalui **salah satu dari 3 cara**:

### Opsi A: Melalui HTTP Headers (Direkomendasikan untuk REST & Mobile)
```http
Authorization: Bearer <API_TOKEN>
X-Router-Host: ath.vpnbersama.us
X-Router-Port: 51121
X-Router-User: admin
X-Router-Pass: password123
Content-Type: application/json
```

### Opsi B: Melalui JSON Body
```json
{
  "router": {
    "host": "ath.vpnbersama.us",
    "port": 51121,
    "user": "admin",
    "password": "password123"
  }
}
```

### Opsi C: Melalui Router ID (Yang sudah terdaftar di `config.toml`)
```json
{
  "router_id": "main"
}
```

---

## 2. Katalog Lengkap Seluruh Modul Endpoint

### ⚡ 0. Fast-Path Overview Snapshot (Sub-Millisecond Aggregation)
Endpoint ini menggabungkan 6 query router terpisah menjadi 1 eksekusi simultan (`tokio::join!`) di single persistent socket:
| Method | Endpoint | Kegunaan | Payload |
|---|---|---|---|
| `POST` | `/api/v1/overview` | Aggregated Snapshot: Identity, System Resource, RouterBOARD, Hotspot Active, PPP Active, Interface states | `{}` |

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




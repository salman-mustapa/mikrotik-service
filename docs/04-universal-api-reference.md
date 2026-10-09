# Referensi Lengkap Universal API Gateway

Gateway MikroTik Rust mengekspos endpoint REST dan SSE universal. Semua request dapat dijalankan oleh stack bahasa apa pun (Web, Mobile, Backend) dengan format JSON standar.

---

## 1. Mekanisme Spesifikasi Router Target

Setiap request ke endpoint `/api/v1/*` dapat menentukan router target melalui **salah satu dari 3 cara**:

### Cara A: Melalui HTTP Headers (Sangat direkomendasikan untuk REST & Mobile)
```http
Authorization: Bearer <API_TOKEN>
X-Router-Host: ath.vpnbersama.us
X-Router-Port: 51121
X-Router-User: admin
X-Router-Pass: password123
Content-Type: application/json
```

### Cara B: Melalui JSON Body (Objek `router`)
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

### Cara C: Melalui Router ID (Jika sudah didaftarkan di `config.toml`)
```json
{
  "router_id": "main"
}
```
*(Atau header `X-Router-Id: main`)*

---

## 2. Katalog Endpoint Modular

### A. System Management (`/api/v1/system/*`)
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

### B. IPv4 & Routing (`/api/v1/ip/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/ip/addresses` | Daftar IP Address | `{"filter": {"?disabled": "no"}}` |
| `POST` | `/api/v1/ip/address/add` | Tambah IP Address | `{"address": "192.168.10.1/24", "interface": "ether2", "comment": "LAN"}` |
| `POST` | `/api/v1/ip/address/remove` | Hapus IP Address | `{"id": "*1"}` |
| `POST` | `/api/v1/ip/routes` | Tabel routing IPv4 | `{}` |
| `POST` | `/api/v1/ip/dns` | Konfigurasi DNS Server | `{}` |
| `POST` | `/api/v1/ip/pools` | Daftar IP Pool | `{}` |

---

### C. IPv6 Management (`/api/v1/ipv6/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/ipv6/addresses` | Daftar IPv6 Address | `{}` |
| `POST` | `/api/v1/ipv6/address/add` | Tambah IPv6 Address | `{"address": "2001:db8::1/64", "interface": "ether1", "advertise": true}` |
| `POST` | `/api/v1/ipv6/address/remove`| Hapus IPv6 Address | `{"id": "*1"}` |
| `POST` | `/api/v1/ipv6/routes` | Tabel routing IPv6 | `{}` |
| `POST` | `/api/v1/ipv6/pools` | IPv6 Pool | `{}` |
| `POST` | `/api/v1/ipv6/nd` | Neighbor Discovery (ND) | `{}` |

---

### D. DHCP Server & Leases (`/api/v1/dhcp/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/dhcp/servers` | Daftar server DHCP | `{}` |
| `POST` | `/api/v1/dhcp/leases` | Daftar klien yang mendapat IP DHCP | `{}` |
| `POST` | `/api/v1/dhcp/lease/make-static`| Kunci IP dinamis jadi IP statis | `{"id": "*2"}` |
| `POST` | `/api/v1/dhcp/lease/remove` | Hapus lease klien | `{"id": "*2"}` |

---

### E. Hotspot & Vouchers (`/api/v1/hotspot/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/hotspot/users` | Daftar seluruh voucher/user | `{}` |
| `POST` | `/api/v1/hotspot/user/create` | Buat voucher baru | `{"name": "vc10", "password": "pass", "profile": "default", "timelimit": "3h"}` |
| `POST` | `/api/v1/hotspot/user/remove` | Hapus voucher/user | `{"id": "*5"}` |
| `POST` | `/api/v1/hotspot/active` | Daftar user yang **sedang online** | `{}` |
| `POST` | `/api/v1/hotspot/kick` | **Kick** (putuskan koneksi) user online | `{"id": "*3"}` |
| `POST` | `/api/v1/hotspot/profiles` | Profil paket voucher | `{}` |
| `POST` | `/api/v1/hotspot/ip-bindings`| Bypass MAC Address (tanpa login voucher) | `{}` |

---

### F. PPP & PPPoE (`/api/v1/ppp/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/ppp/secrets` | Daftar akun pelanggan PPPoE | `{}` |
| `POST` | `/api/v1/ppp/secret/create` | Tambah akun PPPoE baru | `{"name": "user1", "password": "123", "service": "pppoe", "profile": "default"}` |
| `POST` | `/api/v1/ppp/secret/remove` | Hapus akun PPPoE | `{"id": "*1"}` |
| `POST` | `/api/v1/ppp/active` | Pelanggan PPPoE yang sedang online | `{}` |
| `POST` | `/api/v1/ppp/disconnect` | Putuskan koneksi pelanggan aktif | `{"id": "*4"}` |
| `POST` | `/api/v1/ppp/profiles` | Daftar profil paket PPPoE | `{}` |

---

### G. Firewall & Keamanan (`/api/v1/firewall/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/firewall/filters` | Aturan firewall filter | `{}` |
| `POST` | `/api/v1/firewall/nat` | Aturan NAT (Port forwarding/masquerade) | `{}` |
| `POST` | `/api/v1/firewall/address-lists`| Daftar IP Address List | `{}` |
| `POST` | `/api/v1/firewall/block-ip` | **Blokir IP** (masukkan ke address list) | `{"address": "1.2.3.4", "list": "BLACKLIST", "timeout": "7d"}` |
| `POST` | `/api/v1/firewall/unblock-ip` | Hapus IP dari daftar blokir | `{"id": "*1"}` |

---

### H. Simple Queues (`/api/v1/queues/*`)
| Method | Endpoint | Kegunaan | Payload Tambahan |
|---|---|---|---|
| `POST` | `/api/v1/queues/simple` | Daftar aturan limit bandwidth | `{}` |
| `POST` | `/api/v1/queues/simple/add` | Tambah aturan limit kecepatan | `{"name": "Limit-1", "target": "192.168.1.50/32", "max_limit": "2M/10M"}` |
| `POST` | `/api/v1/queues/simple/set-limit`| Ubah limit kecepatan | `{"id": "*1", "max_limit": "5M/20M"}` |
| `POST` | `/api/v1/queues/simple/remove` | Hapus aturan limit | `{"id": "*1"}` |

---

### I. Interfaces & Realtime Traffic Streaming
| Method | Endpoint | Kegunaan |
|---|---|---|
| `POST` | `/api/v1/interfaces/all` | Daftar semua interface ethernet & bridge |
| `POST` | `/api/v1/interfaces/sample-traffic` | Ambil 1 snapshot kecepatan traffic saat ini (`{"interface": "ether1"}`) |
| `GET` | `/api/v1/interfaces/stream` | **SSE Live Stream Traffic** (Query params: `host`, `port`, `user`, `password`, `interface=ether1`) |

---

### J. Universal Raw Command Runner (`/api/v1/command`)
Dapat menjalankan perintah RouterOS apa saja tanpa batas:
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

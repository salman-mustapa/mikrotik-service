# Katalog Lengkap Command MikroTik RouterOS API

Dokumen ini adalah kamus referensi command yang dapat dieksekusi melalui endpoint `POST /routers/:id/command` pada Rust Gateway.

---

## 1. Aturan Penulisan Command & Arguments

Setiap request ke Gateway Rust menggunakan format JSON:
```json
{
  "command": "<NAMA_COMMAND>",
  "args": {
    "<KEY>": "<VALUE>"
  }
}
```

### Konvensi Prefix Key:
| Prefix | Fungsi | Contoh | Arti |
|---|---|---|---|
| *(Tanpa prefix)* | Menyetel parameter / atribut | `"name": "user1"` | Menyetel atribut `name` bernilai `user1` |
| `.` | Kontrol API MikroTik | `".proplist": ".id,name"` | Hanya meminta field `.id` dan `name` (menghemat bandwidth & CPU) |
| `?` | Filter / Pencarian data | `"?disabled": "no"` | Hanya ambil record yang properti `disabled` bernilai `no` |

---

## 2. Katalog Berdasarkan Modul

### A. System & Hardware
| Command | Kegunaan | Contoh Args |
|---|---|---|
| `/system/resource/print` | Info CPU, RAM, Uptime, Versi OS | `{}` |
| `/system/routerboard/print` | Model router, Serial Number, Firmware | `{}` |
| `/system/identity/print` | Nama host / identitas router | `{}` |
| `/system/identity/set` | Mengubah nama host router | `{"name": "Router-Utama"}` |
| `/system/reboot` | Reboot router | `{}` |
| `/system/clock/print` | Jam, tanggal, timezone | `{}` |
| `/system/package/print` | Daftar package terpasang | `{}` |

---

### B. Interface & Trafik
| Command | Kegunaan | Contoh Args |
|---|---|---|
| `/interface/print` | Daftar semua interface fisik/virtual | `{".proplist": ".id,name,type,running,disabled"}` |
| `/interface/set` | Ubah status / comment interface | `{"disabled": "yes"}`, `{"comment": "ISP 1"}` |
| `/interface/monitor-traffic` | **Live Traffic Monitoring (Stream via SSE)** | `{"interface": "ether1"}` |
| `/interface/ethernet/print` | Info port ethernet (speed, link, duplex) | `{}` |
| `/interface/bridge/print` | Daftar bridge | `{}` |
| `/interface/bridge/port/print` | Daftar port anggota bridge | `{}` |

---

### C. IP Addressing & Jaringan
| Command | Kegunaan | Contoh Args |
|---|---|---|
| `/ip/address/print` | Lihat alokasi IP Address | `{"?disabled": "no"}` |
| `/ip/address/add` | Tambah IP Address ke interface | `{"address": "192.168.10.1/24", "interface": "ether2", "comment": "LAN Kantor"}` |
| `/ip/address/remove` | Hapus IP Address | `{".id": "*1"}` |
| `/ip/route/print` | Tabel routing | `{".proplist": ".id,dst-address,gateway,distance,active"}` |
| `/ip/route/add` | Tambah static route / default gateway | `{"dst-address": "0.0.0.0/0", "gateway": "192.168.1.1"}` |
| `/ip/dns/print` | Konfigurasi DNS | `{}` |
| `/ip/dns/set` | Set DNS server | `{"servers": "8.8.8.8,1.1.1.1", "allow-remote-requests": "yes"}` |

---

### D. Hotspot (Voucher & Users - Mikhmon Style)
| Command | Kegunaan | Contoh Args |
|---|---|---|
| `/ip/hotspot/print` | Daftar server hotspot | `{}` |
| `/ip/hotspot/user/profile/print` | Daftar profil paket voucher | `{".proplist": ".id,name,rate-limit,shared-users"}` |
| `/ip/hotspot/user/profile/add` | Buat paket voucher baru | `{"name": "Paket-3Jam", "rate-limit": "2M/5M", "shared-users": "1"}` |
| `/ip/hotspot/user/print` | Daftar semua user / voucher | `{".proplist": ".id,name,profile,uptime,bytes-in,bytes-out,comment"}` |
| `/ip/hotspot/user/add` | Generate voucher baru | `{"name": "vc102", "password": "pass", "profile": "Paket-3Jam", "limit-uptime": "3h"}` |
| `/ip/hotspot/user/set` | Ubah password / profil user | `{".id": "*3", "password": "newpassword"}` |
| `/ip/hotspot/user/remove` | Hapus user / voucher | `{".id": "*3"}` |
| `/ip/hotspot/active/print` | User yang **sedang login saat ini** | `{".proplist": ".id,user,address,mac-address,uptime"}` |
| `/ip/hotspot/active/remove` | **Kick** (keluarkan) user yang sedang online | `{".id": "*5"}` |
| `/ip/hotspot/host/print` | Daftar perangkat terhubung ke WiFi Hotspot | `{}` |
| `/ip/hotspot/ip-binding/print` | Bypass perangkat (tanpa login voucher) | `{}` |
| `/ip/hotspot/ip-binding/add` | Tambah bypass MAC Address | `{"mac-address": "AA:BB:CC:DD:EE:FF", "type": "bypassed"}` |

---

### E. PPPoE & VPN
| Command | Kegunaan | Contoh Args |
|---|---|---|
| `/ppp/profile/print` | Profil paket PPPoE | `{}` |
| `/ppp/secret/print` | Daftar akun pelanggan PPPoE | `{".proplist": ".id,name,service,profile,remote-address"}` |
| `/ppp/secret/add` | Buat akun pelanggan PPPoE baru | `{"name": "pelanggan_01", "password": "secret", "service": "pppoe", "profile": "50Mbps"}` |
| `/ppp/secret/set` | Nonaktifkan / aktifkan akun pelanggan | `{".id": "*2", "disabled": "yes"}` |
| `/ppp/secret/remove` | Hapus akun PPPoE | `{".id": "*2"}` |
| `/ppp/active/print` | Pelanggan PPPoE yang sedang online | `{".proplist": ".id,name,service,address,uptime,caller-id"}` |
| `/ppp/active/remove` | Putuskan koneksi pelanggan aktif | `{".id": "*4"}` |

---

### F. DHCP Server & Leases
| Command | Kegunaan | Contoh Args |
|---|---|---|
| `/ip/dhcp-server/print` | Daftar DHCP Server | `{}` |
| `/ip/dhcp-server/lease/print` | Daftar IP yang dibagikan ke klien | `{".proplist": ".id,address,mac-address,host-name,status,dynamic"}` |
| `/ip/dhcp-server/lease/make-static` | Kunci IP DHCP menjadi IP Tetap | `{".id": "*1"}` |
| `/ip/dhcp-server/lease/remove` | Hapus lease | `{".id": "*1"}` |

---

### G. Simple Queues (Bandwidth Limiter)
| Command | Kegunaan | Contoh Args |
|---|---|---|
| `/queue/simple/print` | Daftar aturan limit bandwidth | `{".proplist": ".id,name,target,max-limit,bytes"}` |
| `/queue/simple/add` | Buat limit bandwidth baru | `{"name": "Limit-Admin", "target": "192.168.1.100/32", "max-limit": "5M/10M"}` |
| `/queue/simple/set` | Ubah limit kecepatan | `{".id": "*1", "max-limit": "10M/20M"}` |
| `/queue/simple/remove` | Hapus aturan limit | `{".id": "*1"}` |

---

### H. Firewall & Security
| Command | Kegunaan | Contoh Args |
|---|---|---|
| `/ip/firewall/filter/print` | Aturan firewall filter | `{}` |
| `/ip/firewall/nat/print` | Aturan NAT (Port forwarding, Masquerade) | `{}` |
| `/ip/firewall/nat/add` | Tambah Port Forwarding | `{"chain": "dstnat", "action": "dst-nat", "protocol": "tcp", "dst-port": "8080", "to-addresses": "192.168.1.10", "to-ports": "80"}` |
| `/ip/firewall/address-list/print` | Daftar Address List (Daftar IP Blokir) | `{}` |
| `/ip/firewall/address-list/add` | Masukkan IP ke daftar blokir | `{"list": "DROP_ATTACKERS", "address": "1.2.3.4", "timeout": "7d"}` |

---

### I. Diagnostic & Tools
| Command | Kegunaan | Contoh Args |
|---|---|---|
| `/ping` | Ping IP dari router | `{"address": "8.8.8.8", "count": "4"}` |
| `/tool/traceroute` | Traceroute dari router | `{"address": "1.1.1.1", "count": "1"}` |
| `/log/print` | Baca log sistem router | `{".proplist": ".id,time,topics,message"}` |

---

## 3. Tips Optimasi Performa Maksimal

1. **Selalu gunakan `.proplist` jika memungkinkan**:
   MikroTik memiliki ratusan properti per item (seperti statistik byte per detik, packet counter, dll). Menggunakan `.proplist` membuat MikroTik hanya mengirimkan kolom yang dibutuhkan, memangkas beban CPU MikroTik hingga 80% dan membuat respons mendekati instan.
2. **Gunakan filter query `?` di router, bukan di PHP**:
   Daripada menarik 5.000 user lalu difilter di Laravel, gunakan `"?disabled": "no"` langsung di args request agar pemfilteran dilakukan di memori MikroTik.

# MikroTik PHP & Laravel SDK (Powered by Rust Core)

SDK PHP modern (PHP 8.1+) untuk berinteraksi dengan RouterOS MikroTik menggunakan gateway berkinerja tinggi berbasis **Rust**.

---

## ⚡ Cara Kerja

```
Aplikasi Laravel (PHP) ---> HTTP JSON (0.2ms) ---> Rust Core Gateway ---> Persistent TCP (8728) ---> MikroTik
```

SDK ini tidak membuka socket mentah di setiap request (yang lambat dan membebani router). Sebagai gantinya, SDK mengirimkan instruksi ke **Rust Core Gateway** lokal yang menjaga koneksi persisten dan multiplexing tag `.tag`.

---

## 🚀 Cara Penggunaan di Laravel

### 1. Inisialisasi Instan (Direct OOP)
```php
use Mikrotik\MikrotikApp;

// Terhubung ke router lokal atau remote via VPN:
$mikrotikIp   = '192.168.88.1';
$mikrotikPort = 8728;
$mikrotikUser = 'admin';
$mikrotikPass = 'secret';

$app = new MikrotikApp($mikrotikIp, $mikrotikPort, $mikrotikUser, $mikrotikPass);
// atau static shortcut:
$app = MikrotikApp::connect($mikrotikIp, $mikrotikPort, $mikrotikUser, $mikrotikPass);
```

### 2. Membaca System & Resource
```php
// Info CPU, RAM, Uptime
$resource = $app->system()->resource();
echo "CPU Load: " . $resource['cpu-load'] . "%\n";
echo "Uptime: " . $resource['uptime'] . "\n";

// Info Hardware & Serial Number
$board = $app->system()->routerboard();
$identity = $app->system()->identity(); // Nama router
```

### 3. Mengelola IP Address & DHCP (IPv4 & IPv6)
```php
// Ambil semua IPv4
$ips = $app->ip()->addresses();

// Tambah IPv4 baru
$app->ip()->addAddress('192.168.100.1/24', 'ether2', 'LAN Baru');

// Ambil DHCP Leases
$leases = $app->ip()->dhcpLeases();

// Manajemen IPv6
$ipv6Addresses = $app->ipv6()->addresses();
```

### 4. Manajemen Hotspot & Voucher (Mikhmon Style)
```php
// 1. Ambil semua voucher
$users = $app->hotspot()->users();

// 2. Ambil user yang SEDANG ONLINE saat ini
$active = $app->hotspot()->active();

// 3. Generate voucher baru dengan limit waktu 3 jam
$app->hotspot()->createUser(
    username: 'voucher01',
    password: 'password123',
    profile: 'Paket-3Jam',
    timelimit: '3h',
    comment: 'Dibuat via Laravel'
);

// 4. Kick user yang sedang online
$app->hotspot()->kick('*4');
```

### 5. PPPoE & Pelanggan Rumahan
```php
// Daftar akun pelanggan
$secrets = $app->ppp()->secrets();

// Pelanggan yang sedang aktif
$onlinePpp = $app->ppp()->active();

// Tambah pelanggan baru
$app->ppp()->createSecret('user_budi', 'pass123', 'pppoe', 'Paket-20Mbps');
```

### 6. Limit Kecepatan (Simple Queues)
```php
// Buat batasan bandwidth 2 Mbps Upload / 10 Mbps Download
$app->queues()->addSimple('Limit-Warnet', '192.168.1.50/32', '2M/10M');
```

### 7. Keamanan & Blokir IP (Firewall)
```php
// Masukkan IP penyerang / brute force ke Address List
$app->firewall()->blockIp('203.0.113.50', 'BLACKLIST_DROP', '7d', 'Brute force port 80');
```

### 8. Custom Command MikroTik Apa Saja
```php
// Anda dapat menjalankan perintah RouterOS apa pun:
$custom = $app->command('/tool/ping', [
    'address' => '8.8.8.8',
    'count'   => '3',
]);
```

---

## 🛡️ Error Handling
Ketika MikroTik menolak perintah (misalnya user voucher duplikat atau interface tidak ditemukan):
```php
use Mikrotik\Exceptions\MikrotikTrapException;
use Mikrotik\Exceptions\MikrotikException;

try {
    $app->hotspot()->createUser('user_sudah_ada', 'pass');
} catch (MikrotikTrapException $e) {
    // Menangkap pesan asli dari MikroTik (!trap)
    echo "Ditolak MikroTik: " . $e->getMessage();
} catch (MikrotikException $e) {
    echo "Gateway Error: " . $e->getMessage();
}
```

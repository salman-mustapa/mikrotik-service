<?php

namespace Mikrotik\Modules;

use Mikrotik\Client;

class SystemModule
{
    protected Client $client;

    public function __construct(Client $client)
    {
        $this->client = $client;
    }

    /** Ambil CPU, Memory, Uptime, Architecture, Version */
    public function resource(): array
    {
        $res = $this->client->command('/system/resource/print');
        return $res[0] ?? [];
    }

    /** Ambil Model Router, Serial Number, Current Firmware, Upgrade Firmware */
    public function routerboard(): array
    {
        $res = $this->client->command('/system/routerboard/print');
        return $res[0] ?? [];
    }

    /** Ambil Nama Identitas Router */
    public function identity(): string
    {
        $res = $this->client->command('/system/identity/print');
        return $res[0]['name'] ?? 'MikroTik';
    }

    /** Ubah Nama Identitas Router */
    public function setIdentity(string $name): array
    {
        return $this->client->command('/system/identity/set', ['name' => $name]);
    }

    /** Ambil Info Jam, Tanggal, dan Timezone */
    public function clock(): array
    {
        $res = $this->client->command('/system/clock/print');
        return $res[0] ?? [];
    }

    /** Ambil Health Router (Voltage, Temperature, Fan) jika router mendukung */
    public function health(): array
    {
        return $this->client->command('/system/health/print');
    }

    /** Ambil Daftar Paket / Packages RouterOS */
    public function packages(): array
    {
        return $this->client->command('/system/package/print');
    }

    /** Cek Update RouterOS Versi Terbaru */
    public function checkUpdate(): array
    {
        $res = $this->client->command('/system/package/update/check-for-updates');
        return $res[0] ?? [];
    }

    /** Download dan Upgrade RouterOS ke Versi Terbaru */
    public function installUpdate(): array
    {
        return $this->client->command('/system/package/update/install');
    }

    /** Upgrade Firmware RouterBOARD (setelah upgrade RouterOS) */
    public function upgradeFirmware(): array
    {
        return $this->client->command('/system/routerboard/upgrade');
    }

    /** Reboot Router */
    public function reboot(): array
    {
        return $this->client->command('/system/reboot');
    }

    /** Shutdown Router */
    public function shutdown(): array
    {
        return $this->client->command('/system/shutdown');
    }
}

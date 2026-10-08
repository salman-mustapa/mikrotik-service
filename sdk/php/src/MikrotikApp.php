<?php

namespace Mikrotik;

use Mikrotik\Modules\SystemModule;
use Mikrotik\Modules\IpModule;
use Mikrotik\Modules\Ipv6Module;
use Mikrotik\Modules\HotspotModule;
use Mikrotik\Modules\PppModule;
use Mikrotik\Modules\InterfaceModule;
use Mikrotik\Modules\FirewallModule;
use Mikrotik\Modules\QueueModule;
use Mikrotik\Modules\LogModule;

class MikrotikApp
{
    protected Client $client;

    protected ?SystemModule $system = null;
    protected ?IpModule $ip = null;
    protected ?Ipv6Module $ipv6 = null;
    protected ?HotspotModule $hotspot = null;
    protected ?PppModule $ppp = null;
    protected ?InterfaceModule $interfaces = null;
    protected ?FirewallModule $firewall = null;
    protected ?QueueModule $queues = null;
    protected ?LogModule $logs = null;

    public function __construct(
        string $host = '127.0.0.1',
        int $port = 8728,
        string $user = 'admin',
        string $password = '',
        string $gatewayUrl = 'http://127.0.0.1:8080',
        string $gatewayToken = 'change-me-to-a-long-random-string'
    ) {
        $this->client = new Client($host, $port, $user, $password, $gatewayUrl, $gatewayToken);
    }

    /**
     * Fluent Static Constructor
     */
    public static function connect(
        string $host,
        int $port = 8728,
        string $user = 'admin',
        string $password = '',
        string $gatewayUrl = 'http://127.0.0.1:8080',
        string $gatewayToken = 'change-me-to-a-long-random-string'
    ): self {
        return new self($host, $port, $user, $password, $gatewayUrl, $gatewayToken);
    }

    public function getClient(): Client
    {
        return $this->client;
    }

    /**
     * Eksekusi perintah arbitrary / custom secara langsung
     */
    public function command(string $command, array $args = []): array
    {
        return $this->client->command($command, $args);
    }

    /**
     * Modul Manajemen Sistem, Resource, Routerboard, Reboot & Upgrade
     */
    public function system(): SystemModule
    {
        return $this->system ??= new SystemModule($this->client);
    }

    /**
     * Modul IPv4, DHCP Server/Lease, Pools, Routes, DNS, ARP
     */
    public function ip(): IpModule
    {
        return $this->ip ??= new IpModule($this->client);
    }

    /**
     * Modul IPv6, Addresses, Routes, Pools, ND
     */
    public function ipv6(): Ipv6Module
    {
        return $this->ipv6 ??= new Ipv6Module($this->client);
    }

    /**
     * Modul Hotspot, Vouchers, Active Users, Profiles, IP Bindings
     */
    public function hotspot(): HotspotModule
    {
        return $this->hotspot ??= new HotspotModule($this->client);
    }

    /**
     * Modul PPPoE & VPN Secrets, Active, Profiles
     */
    public function ppp(): PppModule
    {
        return $this->ppp ??= new PppModule($this->client);
    }

    /**
     * Modul Interfaces, Ethernet, Bridge, Realtime Monitor Traffic
     */
    public function interfaces(): InterfaceModule
    {
        return $this->interfaces ??= new InterfaceModule($this->client);
    }

    /**
     * Modul Firewall Filter, NAT, Mangle, Address List (Block/Unblock)
     */
    public function firewall(): FirewallModule
    {
        return $this->firewall ??= new FirewallModule($this->client);
    }

    /**
     * Modul Simple Queue & Queue Tree (Bandwidth Limiter)
     */
    public function queues(): QueueModule
    {
        return $this->queues ??= new QueueModule($this->client);
    }

    /**
     * Modul Log Router
     */
    public function logs(): LogModule
    {
        return $this->logs ??= new LogModule($this->client);
    }
}

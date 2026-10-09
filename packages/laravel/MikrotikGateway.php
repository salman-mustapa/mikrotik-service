<?php

namespace App\Services;

use Illuminate\Support\Facades\Http;
use Exception;

/**
 * MikroTik Universal Rust Engine - Laravel Client SDK
 * 
 * Helper client siap pakai untuk Laravel (PHP) layaknya plugin native.
 * Memanggil Rust Gateway berkecepatan tinggi tanpa membebani CPU router MikroTik.
 */
class MikrotikGateway
{
    protected string $gatewayUrl;
    protected string $token;
    protected array $router;

    public function __construct(?string $host = null, ?int $port = null, ?string $user = null, ?string $password = null)
    {
        $this->gatewayUrl = env('MIKROTIK_GATEWAY_URL', 'http://127.0.0.1:8080');
        $this->token = env('MIKROTIK_GATEWAY_TOKEN', 'change-me-to-a-long-random-string');

        $this->router = [
            'host'     => $host ?? env('MIKROTIK_ROUTER_HOST', '192.168.88.1'),
            'port'     => $port ?? (int) env('MIKROTIK_ROUTER_PORT', 8728),
            'user'     => $user ?? env('MIKROTIK_ROUTER_USER', 'admin'),
            'password' => $password ?? env('MIKROTIK_ROUTER_PASS', ''),
        ];
    }

    /**
     * Helper pemanggil endpoint gateway
     */
    protected function post(string $endpoint, array $payload = []): array
    {
        $response = Http::timeout(15)
            ->withToken($this->token)
            ->withHeaders([
                'X-Router-Host' => $this->router['host'],
                'X-Router-Port' => (string) $this->router['port'],
                'X-Router-User' => $this->router['user'],
                'X-Router-Pass' => $this->router['password'],
            ])
            ->post("{$this->gatewayUrl}{$endpoint}", $payload);

        if (!$response->successful()) {
            throw new Exception("MikroTik Gateway Error [{$response->status()}]: " . $response->body());
        }

        return $response->json();
    }

    // ==========================================
    // ⚡ Fast-Path Overview & Live Diagnostics
    // ==========================================

    public function getOverview(): array
    {
        return $this->post('/api/v1/overview');
    }

    public function getConnectedDevices(): array
    {
        return $this->post('/api/v1/network/connected-devices');
    }

    // ==========================================
    // 🔍 Queue & User Bandwidth Inspection
    // ==========================================

    public function inspectUser(string $queryIpOrUser): array
    {
        return $this->post('/api/v1/queues/inspect-user', ['query' => $queryIpOrUser]);
    }

    public function getBandwidthSummary(): array
    {
        return $this->post('/api/v1/queues/overview-summary');
    }

    public function testQueueLimit(string $targetIp): array
    {
        return $this->post('/api/v1/queues/test-limit', ['target_ip' => $targetIp]);
    }

    // ==========================================
    // ⚖️ Multi-WAN PCC Load Balancing
    // ==========================================

    public function setupLoadBalance(string $lanInterface, array $wans, string $matcher = 'both-addresses', bool $autoFailover = true): array
    {
        return $this->post('/api/v1/load-balance/pcc/setup', [
            'lan_interface' => $lanInterface,
            'wans'          => $wans,
            'matcher'       => $matcher,
            'auto_failover' => $autoFailover,
            'add_masquerade'=> true,
        ]);
    }

    public function getLoadBalanceStatus(): array
    {
        return $this->post('/api/v1/load-balance/status');
    }

    public function removeLoadBalance(): array
    {
        return $this->post('/api/v1/load-balance/remove');
    }

    // ==========================================
    // 🚫 Security & Protection (Anti-Tethering & Blocker)
    // ==========================================

    public function enableAntiTethering(string $hotspotInterface = 'bridge'): array
    {
        return $this->post('/api/v1/security/anti-tethering/enable', [
            'hotspot_interface'        => $hotspotInterface,
            'enforce_shared_users_one' => true,
        ]);
    }

    public function blockApp(string $appType, ?string $customDomain = null): array
    {
        $payload = ['app_type' => $appType];
        if ($customDomain) {
            $payload['custom_domain'] = $customDomain;
        }
        return $this->post('/api/v1/security/app-block', $payload);
    }

    public function unblockApp(string $appType): array
    {
        return $this->post('/api/v1/security/app-unblock', ['app_type' => $appType]);
    }

    // ==========================================
    // 🧙‍♂️ Hotspot & Voucher Wizard
    // ==========================================

    public function setupHotspotWizard(array $config): array
    {
        return $this->post('/api/v1/hotspot/wizard/setup', $config);
    }

    public function generateVouchers(int $qty, string $prefix = 'V-', string $profile = 'default', ?string $timelimit = null): array
    {
        $payload = [
            'qty'     => $qty,
            'prefix'  => $prefix,
            'profile' => $profile,
        ];
        if ($timelimit) {
            $payload['timelimit'] = $timelimit;
        }
        return $this->post('/api/v1/hotspot/generate-batch', $payload);
    }

    public function getHotspotUsers(): array
    {
        return $this->post('/api/v1/hotspot/users');
    }

    public function kickHotspotUser(string $activeId): array
    {
        return $this->post('/api/v1/hotspot/kick', ['id' => $activeId]);
    }

    // ==========================================
    // 🔀 Port Forwarding (Dst-NAT)
    // ==========================================

    public function addPortForward(string $dstPort, string $toAddress, string $toPort, string $protocol = 'tcp', ?string $comment = null): array
    {
        return $this->post('/api/v1/firewall/port-forward', [
            'dst_port'     => $dstPort,
            'to_addresses' => $toAddress,
            'to_ports'     => $toPort,
            'protocol'     => $protocol,
            'comment'      => $comment,
        ]);
    }
}

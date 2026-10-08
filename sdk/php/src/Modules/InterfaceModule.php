<?php

namespace Mikrotik\Modules;

use Mikrotik\Client;

class InterfaceModule
{
    protected Client $client;

    public function __construct(Client $client)
    {
        $this->client = $client;
    }

    public function all(array $filter = []): array
    {
        return $this->client->command('/interface/print', $filter);
    }

    public function get(string $name): array
    {
        $res = $this->client->command('/interface/print', ['?name' => $name]);
        return $res[0] ?? [];
    }

    public function enable(string $id): array
    {
        return $this->client->command('/interface/enable', ['.id' => $id]);
    }

    public function disable(string $id): array
    {
        return $this->client->command('/interface/disable', ['.id' => $id]);
    }

    public function setComment(string $id, string $comment): array
    {
        return $this->client->command('/interface/set', ['.id' => $id, 'comment' => $comment]);
    }

    // --- Bridge ---

    public function bridges(): array
    {
        return $this->client->command('/interface/bridge/print');
    }

    public function bridgePorts(): array
    {
        return $this->client->command('/interface/bridge/port/print');
    }

    // --- Ethernet ---

    public function ethernet(): array
    {
        return $this->client->command('/interface/ethernet/print');
    }

    // --- Realtime Traffic URL for Browser / SSE ---

    public function getMonitorTrafficUrl(string $interface): string
    {
        return $this->client->getStreamUrl('/interface/monitor-traffic', [
            'interface' => $interface,
        ]);
    }

    /** Ambil 1 sampel traffic saat ini secara sinkron */
    public function sampleTraffic(string $interface): array
    {
        $res = $this->client->command('/interface/monitor-traffic', [
            'interface' => $interface,
            'once' => '',
        ]);
        return $res[0] ?? [];
    }
}

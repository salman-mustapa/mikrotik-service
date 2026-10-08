<?php

namespace Mikrotik\Modules;

use Mikrotik\Client;

class FirewallModule
{
    protected Client $client;

    public function __construct(Client $client)
    {
        $this->client = $client;
    }

    // --- Filter Rules ---

    public function filters(array $filter = []): array
    {
        return $this->client->command('/ip/firewall/filter/print', $filter);
    }

    public function addFilter(array $params): array
    {
        return $this->client->command('/ip/firewall/filter/add', $params);
    }

    public function removeFilter(string $id): array
    {
        return $this->client->command('/ip/firewall/filter/remove', ['.id' => $id]);
    }

    // --- NAT Rules ---

    public function nat(array $filter = []): array
    {
        return $this->client->command('/ip/firewall/nat/print', $filter);
    }

    public function addPortForward(
        string $protocol,
        int $dstPort,
        string $toAddress,
        int $toPort,
        ?string $comment = null
    ): array {
        $args = [
            'chain'        => 'dstnat',
            'action'       => 'dst-nat',
            'protocol'     => $protocol,
            'dst-port'     => (string) $dstPort,
            'to-addresses' => $toAddress,
            'to-ports'     => (string) $toPort,
        ];
        if ($comment !== null) $args['comment'] = $comment;
        return $this->client->command('/ip/firewall/nat/add', $args);
    }

    // --- Mangle Rules ---

    public function mangle(array $filter = []): array
    {
        return $this->client->command('/ip/firewall/mangle/print', $filter);
    }

    // --- Address Lists (Block / Whitelist IP) ---

    public function addressLists(array $filter = []): array
    {
        return $this->client->command('/ip/firewall/address-list/print', $filter);
    }

    public function blockIp(string $ip, string $listName = 'BLACKLIST', ?string $timeout = '7d', ?string $comment = null): array
    {
        $args = [
            'list'    => $listName,
            'address' => $ip,
        ];
        if ($timeout !== null) $args['timeout'] = $timeout;
        if ($comment !== null) $args['comment'] = $comment;
        return $this->client->command('/ip/firewall/address-list/add', $args);
    }

    public function unblockIp(string $id): array
    {
        return $this->client->command('/ip/firewall/address-list/remove', ['.id' => $id]);
    }

    // --- Connections (Conntrack) ---

    public function connections(array $filter = []): array
    {
        return $this->client->command('/ip/firewall/connection/print', $filter);
    }
}

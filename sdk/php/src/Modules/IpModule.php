<?php

namespace Mikrotik\Modules;

use Mikrotik\Client;

class IpModule
{
    protected Client $client;

    public function __construct(Client $client)
    {
        $this->client = $client;
    }

    // --- IP Address ---

    public function addresses(array $filter = []): array
    {
        return $this->client->command('/ip/address/print', $filter);
    }

    public function addAddress(string $address, string $interface, ?string $comment = null): array
    {
        $args = ['address' => $address, 'interface' => $interface];
        if ($comment !== null) $args['comment'] = $comment;
        return $this->client->command('/ip/address/add', $args);
    }

    public function removeAddress(string $id): array
    {
        return $this->client->command('/ip/address/remove', ['.id' => $id]);
    }

    // --- DHCP Server & Leases ---

    public function dhcpServers(): array
    {
        return $this->client->command('/ip/dhcp-server/print');
    }

    public function dhcpLeases(array $filter = []): array
    {
        return $this->client->command('/ip/dhcp-server/lease/print', $filter);
    }

    public function makeDhcpLeaseStatic(string $id): array
    {
        return $this->client->command('/ip/dhcp-server/lease/make-static', ['.id' => $id]);
    }

    public function removeDhcpLease(string $id): array
    {
        return $this->client->command('/ip/dhcp-server/lease/remove', ['.id' => $id]);
    }

    // --- IP Pool ---

    public function pools(): array
    {
        return $this->client->command('/ip/pool/print');
    }

    public function addPool(string $name, string $ranges): array
    {
        return $this->client->command('/ip/pool/add', ['name' => $name, 'ranges' => $ranges]);
    }

    // --- Routing & DNS ---

    public function routes(array $filter = []): array
    {
        return $this->client->command('/ip/route/print', $filter);
    }

    public function addRoute(string $dstAddress, string $gateway, ?int $distance = null): array
    {
        $args = ['dst-address' => $dstAddress, 'gateway' => $gateway];
        if ($distance !== null) $args['distance'] = (string) $distance;
        return $this->client->command('/ip/route/add', $args);
    }

    public function dns(): array
    {
        $res = $this->client->command('/ip/dns/print');
        return $res[0] ?? [];
    }

    public function setDns(string $servers, bool $allowRemoteRequests = true): array
    {
        return $this->client->command('/ip/dns/set', [
            'servers' => $servers,
            'allow-remote-requests' => $allowRemoteRequests ? 'yes' : 'no',
        ]);
    }

    // --- ARP Table ---

    public function arp(array $filter = []): array
    {
        return $this->client->command('/ip/arp/print', $filter);
    }

    // --- IP Services (api, www, ssh, winbox) ---

    public function services(): array
    {
        return $this->client->command('/ip/service/print');
    }

    public function setServicePort(string $serviceName, int $port): array
    {
        $services = $this->client->command('/ip/service/print', ['?name' => $serviceName]);
        if (empty($services[0]['.id'])) {
            throw new \Exception("Service {$serviceName} not found");
        }
        return $this->client->command('/ip/service/set', [
            '.id' => $services[0]['.id'],
            'port' => (string) $port,
        ]);
    }
}

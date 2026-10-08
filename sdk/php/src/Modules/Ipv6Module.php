<?php

namespace Mikrotik\Modules;

use Mikrotik\Client;

class Ipv6Module
{
    protected Client $client;

    public function __construct(Client $client)
    {
        $this->client = $client;
    }

    /** Ambil Daftar IPv6 Address */
    public function addresses(array $filter = []): array
    {
        return $this->client->command('/ipv6/address/print', $filter);
    }

    /** Tambah Alamat IPv6 ke Interface */
    public function addAddress(string $address, string $interface, bool $advertise = false, ?string $comment = null): array
    {
        $args = [
            'address' => $address,
            'interface' => $interface,
            'advertise' => $advertise ? 'yes' : 'no',
        ];
        if ($comment !== null) $args['comment'] = $comment;
        return $this->client->command('/ipv6/address/add', $args);
    }

    /** Hapus Alamat IPv6 */
    public function removeAddress(string $id): array
    {
        return $this->client->command('/ipv6/address/remove', ['.id' => $id]);
    }

    /** Tabel Routing IPv6 */
    public function routes(array $filter = []): array
    {
        return $this->client->command('/ipv6/route/print', $filter);
    }

    /** Tambah Route IPv6 */
    public function addRoute(string $dstAddress, string $gateway): array
    {
        return $this->client->command('/ipv6/route/add', [
            'dst-address' => $dstAddress,
            'gateway' => $gateway,
        ]);
    }

    /** IPv6 Pool */
    public function pools(): array
    {
        return $this->client->command('/ipv6/pool/print');
    }

    /** Neighbor Discovery (ND) */
    public function nd(): array
    {
        return $this->client->command('/ipv6/nd/print');
    }

    /** IPv6 DHCP Server Binding / Leases */
    public function dhcpServerBindings(): array
    {
        return $this->client->command('/ipv6/dhcp-server/binding/print');
    }
}

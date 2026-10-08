<?php

namespace Mikrotik\Modules;

use Mikrotik\Client;

class HotspotModule
{
    protected Client $client;

    public function __construct(Client $client)
    {
        $this->client = $client;
    }

    // --- Users (Voucher) ---

    public function users(array $filter = []): array
    {
        return $this->client->command('/ip/hotspot/user/print', $filter);
    }

    public function createUser(
        string $username,
        string $password,
        string $profile = 'default',
        ?string $timelimit = null,
        ?string $datalimit = null,
        ?string $comment = null
    ): array {
        $args = [
            'name'     => $username,
            'password' => $password,
            'profile'  => $profile,
        ];
        if ($timelimit !== null) $args['limit-uptime'] = $timelimit;
        if ($datalimit !== null) $args['limit-bytes-total'] = $datalimit;
        if ($comment !== null)   $args['comment'] = $comment;

        return $this->client->command('/ip/hotspot/user/add', $args);
    }

    public function updateUser(string $id, array $params): array
    {
        $params['.id'] = $id;
        return $this->client->command('/ip/hotspot/user/set', $params);
    }

    public function removeUser(string $id): array
    {
        return $this->client->command('/ip/hotspot/user/remove', ['.id' => $id]);
    }

    // --- Active Users & Kick ---

    public function active(array $filter = []): array
    {
        return $this->client->command('/ip/hotspot/active/print', $filter);
    }

    public function kick(string $activeId): array
    {
        return $this->client->command('/ip/hotspot/active/remove', ['.id' => $activeId]);
    }

    // --- Profiles ---

    public function profiles(): array
    {
        return $this->client->command('/ip/hotspot/user/profile/print');
    }

    public function addProfile(string $name, ?string $rateLimit = null, int $sharedUsers = 1): array
    {
        $args = [
            'name'         => $name,
            'shared-users' => (string) $sharedUsers,
        ];
        if ($rateLimit !== null) $args['rate-limit'] = $rateLimit;
        return $this->client->command('/ip/hotspot/user/profile/add', $args);
    }

    // --- IP Binding (Bypass MAC / IP) ---

    public function ipBindings(): array
    {
        return $this->client->command('/ip/hotspot/ip-binding/print');
    }

    public function addIpBinding(string $macAddress, string $type = 'bypassed', ?string $comment = null): array
    {
        $args = [
            'mac-address' => $macAddress,
            'type'        => $type,
        ];
        if ($comment !== null) $args['comment'] = $comment;
        return $this->client->command('/ip/hotspot/ip-binding/add', $args);
    }

    // --- Hosts & Cookies ---

    public function hosts(): array
    {
        return $this->client->command('/ip/hotspot/host/print');
    }

    public function cookies(): array
    {
        return $this->client->command('/ip/hotspot/cookie/print');
    }

    public function clearCookies(): array
    {
        $cookies = $this->cookies();
        foreach ($cookies as $c) {
            if (!empty($c['.id'])) {
                $this->client->command('/ip/hotspot/cookie/remove', ['.id' => $c['.id']]);
            }
        }
        return ['status' => 'cleared'];
    }
}

<?php

namespace Mikrotik\Modules;

use Mikrotik\Client;

class PppModule
{
    protected Client $client;

    public function __construct(Client $client)
    {
        $this->client = $client;
    }

    // --- Secrets (Accounts) ---

    public function secrets(array $filter = []): array
    {
        return $this->client->command('/ppp/secret/print', $filter);
    }

    public function createSecret(
        string $name,
        string $password,
        string $service = 'pppoe',
        string $profile = 'default',
        ?string $remoteAddress = null,
        ?string $comment = null
    ): array {
        $args = [
            'name'     => $name,
            'password' => $password,
            'service'  => $service,
            'profile'  => $profile,
        ];
        if ($remoteAddress !== null) $args['remote-address'] = $remoteAddress;
        if ($comment !== null)       $args['comment'] = $comment;

        return $this->client->command('/ppp/secret/add', $args);
    }

    public function removeSecret(string $id): array
    {
        return $this->client->command('/ppp/secret/remove', ['.id' => $id]);
    }

    public function setSecretStatus(string $id, bool $enable): array
    {
        return $this->client->command('/ppp/secret/set', [
            '.id' => $id,
            'disabled' => $enable ? 'no' : 'yes',
        ]);
    }

    // --- Active Connections ---

    public function active(array $filter = []): array
    {
        return $this->client->command('/ppp/active/print', $filter);
    }

    public function disconnect(string $activeId): array
    {
        return $this->client->command('/ppp/active/remove', ['.id' => $activeId]);
    }

    // --- Profiles ---

    public function profiles(): array
    {
        return $this->client->command('/ppp/profile/print');
    }

    public function addProfile(string $name, string $localAddress, string $remoteAddress, ?string $rateLimit = null): array
    {
        $args = [
            'name'           => $name,
            'local-address'  => $localAddress,
            'remote-address' => $remoteAddress,
        ];
        if ($rateLimit !== null) $args['rate-limit'] = $rateLimit;
        return $this->client->command('/ppp/profile/add', $args);
    }

    // --- PPPoE Server Service ---

    public function servers(): array
    {
        return $this->client->command('/interface/pppoe-server/server/print');
    }
}

<?php

namespace Mikrotik\Modules;

use Mikrotik\Client;

class QueueModule
{
    protected Client $client;

    public function __construct(Client $client)
    {
        $this->client = $client;
    }

    // --- Simple Queues ---

    public function simple(array $filter = []): array
    {
        return $this->client->command('/queue/simple/print', $filter);
    }

    public function addSimple(string $name, string $target, string $maxLimit, ?string $comment = null): array
    {
        $args = [
            'name'      => $name,
            'target'    => $target,
            'max-limit' => $maxLimit, // e.g. "2M/10M"
        ];
        if ($comment !== null) $args['comment'] = $comment;
        return $this->client->command('/queue/simple/add', $args);
    }

    public function setLimit(string $id, string $maxLimit): array
    {
        return $this->client->command('/queue/simple/set', [
            '.id'       => $id,
            'max-limit' => $maxLimit,
        ]);
    }

    public function removeSimple(string $id): array
    {
        return $this->client->command('/queue/simple/remove', ['.id' => $id]);
    }

    // --- Queue Tree ---

    public function tree(): array
    {
        return $this->client->command('/queue/tree/print');
    }

    // --- Queue Types ---

    public function types(): array
    {
        return $this->client->command('/queue/type/print');
    }
}

<?php

namespace Mikrotik\Modules;

use Mikrotik\Client;

class LogModule
{
    protected Client $client;

    public function __construct(Client $client)
    {
        $this->client = $client;
    }

    public function all(array $filter = []): array
    {
        return $this->client->command('/log/print', $filter);
    }

    public function recent(int $limit = 50): array
    {
        return $this->client->command('/log/print');
    }

    public function byTopic(string $topic): array
    {
        return $this->client->command('/log/print', [
            '?topics' => $topic,
        ]);
    }
}

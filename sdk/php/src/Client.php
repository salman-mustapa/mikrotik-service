<?php

namespace Mikrotik;

use GuzzleHttp\Client as HttpClient;
use GuzzleHttp\Exception\RequestException;
use Mikrotik\Exceptions\MikrotikException;
use Mikrotik\Exceptions\MikrotikTrapException;

class Client
{
    protected HttpClient $http;
    protected string $host;
    protected int $port;
    protected string $user;
    protected string $password;
    protected string $gatewayUrl;
    protected string $gatewayToken;

    public function __construct(
        string $host = '127.0.0.1',
        int $port = 8728,
        string $user = 'admin',
        string $password = '',
        string $gatewayUrl = 'http://127.0.0.1:8080',
        string $gatewayToken = 'change-me-to-a-long-random-string',
        int $timeout = 10
    ) {
        $this->host = $host;
        $this->port = $port;
        $this->user = $user;
        $this->password = $password;
        $this->gatewayUrl = rtrim($gatewayUrl, '/');
        $this->gatewayToken = $gatewayToken;

        $this->http = new HttpClient([
            'base_uri' => $this->gatewayUrl,
            'timeout'  => $timeout,
            'headers'  => [
                'Authorization' => "Bearer {$this->gatewayToken}",
                'Content-Type'  => 'application/json',
                'Accept'        => 'application/json',
            ],
        ]);
    }

    /**
     * Eksekusi perintah RouterOS ke Rust Gateway
     */
    public function command(string $command, array $args = []): array
    {
        $payload = [
            'router' => [
                'host'     => $this->host,
                'port'     => $this->port,
                'user'     => $this->user,
                'password' => $this->password,
            ],
            'command' => $command,
            'args'    => (object) $args,
        ];

        try {
            $response = $this->http->post('/api/v1/command', [
                'json' => $payload,
            ]);

            $body = (string) $response->getBody();
            return json_decode($body, true) ?? [];
        } catch (RequestException $e) {
            $response = $e->getResponse();
            if ($response) {
                $status = $response->getStatusCode();
                $body = json_decode((string) $response->getBody(), true);
                $errMsg = $body['error'] ?? $e->getMessage();
                $category = $body['category'] ?? null;

                if ($status === 422) {
                    throw new MikrotikTrapException($errMsg, $category, $status);
                }
                throw new MikrotikException("Gateway Error [{$status}]: {$errMsg}", $status);
            }
            throw new MikrotikException("Network Connection Error: " . $e->getMessage(), 0, $e);
        }
    }

    /**
     * Dapatkan URL Server-Sent Events (SSE) untuk realtime monitoring
     */
    public function getStreamUrl(string $command, array $params = []): string
    {
        $query = array_merge([
            'host'     => $this->host,
            'port'     => $this->port,
            'user'     => $this->user,
            'password' => $this->password,
            'command'  => $command,
        ], $params);

        return "{$this->gatewayUrl}/api/v1/listen?" . http_build_query($query);
    }
}

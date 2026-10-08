<?php

namespace Mikrotik\Laravel;

use Illuminate\Support\ServiceProvider;
use Mikrotik\MikrotikApp;

class MikrotikServiceProvider extends ServiceProvider
{
    public function register(): void
    {
        $this->mergeConfigFrom(__DIR__ . '/../../config/mikrotik.php', 'mikrotik');

        $this->app->singleton(MikrotikApp::class, function ($app) {
            $config = $app['config']['mikrotik'] ?? [];

            return new MikrotikApp(
                host: $config['host'] ?? '127.0.0.1',
                port: (int) ($config['port'] ?? 8728),
                user: $config['user'] ?? 'admin',
                password: $config['password'] ?? '',
                gatewayUrl: $config['gateway_url'] ?? 'http://127.0.0.1:8080',
                gatewayToken: $config['gateway_token'] ?? 'change-me-to-a-long-random-string'
            );
        });

        $this->app->alias(MikrotikApp::class, 'mikrotik');
    }

    public function boot(): void
    {
        if ($this->app->runningInConsole()) {
            $this->publishes([
                __DIR__ . '/../../config/mikrotik.php' => config_path('mikrotik.php'),
            ], 'mikrotik-config');
        }
    }
}

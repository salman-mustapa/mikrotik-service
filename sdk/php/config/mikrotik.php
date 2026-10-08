<?php

return [
    /*
    |--------------------------------------------------------------------------
    | Default MikroTik Connection Credentials
    |--------------------------------------------------------------------------
    */
    'host'          => env('MIKROTIK_HOST', '192.168.88.1'),
    'port'          => (int) env('MIKROTIK_PORT', 8728),
    'user'          => env('MIKROTIK_USER', 'admin'),
    'password'      => env('MIKROTIK_PASSWORD', ''),

    /*
    |--------------------------------------------------------------------------
    | Rust Gateway URL & Bearer Auth Token
    |--------------------------------------------------------------------------
    | The high-performance Rust daemon running locally or in your private network.
    */
    'gateway_url'   => env('MIKROTIK_GATEWAY_URL', 'http://127.0.0.1:8080'),
    'gateway_token' => env('MIKROTIK_GATEWAY_TOKEN', 'change-me-to-a-long-random-string'),
];

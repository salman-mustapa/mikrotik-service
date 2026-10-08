<?php

namespace Mikrotik\Laravel\Facades;

use Illuminate\Support\Facades\Facade;
use Mikrotik\MikrotikApp;

/**
 * @method static \Mikrotik\Modules\SystemModule system()
 * @method static \Mikrotik\Modules\IpModule ip()
 * @method static \Mikrotik\Modules\Ipv6Module ipv6()
 * @method static \Mikrotik\Modules\HotspotModule hotspot()
 * @method static \Mikrotik\Modules\PppModule ppp()
 * @method static \Mikrotik\Modules\InterfaceModule interfaces()
 * @method static \Mikrotik\Modules\FirewallModule firewall()
 * @method static \Mikrotik\Modules\QueueModule queues()
 * @method static \Mikrotik\Modules\LogModule logs()
 * @method static array command(string $command, array $args = [])
 * @method static \Mikrotik\MikrotikApp connect(string $host, int $port = 8728, string $user = 'admin', string $password = '')
 *
 * @see \Mikrotik\MikrotikApp
 */
class Mikrotik extends Facade
{
    protected static function getFacadeAccessor(): string
    {
        return MikrotikApp::class;
    }
}

<?php

namespace Mikrotik\Exceptions;

class MikrotikTrapException extends MikrotikException
{
    protected ?string $category;

    public function __construct(string $message, ?string $category = null, int $code = 422)
    {
        parent::__construct($message, $code);
        $this->category = $category;
    }

    public function getCategory(): ?string
    {
        return $this->category;
    }
}

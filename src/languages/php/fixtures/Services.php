<?php

namespace MyApp\Services;

use MyApp\Models\User;
use MyApp\Models\Repository;

class Calculator
{
    public function add(int $a, int $b): int
    {
        return $a + $b;
    }

    public function multiply(int $a, int $b): int
    {
        return $a * $b;
    }
}

class UserRepository implements Repository
{
    public function find(int $id): ?User
    {
        return new User("Unknown", 0);
    }

    public function save(mixed $entity): void
    {
        // Save logic
    }
}

const DEFAULT_TIMEOUT = 30;
const MAX_RETRIES = 3;

function helper_function(): string
{
    return "helper";
}

<?php

namespace MyApp\Models;

class User
{
    private string $name;
    private int $age;

    public function __construct(string $name, int $age)
    {
        $this->name = $name;
        $this->age = $age;
    }

    public function getName(): string
    {
        return $this->name;
    }
}

class Order
{
    public int $id;
    public OrderStatus $status;

    public function __construct(int $id, OrderStatus $status)
    {
        $this->id = $id;
        $this->status = $status;
    }
}

enum OrderStatus
{
    case Pending;
    case Active;
    case Completed;
}

interface Repository
{
    public function find(int $id): mixed;
    public function save(mixed $entity): void;
}

trait Timestampable
{
    public function touch(): void
    {
        // Update timestamp
    }
}

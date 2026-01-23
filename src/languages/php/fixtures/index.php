<?php

namespace MyApp;

use MyApp\Models\User;
use MyApp\Models\Order;
use MyApp\Services\Calculator;
use MyApp\Services\UserRepository;

class Application
{
    public function run(): void
    {
        $user = new User("Alice", 30);
        $calculator = new Calculator();

        $result = $calculator->add(1, 2);
        echo "Hello {$user->getName()}, result is {$result}\n";

        $order = new Order(1, OrderStatus::Active);
        $repo = new UserRepository();
    }
}

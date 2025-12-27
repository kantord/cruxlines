mod models;
mod utils;

use models::{Status, User};
use utils::{add, Counter, PI};

fn greet(name: &str) -> String {
    format!("Hello, {name}")
}

fn main() {
    let user = User::new("Ada", Status::Active);
    let total = add(2, 3);
    let mut counter = Counter::new(1);
    counter.inc();
    println!("{}", greet(&user.name));
    println!("total={total}, pi={PI}, counter={}", counter.value());
}

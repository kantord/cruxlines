pub const PI: f64 = 3.14159;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub struct Counter {
    value: i32,
}

impl Counter {
    pub fn new(start: i32) -> Self {
        Self { value: start }
    }

    pub fn inc(&mut self) {
        self.value += 1;
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

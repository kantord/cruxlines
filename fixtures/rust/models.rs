#[derive(Clone, Debug)]
pub enum Status {
    Active,
    Inactive,
}

pub struct User {
    pub name: String,
    pub status: Status,
}

impl User {
    pub fn new(name: &str, status: Status) -> Self {
        Self {
            name: name.to_string(),
            status,
        }
    }
}

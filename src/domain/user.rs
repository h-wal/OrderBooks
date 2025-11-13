use crate::domain::Trade;

#[derive(Clone, Debug)]
pub struct User {
    pub email: String,
    pub password: String,
    pub balance: u64,
    pub holdings: u64,
    pub trades: Vec<Trade>
}

impl User {
    pub fn new(email: String, password: String) -> Self {
        Self {
            email,
            password,
            balance: 0,
            holdings: 0,
            trades: vec![]
        }
    }
}


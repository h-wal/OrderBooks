use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Order, User};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Trade {
    pub id: Uuid,
    pub buyer: String,
    pub seller: String,
    pub qty: u64,
    pub price: u64,
}

impl Trade {
    pub fn new(buyer: &User, order: &Order, seler: &User) -> Self{
        Self { 
            id: Uuid::new_v4(),
            buyer: buyer.email.clone(),
            seller: seler.email.clone(),
            qty: order.qty,
            price: order.price
        }
    }
}
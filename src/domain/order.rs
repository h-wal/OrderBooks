use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Side {
    /// A buy order 
    Bid,
    /// A sell order
    Ask,
}

///Order with its types ...
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct  Order {
    pub id: Uuid,
    pub user_id: String,
    pub qty: u64,
    pub price: u64,
    pub side: Side,
}

impl Order {
    pub fn new(user_id: String, qty: u64, price: u64, side: Side) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            qty,
            price,
            side,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderSummary {
    pub owner: String,
    pub qty: u64,
    pub price: u64,
    pub side: Side,
}


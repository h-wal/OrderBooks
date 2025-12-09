use serde::Deserialize;
use uuid::Uuid;
use crate::domain::Side;

#[derive(Deserialize)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct OnRampHttpRequest {
    pub user_email: String,
    pub balance: u64,
    pub holding: u64,
}

#[derive(Deserialize)]
pub struct OrderInput {
    pub qty: u64,
    pub price: u64,
    pub side: Side,
}

#[derive(Deserialize)]
pub struct CreateLimitOrderRequest {
    pub market_id: u64,
    pub user_email: String,
    pub order: OrderInput,
}

#[derive(Deserialize)]
pub struct CreateMarketOrderRequest {
    pub market_id: u64,
    pub user_email: String,
    pub order: OrderInput,
}

#[derive(Deserialize)]
pub struct CancelOrderRequest {
    pub market_id: u64,
    pub side: Side,
    pub order_id: Uuid,
}

#[derive(Deserialize)]
pub struct CreateMarketRequest {
    pub market_id: u64,
}

#[derive(Deserialize)]
pub struct GetOrderBookRequest {
    pub user_email: String,
    pub market_id: u64,
}


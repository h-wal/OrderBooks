use axum::{routing::post, Router};
use crate::app::AppState;
use crate::handlers::{auth, market, orders};

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", post(|| async { "Hello World!" }))
        .route("/signup", post(auth::signup_handler))
        .route("/signin", post(auth::signin_handler))
        .route("/onramp", post(auth::onramp_handler))
        .route("/createLimitOrder", post(orders::create_limit_order_handler))
        .route("/getorderbook", post(market::get_order_book_handler))
        .route("/createMarketOrder", post(orders::create_market_order_handler))
        .route("/cancelorder", post(orders::cancel_order_handler))
        .route("/createmarket", post(market::create_market_handler))
        .route("/listmarkets", post(market::list_markets_handler))
}


use axum::{routing::post, Router};
use crate::app::AppState;
use crate::handlers::{auth, orders, market};

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", post(|| async { "Hello World!" }))
        .route("/signup", post(auth::signup_handler))
        .route("/signin", post(auth::signin_handler))
        .route("/onramp", post(auth::onramp_handler))
        .route("/createLimitOrder", post(orders::create_limit_order_handler))
        .route("/getorderbook", post(market::get_order_book_handler))
}

//create market order

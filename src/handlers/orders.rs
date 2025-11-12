use axum::http::response;
use axum::{extract::State, http::StatusCode, Json};
use tokio::sync::oneshot;
use crate::actors::db;
use crate::app::AppState;
use crate::actors::orderbook::OrderbookCommand;
use crate::dto::{CreateMarketOrderRequest, CreateMarketOrderResponse};

pub async fn create_limit_order_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateMarketOrderRequest>
) -> CreateMarketOrderResponse {

    let ob_tx = state.ob_tx.clone();
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    
    let _ = ob_tx.send(OrderbookCommand::NewLimitOrder { 
        market_id: payload.market_id, 
        user_id: payload.user_email,
        side: payload.order.side, 
        qty: payload.order.qty, 
        price: payload.order.price, 
        resp: oneshot_tx 
    }).await;

    match oneshot_rx.await {
        Ok(response) => {
            if response.status.contains("Successfull") {
                CreateMarketOrderResponse {
                    message: response.status,
                    trades: response.fills,
                    status: StatusCode::OK
                }
            } else {
                CreateMarketOrderResponse { 
                    message: response.status.to_string(), 
                    trades: vec![], 
                    status: StatusCode::EXPECTATION_FAILED
                }
            }
        },
        Err(_) => {
            CreateMarketOrderResponse {
                message: "Error Creating Market Order".to_string(),
                trades: vec![],
                status: StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
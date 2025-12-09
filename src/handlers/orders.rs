use axum::{extract::State, http::StatusCode, Json};
use tokio::sync::oneshot;
use crate::app::AppState;
use crate::actors::orderbook::OrderbookCommand;
use crate::dto::{
    CancelOrderRequest, CancelOrderResponse, CreateLimitOrderRequest, CreateLimitOrderResponse,
    CreateMarketOrderRequest, CreateMarketOrderResponse,
};

pub async fn create_limit_order_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateLimitOrderRequest>
) -> CreateLimitOrderResponse {

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
            if response.status.contains("Success") {
                CreateLimitOrderResponse {
                    message: response.status,
                    trades: response.fills,
                    status: StatusCode::OK
                }
            } else {
                CreateLimitOrderResponse { 
                    message: response.status.to_string(), 
                    trades: vec![], 
                    status: StatusCode::EXPECTATION_FAILED
                }
            }
        },
        Err(_) => {
            CreateLimitOrderResponse {
                message: "Error Creating Market Order".to_string(),
                trades: vec![],
                status: StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

pub async fn create_market_order_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateMarketOrderRequest>
) -> CreateMarketOrderResponse {

    let ob_tx = state.ob_tx.clone();
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    let _ = ob_tx.send(OrderbookCommand::NewMarketOrder { 
        market_id: payload.market_id, 
        user_id: payload.user_email, 
        side: payload.order.side, 
        qty: payload.order.qty, 
        resp: oneshot_tx 
    }).await;

    match oneshot_rx.await {
        Ok(response) => {
            
            if response.status.contains("Success"){
                CreateMarketOrderResponse::created(response.status, response.fills)
            } else {
                CreateMarketOrderResponse::failed(response.status, response.fills)
            }
           
        }
        Err(_) => {
            CreateMarketOrderResponse::error(
                "Internal Servor Error",
                vec![]
            )
        }
    }
}

pub async fn cancel_order_handler(
    State(state): State<AppState>,
    Json(payload): Json<CancelOrderRequest>,
) -> CancelOrderResponse {
    let ob_tx = state.ob_tx.clone();
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    let _ = ob_tx.send(OrderbookCommand::CancelOrder {
        market_id: payload.market_id,
        side: payload.side,
        order_id: payload.order_id,
        resp: oneshot_tx,
    }).await;

    match oneshot_rx.await {
        Ok(response) => {
            if response.canceled {
                CancelOrderResponse::ok(response.status)
            } else {
                CancelOrderResponse::failed(response.status)
            }
        }
        Err(e) => CancelOrderResponse::failed(format!("Actor error: {}", e)),
    }
}
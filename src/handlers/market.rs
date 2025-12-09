use axum::{extract::State, http::StatusCode, Json};
use tokio::sync::oneshot;
use crate::app::AppState;
use crate::actors::{db::DbCommand, orderbook::OrderbookCommand};
use crate::dto::{
    CreateMarketRequest, CreateMarketResponse, GetOrderBookRequest, GetOrderBookResponse,
    ListMarketsResponse,
};
use crate::domain::{MarketBook, Side};

pub async fn get_order_book_handler(
    State(state): State<AppState>,
    Json(payload): Json<GetOrderBookRequest>
) -> GetOrderBookResponse {
    let db_tx = state.db_tx.clone();
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    
    let _ = db_tx.send(DbCommand::CheckUser { 
        user_email: payload.user_email.clone(),
        response_status: oneshot_tx
    }).await;

    match oneshot_rx.await {
        Ok(response) => {
            if response.user_exists {
                let ob_tx = state.ob_tx.clone();
                let (oneshot_tx, oneshot_rx) = oneshot::channel();
                
                let _ = ob_tx.send(OrderbookCommand::GetBook { 
                    market_id: payload.market_id, 
                    resp: oneshot_tx
                }).await;

                match oneshot_rx.await {
                    Ok(response) => {
                        if response.status.contains("Success") {

                            GetOrderBookResponse {
                                status: StatusCode::OK,
                                message: "Succesfully fetched the Order Book".to_string(),
                                bids: response.bids,
                                asks: response.asks
                            }

                        } else {
                            GetOrderBookResponse { 
                                status: StatusCode::NOT_FOUND, 
                                message: "Error fetching Order Book".to_string(), 
                                bids: None, 
                                asks: None
                            }
                        }
                    } 
                    Err(e) => {
                        GetOrderBookResponse { 
                            status: StatusCode::INTERNAL_SERVER_ERROR, 
                            message: e.to_string(), 
                            bids: None,
                            asks: None
                        }
                    }
                }
            } else {
                GetOrderBookResponse { 
                    status: StatusCode::NOT_ACCEPTABLE, 
                    message: "User does not exist".to_string(), 
                    bids: None, 
                    asks: None
                }
            }
        } 
        Err(e) => {
            GetOrderBookResponse { 
                status: StatusCode::INTERNAL_SERVER_ERROR, 
                message: e.to_string(), 
                bids: None, 
                asks: None
            }
        }
    }
}

pub async fn create_market_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateMarketRequest>,
) -> CreateMarketResponse {
    let ob_tx = state.ob_tx.clone();
    let (tx, rx) = oneshot::channel();

    let _ = ob_tx
        .send(OrderbookCommand::CreateMarket {
            market_id: payload.market_id,
            resp: tx,
        })
        .await;

    match rx.await {
        Ok(response) => {
            if response.status.contains("created") {
                CreateMarketResponse::created(response.status, response.market_ids)
            } else {
                CreateMarketResponse::failed(response.status)
            }
        }
        Err(e) => CreateMarketResponse::failed(format!("Actor error: {}", e)),
    }
}

pub async fn list_markets_handler(
    State(state): State<AppState>,
) -> ListMarketsResponse {
    let ob_tx = state.ob_tx.clone();
    let (tx, rx) = oneshot::channel();

    let _ = ob_tx.send(OrderbookCommand::ListMarkets { resp: tx }).await;

    match rx.await {
        Ok(response) => {
            let markets = response.market_ids.unwrap_or_default();
            ListMarketsResponse::ok("Markets listed", markets)
        }
        Err(e) => ListMarketsResponse::ok(format!("Actor error: {}", e), vec![]),
    }
}
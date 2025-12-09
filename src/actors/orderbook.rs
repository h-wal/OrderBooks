use std::collections::{BTreeMap, HashMap};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::actors::db::{DbCommand, DbSender};
use crate::domain::{MarketBook, Order, Side};

pub enum OrderbookCommand {
    CreateMarket {
        market_id: u64,
        resp: oneshot::Sender<OrderbookResponse>,
    },
    ListMarkets {
        resp: oneshot::Sender<OrderbookResponse>,
    },
    NewLimitOrder {
        market_id: u64,
        user_id: String,
        side: Side,
        qty: u64,
        price: u64,
        resp: oneshot::Sender<OrderbookResponse>,
    },
    NewMarketOrder {
        market_id: u64,
        user_id: String,
        side: Side,
        qty: u64,
        resp: oneshot::Sender<OrderbookResponse>,
    },
    CancelOrder {
        market_id: u64,
        side: Side,
        order_id: Uuid,
        resp: oneshot::Sender<OrderbookResponse>,
    },
    GetBook {
        market_id: u64,
        resp: oneshot::Sender<OrderbookResponse>,
    },
}

pub struct OrderbookResponse {
    pub status: String,
    pub fills: Vec<crate::domain::Trade>,
    pub remaining_qty: u64,
    pub bids: Option<BTreeMap<u64, std::collections::VecDeque<Order>>>,
    pub asks: Option<BTreeMap<u64, std::collections::VecDeque<Order>>>,
    pub market_ids: Option<Vec<u64>>,
    pub canceled: bool,
}

impl OrderbookResponse {
    fn empty(status: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            fills: vec![],
            remaining_qty: 0,
            bids: None,
            asks: None,
            market_ids: None,
            canceled: false,
        }
    }
}

pub async fn start_orderbook_actor(mut rx: mpsc::Receiver<OrderbookCommand>, db_tx: DbSender) {
    let mut order_book: HashMap<u64, MarketBook> = HashMap::new();

    println!("Orderbook actor started");

    while let Some(cmd) = rx.recv().await {
        match cmd {
            OrderbookCommand::CreateMarket { market_id, resp } => {
                let response = if order_book.contains_key(&market_id) {
                    OrderbookResponse::empty(format!("Market {} already exists", market_id))
                } else {
                    order_book.insert(market_id, MarketBook::new());
                    OrderbookResponse {
                        market_ids: Some(order_book.keys().cloned().collect()),
                        ..OrderbookResponse::empty(format!("Market {} created", market_id))
                    }
                };
                let _ = resp.send(response);
            }
            OrderbookCommand::ListMarkets { resp } => {
                let ids = order_book.keys().cloned().collect::<Vec<_>>();
                let response = OrderbookResponse {
                    market_ids: Some(ids),
                    ..OrderbookResponse::empty("Markets listed")
                };
                let _ = resp.send(response);
            }
            OrderbookCommand::NewLimitOrder { market_id, user_id, side, qty, price, resp } => {
                let response = if let Some(book) = order_book.get_mut(&market_id) {
                    let (oneshot_tx, oneshot_rx) = oneshot::channel();
                    let _ = db_tx.send(DbCommand::GetUser {
                        user_email: user_id.clone(),
                        response_status: oneshot_tx,
                    }).await;

                    match oneshot_rx.await {
                        Ok(response) => match response.user {
                            Some(user) => {
                                let response = match side {
                                    Side::Bid if price * qty > user.balance => {
                                        OrderbookResponse::empty("Insufficient balance")
                                    }
                                    Side::Ask if qty > user.holdings => {
                                        OrderbookResponse::empty("Insufficient holdings")
                                    }
                                    _ => {
                                        let order = Order::new(user_id.clone(), qty, price, side);
                                        let (trades, remaining_order) = book.match_order(order);

                                        let (tx, rx) = oneshot::channel();
                                        let _ = db_tx.send(DbCommand::Reconciliation {
                                            trades: trades.clone(),
                                            response_status: tx,
                                        }).await;
                                        let _ = rx.await;

                                        if let Some(order) = remaining_order {
                                            book.insert_order(order);
                                            OrderbookResponse {
                                                status: "Success, resting remaining order".to_string(),
                                                fills: trades,
                                                remaining_qty: 0,
                                                bids: None,
                                                asks: None,
                                                market_ids: None,
                                                canceled: false,
                                            }
                                        } else {
                                            OrderbookResponse {
                                                status: "Success, fully matched".to_string(),
                                                fills: trades,
                                                remaining_qty: 0,
                                                bids: None,
                                                asks: None,
                                                market_ids: None,
                                                canceled: false,
                                            }
                                        }
                                    }
                                };
                                response
                            }
                            None => OrderbookResponse::empty("User does not exist"),
                        },
                        Err(_) => OrderbookResponse::empty("Database error"),
                    }
                } else {
                    OrderbookResponse::empty("Market does not exist")
                };

                let _ = resp.send(response);
            }
            OrderbookCommand::NewMarketOrder { market_id, user_id, side , qty , resp } => {
                let response = if let Some(book) = order_book.get_mut(&market_id) {
                    let (oneshot_tx, oneshot_rx) = oneshot::channel();
                    let _ = db_tx.send(DbCommand::GetUser { 
                        user_email: user_id.clone(), 
                        response_status: oneshot_tx 
                    }).await;

                    match oneshot_rx.await {
                        Ok(response) => {
                            match response.user {
                                Some(user) => {
                                    println!("User {} has balance {}", user.email, user.balance );
                                    let order = Order::new(user_id.clone(), qty, 0, side);

                                    let (trades, _remaining_order) = book.match_order(order);

                                    let (tx, rx) = oneshot::channel();
                                    let _ = db_tx.send(DbCommand::Reconciliation { trades: trades.clone(), response_status: tx }).await;
                                    let _ = rx.await;

                                    OrderbookResponse {
                                        status: "Market order processed".to_string(),
                                        fills: trades,
                                        remaining_qty: 0,
                                        bids: None,
                                        asks: None,
                                        market_ids: None,
                                        canceled: false,
                                    }
                                }
                                None => OrderbookResponse::empty("Error finding user"),
                            }
                        }
                        Err(_) => OrderbookResponse::empty("Database error"),
                    }
                } else {
                    OrderbookResponse::empty("Market does not exist")
                };

                let _ = resp.send(response);
            }
            OrderbookCommand::CancelOrder { market_id, side, order_id, resp } => {
                let response = if let Some(book) = order_book.get_mut(&market_id) {
                    let removed = book.cancel_order(side, order_id);
                    if removed {
                        OrderbookResponse {
                            status: "Order canceled".to_string(),
                            fills: vec![],
                            remaining_qty: 0,
                            bids: None,
                            asks: None,
                            market_ids: None,
                            canceled: true,
                        }
                    } else {
                        OrderbookResponse::empty("Order not found")
                    }
                } else {
                    OrderbookResponse::empty("Market does not exist")
                };
                let _ = resp.send(response);
            }
            OrderbookCommand::GetBook { market_id, resp } => {
                let response = if let Some(book) = order_book.get(&market_id) {
                    OrderbookResponse {
                        status: "Successful! Current order book snapshot".to_string(),
                        fills: vec![],
                        remaining_qty: 0,
                        bids: Some(book.bids.clone()),
                        asks: Some(book.asks.clone()),
                        market_ids: None,
                        canceled: false,
                    }
                } else {
                    OrderbookResponse::empty("Market does not exist")
                };
                let _ = resp.send(response);
            }
        }
    }
}
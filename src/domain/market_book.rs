use std::collections::{BTreeMap, VecDeque};
use crate::domain::{Order, OrderSummary, Side, order, trade};
use crate::domain::Trade;

pub struct MarketBook {
    pub bids: BTreeMap<u64, VecDeque<Order>>,
    pub asks: BTreeMap<u64, VecDeque<Order>>,
}

impl MarketBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn insert_order(&mut self, order: Order) {
        let target_side = match order.side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks
        };

        let entry = target_side.entry(order.price).or_insert_with(VecDeque::new);
        entry.push_back(order);
    }

    pub fn match_order(&mut self , incoming_order: Order) -> (Vec<Trade>, Option<Order>){
        
        let mut fills = Vec::new();
        let mut remaining_qty = incoming_order.qty;

        match incoming_order.side {
            
            Side::Bid => {
                 //first lets get the whole arrray of all the availaible ask prices 
                let ask_prices: Vec<u64> = self.asks.keys().cloned().collect();
                

                // now lets first iterate over the array to get the best price to match the order
                for ask_price in ask_prices {

                    // check if qty is already 0 or price is too high 
                    if remaining_qty == 0 || ask_price > incoming_order.price {
                        break;
                    }

                    //getting all the availaible orders at this price 
                    if let Some(ask_orders) = self.asks.get_mut(&ask_price){
                        
                        while let Some(mut best_ask_order) = ask_orders.pop_front(){
                            
                            if remaining_qty == 0 {
                                // Put back the order we just popped
                                ask_orders.push_front(best_ask_order);
                                break;
                            }

                            let trade_qty = remaining_qty.min(best_ask_order.qty);

                            let trade = Trade{
                                id: uuid::Uuid::new_v4(),
                                buyer: incoming_order.user_id.clone(),
                                seller: best_ask_order.user_id.clone(),
                                qty: trade_qty,
                                price: ask_price
                            };
                            fills.push(trade);

                            remaining_qty -= trade_qty;
                            best_ask_order.qty -= trade_qty;

                            if best_ask_order.qty > 0 {
                                ask_orders.push_front(best_ask_order);
                                break; // Move to next price level
                            }

                        }

                        if ask_orders.is_empty() {
                            self.asks.remove(&ask_price);
                        }
                    }
                }

            }

            Side::Ask => {
                
                //get the array of all the prices availaible in reversed order ie highest first
                let bid_prices: Vec<u64> = self.bids.keys().rev().cloned().collect();

                for bid_price in bid_prices {

                    if remaining_qty == 0 || bid_price < incoming_order.price{
                        break;
                    }

                    if let Some(mut bid_orders) = self.bids.get_mut(&bid_price){

                        while let Some(mut bid_order) = bid_orders.pop_front(){

                            if remaining_qty == 0 {
                                bid_orders.push_front(bid_order);
                                break;
                            }

                            let trade_qty = remaining_qty.min(bid_order.qty);

                            let trade = Trade{
                                id: uuid::Uuid::new_v4(),
                                buyer: bid_order.user_id.clone(),
                                seller: incoming_order.user_id.clone(),
                                qty: trade_qty,
                                price: bid_price
                            };

                            fills.push(trade);

                            remaining_qty -= trade_qty;
                            bid_order.qty -= trade_qty;

                            if bid_order.qty > 0 {
                                bid_orders.push_front(bid_order);
                                break; // Move to next price level
                            }
                        }

                        if bid_orders.is_empty() {
                            self.bids.remove(&bid_price);
                        }
                    }

                }

                

            }
        }

        let remaining_order = if remaining_qty > 0 {
            Some(Order {
                user_id: incoming_order.user_id,
                qty: remaining_qty,
                price: incoming_order.price,
                side: incoming_order.side,
            })
        } else {
            None
        };

        (fills, remaining_order)
    }

}


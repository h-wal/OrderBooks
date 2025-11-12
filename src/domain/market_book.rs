use std::collections::{BTreeMap, VecDeque};
use crate::domain::{Order, OrderSummary, Side};
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

    pub fn summarize_side(side: &BTreeMap<u64, VecDeque<Order>>, side_type: Side) -> Vec<Trade> {
        let mut fills: Vec<Trade> = Vec::new();
        for (price, orders) in side.iter() {
            for order in orders.iter() {
                fills.push(Trade { 
                    buyer: "".to_string(), 
                    seller: "".to_string(), 
                    qty: 0, 
                    price: 0 
                });
            }
        }
        fills
    }
}


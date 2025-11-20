use super::types::{Order, Price, Side};

use std::collections::BTreeMap;

pub struct OrderBook {
    pub bids: BTreeMap<Price, Vec<Order>>,
    pub asks: BTreeMap<Price, Vec<Order>>,
}

impl OrderBook {
    pub fn add_order(&mut self, order: Order) {
        match order.side {
            Side::Bid => &mut self
                .bids
                .entry(order.price)
                .or_insert_with(Vec::new)
                .push(order),
            Side::Ask => &mut self
                .asks
                .entry(order.price)
                .or_insert_with(Vec::new)
                .push(order),
        };
    }

    pub fn print(&self) {
        println!("=== ORDER BOOK ===");

        println!("-- Asks (sell orders) --");
        for (price, orders) in &self.asks {
            println!("@ {}: {:?}", price, orders)
        }

        println!("-- Bids (buy orders) --");
        for (price, orders) in self.bids.iter().rev() {
            println!("@ {}: {:?}", price, orders)
        }
    }

    pub fn cleanup_price_level(level: &mut BTreeMap<Price, Vec<Order>>, price: Price) {
        if let Some(orders) = level.get(&price) {
            if orders.is_empty() {
                level.remove(&price);
            }
        }
    }
}

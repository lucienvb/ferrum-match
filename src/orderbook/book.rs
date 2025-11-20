use super::types::{Order, Price, Side};

use std::collections::BTreeMap;

pub struct OrderBook {
    pub bids: BTreeMap<Price, Vec<Order>>,
    pub asks: BTreeMap<Price, Vec<Order>>,
}

impl OrderBook {
    pub fn add_order(&mut self, order: Order) {
        let book_side = match order.side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };

        book_side
            .entry(order.price)
            .or_insert_with(Vec::new)
            .push(order);
    }

    pub fn print(&self) {
        println!("=== ORDER BOOK ===");

        println!("-- Asks (sell orders) --");
        for (price, orders) in &self.asks {
            println!("@ {}: {:?}", price, orders)
        }

        println!("-- Bids (buy orders) --");
        for (price, orders) in &self.bids {
            println!("@ {}: {:?}", price, orders)
        }
    }
}

use super::types::{Order, OrderId, Price, Side};

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

static ORDER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub struct OrderBook {
    pub bids: BTreeMap<Price, Vec<Order>>,
    pub asks: BTreeMap<Price, Vec<Order>>,
}

pub fn next_order_id() -> OrderId {
    OrderId(ORDER_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
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

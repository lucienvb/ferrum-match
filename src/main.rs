mod orderbook;

use chrono::{DateTime, Utc};
use core::pin::Pin;
use orderbook::types::{Order, OrderId, Side};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

static ORDER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn next_order_id() -> OrderId {
    OrderId(ORDER_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn main() {
    let order1 = Order {
        id: next_order_id(),
        price: 15.3,
        quantity: 5,
        side: Side::Bid,
        timestamp: SystemTime::now(),
    };
    let order2 = Order {
        id: next_order_id(),
        price: 25.3,
        quantity: 8,
        side: Side::Ask,
        timestamp: SystemTime::now(),
    };
    {
        let price = order1.price;
        let quantity = order1.quantity;
        let side = order1.side == Side::Bid;
        let timestamp: DateTime<Utc> = order1.timestamp.into();
        println!("ORDER 1 --> id: ..., price: {price}, quantity: {quantity}, side: {side}, timestamp: {}\n", {timestamp.format("%d/%m/%Y %T")});
    }
    {
        let price = order2.price;
        let quantity = order1.quantity;
        let side = order2.side == Side::Bid;
        let timestamp: DateTime<Utc> = order2.timestamp.into();
        println!("ORDER 2 --> id: ..., price: {price}, quantity: {quantity}, side: {side}, timestamp: {}\n", {timestamp.format("%d/%m/%Y %T")});
    }
}

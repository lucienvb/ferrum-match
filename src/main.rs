mod orderbook;

use orderbook::book::OrderBook;
use orderbook::types::{Order, OrderId, Side};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

static ORDER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn next_order_id() -> OrderId {
    OrderId(ORDER_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn main() {
    let mut orderbook = OrderBook {
        bids: BTreeMap::new(),
        asks: BTreeMap::new(),
    };
    let order1 = Order {
        id: next_order_id(),
        price: 15,
        quantity: 5,
        side: Side::Bid,
        timestamp: SystemTime::now(),
    };
    let order2 = Order {
        id: next_order_id(),
        price: 25,
        quantity: 8,
        side: Side::Ask,
        timestamp: SystemTime::now(),
    };
    let order3 = Order {
        id: next_order_id(),
        price: 22,
        quantity: 3,
        side: Side::Ask,
        timestamp: SystemTime::now(),
    };
    let order4 = Order {
        id: next_order_id(),
        price: 12,
        quantity: 4,
        side: Side::Bid,
        timestamp: SystemTime::now(),
    };
    let order5  {
        id: next_order_id(),
        price: 12,
        quantity: 7,
        side: Side::Bid,
        timestamp: SystemTime::now(),
    };
    orderbook.add_order(order1.clone());
    orderbook.add_order(order2.clone());
    orderbook.add_order(order3.clone());
    orderbook.add_order(order4.clone());
    orderbook.add_order(order5.clone());
    orderbook.print();
}

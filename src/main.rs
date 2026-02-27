mod orderbook;

use orderbook::book::{next_order_id, OrderBook};
use orderbook::types::{Order, OrderId, Price, Quantity, Side};
use std::collections::BTreeMap;
use std::ptr::null;
use std::time::SystemTime;

fn make_order(
    id: OrderId,
    price: Price,
    quantity: Quantity,
    side: Side,
    timestamp: SystemTime,
) -> Order {
    return Order {
        id,
        price,
        quantity,
        side,
        timestamp,
    };
}

fn main() {
    let mut orderbook = OrderBook {
        bids: BTreeMap::new(),
        asks: BTreeMap::new(),
    };
    let order1 = make_order(next_order_id(), 15, 5, Side::Bid, SystemTime::now());
    let order2 = make_order(next_order_id(), 25, 8, Side::Ask, SystemTime::now());
    let order3 = make_order(next_order_id(), 22, 3, Side::Ask, SystemTime::now());
    let order4 = make_order(next_order_id(), 12, 4, Side::Bid, SystemTime::now());
    let order5 = make_order(next_order_id(), 12, 7, Side::Bid, SystemTime::now());
    orderbook.add_order(order1.clone());
    orderbook.add_order(order2.clone());
    orderbook.add_order(order3.clone());
    orderbook.add_order(order4.clone());
    orderbook.add_order(order5.clone());

    let incoming = make_order(next_order_id(), 12, 5, Side::Ask, SystemTime::now());
    let trades = orderbook.matching_order(incoming);
    let incoming2 = make_order(next_order_id(), 25, 100, Side::Bid, SystemTime::now());
    let trades2 = orderbook.matching_order(incoming2);
    println!("--> trades: {:?}", trades);
    println!("--> trades2: {:?}", trades2);
}

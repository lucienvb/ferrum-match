mod orderbook;
use orderbook::book::{next_order_id, OrderBook};
use orderbook::types::Side;
use std::collections::BTreeMap;

fn main() {
    let mut orderbook = OrderBook {
        next_seq: 0,
        bids: BTreeMap::new(),
        asks: BTreeMap::new(),
    };
    orderbook.add_order(next_order_id(), 15, 5, Side::Bid);
    orderbook.add_order(next_order_id(), 25, 8, Side::Ask);
    orderbook.add_order(next_order_id(), 22, 3, Side::Ask);
    orderbook.add_order(next_order_id(), 12, 4, Side::Bid);
    orderbook.add_order(next_order_id(), 12, 4, Side::Bid);

    let trades = orderbook.matching_order(OrderBook::make_order_request(
        next_order_id(),
        12,
        5,
        Side::Ask,
    ));
    let trades2 = orderbook.matching_order(OrderBook::make_order_request(
        next_order_id(),
        25,
        100,
        Side::Bid,
    ));
    println!("\n--> trades: {:?}", trades);
    println!("--> trades2: {:?}\n", trades2);
    orderbook.print();
}

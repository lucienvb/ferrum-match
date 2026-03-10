mod orderbook;
use orderbook::types::Side;
use std::collections::BTreeMap;

use crate::orderbook::types::OrderBook;

fn main() {
    let mut orderbook = OrderBook {
        next_seq: 0,
        order_id_counter: 1,
        bids: BTreeMap::new(),
        asks: BTreeMap::new(),
    };
    orderbook.add_order_external(15, 5, Side::Bid);
    orderbook.add_order_external(25, 8, Side::Ask);
    orderbook.add_order_external(22, 3, Side::Ask);
    orderbook.add_order_external(12, 4, Side::Bid);
    orderbook.add_order_external(12, 4, Side::Bid);

    let trades = orderbook.matching_order(OrderBook::make_order_request(12, 5, Side::Ask));
    let trades2 = orderbook.matching_order(OrderBook::make_order_request(25, 100, Side::Bid));
    println!("\n--> trades: {:?}", trades);
    println!("--> trades2: {:?}\n", trades2);
    orderbook.print();
}

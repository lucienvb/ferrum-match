use ferrum_match::orderbook::book::{next_order_id, OrderBook};
use ferrum_match::orderbook::types::{Order, Side};

use std::collections::BTreeMap;
use std::time::SystemTime;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_test() {
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

        orderbook.add_order(order1.clone());

        assert_eq!(orderbook.bids[&15][0].quantity, 5);
        assert_eq!(orderbook.bids[&15][0].side, Side::Bid);
    }
}

use ferrum_match::orderbook::book::OrderBook;
use ferrum_match::orderbook::types::{Order, OrderId, Price, Quantity, Side};
use std::collections::BTreeMap;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let mut book = OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        };
        book.add_order(make_order(
            OrderId(1),
            100,
            10,
            Side::Ask,
            SystemTime::now(),
        ));

        let trades = book.matching_order(make_order(
            OrderId(2),
            100,
            10,
            Side::Bid,
            SystemTime::now(),
        ));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].taker_order_id, OrderId(2));
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].quantity, 10);
        assert!(book.bids.is_empty());
        assert!(book.asks.is_empty());
    }

    #[test]
    fn test_partial_fill_incoming_greater() {
        let mut book = OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        };
        book.add_order(make_order(OrderId(1), 100, 5, Side::Ask, SystemTime::now()));

        let trades = book.matching_order(make_order(
            OrderId(2),
            100,
            20,
            Side::Bid,
            SystemTime::now(),
        ));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 5);
        let best_bid = book.bids.get(&100).unwrap();
        assert_eq!(best_bid[0].quantity, 15);
        assert_eq!(best_bid[0].id, OrderId(2));
    }

    #[test]
    fn test_partial_fill_book_greater() {
        let mut book = OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        };
        book.add_order(make_order(
            OrderId(1),
            100,
            25,
            Side::Ask,
            SystemTime::now(),
        ));

        let trades = book.matching_order(make_order(
            OrderId(2),
            100,
            10,
            Side::Bid,
            SystemTime::now(),
        ));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 10);
        let level = book.asks.get(&100).unwrap();
        assert_eq!(level[0].quantity, 15);
        assert_eq!(level[0].id, OrderId(1),);
        assert!(book.bids.is_empty());
    }

    #[test]
    fn test_no_crossing() {
        let mut book = OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        };
        book.add_order(make_order(
            OrderId(1),
            105,
            10,
            Side::Ask,
            SystemTime::now(),
        ));

        let trades = book.matching_order(make_order(
            OrderId(2),
            100,
            10,
            Side::Bid,
            SystemTime::now(),
        ));
        println!("trades {:?}", trades);
        assert!(trades.is_empty());
        let bids = book.bids.get(&100).unwrap();
        assert_eq!(bids[0].id, OrderId(1));
    }

    #[test]
    fn test_fifo_within_price_level() {
        let mut book = OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        };
        book.add_order(make_order(OrderId(1), 100, 5, Side::Ask, SystemTime::now()));
        book.add_order(make_order(OrderId(2), 100, 5, Side::Ask, SystemTime::now()));

        let trades =
            book.matching_order(make_order(OrderId(3), 100, 7, Side::Bid, SystemTime::now()));

        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].maker_order_id, OrderId(1),);
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[1].maker_order_id, OrderId(2),);
        assert_eq!(trades[1].quantity, 2);
        let level = book.asks.get(&100).unwrap();
        assert_eq!(level[0].id, OrderId(2),);
        assert_eq!(level[0].quantity, 3);
    }

    #[test]
    fn test_multi_price_levels() {
        let mut book = OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        };
        book.add_order(make_order(OrderId(1), 100, 5, Side::Ask, SystemTime::now()));
        book.add_order(make_order(OrderId(2), 102, 5, Side::Ask, SystemTime::now()));
        book.add_order(make_order(OrderId(3), 105, 5, Side::Ask, SystemTime::now()));

        let trades = book.matching_order(make_order(
            OrderId(4),
            105,
            12,
            Side::Bid,
            SystemTime::now(),
        ));

        assert_eq!(trades.len(), 3);
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[1].price, 102);
        assert_eq!(trades[2].price, 105);
        let remaining = book.asks.get(&105).unwrap();
        assert_eq!(remaining[0].quantity, 3);
    }
}

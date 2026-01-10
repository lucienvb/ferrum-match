use ferrum_match::orderbook::book::OrderBook;
use ferrum_match::orderbook::types::{Order, OrderId, Price, Quantity, Side};
use std::collections::BTreeMap;
use std::time::Duration;
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

fn ts(n: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(n)
}

fn empty_book() -> OrderBook {
    return OrderBook {bids: BTreeMap::new(),
            asks: BTreeMap::new(),}
}

#[cfg(test)]
mod orderbook {
    use super::*;

    #[test]
    fn test_exact_match_buy_order() {
        let mut book = empty_book();
        book.add_order(make_order(OrderId(1), 100, 10, Side::Ask, ts(1)));

        let trades = book.matching_order(make_order(OrderId(2), 100, 10, Side::Bid, ts(2)));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].taker_order_id, OrderId(2));
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].quantity, 10);
        assert!(book.bids.is_empty());
        assert!(book.asks.is_empty());
    }

    #[test]
    fn test_exact_match_sell_order() {
        let mut book = empty_book();
        book.add_order(make_order(OrderId(1), 150, 25, Side::Bid, ts(1)));

        let trades = book.matching_order(make_order(OrderId(2), 150, 25, Side::Ask, ts(2)));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].taker_order_id, OrderId(2));
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].price, 150);
        assert_eq!(trades[0].quantity, 25);
        assert!(book.bids.is_empty());
        assert!(book.asks.is_empty());
    }

    #[test]
    fn test_partial_fill_incoming_greater_buy_order() {
        let mut book = empty_book();
        book.add_order(make_order(OrderId(1), 100, 5, Side::Ask, ts(1)));

        let trades = book.matching_order(make_order(OrderId(2), 100, 20, Side::Bid, ts(2)));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].taker_order_id, OrderId(2));
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].quantity, 5);
        let best_bid = book.bids.get(&100).unwrap();
        assert_eq!(best_bid[0].quantity, 15);
        assert_eq!(best_bid[0].price, 100);
        assert_eq!(best_bid[0].id, OrderId(2));
        assert!(book.asks.is_empty());
    }

    #[test]
    fn test_partial_fill_incoming_greater_sell_order() {
        let mut book = empty_book();
        book.add_order(make_order(OrderId(1), 150, 12, Side::Bid, ts(1)));

        let trades = book.matching_order(make_order(OrderId(2), 150, 23, Side::Ask, ts(2)));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].taker_order_id, OrderId(2));
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].price, 150);
        assert_eq!(trades[0].quantity, 12);
        let best_ask = book.asks.get(&150).unwrap();
        assert_eq!(best_ask[0].quantity, 11);
        assert_eq!(best_ask[0].price, 150);
        assert_eq!(best_ask[0].id, OrderId(2));
        assert!(book.bids.is_empty());
    }

    #[test]
    fn test_partial_fill_book_greater_buy_order() {
        let mut book = empty_book();
        book.add_order(make_order(OrderId(1), 180, 25, Side::Ask, ts(1)));

        let trades = book.matching_order(make_order(OrderId(2), 180, 10, Side::Bid, ts(2)));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].taker_order_id, OrderId(2));
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].quantity, 10);
        assert_eq!(trades[0].price, 180);
        let best_ask = book.asks.get(&180).unwrap();
        assert_eq!(best_ask[0].quantity, 15);
        assert_eq!(best_ask[0].price, 180);
        assert_eq!(best_ask[0].id, OrderId(1),);
        assert!(book.bids.is_empty());
    }

    #[test]
    fn test_partial_fill_book_greater_sell_order() {
        let mut book = empty_book();
        book.add_order(make_order(OrderId(1), 90, 35, Side::Bid, ts(1)));

        let trades = book.matching_order(make_order(OrderId(2), 90, 8, Side::Ask, ts(2)));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].taker_order_id, OrderId(2));
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].quantity, 8);
        assert_eq!(trades[0].price, 90);
        let best_bid = book.bids.get(&90).unwrap();
        assert_eq!(best_bid[0].quantity, 27);
        assert_eq!(best_bid[0].price, 90);
        assert_eq!(best_bid[0].id, OrderId(1),);
        assert!(book.asks.is_empty());
    }

    #[test]
    fn test_no_crossing_buy_order() {
        let mut book = empty_book();
        book.add_order(make_order(OrderId(1), 105, 10, Side::Ask, ts(1)));
        book.add_order(make_order(OrderId(2), 110, 10, Side::Ask, ts(2)));

        let trades = book.matching_order(make_order(OrderId(3), 100, 10, Side::Bid, ts(3)));
        assert!(trades.is_empty());
        let bids = book.bids.get(&100).unwrap();
        assert_eq!(bids[0].id, OrderId(3));
        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.asks.len(), 2);
    }

    #[test]
    fn test_no_crossing_sell_order() {
        let mut book = empty_book();
        book.add_order(make_order(OrderId(1), 95, 10, Side::Bid, ts(1)));
        book.add_order(make_order(OrderId(2), 90, 10, Side::Bid, ts(2)));

        let trades = book.matching_order(make_order(OrderId(3), 100, 10, Side::Ask, ts(2)));
        assert!(trades.is_empty());
        let asks = book.asks.get(&100).unwrap();
        assert_eq!(asks[0].id, OrderId(3));
        assert_eq!(book.asks.len(), 1);
        assert_eq!(book.bids.len(), 2);
    }

    #[test]
    fn test_fifo_within_price_level_buy_order() {
        let mut book = empty_book();
        book.add_order(make_order(OrderId(1), 100, 5, Side::Ask, ts(1)));
        book.add_order(make_order(OrderId(2), 100, 5, Side::Ask, ts(2)));

        let trades = book.matching_order(make_order(OrderId(3), 100, 7, Side::Bid, ts(3)));

        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].taker_order_id, OrderId(3));
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[1].maker_order_id, OrderId(2));
        assert_eq!(trades[1].taker_order_id, OrderId(3));
        assert_eq!(trades[1].quantity, 2);
        let best_ask = book.asks.get(&100).unwrap();
        assert_eq!(best_ask[0].id, OrderId(2),);
        assert_eq!(best_ask[0].quantity, 3);
        assert!(book.bids.is_empty());
    }

    #[test]
    fn test_fifo_within_price_level_sell_order() {
        let mut book = empty_book();
        book.add_order(make_order(OrderId(1), 70, 3, Side::Bid, ts(1)));
        book.add_order(make_order(OrderId(2), 70, 4, Side::Bid, ts(2)));

        let trades = book.matching_order(make_order(OrderId(3), 70, 6, Side::Ask, ts(3)));

        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].taker_order_id, OrderId(3));
        assert_eq!(trades[0].quantity, 3);
        assert_eq!(trades[1].maker_order_id, OrderId(2));
        assert_eq!(trades[1].taker_order_id, OrderId(3));
        assert_eq!(trades[1].quantity, 3);
        let best_bid = book.bids.get(&70).unwrap();
        assert_eq!(best_bid[0].id, OrderId(2),);
        assert_eq!(best_bid[0].quantity, 1);
        assert!(book.asks.is_empty());
    }

    #[test]
    fn test_multi_price_levels_buy_order() {
        let mut book = empty_book();
        book.add_order(make_order(OrderId(1), 100, 5, Side::Ask, ts(1)));
        book.add_order(make_order(OrderId(2), 102, 6, Side::Ask, ts(2)));
        book.add_order(make_order(OrderId(3), 105, 7, Side::Ask, ts(3)));

        let trades = book.matching_order(make_order(OrderId(4), 105, 12, Side::Bid, ts(4)));

        assert_eq!(trades.len(), 3);
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].taker_order_id, OrderId(4));
        assert_eq!(trades[1].price, 102);
        assert_eq!(trades[1].maker_order_id, OrderId(2));
        assert_eq!(trades[1].taker_order_id, OrderId(4));
        assert_eq!(trades[2].price, 105);
        assert_eq!(trades[2].maker_order_id, OrderId(3));
        assert_eq!(trades[2].taker_order_id, OrderId(4));
        let remaining = book.asks.get(&105).unwrap();
        assert_eq!(remaining[0].quantity, 6);
        assert!(book.bids.is_empty());
        assert_eq!(book.asks.len(), 1);
    }

    #[test]
    fn test_multi_price_levels_sell_order() {
        let mut book = empty_book();
        book.add_order(make_order(OrderId(1), 100, 5, Side::Bid, ts(1)));
        book.add_order(make_order(OrderId(2), 102, 6, Side::Bid, ts(2)));
        book.add_order(make_order(OrderId(3), 105, 7, Side::Bid, ts(3)));

        let trades = book.matching_order(make_order(OrderId(4), 95, 12, Side::Ask, ts(4)));

        assert_eq!(trades.len(), 2,);
        assert_eq!(trades[0].price, 105);
        assert_eq!(trades[0].maker_order_id, OrderId(3));
        assert_eq!(trades[0].taker_order_id, OrderId(4));
        assert_eq!(trades[1].price, 102);
        assert_eq!(trades[1].maker_order_id, OrderId(2));
        assert_eq!(trades[1].taker_order_id, OrderId(4));
        let remaining_102 = book.bids.get(&102).unwrap();
        assert_eq!(remaining_102[0].quantity, 1);
        let remaining_100 = book.bids.get(&100).unwrap();
        assert_eq!(remaining_100[0].quantity, 5);
        assert_eq!(book.bids.len(), 2);
        assert!(book.asks.is_empty());
    }
}

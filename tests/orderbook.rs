use ferrum_match::orderbook::book::OrderBook;
use ferrum_match::orderbook::types::{OrderId, Side};
use std::collections::BTreeMap;

fn empty_book() -> OrderBook {
    return OrderBook {
        next_seq: 0,
        bids: BTreeMap::new(),
        asks: BTreeMap::new(),
    };
}

#[cfg(test)]
mod orderbook {
    use super::*;

    #[test]
    fn should_match_exact_when_buy_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 100, 10, Side::Ask);

        let order2 = OrderBook::make_order_request(OrderId(2), 100, 10, Side::Bid);
        let trades = book.matching_order(order2);

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].taker_order_id, OrderId(2));
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].quantity, 10);
        assert!(book.bids.is_empty());
        assert!(book.asks.is_empty());
    }

    #[test]
    fn should_match_exact_when_sell_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 150, 25, Side::Bid);

        let trades = book.matching_order(OrderBook::make_order_request(
            OrderId(2),
            150,
            25,
            Side::Ask,
        ));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].taker_order_id, OrderId(2));
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].price, 150);
        assert_eq!(trades[0].quantity, 25);
        assert!(book.bids.is_empty());
        assert!(book.asks.is_empty());
    }

    #[test]
    fn should_partial_fill_when_incoming_buy_order_is_larger() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 100, 5, Side::Ask);

        let trades = book.matching_order(OrderBook::make_order_request(
            OrderId(2),
            100,
            20,
            Side::Bid,
        ));

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
    fn should_partial_fill_when_incoming_sell_order_is_larger() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 150, 12, Side::Bid);

        let trades = book.matching_order(OrderBook::make_order_request(
            OrderId(2),
            150,
            23,
            Side::Ask,
        ));

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
    fn should_partial_fill_when_book_sell_order_is_larger() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 180, 25, Side::Ask);

        let trades = book.matching_order(OrderBook::make_order_request(
            OrderId(2),
            180,
            10,
            Side::Bid,
        ));

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
    fn should_partial_fill_when_book_buy_order_is_larger() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 90, 35, Side::Bid);

        let trades =
            book.matching_order(OrderBook::make_order_request(OrderId(2), 90, 8, Side::Ask));

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
    fn should_not_cross_asks_when_buy_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 105, 10, Side::Ask);
        book.add_order(OrderId(2), 110, 10, Side::Ask);

        let trades = book.matching_order(OrderBook::make_order_request(
            OrderId(3),
            100,
            10,
            Side::Bid,
        ));
        assert!(trades.is_empty());
        let bids = book.bids.get(&100).unwrap();
        assert_eq!(bids[0].id, OrderId(3));
        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.asks.len(), 2);
    }

    #[test]
    fn should_not_cross_bids_when_sell_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 95, 10, Side::Bid);
        book.add_order(OrderId(2), 90, 10, Side::Bid);

        let trades = book.matching_order(OrderBook::make_order_request(
            OrderId(3),
            100,
            10,
            Side::Ask,
        ));
        assert!(trades.is_empty());
        let asks = book.asks.get(&100).unwrap();
        assert_eq!(asks[0].id, OrderId(3));
        assert_eq!(book.asks.len(), 1);
        assert_eq!(book.bids.len(), 2);
    }

    #[test]
    fn should_match_fifo_within_price_level_when_buy_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 100, 5, Side::Ask);
        book.add_order(OrderId(2), 100, 5, Side::Ask);
        book.add_order(OrderId(3), 100, 5, Side::Ask);

        let trades = book.matching_order(OrderBook::make_order_request(
            OrderId(4),
            100,
            12,
            Side::Bid,
        ));

        assert_eq!(trades.len(), 3);
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].maker_arrival_seq, 0);
        assert_eq!(trades[0].taker_order_id, OrderId(4));
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[1].maker_order_id, OrderId(2));
        assert_eq!(trades[1].maker_arrival_seq, 1);
        assert_eq!(trades[1].taker_order_id, OrderId(4));
        assert_eq!(trades[1].quantity, 5);
        assert_eq!(trades[2].maker_order_id, OrderId(3));
        assert_eq!(trades[2].maker_arrival_seq, 2);
        assert_eq!(trades[2].taker_order_id, OrderId(4));
        assert_eq!(trades[2].quantity, 2);
        let best_ask = book.asks.get(&100).unwrap();
        assert_eq!(best_ask[0].id, OrderId(3),);
        assert_eq!(best_ask[0].quantity, 3);
        assert!(book.bids.is_empty());
    }

    #[test]
    fn should_match_fifo_within_price_level_when_sell_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 70, 3, Side::Bid);
        book.add_order(OrderId(2), 70, 4, Side::Bid);

        let trades =
            book.matching_order(OrderBook::make_order_request(OrderId(3), 70, 6, Side::Ask));

        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].maker_arrival_seq, 0);
        assert_eq!(trades[0].taker_order_id, OrderId(3));
        assert_eq!(trades[0].quantity, 3);
        assert_eq!(trades[1].maker_order_id, OrderId(2));
        assert_eq!(trades[1].maker_arrival_seq, 1);
        assert_eq!(trades[1].taker_order_id, OrderId(3));
        assert_eq!(trades[1].quantity, 3);
        let best_bid = book.bids.get(&70).unwrap();
        assert_eq!(best_bid[0].id, OrderId(2),);
        assert_eq!(best_bid[0].quantity, 1);
        assert!(book.asks.is_empty());
    }

    #[test]
    fn should_match_multi_price_levels_when_buy_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 100, 5, Side::Ask);
        book.add_order(OrderId(2), 102, 6, Side::Ask);
        book.add_order(OrderId(3), 105, 7, Side::Ask);

        let trades = book.matching_order(OrderBook::make_order_request(
            OrderId(4),
            105,
            12,
            Side::Bid,
        ));

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
    fn should_match_multi_price_levels_when_sell_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 100, 5, Side::Bid);
        book.add_order(OrderId(2), 102, 6, Side::Bid);
        book.add_order(OrderId(3), 105, 7, Side::Bid);

        let trades =
            book.matching_order(OrderBook::make_order_request(OrderId(4), 95, 12, Side::Ask));

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

    #[test]
    fn should_cross_partially_due_to_price_limit_when_buy_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 100, 5, Side::Ask);
        book.add_order(OrderId(2), 101, 5, Side::Ask);
        book.add_order(OrderId(3), 102, 5, Side::Ask);

        let trades = book.matching_order(OrderBook::make_order_request(
            OrderId(4),
            101,
            20,
            Side::Bid,
        ));

        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].taker_order_id, OrderId(4));
        assert_eq!(trades[1].price, 101);
        assert_eq!(trades[1].quantity, 5);
        assert_eq!(trades[1].maker_order_id, OrderId(2));
        assert_eq!(trades[1].taker_order_id, OrderId(4));
        let remaining = book.bids.get(&101).unwrap();
        assert_eq!(remaining[0].quantity, 10);
        assert_eq!(book.asks.len(), 1);
    }

    #[test]
    fn should_cross_partially_due_to_price_limit_when_sell_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 100, 5, Side::Bid);
        book.add_order(OrderId(2), 101, 5, Side::Bid);
        book.add_order(OrderId(3), 102, 5, Side::Bid);

        let trades = book.matching_order(OrderBook::make_order_request(
            OrderId(4),
            101,
            30,
            Side::Ask,
        ));

        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].price, 102);
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[0].maker_order_id, OrderId(3));
        assert_eq!(trades[0].taker_order_id, OrderId(4));
        assert_eq!(trades[1].price, 101);
        assert_eq!(trades[1].quantity, 5);
        assert_eq!(trades[1].maker_order_id, OrderId(2));
        assert_eq!(trades[1].taker_order_id, OrderId(4));
        let remaining = book.asks.get(&101).unwrap();
        assert_eq!(remaining[0].quantity, 20);
        assert_eq!(book.bids.len(), 1);
    }

    #[test]
    fn should_process_trade_as_buy_order_when_book_is_empty() {
        let mut book = empty_book();

        let trades =
            book.matching_order(OrderBook::make_order_request(OrderId(1), 100, 3, Side::Bid));

        assert!(trades.is_empty());
        let level = book.bids.get(&100).unwrap();
        assert_eq!(level[0].quantity, 3);
        assert_eq!(level[0].price, 100);
        assert_eq!(level[0].id, OrderId(1));
        assert!(book.asks.is_empty());
    }

    #[test]
    fn should_process_trade_as_sell_order_when_book_is_empty() {
        let mut book = empty_book();

        let trades =
            book.matching_order(OrderBook::make_order_request(OrderId(1), 99, 6, Side::Ask));

        assert!(trades.is_empty());
        let level = book.asks.get(&99).unwrap();
        assert_eq!(level[0].quantity, 6);
        assert_eq!(level[0].price, 99);
        assert_eq!(level[0].id, OrderId(1));
        assert!(book.bids.is_empty());
    }

    #[test]
    fn should_cleanup_price_level_when_buy_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 50, 2, Side::Ask);

        book.matching_order(OrderBook::make_order_request(OrderId(2), 50, 2, Side::Bid));

        assert!(!book.asks.contains_key(&50));
        assert!(book.asks.is_empty());
    }

    #[test]
    fn should_cleanup_price_level_when_sell_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 70, 6, Side::Bid);

        book.matching_order(OrderBook::make_order_request(OrderId(2), 70, 6, Side::Ask));

        assert!(!book.bids.contains_key(&70));
        assert!(book.bids.is_empty());
    }

    #[test]
    fn should_handle_invalid_quantity_when_buy_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 70, 0, Side::Bid);

        book.matching_order(OrderBook::make_order_request(OrderId(2), 70, 0, Side::Ask));

        assert!(book.asks.is_empty());
        assert!(book.bids.is_empty());
    }

    #[test]
    fn should_handle_invalid_quantity_when_sell_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 70, 0, Side::Ask);

        book.matching_order(OrderBook::make_order_request(OrderId(2), 70, 0, Side::Bid));

        assert!(book.asks.is_empty());
        assert!(book.bids.is_empty());
    }

    #[test]
    fn should_handle_invalid_price_when_buy_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 0, 5, Side::Bid);

        book.matching_order(OrderBook::make_order_request(OrderId(2), 0, 5, Side::Ask));

        assert!(book.asks.is_empty());
        assert!(book.bids.is_empty());
    }

    #[test]
    fn should_handle_invalid_price_when_sell_order() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 0, 5, Side::Ask);

        book.matching_order(OrderBook::make_order_request(OrderId(2), 0, 5, Side::Bid));

        assert!(book.asks.is_empty());
        assert!(book.bids.is_empty());
    }

    #[test]
    fn should_handle_orders_based_on_arrival_sequence() {
        let mut book = empty_book();
        book.add_order(OrderId(1), 5, 10, Side::Bid);
        book.add_order(OrderId(2), 5, 10, Side::Bid);

        let trades =
            book.matching_order(OrderBook::make_order_request(OrderId(3), 5, 10, Side::Ask));

        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].maker_arrival_seq, 0);
        assert!(book.asks.is_empty());
        assert_eq!(book.bids.len(), 1);
    }
}

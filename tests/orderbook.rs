use ferrum_match::orderbook::types::{OrderBook, OrderId, Side};
use proptest::prelude::*;
use std::collections::{BTreeMap, HashMap};

fn empty_book() -> OrderBook {
    return OrderBook {
        next_seq: 0,
        order_id_counter: 1,
        bids: BTreeMap::new(),
        asks: BTreeMap::new(),
        order_index: HashMap::new(),
    };
}

#[cfg(test)]
mod orderbook {
    use super::*;

    #[test]
    fn should_match_exact_when_buy_order() {
        let mut book = empty_book();
        book.add_order_external(100, 10, Side::Ask);

        let order2 = OrderBook::make_order_request(100, 10, Side::Bid);
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
        book.add_order_external(150, 25, Side::Bid);

        let trades = book.matching_order(OrderBook::make_order_request(150, 25, Side::Ask));

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
        book.add_order_external(100, 5, Side::Ask);

        let trades = book.matching_order(OrderBook::make_order_request(100, 20, Side::Bid));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].taker_order_id, OrderId(2));
        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].quantity, 5);
        let best_bid = book.bids.get(&100).unwrap();
        assert_eq!(best_bid[0].quantity, 15);
        assert_eq!(best_bid[0].price, 100);
        println!("BEST BID {:?}", best_bid);
        assert_eq!(best_bid[0].id, OrderId(2));
        assert!(book.asks.is_empty());
    }

    #[test]
    fn should_partial_fill_when_incoming_sell_order_is_larger() {
        let mut book = empty_book();
        book.add_order_external(150, 12, Side::Bid);

        let trades = book.matching_order(OrderBook::make_order_request(150, 23, Side::Ask));

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
        book.add_order_external(180, 25, Side::Ask);

        let trades = book.matching_order(OrderBook::make_order_request(180, 10, Side::Bid));

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
        book.add_order_external(90, 35, Side::Bid);

        let trades = book.matching_order(OrderBook::make_order_request(90, 8, Side::Ask));

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
        book.add_order_external(105, 10, Side::Ask);
        book.add_order_external(110, 10, Side::Ask);

        let trades = book.matching_order(OrderBook::make_order_request(100, 10, Side::Bid));
        assert!(trades.is_empty());
        let bids = book.bids.get(&100).unwrap();
        assert_eq!(bids[0].id, OrderId(3));
        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.asks.len(), 2);
    }

    #[test]
    fn should_not_cross_bids_when_sell_order() {
        let mut book = empty_book();
        book.add_order_external(95, 10, Side::Bid);
        book.add_order_external(90, 10, Side::Bid);

        let trades = book.matching_order(OrderBook::make_order_request(100, 10, Side::Ask));
        assert!(trades.is_empty());
        let asks = book.asks.get(&100).unwrap();
        assert_eq!(asks[0].id, OrderId(3));
        assert_eq!(book.asks.len(), 1);
        assert_eq!(book.bids.len(), 2);
    }

    #[test]
    fn should_match_fifo_within_price_level_when_buy_order() {
        let mut book = empty_book();
        book.add_order_external(100, 5, Side::Ask);
        book.add_order_external(100, 5, Side::Ask);
        book.add_order_external(100, 5, Side::Ask);

        let trades = book.matching_order(OrderBook::make_order_request(100, 12, Side::Bid));

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
        book.add_order_external(70, 3, Side::Bid);
        book.add_order_external(70, 4, Side::Bid);

        let trades = book.matching_order(OrderBook::make_order_request(70, 6, Side::Ask));

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
        book.add_order_external(100, 5, Side::Ask);
        book.add_order_external(102, 6, Side::Ask);
        book.add_order_external(105, 7, Side::Ask);

        let trades = book.matching_order(OrderBook::make_order_request(105, 12, Side::Bid));

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
        book.add_order_external(100, 5, Side::Bid);
        book.add_order_external(102, 6, Side::Bid);
        book.add_order_external(105, 7, Side::Bid);

        let trades = book.matching_order(OrderBook::make_order_request(95, 12, Side::Ask));

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
        book.add_order_external(100, 5, Side::Ask);
        book.add_order_external(101, 5, Side::Ask);
        book.add_order_external(102, 5, Side::Ask);

        let trades = book.matching_order(OrderBook::make_order_request(101, 20, Side::Bid));

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
        book.add_order_external(100, 5, Side::Bid);
        book.add_order_external(101, 5, Side::Bid);
        book.add_order_external(102, 5, Side::Bid);

        let trades = book.matching_order(OrderBook::make_order_request(101, 30, Side::Ask));

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

        let trades = book.matching_order(OrderBook::make_order_request(100, 3, Side::Bid));

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

        let trades = book.matching_order(OrderBook::make_order_request(99, 6, Side::Ask));

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
        book.add_order_external(50, 2, Side::Ask);

        book.matching_order(OrderBook::make_order_request(50, 2, Side::Bid));

        assert!(!book.asks.contains_key(&50));
        assert!(book.asks.is_empty());
    }

    #[test]
    fn should_cleanup_price_level_when_sell_order() {
        let mut book = empty_book();
        book.add_order_external(70, 6, Side::Bid);

        book.matching_order(OrderBook::make_order_request(70, 6, Side::Ask));

        assert!(!book.bids.contains_key(&70));
        assert!(book.bids.is_empty());
    }

    #[test]
    fn should_handle_invalid_quantity_when_buy_order() {
        let mut book = empty_book();
        book.add_order_external(70, 0, Side::Bid);

        book.matching_order(OrderBook::make_order_request(70, 0, Side::Ask));

        assert!(book.asks.is_empty());
        assert!(book.bids.is_empty());
    }

    #[test]
    fn should_handle_invalid_quantity_when_sell_order() {
        let mut book = empty_book();
        book.add_order_external(70, 0, Side::Ask);

        book.matching_order(OrderBook::make_order_request(70, 0, Side::Bid));

        assert!(book.asks.is_empty());
        assert!(book.bids.is_empty());
    }

    #[test]
    fn should_handle_invalid_price_when_buy_order() {
        let mut book = empty_book();
        book.add_order_external(0, 5, Side::Bid);

        book.matching_order(OrderBook::make_order_request(0, 5, Side::Ask));

        assert!(book.asks.is_empty());
        assert!(book.bids.is_empty());
    }

    #[test]
    fn should_handle_invalid_price_when_sell_order() {
        let mut book = empty_book();
        book.add_order_external(0, 5, Side::Ask);

        book.matching_order(OrderBook::make_order_request(0, 5, Side::Bid));

        assert!(book.asks.is_empty());
        assert!(book.bids.is_empty());
    }

    #[test]
    fn should_handle_orders_based_on_arrival_sequence() {
        let mut book = empty_book();
        book.add_order_external(5, 10, Side::Bid);
        book.add_order_external(5, 10, Side::Bid);

        let trades = book.matching_order(OrderBook::make_order_request(5, 10, Side::Ask));

        assert_eq!(trades[0].maker_order_id, OrderId(1));
        assert_eq!(trades[0].maker_arrival_seq, 0);
        assert!(book.asks.is_empty());
        assert_eq!(book.bids.len(), 1);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    #[test]
    fn test_volume_conservation(
        initial_qty in 1u64..1000u64,
        incoming_qty in 1u64..1000u64,
        price in 1u64..1000u64
    ) {
        let mut book = empty_book();

        book.add_order_external( price, initial_qty, Side::Ask);

        let taker_req = OrderBook::make_order_request( price, incoming_qty, Side::Bid);
        let trades = book.matching_order(taker_req);

        let traded_volume: u64 = trades.iter().map(|t| t.quantity).sum();

        let remaining_ask_volume: u64 = book.asks.values()
            .flatten()
            .map(|o| o.quantity)
            .sum();

        let remaining_bid_volume: u64 = book.bids.values()
            .flatten()
            .map(|o| o.quantity)
            .sum();

        let total_after = traded_volume * 2 + remaining_ask_volume + remaining_bid_volume;
        let total_before = initial_qty + incoming_qty;

        prop_assert_eq!(total_after, total_before,
            "Lost volume! Before: {}, After: {}", total_before, total_after);
    }
}

proptest! {
        #[test]
        fn test_time_priority_is_respected(
            maker_qty_list in prop::collection::vec(1u64..100u64, 2..10)
        ) {
            let mut book = empty_book();
            let price = 100;

            for (_i, qty) in maker_qty_list.iter().enumerate() {
                book.add_order_external( price, *qty, Side::Ask);
            }

            let total_taker_qty: u64 = maker_qty_list.iter().sum();
            let taker_req = OrderBook::make_order_request( price, total_taker_qty, Side::Bid);
            let trades = book.matching_order(taker_req);

            for i in 0..(trades.len() - 1) {
                prop_assert!(
                    trades[i].maker_arrival_seq < trades[i+1].maker_arrival_seq,
                    "Time priority violated! Trade {} (seq {}) came after Trade {} (seq {})",
                    i+1, trades[i+1].maker_arrival_seq, i, trades[i].maker_arrival_seq
                );
            }
        }
}

use crate::orderbook::types::Quantity;

use super::types::{Order, OrderId, Price, Side, Trade, Trades};

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

static ORDER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub struct OrderBook {
    pub bids: BTreeMap<Price, Vec<Order>>,
    pub asks: BTreeMap<Price, Vec<Order>>,
}

pub fn next_order_id() -> OrderId {
    OrderId(ORDER_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
}

impl OrderBook {
    pub fn add_order(&mut self, order: Order) {
        match order.side {
            Side::Bid => &mut self
                .bids
                .entry(order.price)
                .or_insert_with(Vec::new)
                .push(order),
            Side::Ask => &mut self
                .asks
                .entry(order.price)
                .or_insert_with(Vec::new)
                .push(order),
        };
    }

    pub fn print(&self) {
        println!("=== ORDER BOOK ===");

        println!("-- Asks (sell orders) --");
        for (price, orders) in &self.asks {
            println!("@ {}: {:?}", price, orders)
        }

        println!("-- Bids (buy orders) --");
        for (price, orders) in self.bids.iter().rev() {
            println!("@ {}: {:?}", price, orders)
        }
    }

    pub fn cleanup_price_level(level: &mut BTreeMap<Price, Vec<Order>>, price: Price) {
        if let Some(orders) = level.get(&price) {
            if orders.is_empty() {
                level.remove(&price);
            }
        }
    }

    pub fn make_trade(
        taker_order_id: OrderId,
        maker_order_id: OrderId,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    ) -> Trade {
        Trade {
            taker_order_id,
            maker_order_id,
            price,
            quantity,
            timestamp,
        }
    }

    fn matching_ask_order(&mut self, mut incoming: Order) -> Trades {
        let mut trades = Trades::default();
        let mut count = 0;
        println!("matching_ask_order");

        while incoming.quantity > 0 {
            println!("incoming.quantity: {}", incoming.quantity);
            let price_of_best_bid = match self.bids.keys().next_back().map(|&p| p) {
                Some(p) => p,
                None => break,
            };
            if price_of_best_bid < incoming.price {
                break;
            };

            let level = match self.bids.get_mut(&price_of_best_bid) {
                Some(v) if !v.is_empty() => v,
                _ => {
                    self.bids.remove(&price_of_best_bid);
                    continue;
                }
            };

            let best = &mut level[0];

            let qty_traded = best.quantity.min(incoming.quantity);

            trades.push(Self::make_trade(
                incoming.id,
                best.id,
                price_of_best_bid,
                qty_traded,
                SystemTime::now(),
            ));

            best.quantity -= qty_traded;
            incoming.quantity -= qty_traded;

            if best.quantity == 0 {
                level.remove(0);
            }
            if level.is_empty() {
                self.bids.remove(&price_of_best_bid);
            }
            count += 1;
        }
        if incoming.quantity > 0 {
            self.add_order(incoming.clone());
        }

        trades
    }

    fn matching_bid_order(&mut self, mut incoming: Order) -> Trades {
        let mut trades = Trades::default();
        let mut count = 0;
        println!("matching_bid_order");

        while incoming.quantity > 0 {
            println!("incoming.quantity: {}", incoming.quantity);
            let price_of_best_ask = match self.asks.keys().next().map(|&p| p) {
                Some(p) => p,
                None => break,
            };
            if price_of_best_ask > incoming.price {
                break;
            };
            println!("price of best ask: {}", price_of_best_ask);

            let level = match self.asks.get_mut(&price_of_best_ask) {
                Some(v) if !v.is_empty() => v,
                _ => {
                    self.asks.remove(&price_of_best_ask);
                    continue;
                }
            };

            let best = &mut level[0];

            let qty_traded = best.quantity.min(incoming.quantity);

            trades.push(Self::make_trade(
                incoming.id,
                best.id,
                price_of_best_ask,
                qty_traded,
                SystemTime::now(),
            ));

            best.quantity -= qty_traded;
            incoming.quantity -= qty_traded;

            if best.quantity == 0 {
                level.remove(0);
            }
            if level.is_empty() {
                self.asks.remove(&price_of_best_ask);
            }
            count += 1;
        }
        if incoming.quantity > 0 {
            self.add_order(incoming.clone());
        }

        trades
    }

    pub fn matching_order(&mut self, incoming: Order) -> Trades {
        match incoming.side {
            Side::Ask => self.matching_ask_order(incoming),
            Side::Bid => self.matching_bid_order(incoming),
        }
    }
}

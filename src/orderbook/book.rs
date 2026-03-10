use crate::orderbook::types::{OrderBook, OrderRequest, Quantity};

use super::types::{Order, OrderId, Price, Side, Trade, Trades};

use std::time::SystemTime;

impl OrderBook {
    pub fn next_order_id(&mut self) -> OrderId {
        let id = OrderId(self.order_id_counter);
        self.order_id_counter += 1;
        id
    }

    pub fn make_order_request(price: Price, quantity: Quantity, side: Side) -> OrderRequest {
        return OrderRequest {
            price,
            quantity,
            side,
        };
    }

    pub fn make_order(&mut self, price: Price, quantity: Quantity) -> Order {
        let order = Order {
            id: Self::next_order_id(self),
            price,
            quantity,
            arrival_seq: self.next_seq,
        };
        self.next_seq += 1;
        return order;
    }

    pub fn add_order_external(&mut self, price: Price, quantity: Quantity, side: Side) {
        if quantity == 0 {
            return;
        }

        let order = self.make_order(price, quantity);

        let target_map = match side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };

        target_map
            .entry(order.price)
            .or_insert_with(Vec::new)
            .push(order);
    }

    pub fn add_order_internal(&mut self, order: Order, side: Side) {
        if order.quantity == 0 {
            return;
        }

        let target_map = match side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };

        target_map
            .entry(order.price)
            .or_insert_with(Vec::new)
            .push(order);
    }

    pub fn make_trade(
        taker_order_id: OrderId,
        maker_order_id: OrderId,
        maker_arrival_seq: u64,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    ) -> Trade {
        Trade {
            taker_order_id,
            maker_order_id,
            maker_arrival_seq,
            price,
            quantity,
            timestamp,
        }
    }

    fn matching_ask_order(&mut self, mut incoming: Order) -> Trades {
        let mut trades = Trades::default();
        println!("matching_ask_order for quantity: {}", incoming.quantity);

        while incoming.quantity > 0 {
            let Some(&price_of_best_bid) = self.bids.keys().next_back() else {
                break;
            };

            if price_of_best_bid < incoming.price {
                break;
            };

            if let Some(level) = self.bids.get_mut(&price_of_best_bid) {
                let best = &mut level[0];
                let qty_traded = best.quantity.min(incoming.quantity);

                trades.push(Self::make_trade(
                    incoming.id,
                    best.id,
                    best.arrival_seq,
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
            } else {
                self.bids.remove(&price_of_best_bid);
            };
        }

        if incoming.quantity > 0 {
            self.add_order_internal(incoming, Side::Ask);
        }

        trades
    }

    fn matching_bid_order(&mut self, mut incoming: Order) -> Trades {
        let mut trades = Trades::default();
        println!("matching_bid_order for quantity: {}", incoming.quantity);

        while incoming.quantity > 0 {
            let Some(&price_of_best_ask) = self.asks.keys().next() else {
                break;
            };

            if price_of_best_ask > incoming.price {
                break;
            };

            if let Some(level) = self.asks.get_mut(&price_of_best_ask) {
                let best = &mut level[0];

                let qty_traded = best.quantity.min(incoming.quantity);

                trades.push(Self::make_trade(
                    incoming.id,
                    best.id,
                    best.arrival_seq,
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
            } else {
                self.asks.remove(&price_of_best_ask);
            };
        }

        if incoming.quantity > 0 {
            self.add_order_internal(incoming, Side::Bid);
        }

        trades
    }

    pub fn matching_order(&mut self, incoming: OrderRequest) -> Trades {
        let order = self.make_order(incoming.price, incoming.quantity);
        match incoming.side {
            Side::Ask => self.matching_ask_order(order),
            Side::Bid => self.matching_bid_order(order),
        }
    }

    pub fn print(&self) {
        println!("=== ORDER BOOK ===");

        println!("-- Asks (sell orders) --");
        self.asks
            .iter()
            .for_each(|(price, orders)| println!("@ {}: {:?}", price, orders));

        println!("-- Bids (buy orders) --");
        self.bids
            .iter()
            .rev()
            .for_each(|(price, orders)| println!("@ {}: {:?}", price, orders));
    }
}

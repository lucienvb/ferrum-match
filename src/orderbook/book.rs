use tracing::{debug, info, instrument, trace};

use crate::orderbook::types::{OrderBook, OrderRequest, Quantity};

use super::types::{Order, OrderId, Price, Side, Trade, Trades};

use std::time::SystemTime;

impl OrderBook {
    pub fn next_order_id(&mut self) -> OrderId {
        let id = OrderId(self.order_id_counter);
        self.order_id_counter += 1;
        trace!(new_id = id.0, "Generated next order ID");
        id
    }

    pub fn make_order_request(price: Price, quantity: Quantity, side: Side) -> OrderRequest {
        return OrderRequest {
            price,
            quantity,
            side,
        };
    }

    #[instrument(skip(self), fields(order_id))]
    pub fn make_order(&mut self, price: Price, quantity: Quantity) -> Order {
        let id = self.next_order_id();
        tracing::Span::current().record("order_id", id.0);

        let order = Order {
            id,
            price,
            quantity,
            arrival_seq: self.next_seq,
        };

        self.next_seq += 1;
        debug!(id = %order.id.0, price = %order.price, qty = %order.quantity, "Order struct initialized");
        order
    }

    #[instrument(skip(self))]
    pub fn add_order_external(&mut self, price: Price, quantity: Quantity, side: Side) {
        if quantity == 0 {
            return;
        }

        let order = self.make_order(price, quantity);
        info!(side = ?side, price = %order.price, qty = %order.quantity, "Adding external order to book");
        self.add_order_internal(order, side);
    }

    #[instrument(skip(self, order))]
    pub fn add_order_internal(&mut self, order: Order, side: Side) {
        if order.quantity == 0 {
            return;
        }

        trace!(id = %order.id.0, side = ?side, "Inserting order into BTreeMap");
        let target_map = match side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };

        target_map
            .entry(order.price)
            .or_insert_with(Vec::new)
            .push(order);
    }

    #[instrument(fields(order_id))]
    pub fn make_trade(
        taker_order_id: OrderId,
        maker_order_id: OrderId,
        maker_arrival_seq: u64,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    ) -> Trade {
        let trade = Trade {
            taker_order_id,
            maker_order_id,
            maker_arrival_seq,
            price,
            quantity,
            timestamp,
        };

        info!(
            target = "trades",
            taker = %trade.taker_order_id.0,
            maker = %trade.maker_order_id.0,
            price = %trade.price,
            qty = %trade.quantity,
            "Execution occurred"
        );

        trade
    }

    #[instrument(skip(self, incoming), fields(incoming_id = %incoming.id.0))]
    fn matching_ask_order(&mut self, mut incoming: Order) -> Trades {
        let mut trades = Trades::default();
        debug!(
            qty = incoming.quantity,
            price = incoming.price,
            "Start matching ask order"
        );

        while incoming.quantity > 0 {
            let Some(&price_of_best_bid) = self.bids.keys().next_back() else {
                trace!("No more bids available in the book");
                break;
            };

            if price_of_best_bid < incoming.price {
                trace!(best_bid = %price_of_best_bid, incoming_price = %incoming.price, "Price gap reached; stopping match");
                break;
            };

            if let Some(level) = self.bids.get_mut(&price_of_best_bid) {
                let best = &mut level[0];
                let qty_traded = best.quantity.min(incoming.quantity);

                trace!(maker_id = %best.id.0, qty = %qty_traded, "Matching against price level");

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
                    trace!(maker_id = %best.id.0, "Maker order fully filled, removing from level");
                    level.remove(0);
                }
                if level.is_empty() {
                    trace!(price = %price_of_best_bid, "Price level empty, removing from book");
                    self.bids.remove(&price_of_best_bid);
                }
            }
        }

        if incoming.quantity > 0 {
            debug!(remaining_qty = %incoming.quantity, "Ask order not fully filled, adding remainder to book");
            self.add_order_internal(incoming, Side::Ask);
        }

        trades
    }

    #[instrument(skip(self, incoming), fields(incoming_id = %incoming.id.0))]
    fn matching_bid_order(&mut self, mut incoming: Order) -> Trades {
        let mut trades = Trades::default();
        debug!(
            qty = incoming.quantity,
            price = incoming.price,
            "Start matching bid order"
        );

        while incoming.quantity > 0 {
            let Some(&price_of_best_ask) = self.asks.keys().next() else {
                trace!("No more asks available in the book");
                break;
            };

            if price_of_best_ask > incoming.price {
                trace!(best_ask = %price_of_best_ask, incoming_price = %incoming.price, "Price gap reached; stopping match");
                break;
            };

            if let Some(level) = self.asks.get_mut(&price_of_best_ask) {
                let best = &mut level[0];
                let qty_traded = best.quantity.min(incoming.quantity);

                trace!(maker_id = %best.id.0, qty = %qty_traded, "Matching against price level");

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
                    trace!(maker_id = %best.id.0, "Maker order fully filled, removing from level");
                    level.remove(0);
                }
                if level.is_empty() {
                    trace!(price = %price_of_best_ask, "Price level empty, removing from book");
                    self.asks.remove(&price_of_best_ask);
                }
            }
        }

        if incoming.quantity > 0 {
            debug!(remaining_qty = %incoming.quantity, "Bid order not fully filled, adding remainder to book");
            self.add_order_internal(incoming, Side::Bid);
        }

        trades
    }

    #[instrument(skip(self), fields(side = ?incoming.side, p = %incoming.price, q = %incoming.quantity))]
    pub fn matching_order(&mut self, incoming: OrderRequest) -> Trades {
        info!("Processing incoming order request");
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

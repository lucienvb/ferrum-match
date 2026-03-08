use std::time::SystemTime;

pub type Price = u64;
pub type Quantity = u64;
pub type ArrivalSeq = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub enum Side {
    Bid,
    Ask,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Order {
    pub id: OrderId,
    pub side: Side,
    pub price: Price,
    pub quantity: Quantity,
    pub arrival_seq: ArrivalSeq,
}

#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub id: OrderId,
    pub side: Side,
    pub price: Price,
    pub quantity: Quantity,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Trade {
    pub taker_order_id: OrderId,
    pub maker_order_id: OrderId,
    pub maker_arrival_seq: u64,
    pub price: Price,
    pub quantity: Quantity,
    pub timestamp: SystemTime,
}

pub type Trades = Vec<Trade>;

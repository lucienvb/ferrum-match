struct Order {
    id: u64,
    price: f64,
    quantity: u64,
    is_buy: bool
}

fn main() {
    let order1 = Order {id :1, price: 15.0, quantity: 5, is_buy: true};
    let id = order1.id;
    let price = order1.price;
    let quantity = order1.quantity;
    let is_buy = order1.is_buy;
    print!("id: {id}, price: {price}, quantity: {quantity}, is_buy: {is_buy}\n");
}

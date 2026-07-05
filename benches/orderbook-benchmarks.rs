use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ferrum_match::orderbook::types::{OrderBook, OrderId, Side};

const LIMIT: u64 = 100000;
const CANCEL_LIMIT: u64 = 10000;

fn bench_insert_no_match(c: &mut Criterion) {
    c.bench_function("no match 100k bids", |b| {
        b.iter_batched(
            || OrderBook::new(),
            |mut book: OrderBook| {
                for i in 0..LIMIT {
                    black_box(book.matching_order(OrderBook::make_order_request(
                        100 + (i % 10),
                        10,
                        Side::Bid,
                    )));
                }
            },
            criterion::BatchSize::LargeInput,
        );
    });
}

fn bench_insert_full_match(c: &mut Criterion) {
    c.bench_function("full match 50/50 100k", |b| {
        b.iter_batched(
            || OrderBook::new(),
            |mut book| {
                for i in 0..LIMIT {
                    let order = if i % 2 == 0 {
                        OrderBook::make_order_request(200, 11, Side::Bid)
                    } else {
                        OrderBook::make_order_request(100, 11, Side::Ask)
                    };
                    black_box(book.matching_order(order));
                }
            },
            criterion::BatchSize::LargeInput,
        );
    });
}

fn bench_insert_partial_match(c: &mut Criterion) {
    c.bench_function("partial match 100k", |b| {
        b.iter_batched(
            || OrderBook::new(),
            |mut book| {
                for i in 0..LIMIT {
                    black_box(book.matching_order(OrderBook::make_order_request(
                        100 + (i % 10),
                        12,
                        get_side(i, 3),
                    )));
                }
            },
            criterion::BatchSize::LargeInput,
        );
    });
}

fn bench_deep_book_insert(c: &mut Criterion) {
    c.bench_function("deep book single price 100k", |b| {
        b.iter_batched(
            || OrderBook::new(),
            |mut book| {
                for i in 0..50000 {
                    black_box(book.matching_order(OrderBook::make_order_request(
                        100 + (i % 50),
                        13,
                        Side::Ask,
                    )));
                }
                for _ in 0..50000 {
                    black_box(book.matching_order(OrderBook::make_order_request(
                        200,
                        13,
                        Side::Bid,
                    )));
                }
            },
            criterion::BatchSize::LargeInput,
        );
    });
}

fn bench_cancel_order(c: &mut Criterion) {
    c.bench_function("cancel 10k orders", |b| {
        b.iter_batched(
            || {
                let mut book = OrderBook::new();
                let mut ids: Vec<OrderId> = Vec::with_capacity(CANCEL_LIMIT as usize);

                for i in 0..CANCEL_LIMIT {
                    book.matching_order(
                        OrderBook::make_order_request(999999, 14, Side::Ask)
                    );
                    ids.push(OrderId(i + 1));
                }

                (book, ids)
            },
            |(mut book, ids)| {
                for id in ids {
                    black_box(book.cancel_order(id));
                }
            },
            criterion::BatchSize::LargeInput,
        );
    });
}

fn get_side(i: u64, modulo: u64) -> Side {
    if i % modulo == 0 {
        Side::Bid
    } else {
        Side::Ask
    }
}

criterion_group!(
    benches,
    bench_insert_no_match,
    bench_insert_full_match,
    bench_insert_partial_match,
    bench_deep_book_insert,
    bench_cancel_order
);
criterion_main!(benches);

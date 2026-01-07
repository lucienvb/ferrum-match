#!/bin/bash
cargo fmt && RUST_BACKTRACE=1 cargo test --test orderbook
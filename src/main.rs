#![allow(unexpected_cfgs)]

mod modes;
mod orderbook;

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::collections::BTreeMap;
use std::env;
use std::ops::ControlFlow;
use tracing::info;

use crate::modes::interactive::interactive_mode;
use crate::orderbook::types::OrderBook;

fn main() -> Result<()> {
    // tracing_subscriber::fmt().with_env_filter("debug").init();
    let args: Vec<String> = env::args().collect();

    info!("Matching engine started");

    let mut orderbook = OrderBook {
        next_seq: 0,
        order_id_counter: 1,
        bids: BTreeMap::new(),
        asks: BTreeMap::new(),
    };

    info!("Created empty orderbook");

    let mut rl = DefaultEditor::new()?;
    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("ferrum> ");
        match readline {
            Ok(line) => {
                if args.get(1).map(|s| s.as_str()) == Some("interactive") {
                    if let ControlFlow::Break(_) = interactive_mode(&mut orderbook, line.as_str()) {
                        break;
                    }
                } else {
                    println!("Command mode not implemented yet.")
                }

                rl.add_history_entry(line.as_str())?;
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    #[cfg(feature = "with-file-history")]
    rl.save_history("history.txt");
    Ok(())
}

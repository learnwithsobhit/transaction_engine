mod engine;
use crate::engine::transaction_engine::TransactionEngine;
fn main() {
    let mut engine = TransactionEngine::new();
    let res = engine.read_input();
    if let Some(e) = res.err() {
        println!("{:?}", e.to_string());
    }
}

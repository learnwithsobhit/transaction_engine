mod engine;
use crate::engine::transaction_engine::TransactionEngine;
fn main() {
    let mut engine = TransactionEngine::new();
    let _res = engine.read_input().is_ok();
}

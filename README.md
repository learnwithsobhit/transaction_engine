# transaction_engine

## for running sample csv
cargo run -- sample.csv

or 

cargo run -- sample.csv > output.csv

## assumptions --
1. withdrawal should not be performed if account is locked.
2. resolve and charge-back performed only if tx is disputed earlier.

## testing
testing is performed using sample csv and output csv generated using "cargo run -- sample.csv > output.csv".

## error handling 
no specific error is handled except input csv not provided, but each API return std::error::Error so it can utilized to propagate errors.
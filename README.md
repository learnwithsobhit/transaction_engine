# transaction_engine

## for running sample csv

cargo run -- sample.csv

or 

cargo run -- sample.csv > output.csv

## assumptions --

1. withdrawal should not be performed if account is locked.
2. resolve and chargeback performed only if tx is disputed earlier.
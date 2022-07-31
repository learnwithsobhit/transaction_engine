use std::{collections::HashMap, env, io::Error};

use serde::Deserialize;

use super::client::Client;

#[derive(Debug, Deserialize)]
pub struct Record {
    r#type: String,
    client: Option<u16>,
    tx: Option<u32>,
    amount: Option<f32>,
}

///
/// Transaction engine to process all the transactions in real time with respect to each client.
///
pub struct TransactionEngine {
    /* clients map hold the client ID and transaction metadata related to the client*/
    clients: HashMap<u16, Client>,
}

impl TransactionEngine {
    pub fn new() -> Self {
        TransactionEngine {
            clients: HashMap::default(),
        }
    }

    pub fn process_transactions(&mut self, record: Record) -> Result<(), Error> {
        let transaction_type = record.r#type;
        let client_id = record.client.unwrap();
        let tx_id = record.tx.unwrap();
        let amount = record.amount.unwrap_or_default();
        let transaction_type = transaction_type.into();
        if let std::collections::hash_map::Entry::Vacant(e) = self.clients.entry(client_id) {
            let client = Client::new(client_id, tx_id, transaction_type, amount);
            e.insert(client);
        } else {
            self.clients
                .get_mut(&client_id)
                .unwrap()
                .execute_transaction(tx_id, transaction_type, amount)?;
        }
        Ok(())
    }

    pub fn read_input(&mut self) -> Result<(), Error> {
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            let mut rdr = csv::Reader::from_path(&args[1])?;
            let _header = rdr.headers()?;
            for result in rdr.deserialize() {
                let record: Record = result?;
                self.process_transactions(record)?;
            }
            self.display_result();
        } else {
            println!("input csv file not found!");
        }
        Ok(())
    }

    pub fn display_result(&mut self) {
        println!("client,available,held,total,locked");
        for client in self.clients.values_mut() {
            client.show_info();
        }
    }
}

impl Default for TransactionEngine {
    fn default() -> Self {
        Self::new()
    }
}

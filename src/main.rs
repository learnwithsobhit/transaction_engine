use serde::Deserialize;
use std::{collections::HashMap, env, error::Error};

///
/// transaction type
///
#[derive(Debug, Deserialize, PartialEq)]
pub enum TransactionType {
    Deposit = 0,
    Withdrawal = 1,
    Dispute = 2,
    Resolve = 3,
    Chargeback = 4,
}

impl From<String> for TransactionType {
    fn from(value: String) -> Self {
        if value == "deposit" {
            TransactionType::Deposit
        } else if value == "withdrawal" {
            TransactionType::Withdrawal
        } else if value == "resolve" {
            TransactionType::Resolve
        } else if value == "chargeback" {
            TransactionType::Chargeback
        } else {
            TransactionType::Dispute
        }
    }
}

///
/// records to be fetched from input csv
/// 
#[derive(Debug, Deserialize)]
pub struct Record {
    r#type: String,
    client: Option<u16>,
    tx: Option<u32>,
    amount: Option<f32>,
}

///
/// Client struct which hold required information for the client
///
pub struct Client {
    /*id of the client*/
    _id: u16,
    /*The total funds that are available for trading, staking, withdrawal, etc. This should be equal to the total - held amounts*/
    available: f32,
    /*The total funds that are held for dispute. This should be equal to total - available amounts*/
    held: f32,
    /*The total funds that are available or held. This should be equal to available + held*/
    total: f64,
    /*Whether the account is locked. An account is locked if a charge back occurs*/
    locked: bool,
    /* all transaction performed by this client needed in case of dispute/resolved/charge-back transaction id : (transaction type, amount)*/
    transactions: HashMap<u32, (TransactionType, f32)>,
}

impl Client {
    ///
    /// create new client instance
    /// 
    pub fn new(id: u16, tx_id: u32, transaction_type: TransactionType, amount: f32) -> Self {
        let total = match transaction_type {
            TransactionType::Deposit => amount as f64,
            _ => 0f64,
        };

        let mut transaction = HashMap::new();
        transaction.insert(tx_id, (transaction_type, amount));
        Client {
            _id: id,
            available: amount,
            held: 0f32,
            total,
            locked: false,
            transactions: transaction,
        }
    }

    ///
    /// perform deposit operation
    /// 
    fn perform_deposit(
        &mut self,
        tx_id: u32,
        tx_type: TransactionType,
        amount: f32,
    ) -> Result<(), Box<dyn Error>> {
        self.available += amount;
        self.total = (self.available + self.held) as f64;
        self.transactions.insert(tx_id, (tx_type, amount));
        Ok(())
    }

    ///
    /// perform withdrawal operation
    /// if account is locked withdrawal can't be performed only deposit can be done
    ///
    fn perform_withdrawal(
        &mut self,
        tx_id: u32,
        tx_type: TransactionType,
        amount: f32,
    ) -> Result<(), Box<dyn Error>> {
        if self.available > amount && !self.locked {
            self.available -= amount;
            self.total = (self.available + self.held) as f64;
            self.transactions.insert(tx_id, (tx_type, amount));
        }
        Ok(())
    }

    ///
    /// perform dispute operation
    /// 
    fn perform_dispute(&mut self, tx_id: u32) -> Result<(), Box<dyn Error>> {
        if self.transactions.contains_key(&tx_id) {
            let tx = self.transactions.get(&tx_id).unwrap();
            let amount = tx.1;
            self.available -= tx.1;
            self.held += tx.1;
            self.transactions
                .insert(tx_id, (TransactionType::Dispute, amount));
        }
        Ok(())
    }

    ///
    /// perform resolve operation
    /// resolve should be applied for disputed and non frozen transactions
    ///
    fn perform_resolve(&mut self, tx_id: u32) -> Result<(), Box<dyn Error>> {
        if self.transactions.contains_key(&tx_id) {
            let tx = self.transactions.get(&tx_id).unwrap();
            let amount = tx.1;
            if tx.0 == TransactionType::Dispute && !self.locked {
                self.available += tx.1;
                self.held -= tx.1;
                self.transactions
                    .insert(tx_id, (TransactionType::Resolve, amount));
            }
        }
        Ok(())
    }

    ///
    /// perform chargeback operation
    /// Chargeback should be applied after resolved
    ///
    fn perform_chargeback(&mut self, tx_id: u32) -> Result<(), Box<dyn Error>> {
        if self.transactions.contains_key(&tx_id) {
            let tx = self.transactions.get(&tx_id).unwrap();
            let amount = tx.1;
            if tx.0 == TransactionType::Dispute {
                self.held -= tx.1;
                self.total = (self.available + self.held) as f64;
                self.locked = true;
                self.transactions
                    .insert(tx_id, (TransactionType::Chargeback, amount));
            }
        }
        Ok(())
    }

    ///
    /// execute each transaction based on input csv
    /// 
    pub fn execute_transaction(
        &mut self,
        tx_id: u32,
        transaction_type: TransactionType,
        amount: f32,
    ) -> Result<(), Box<dyn Error>> {
        match transaction_type {
            TransactionType::Deposit => self.perform_deposit(tx_id, transaction_type, amount)?,
            TransactionType::Withdrawal => {
                self.perform_withdrawal(tx_id, transaction_type, amount)?
            }
            TransactionType::Dispute => self.perform_dispute(tx_id)?,
            TransactionType::Resolve => self.perform_resolve(tx_id)?,
            TransactionType::Chargeback => self.perform_chargeback(tx_id)?,
        };
        Ok(())
    }
}

///
/// Transaction engine to process all the transactions in real time with respect to each client.
///
pub struct TransactionEngine {
    /* clients map hold the client ID and transaction metadata related to the client*/
    clients: HashMap<u16, Client>,
}

impl TransactionEngine {
    ///
    /// create new engine
    /// 
    pub fn new() -> Self {
        TransactionEngine {
            clients: HashMap::default(),
        }
    }

    ///
    /// process csv inputs
    /// 
    pub fn process_transactions(&mut self, record: Record) -> Result<(), Box<dyn Error>> {
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

    ///
    /// read input transaction csv
    /// 
    pub fn read_input(&mut self) -> Result<(), Box<dyn Error>> {
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
            return Err(From::from("input csv file not found!"));
        }
        Ok(())
    }

    ///
    /// display results
    /// 
    pub fn display_result(&mut self) {
        println!("client,available,held,total,locked");
        for client in self.clients.values() {
            let info = std::format!(
                "{},{},{},{},{}",
                client._id,
                client.available,
                client.held,
                client.total,
                client.locked
            );
            println!("{:?}", info);
        }
    }
}

impl Default for TransactionEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let mut engine = TransactionEngine::new();
    let res = engine.read_input();
    if let Some(e) = res.err() {
        println!("{:?}", e.to_string());
    }
}

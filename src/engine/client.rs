use serde::Deserialize;
use std::{collections::HashMap, io::Error};

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

    fn perform_deposit(
        &mut self,
        tx_id: u32,
        tx_type: TransactionType,
        amount: f32,
    ) -> Result<(), Error> {
        self.available += amount;
        self.total = (self.available + self.held) as f64;
        self.transactions.insert(tx_id, (tx_type, amount));
        Ok(())
    }

    ///
    /// if account is locked withdrawal can't be performed only deposit can be done
    ///
    fn perform_withdrawal(
        &mut self,
        tx_id: u32,
        tx_type: TransactionType,
        amount: f32,
    ) -> Result<(), Error> {
        if self.available > amount && !self.locked {
            self.available -= amount;
            self.total = (self.available + self.held) as f64;
            self.transactions.insert(tx_id, (tx_type, amount));
        }
        Ok(())
    }

    fn perform_dispute(&mut self, tx_id: u32) -> Result<(), Error> {
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
    /// resolve should be applied for disputed and non frozen transactions
    ///
    fn perform_resolve(&mut self, tx_id: u32) -> Result<(), Error> {
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
    /// Chargeback should be applied after resolved
    ///
    fn perform_chargeback(&mut self, tx_id: u32) -> Result<(), Error> {
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

    pub fn execute_transaction(
        &mut self,
        tx_id: u32,
        transaction_type: TransactionType,
        amount: f32,
    ) -> Result<(), Error> {
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

    pub fn show_info(&mut self) {
        print!("{},", self._id);
        print!("{},", self.available);
        print!("{},", self.held);
        print!("{},", self.total);
        print!("{}", self.locked);
    }
}

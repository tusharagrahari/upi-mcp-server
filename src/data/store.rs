use crate::model::Transaction;
use std::{fs::File, io::BufReader};

#[derive(Debug)]
pub struct TransactionStore {
    pub transactions: Vec<Transaction>,
}

impl TransactionStore {
    fn new() -> Self {
        TransactionStore {
            transactions: Vec::new(),
        }
    }

    // this would load from a database or file
    pub fn load() -> anyhow::Result<Self> {
        let mut t = TransactionStore::new();
        // Open the file in read-only mode with buffer.
        let file = File::open("data/transaction.json")?;
        let reader = BufReader::new(file);

        let transactions: Vec<Transaction> = serde_json::from_reader(reader)?;
        t.transactions = transactions;
        Ok(t)
    }
}

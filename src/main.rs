use crate::data::store::TransactionStore;

pub mod data;
pub mod model;

fn main() {
    let _store = match TransactionStore::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error loading transactions: {:?}", e);
            return;
        }
    };
    // println!("Loaded transactions: {:?}", store.transactions);
}

use chrono::DateTime;

use crate::model::{Category, Transaction};

#[derive(Debug)]
pub struct TransactionStore {
    pub transactions: Vec<Transaction>,
}

impl TransactionStore {
    pub fn new() -> Self {
        TransactionStore {
            transactions: Vec::new(),
        }
    }
    // this would load from a database or file
    pub fn load() -> anyhow::Result<Self> {
        let data = include_str!("../../data/transaction.json");
        let t: Vec<Transaction> = serde_json::from_str(data)?;
        Ok(TransactionStore { transactions: t })
    }

    pub fn filter_by_category(&self, category: Category) -> Vec<&Transaction> {
        let t = self
            .transactions
            .iter()
            .filter(|t| t.category == category)
            .collect();
        t
    }

    pub fn filter_by_merchant(&self, merchant: &str) -> Vec<&Transaction> {
        self.transactions
            .iter()
            .filter(|t| {
                t.merchant_name.as_deref().map(|m| m.to_lowercase())
                    == Some(merchant.to_lowercase())
            })
            .collect()
    }

    pub fn filter_by_date_range(
        &self,
        start: DateTime<chrono::Utc>,
        end: DateTime<chrono::Utc>,
    ) -> Vec<&Transaction> {
        self.transactions
            .iter()
            .filter(|t| t.timestamp >= start && t.timestamp <= end)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_load_transactions() {
        let store = TransactionStore::load();
        assert!(store.is_ok());
        let store = store.unwrap();
        assert!(!store.transactions.is_empty());
        println!("Loaded transactions: {:?}", store.transactions);
    }

    #[test]
    fn test_filter_by_category() {
        let store = TransactionStore::load().unwrap();
        let food_transactions = store.filter_by_category(Category::Food);
        assert!(!food_transactions.is_empty());
        for t in food_transactions.clone() {
            assert_eq!(t.category, crate::model::Category::Food);
        }
        println!("Food transactions: {:?}", food_transactions);
    }

    #[test]
    fn test_filter_by_merchant() {
        let store = TransactionStore::load().unwrap();
        let merchant_transactions = store.filter_by_merchant("Zomato");
        assert!(!merchant_transactions.is_empty());
        for t in merchant_transactions.clone() {
            assert_eq!(t.merchant_name.as_deref(), Some("Zomato"));
        }
        println!("Zomato transactions: {:?}", merchant_transactions);
    }

    #[test]
    fn test_filter_by_date_range() {
        let store = TransactionStore::load().unwrap();
        let start = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 30, 12, 0, 0).unwrap();
        let recent_transactions = store.filter_by_date_range(start, end);
        assert!(!recent_transactions.is_empty());
        for t in recent_transactions.clone() {
            assert!(t.timestamp >= start && t.timestamp <= end);
        }
        println!(
            "Transactions from Jan 15 to Jan 30: {:?}",
            recent_transactions
        );
    }
}

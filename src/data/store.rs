use chrono::{DateTime, NaiveDate, Utc};

use crate::model::{self, Category, Transaction};

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

    pub fn filter_combined(
        &self,
        start_date: Option<String>,
        end_date: Option<String>,
        min_amount: Option<f64>,
        max_amount: Option<f64>,
        merchant_name: Option<String>,
        category: Option<String>,
    ) -> Result<Vec<&Transaction>, String> {
        let mut cate = true;
        let results = self
            .transactions
            .iter()
            .filter(|txn| {
                let mut matches = true;
                if let Some(ref start) = start_date {
                    if let Ok(start_dt) = NaiveDate::parse_from_str(start, "%Y-%m-%d")
                        .or_else(|_| {
                            NaiveDate::parse_from_str(&format!("{}-01", start), "%Y-%m-%d")
                        })
                        .and_then(|d| Ok(d.and_hms_opt(0, 0, 0).unwrap().and_utc()))
                    {
                        if txn.timestamp < start_dt.with_timezone(&Utc) {
                            matches = false;
                        }
                    }
                }
                if let Some(ref end) = end_date {
                    if let Ok(end_dt) = NaiveDate::parse_from_str(end, "%Y-%m-%d")
                        // For year-month input (e.g. "2024-01"), we want the last day of that month as the end date.
                        // Strategy: parse as 1st of month → add 1 month → subtract 1 day = last day of month.
                        .or_else(|_| {
                            NaiveDate::parse_from_str(&format!("{}-01", end), "%Y-%m-%d").map(|d| {
                                d.checked_add_months(chrono::Months::new(1))
                                    .unwrap()
                                    .pred_opt()
                                    .unwrap()
                            })
                        })
                        .and_then(|d| Ok(d.and_hms_opt(23, 59, 59).unwrap().and_utc()))
                    {
                        if txn.timestamp > end_dt.with_timezone(&Utc) {
                            matches = false;
                        }
                    }
                }
                if let Some(min) = min_amount {
                    if txn.amount < min {
                        matches = false;
                    }
                }
                if let Some(max) = max_amount {
                    if txn.amount > max {
                        matches = false;
                    }
                }
                if let Some(ref merchant) = merchant_name {
                    let temp = txn.merchant_name.as_deref().map(|m| m.to_lowercase());
                    if let Some(temp) = temp {
                        if !temp.contains(merchant.to_lowercase().as_str()) {
                            matches = false;
                        }
                    } else {
                        matches = false;
                    }
                }
                if let Some(ref cat) = category {
                    let parsed_cat = model::Category::from_str(cat);
                    if parsed_cat.is_none() {
                        cate = false;
                        matches = false;
                    } else if txn.category != parsed_cat.unwrap() {
                        matches = false;
                    }
                }
                matches
            })
            .collect::<Vec<_>>();
        if cate {
            Ok(results)
        } else {
            Err("Invalid category. Valid values: Food, Grocery, Utilities, Entertainment, Transportation, Healthcare, Rental, Salary, Investment, Other.".to_string())
        }
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

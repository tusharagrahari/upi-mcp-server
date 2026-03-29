use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Datelike, NaiveDate, Utc};

use crate::model::{
    self, Category, CategoryBreakdown, MerchantInsight, RecurringTransaction, Transaction,
    TransactionType,
};

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
    pub fn load() -> Result<Self, serde_json::Error> {
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

    pub fn aggregate_by_category(
        &self,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Vec<CategoryBreakdown> {
        let filtered = self
            .filter_combined(start_date, end_date, None, None, None, None)
            .expect("category is None, this cannot fail");
        let mut total_spent = 0.0;
        let mut agg: HashMap<String, (f64, u32)> = HashMap::new();
        for txn in filtered {
            if txn.transaction_type == TransactionType::Credit {
                continue; // Skip credits for spending breakdown
            }
            let entry = agg.entry(format!("{:?}", txn.category)).or_insert((0.0, 0));
            entry.0 += txn.amount;
            entry.1 += 1;
            total_spent += txn.amount;
        }
        // let mut result: Vec<(String, f64, u32)> = agg.into_iter().map(|(cat, (amount, count))| (cat, amount, count)).collect();
        let mut result: Vec<CategoryBreakdown> = agg
            .into_iter()
            .map(|(cat, (amount, count))| CategoryBreakdown {
                category: cat,
                total_amount: amount,
                transaction_count: count,
                percentage_of_total: if total_spent > 0.0 {
                    ((amount / total_spent) * 10000.0).round() / 100.0 // Round to 2 decimal places
                } else {
                    0.0
                },
            })
            .collect();
        result.sort_by(|a, b| b.total_amount.partial_cmp(&a.total_amount).unwrap());
        result
    }

    pub fn recurring_transactions(&self) -> Vec<RecurringTransaction> {
        // Anchor = month of the latest transaction in the store
        let anchor = match self
            .transactions
            .iter()
            .map(|t| t.timestamp.date_naive())
            .max()
        {
            Some(d) => d,
            None => return vec![],
        };

        // Window: 3 months before anchor month, excluding anchor month itself
        let anchor_first = NaiveDate::from_ymd_opt(anchor.year(), anchor.month(), 1)
            .expect("anchor date is always valid");
        let window_end = anchor_first.pred_opt().expect("anchor is never year 0");
        let window_start = anchor_first
            .checked_sub_months(chrono::Months::new(3))
            .expect("anchor month is never within 3 months of year 0");

        // Group debit transactions in the window by (merchant_name, amount_in_paise)
        // Skip transactions with no merchant name
        let mut groups: HashMap<(String, i64), Vec<(NaiveDate, &Transaction)>> = HashMap::new();
        for txn in &self.transactions {
            if txn.transaction_type != TransactionType::Debit {
                continue;
            }
            let merchant = match &txn.merchant_name {
                Some(m) => m.clone(),
                None => continue,
            };
            let date = txn.timestamp.date_naive();
            if date < window_start || date > window_end {
                continue;
            }
            let paise = (txn.amount * 100.0).round() as i64;
            groups
                .entry((merchant, paise))
                .or_default()
                .push((date, txn));
        }

        let mut result = Vec::new();
        for ((merchant, _paise), mut entries) in groups {
            // Must appear exactly once in each of the 3 months
            if entries.len() != 3 {
                continue;
            }
            let unique_months: HashSet<(i32, u32)> =
                entries.iter().map(|(d, _)| (d.year(), d.month())).collect();
            if unique_months.len() != 3 {
                continue;
            }

            // Sort by date, then check both intervals are within 28–32 days
            entries.sort_by_key(|(d, _)| *d);
            let interval1 = (entries[1].0 - entries[0].0).num_days();
            let interval2 = (entries[2].0 - entries[1].0).num_days();
            if !(28..=32).contains(&interval1) || !(28..=32).contains(&interval2) {
                continue;
            }

            result.push(RecurringTransaction {
                merchant_name: merchant,
                amount: entries[0].1.amount,
                category: entries[0].1.category.clone(),
            });
        }
        result
    }

    pub fn get_merchant_insights(
        &self,
        merchant: String,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<MerchantInsight, String> {
        let merchant_txns: Vec<&Transaction> = self
            .filter_combined(
                start_date,
                end_date,
                None,
                None,
                Some(merchant.clone()),
                None,
            )
            .expect("category not passed, this cannot fail");

        if merchant_txns.is_empty() {
            return Err("No transactions found for the specified merchant".to_string());
        }

        let (total_spent, debit_count): (f64, u32) =
            merchant_txns
                .iter()
                .fold((0.0_f64, 0_u32), |(sum, count), txn| {
                    if txn.transaction_type == TransactionType::Debit {
                        (sum + txn.amount, count + 1)
                    } else {
                        (sum, count)
                    }
                });

        let insight = MerchantInsight {
            merchant_name: merchant,
            total_amount: total_spent,
            transaction_count: debit_count,
            average_amount: if debit_count > 0 {
                total_spent / (debit_count as f64)
            } else {
                0.0
            },
        };
        Ok(insight)
    }

    pub fn get_top_merchants(
        &self,
        start_date: Option<String>,
        end_date: Option<String>,
        top_n: Option<usize>,
        sort_by: Option<String>,
    ) -> Vec<MerchantInsight> {
        let filtered = self
            .filter_combined(start_date, end_date, None, None, None, None)
            .expect("category is None, this cannot fail");
        let mut agg: HashMap<String, (f64, u32)> = HashMap::new();
        for txn in filtered {
            if txn.transaction_type == TransactionType::Credit {
                continue; // Skip credits for merchant insights
            }
            let merchant = match &txn.merchant_name {
                Some(m) => m.clone(),
                None => continue,
            };
            let entry = agg.entry(merchant).or_insert((0.0, 0));
            entry.0 += txn.amount;
            entry.1 += 1;
        }
        let mut insights: Vec<MerchantInsight> = agg
            .into_iter()
            .map(
                |(merchant_name, (total_amount, transaction_count))| MerchantInsight {
                    merchant_name,
                    total_amount,
                    transaction_count,
                    average_amount: if transaction_count > 0 {
                        total_amount / (transaction_count as f64)
                    } else {
                        0.0
                    },
                },
            )
            .collect();
        if let Some(sort_by) = sort_by {
            match sort_by.to_lowercase().as_str() {
                "total_amount" | "amount" | "spend" => {
                    insights.sort_by(|a, b| b.total_amount.partial_cmp(&a.total_amount).unwrap())
                }
                "transaction_count" | "count" => {
                    insights.sort_by(|a, b| b.transaction_count.cmp(&a.transaction_count))
                }
                "average_amount" | "average" => insights
                    .sort_by(|a, b| b.average_amount.partial_cmp(&a.average_amount).unwrap()),
                _ => insights.sort_by(|a, b| b.total_amount.partial_cmp(&a.total_amount).unwrap()),
            };
        } else {
            insights.sort_by(|a, b| b.total_amount.partial_cmp(&a.total_amount).unwrap());
        }
        if let Some(n) = top_n {
            insights.truncate(n);
        }
        insights
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

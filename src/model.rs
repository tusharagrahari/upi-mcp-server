use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub amount: f64,
    pub timestamp: DateTime<Utc>,
    pub description: Option<String>,
    pub category: Category,
    pub upi_id: String,
    pub transaction_type: TransactionType,
    pub merchant_name: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Food,
    Utilities,
    Entertainment,
    Transportation,
    Healthcare,
    Rental,
    Salary,
    Investment,
    Other,
    Grocery,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Credit,
    Debit,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
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

#[derive(Debug, serde::Serialize)]
pub struct CategoryBreakdown {
    pub category: String,
    pub total_amount: f64,
    pub transaction_count: u32,
    pub percentage_of_total: f64,
}

impl Category {
    pub fn from_str(s: &str) -> Option<Category> {
        match s.to_lowercase().as_str() {
            "food" => Some(Category::Food),
            "grocery" => Some(Category::Grocery),
            "utilities" => Some(Category::Utilities),
            "entertainment" => Some(Category::Entertainment),
            "transportation" => Some(Category::Transportation),
            "healthcare" => Some(Category::Healthcare),
            "rental" => Some(Category::Rental),
            "salary" => Some(Category::Salary),
            "investment" => Some(Category::Investment),
            "other" => Some(Category::Other),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Credit,
    Debit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecurringTransaction {
    pub merchant_name: String,
    pub amount: f64,
    pub category: Category,
}

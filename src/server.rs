use std::collections::HashMap;

use crate::{data::store::TransactionStore, model};
use rmcp::{
    ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};

/*
Summary of the flow:
  startup → tool_router built from #[tool] methods → stored on struct
  runtime → Claude calls tool → ServerHandler receives it
         → tool_handler delegates to tool_router
         → tool_router finds the right method → calls it → returns result
 */

#[derive(Debug)]
pub struct UpiServer {
    store: TransactionStore,
    tool_router: ToolRouter<Self>, //This router is a lookup table — it maps tool names ("search_transactions") to the actual method that handles them.
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchTxnRequest {
    // Define your search parameters here, e.g.:
    #[schemars(
        description = "Search for transactions that occurred on or after this date. Format: YYYY-MM or YYYY-MM-DD"
    )]
    pub start_date: Option<String>,
    #[schemars(
        description = "Search for transactions that occurred on or before this date. Format: YYYY-MM or YYYY-MM-DD"
    )]
    pub end_date: Option<String>,
    #[schemars(
        description = "Search for transactions with amount greater than or equal to this value."
    )]
    pub min_amount: Option<f64>,
    #[schemars(
        description = "Search for transactions with amount less than or equal to this value."
    )]
    pub max_amount: Option<f64>,
    #[schemars(
        description = "Search for transactions that match this merchant name (case-insensitive)."
    )]
    pub merchant_name: Option<String>,
    #[schemars(
        description = "The category of transactions to search for. For example: 'Food', 'Transport', 'Shopping', etc. Case-insensitive."
    )]
    pub category: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SpendingBreakdownRequest {
    #[schemars(
        description = "The start date for the spending breakdown. Format: YYYY-MM or YYYY-MM-DD"
    )]
    pub start_date: Option<String>,
    #[schemars(
        description = "The end date for the spending breakdown. Format: YYYY-MM or YYYY-MM-DD"
    )]
    pub end_date: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct CategoryBreakdown {
    pub category: String,
    pub total_amount: f64,
    pub transaction_count: u32,
    pub percentage_of_total: f64,
}

#[tool_router]
impl UpiServer {
    pub fn new(store: TransactionStore) -> Self {
        UpiServer {
            store,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Search UPI transactions based on various filters like date range, amount, merchant name, etc."
    )]
    fn search_txn(
        &self,
        Parameters(SearchTxnRequest {
            start_date,
            end_date,
            min_amount,
            max_amount,
            merchant_name,
            category,
        }): Parameters<SearchTxnRequest>,
    ) -> String {
        let results = match TransactionStore::filter_combined(
            &self.store,
            start_date,
            end_date,
            min_amount,
            max_amount,
            merchant_name,
            category,
        ) {
            Ok(res) => res,
            Err(e) => return e, // Return the error message if filtering fails (e.g., due to invalid category)
        };
        serde_json::to_string(&results).unwrap_or_else(|_| "Failed to serialize".to_string())
    }

    #[tool(
        description = "Returns a breakdown of spending by category for a given time period. Shows total amount, transaction count, and percentage share per category, sorted by highest spend. Use this when the user asks about spending patterns, budget analysis, or category-wise expenses."
    )]
    fn get_spending_breakdown(
        &self,
        Parameters(SpendingBreakdownRequest {
            start_date,
            end_date,
        }): Parameters<SpendingBreakdownRequest>,
    ) -> String {
        let results = match TransactionStore::filter_combined(
            &self.store,
            start_date,
            end_date,
            None,
            None,
            None,
            None,
        ) {
            Ok(res) => res,
            Err(e) => return e, // Return the error message if filtering fails (e.g., due to invalid category)
        };
        let mut breakdown: HashMap<&model::Category, (f64, u32)> = HashMap::new();
        let mut total_spent = 0.0;
        for txn in results {
            if txn.transaction_type == model::TransactionType::Credit {
                continue; // Skip credits for spending breakdown
            }
            let entry = breakdown.entry(&txn.category).or_insert((0.0, 0));
            entry.0 += txn.amount;
            entry.1 += 1;
            total_spent += txn.amount;
        }
        let mut breakdown: Vec<CategoryBreakdown> = breakdown
            .into_iter()
            .map(|(cat, (amount, count))| CategoryBreakdown {
                category: format!("{:?}", cat),
                total_amount: amount,
                transaction_count: count,
                percentage_of_total: if total_spent > 0.0 {
                    ((amount / total_spent) * 10000.0).round() / 100.0 // Round to 2 decimal places
                } else {
                    0.0
                },
            })
            .collect();
        breakdown.sort_by(|a, b| b.total_amount.partial_cmp(&a.total_amount).unwrap());
        serde_json::to_string(&breakdown).unwrap_or_else(|_| "Failed to serialize".to_string())
    }
}

#[tool_handler] //When rmcp receives a tools/call request, ServerHandler is what handles it — and #[tool_handler] generates the implementation that delegates to your tool_router.
impl ServerHandler for UpiServer {
    fn get_info(&self) -> ServerInfo {
        //This is where you tell Claude what your server can do. By enabling tools in the capabilities, you're telling Claude that it can call the methods you've defined with #[tool].
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("A UPI transaction intelligence server. Use this to query, filter and analyze UPI transaction history.".to_string())
    }
}

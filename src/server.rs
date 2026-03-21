use std::collections::HashMap;

use crate::{data::store::TransactionStore, model::{self, CategoryBreakdown}};
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
pub struct ComparePeriodRequest {
    #[schemars(
        description = "The first period to compare. Format: YYYY-MM or YYYY-MM-DD for start and end dates."
    )]
    pub period1_start: String,
    pub period1_end: String,
    #[schemars(
        description = "The second period to compare. Format: YYYY-MM or YYYY-MM-DD for start and end dates."
    )]
    pub period2_start: String,
    pub period2_end: String,
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
        let breakdown = self.store.aggregate_by_category(start_date, end_date);
        serde_json::to_string(&breakdown).unwrap_or_else(|_| "Failed to serialize".to_string())
    }

    #[tool(
        description = "Compares spending across two arbitrary time periods, broken down by category. Returns a side-by-side delta showing total spent, transaction count, and percentage change per category for period A vs period B. Categories that appear in only one period are included with 0.0 for the other — absence of spending is meaningful data. Use this when the user asks to compare spending between two months, date ranges, or any two time windows. Do NOT use get_spending_breakdown twice and diff manually — use this tool instead." 
    )]
    fn compare_periods(
        &self,
        Parameters(ComparePeriodRequest {
            period1_start,
            period1_end,
            period2_start,
            period2_end,
        }): Parameters<ComparePeriodRequest>,
    ) -> String {
        let breakdown1 = self.store.aggregate_by_category(Some(period1_start), Some(period1_end));
        let breakdown2 = self.store.aggregate_by_category(Some(period2_start), Some(period2_end));

        // Create a map for easy lookup of categories in breakdown2
        let mut breakdown2_map: HashMap<String, CategoryBreakdown> = HashMap::new();
        for item in breakdown2 {
            breakdown2_map.insert(item.category.clone(), item);
        }

        "return".to_string() // Placeholder, implement the actual comparison logic and return a structured result as JSON string.
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

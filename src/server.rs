use std::collections::HashMap;

use crate::{data::store::TransactionStore, model::CategoryBreakdown};
use rmcp::{
    ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use serde::Serialize;

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
        description = "The start of first period to compare. Format: YYYY-MM or YYYY-MM-DD for start and end dates."
    )]
    pub period1_start: String,
    #[schemars(
        description = "The end of first period to compare. Format: YYYY-MM or YYYY-MM-DD for start and end dates."
    )]
    pub period1_end: String,
    #[schemars(
        description = "The start of second period to compare. Format: YYYY-MM or YYYY-MM-DD for start and end dates."
    )]
    pub period2_start: String,
    #[schemars(
        description = "The end of second period to compare. Format: YYYY-MM or YYYY-MM-DD for start and end dates."
    )]
    pub period2_end: String,
}

#[derive(Debug, Serialize)]
pub struct CompareResult {
    category: String,
    period1_total: f64,
    period1_count: u32,
    period2_total: f64,
    period2_count: u32,
    total_delta: f64,   // period2_total - period1_total
    count_delta: isize, // period2_count - period1_count
}

#[derive(Serialize)]
pub struct RecurringPaymentsResponse<'a> {
    recurring_payments: &'a Vec<crate::model::RecurringTransaction>,
    monthly_recurring_total: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MerchantInsightsRequest {
    #[schemars(
        description = "The merchant name to look up. Partial and case-insensitive — 'zom' will match 'Zomato'."
    )]
    pub merchant_name: String,
    #[schemars(description = "Optional start date for filtering. Format: YYYY-MM or YYYY-MM-DD")]
    pub start_date: Option<String>,
    #[schemars(description = "Optional end date for filtering. Format: YYYY-MM or YYYY-MM-DD")]
    pub end_date: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TopMerchantsRequest {
    #[schemars(
        description = "Sort merchants by: 'spend' (total amount), 'count' (transaction count), or 'average' (average transaction amount). Defaults to 'spend' if not provided."
    )]
    pub sort_by: Option<String>,
    #[schemars(
        description = "Limit results to top N merchants. Returns all merchants if not provided."
    )]
    pub top_n: Option<usize>,
    #[schemars(description = "Optional start date for filtering. Format: YYYY-MM or YYYY-MM-DD")]
    pub start_date: Option<String>,
    #[schemars(description = "Optional end date for filtering. Format: YYYY-MM or YYYY-MM-DD")]
    pub end_date: Option<String>,
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
        description = "Compares spending across two arbitrary time periods, broken down by category. Returns a side-by-side delta showing total spent, transaction count, per category for period A vs period B. Categories that appear in only one period are included with 0.0 for the other — absence of spending is meaningful data. Use this when the user asks to compare spending between two months, date ranges, or any two time windows. Do NOT use get_spending_breakdown twice and diff manually — use this tool instead."
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
        let breakdown1 = self
            .store
            .aggregate_by_category(Some(period1_start), Some(period1_end));
        let breakdown2 = self
            .store
            .aggregate_by_category(Some(period2_start), Some(period2_end));

        // Create a map for easy lookup of categories in breakdown2
        let mut breakdown2_map: HashMap<String, (CategoryBreakdown, bool)> = HashMap::new(); // The bool is a flag to indicate if the category was matched with breakdown1
        for item in breakdown2 {
            breakdown2_map.insert(item.category.clone(), (item, false));
        }

        let mut comparison_results: Vec<CompareResult> = Vec::new();

        for item in breakdown1.iter() {
            if let Some((breakdown2_item, matched)) = breakdown2_map.get_mut(&item.category) {
                // Category exists in both periods, calculate deltas
                let total_delta = breakdown2_item.total_amount - item.total_amount;
                let count_delta =
                    breakdown2_item.transaction_count as isize - item.transaction_count as isize;

                let result = CompareResult {
                    category: item.category.clone(),
                    period1_total: item.total_amount,
                    period1_count: item.transaction_count,
                    period2_total: breakdown2_item.total_amount,
                    period2_count: breakdown2_item.transaction_count,
                    total_delta,
                    count_delta,
                };
                comparison_results.push(result);
                *matched = true; // Update breakdown2_item with the deltas and mark it as matched
            } else {
                // Category exists only in period1, add it to the map with 0 values for period2
                let result = CompareResult {
                    category: item.category.clone(),
                    period1_total: item.total_amount,
                    period1_count: item.transaction_count,
                    period2_total: 0.0,
                    period2_count: 0,
                    total_delta: -item.total_amount, // Since period2 is 0, the delta is negative of period1
                    count_delta: -(item.transaction_count as isize), // Similarly for count
                };
                comparison_results.push(result);
            }
        }

        // Now add categories that exist only in period2
        for (category, (breakdown2_item, matched)) in breakdown2_map.iter() {
            if !*matched {
                let result = CompareResult {
                    category: category.clone(),
                    period1_total: 0.0,
                    period1_count: 0,
                    period2_total: breakdown2_item.total_amount,
                    period2_count: breakdown2_item.transaction_count,
                    total_delta: breakdown2_item.total_amount, // Since period1 is 0, the delta is just the period2 total
                    count_delta: breakdown2_item.transaction_count as isize, // Similarly for count
                };
                comparison_results.push(result);
            }
        }

        serde_json::to_string(&comparison_results)
            .unwrap_or_else(|_| "Failed to serialize".to_string())
    }

    #[tool(
        description = "Detects recurring payments (subscriptions, rent, utilities) in the transaction history. Requires no input — automatically scans the 3 months prior to the latest transaction date. A payment is considered recurring if the same merchant charged the same amount exactly once per month with 28–32 day intervals. Returns a flat list of recurring payments and a monthly_recurring_total. Use this when the user asks about subscriptions, recurring charges, or what they pay every month."
    )]
    fn detect_recurring_payments(&self) -> String {
        let recurring = self.store.recurring_transactions();
        let monthly_recurring_total: f64 = recurring.iter().map(|r| r.amount).sum();

        let response = RecurringPaymentsResponse {
            recurring_payments: &recurring,
            monthly_recurring_total,
        };
        serde_json::to_string(&response).unwrap_or_else(|_| "Failed to serialize".to_string())
    }

    #[tool(
        description = "Returns total spent, transaction count, and average transaction amount for a merchant. Merchant name matching is partial and case-insensitive — all matching merchants are aggregated into a single result. Returns an error message if no transactions are found for the given merchant. Use this when the user asks how much they spent at a specific store or vendor."
    )]
    pub fn get_merchant_insights(
        &self,
        Parameters(MerchantInsightsRequest {
            merchant_name,
            start_date,
            end_date,
        }): Parameters<MerchantInsightsRequest>,
    ) -> String {
        match self
            .store
            .get_merchant_insights(merchant_name, start_date, end_date)
        {
            Ok(insight) => serde_json::to_string(&insight)
                .unwrap_or_else(|_| "Failed to serialize".to_string()),
            Err(e) => e,
        }
    }

    #[tool(
        description = "Returns all merchants ranked by a chosen metric. Optionally accepts sort_by: 'spend' (total amount), 'count' (transaction count), or 'average' (average transaction amount) — defaults to 'spend' if not provided. Optionally accepts top_n to limit results. Use this when the user asks which merchants they spend the most at, their most frequent vendors, or wants a ranked list of merchants."
    )]
    pub fn get_top_merchants(
        &self,
        Parameters(TopMerchantsRequest {
            sort_by,
            top_n,
            start_date,
            end_date,
        }): Parameters<TopMerchantsRequest>,
    ) -> String {
        let insights = self
            .store
            .get_top_merchants(start_date, end_date, top_n, sort_by);
        serde_json::to_string(&insights).unwrap_or_else(|_| "Failed to serialize".to_string())
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

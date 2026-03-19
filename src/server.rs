use crate::{data::store::TransactionStore, model};
use chrono::{NaiveDate, Utc};
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
        let mut cate = None;
        let results = self
            .store
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
                        cate = Some(cat);
                        matches = false;
                    } else if txn.category != parsed_cat.unwrap() {
                        matches = false;
                    }
                }
                matches
            })
            .collect::<Vec<_>>();
        if cate.is_some() {
            return "Invalid category. Valid values: Food, Grocery, Utilities, Entertainment, Transportation, Healthcare, Rental, Salary, Investment, Other.".to_string();
        }
        serde_json::to_string(&results).unwrap_or_else(|_| "Failed to serialize".to_string())
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

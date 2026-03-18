use crate::data::store::TransactionStore;
use rmcp::{
    ServerHandler,
    handler::server::tool::ToolRouter,
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

#[tool_router]
impl UpiServer {
    pub fn new(store: TransactionStore) -> Self {
        UpiServer {
            store,
            tool_router: ToolRouter::new(),
        }
    }

    #[tool(
        description = "Search UPI transactions based on various filters like date range, amount, merchant name, etc."
    )]
    fn search_txn(&self) -> String {
        "This is a placeholder for the search_txn tool".to_string()
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

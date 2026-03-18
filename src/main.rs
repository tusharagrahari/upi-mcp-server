use crate::data::store::TransactionStore;
mod server;
use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use server::UpiServer;
pub mod data;
pub mod model;

#[tokio::main]
async fn main() -> Result<()> {
    let store = match TransactionStore::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error loading transactions: {:?}", e);
            TransactionStore::new() // Start with an empty store if loading fails
        }
    };

    let service = UpiServer::new(store)
        .serve(stdio())
        .await
        .inspect_err(|e| {
            eprintln!("serving error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}

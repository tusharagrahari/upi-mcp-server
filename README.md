# UPI MCP Server

A UPI transaction intelligence server built with Rust and the [Model Context Protocol (MCP)](https://modelcontextprotocol.io). Exposes transaction analytics as tools that Claude can call directly — search, aggregate, detect recurring payments, and rank merchants from natural language queries.

## What it does

Instead of dumping raw transaction data into Claude's context, this server exposes purpose-built tools. Claude routes queries to the right tool, gets back pre-computed results, and answers accurately without doing arithmetic in-context.

## Tools

| Tool | Description |
|------|-------------|
| `search_txn` | Filter raw transactions by date, amount, merchant, category |
| `get_spending_breakdown` | Category-wise spend breakdown for a date range |
| `compare_periods` | Side-by-side delta between two time periods by category |
| `detect_recurring_payments` | Detect monthly subscriptions using interval analysis |
| `get_merchant_insights` | Total spent, count, average for a specific merchant |
| `get_top_merchants` | All merchants ranked by spend, count, or average |

## Stack

- **Rust** — systems language, zero-cost abstractions
- **rmcp** — Rust MCP SDK for tool registration and stdio transport
- **chrono** — date/time handling
- **serde / serde_json** — serialization
- **tokio** — async runtime

## Running locally

```bash
cargo build
cargo run        # starts the server over stdio
cargo check      # fast type-check
cargo clippy     # lint
cargo test       # run unit tests
```

Connect via **Claude Desktop** — the binary communicates over stdio, Claude Desktop spawns and manages the process.

## Architecture

```
src/
  main.rs        # entry: loads TransactionStore, starts stdio MCP server
  server.rs      # UpiServer: tool definitions (#[tool] methods) + ToolRouter
  model.rs       # Transaction, Category, TransactionType, response structs
  data/
    store.rs     # TransactionStore: data loading, filter helpers, analytics methods
data/
  transaction.json  # 202 mock transactions, Jan–Apr 2024 (compiled into binary)
```

Data is embedded at compile time via `include_str!` — no runtime file path needed.

## Design decisions

**Why dedicated tools instead of one generic query tool?**
Claude is a language model, not a calculator. Aggregating 200 transactions and computing percentage deltas in-context will drift. Code-side computation is exact, deterministic, and token-efficient.

**Why `compare_periods` instead of calling `get_spending_breakdown` twice?**
One tool call vs two stdio round-trips. Pre-diffed output with `change_pct` and `total_delta` fields Claude can't compute reliably itself.

**Why interval-based recurring detection instead of day-of-month matching?**
Day-of-month breaks in February and across month boundaries. Checking that consecutive occurrences are 28–32 days apart is robust to subscription drift.

## Known limitations

- Mock data only — no database backend
- `detect_recurring_payments` window is derived from the latest transaction date, not system time
- Category keys use `Debug` formatting — coupled to enum implementation detail

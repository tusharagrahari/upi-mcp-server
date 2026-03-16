# UPI Intelligence Layer — MCP Server in Rust
### A 2-Week Build Roadmap

---

## The Idea

Every UPI user has months of rich transaction data — where they spend, how much, how often — but it's locked behind static bank app screens and scrollable lists. There's no way to just ask *"am I spending more on food than last month?"* and get a straight answer.

This project fixes that.

A **Rust-based MCP server** sits on top of UPI transaction data and exposes it as structured tools that any MCP-compatible AI agent (like Claude Desktop) can call. The user chats in plain English or Hindi. Claude figures out which tool to call, calls the server, gets structured data back, and responds conversationally.

**You don't build any UI. Claude is the UI.**

---

## The NPCI Angle

NPCI sees both sides of every UPI transaction across all banks. No single bank has that complete picture. This MCP server — if connected to real UPI data with user consent — becomes a *"chat with your UPI data"* layer, turning India's payment rail into a personal intelligence layer. No financial advice, no regulatory risk. Just making users' own data accessible through natural conversation.

---

## Tools Exposed by the MCP Server

| Tool | What it does |
|------|-------------|
| `search_transactions` | Filter by merchant, category, date range, amount |
| `get_spending_breakdown` | Category-wise spending for a given period |
| `compare_month_over_month` | Track spending trends over time |
| `detect_recurring_payments` | Identify salary, rent, EMIs, subscriptions |
| `get_merchant_insights` | Deep dive into spending at a specific merchant |

---

## MVP Finish Line

By Day 14, you should be able to open Claude Desktop and type:

> *"How much did I spend on food last month compared to this month?"*

...and get a real, structured, conversational answer from your server.

---

## Time Budget

| Period | Hours/Day | Days | Total |
|--------|-----------|------|-------|
| Weekdays | 2 hrs | 10 | 20 hrs |
| Weekends | 4 hrs | 4 | 16 hrs |
| **Total** | | | **~36 hrs** |

---

## Week 1 — Foundations & Data Layer

### Day 1 · Weekday · 2 hrs — Understand MCP Deeply
Don't touch code yet. Read first.

- Read the [MCP specification overview](https://modelcontextprotocol.io/introduction) — focus on the **Tools** primitive, skip Resources and Prompts for now
- Read [how Claude calls tools](https://docs.anthropic.com/en/docs/build-with-claude/tool-use) — understand the request/response shape
- Skim the [`rmcp` crate README](https://github.com/modelcontextprotocol/rust-sdk) on GitHub

**Goal:** Understand what a tool *is* from the protocol's perspective. A tool is just a name + JSON schema for input + a handler that returns text.

---

### Day 2 · Weekday · 2 hrs — Run a Hello-World MCP Server
- Clone the `rmcp` examples repo
- Get the basic echo/calculator example running locally
- Connect it to Claude Desktop using the `mcpServers` config in `claude_desktop_config.json`
- Actually call a tool from Claude's chat window

**Goal:** See the full loop working — Claude → stdio → your Rust binary → response back. This moment removes all the magic.

📌 Resource: [Claude Desktop MCP setup guide](https://modelcontextprotocol.io/quickstart/user)

---

### Day 3 · Weekday · 2 hrs — Design Your Data Model
- Define `Transaction`, `Category`, `Direction` structs in `models.rs`
- Think through edge cases: UPI credit vs debit, failed transactions
- Write `transactions.json` by hand — 30–40 realistic mock transactions (Zomato, Swiggy, IRCTC, Netflix, rent, salary credits)

**Goal:** A data file you can load and query. This is your "database" for the entire MVP.

📌 Resource: Look at real UPI apps (PhonePe, GPay) to understand what fields exist on a real transaction.

---

### Day 4 · Weekday · 2 hrs — Build the Data Store
- Write `store.rs` — loads `transactions.json` into a `Vec<Transaction>` at startup
- Write basic filter helpers: by date range, by category, by merchant
- Unit test these filters with `#[cfg(test)]` blocks

**Goal:** A `TransactionStore` struct with clean methods you'll call from every tool. Get the boring plumbing right here.

---

### Day 5 · Weekday · 2 hrs — Wire the MCP Server Skeleton
- Set up `main.rs` + `server.rs` with `rmcp`
- Register *one* placeholder tool (`search_transactions`) that returns a hardcoded JSON string
- Connect it to Claude Desktop and verify Claude can see and call the tool

**Goal:** Your server is live. Claude Desktop lists your tool. Even though it returns mock data, the pipe is working.

---

### Day 6 · Weekend · 4 hrs — Build `search_transactions` Properly
- Implement full filter logic: merchant (partial, case-insensitive), category, date range, amount range
- Return real data from your store
- Test conversationally in Claude: *"show me all Zomato transactions above ₹300"*
- Spend 30 mins just *talking to it* and fixing rough edges

**Goal:** Your first real, useful tool is done and feels good in conversation.

---

### Day 7 · Weekend · 4 hrs — Build `get_spending_breakdown`
- Group transactions by category for a given period
- Return total amount + count per category, sorted by spend
- Add a percentage share field (makes Claude's summaries much richer)
- Test: *"show me my spending breakdown for January"*

**Goal:** Two tools working. You now have enough for a decent demo already.

---

## Week 2 — Remaining Tools + Polish

### Day 8 · Weekday · 2 hrs — Build `compare_month_over_month`
- Accept two month parameters (or default to current vs last)
- Run `get_spending_breakdown` logic for each, diff the results
- Return a structured delta: `{ category, last_month, this_month, change_pct }`
- Test: *"am I spending more on food this month?"*

---

### Day 9 · Weekday · 2 hrs — Build `detect_recurring_payments`
The approach:
- Group transactions by merchant
- For each merchant with 2+ transactions, compute intervals between them
- If the median interval is ~7, ~15, or ~30 days (with tolerance), flag it as recurring
- Classify: salary (large credit, monthly), rent (large debit, monthly), subscription (small, fixed amount)

📌 Think of this as a basic time-series pattern problem — no ML needed, just interval statistics.

---

### Day 10 · Weekday · 2 hrs — Build `get_merchant_insights`
- Filter all transactions for a merchant
- Return: total spent, visit count, average transaction, first/last visit, most common time of day
- Test: *"how much have I spent on Swiggy total, and how often do I order?"*

---

### Day 11 · Weekday · 2 hrs — Error Handling + Tool Descriptions
This day is underrated. Your tool *descriptions* are how Claude decides which tool to call.
- Rewrite every tool's description string to be precise about what it does and doesn't do
- Add proper error returns (don't panic, return an error string Claude can reason about)
- Handle edge cases: empty results, invalid date formats, unknown categories

📌 Resource: [MCP tool description best practices](https://modelcontextprotocol.io/docs/concepts/tools)

---

### Day 12 · Weekend · 4 hrs — Add 200+ Mock Transactions + Stress Test
- Write a mock data generator (`mock.rs`) that produces realistic 6-month transaction history
- Realistic patterns: salary credit on 1st, rent on 5th, Netflix on 15th, random food/transport throughout
- Ask Claude 20 different questions in different phrasings — fix anything that breaks or feels off

**Goal:** Stress test the conversational layer, not just the code.

---

### Day 13 · Weekend · 4 hrs — README + Architecture Doc + Demo Script
- Write the final README that explains the NPCI angle clearly (this is your narrative when you show it)
- Document how to run the server and connect to Claude Desktop
- Record or script a 5-minute demo: 5 questions, each hitting a different tool
- Push to GitHub with a clean commit history

---

### Day 14 · Weekday · 2 hrs — Buffer / Stretch Goals

If everything went well, pick one:
- Add a 6th tool: `get_top_merchants` — ranked list of where most money goes
- Test Hindi query support explicitly (Claude handles it, but verify your data layer holds up)
- Write a short blog post or Twitter thread about the NPCI angle

If you hit blockers, this day is your catch-up.

---

## Project Structure (Target)

```
upi-mcp-server/
├── Cargo.toml
├── src/
│   ├── main.rs              # MCP server entrypoint
│   ├── server.rs            # Tool registration + dispatch
│   ├── models.rs            # Transaction, Category, Direction etc.
│   ├── tools/
│   │   ├── mod.rs
│   │   ├── search.rs        # search_transactions
│   │   ├── breakdown.rs     # get_spending_breakdown
│   │   ├── compare.rs       # compare_month_over_month
│   │   ├── recurring.rs     # detect_recurring_payments
│   │   └── merchant.rs      # get_merchant_insights
│   └── data/
│       ├── mod.rs
│       ├── store.rs         # In-memory store (mock data layer)
│       └── mock.rs          # Transaction seed data generator
└── data/
    └── transactions.json    # Seed file for mock UPI data
```

---

## Key Dependencies

```toml
[dependencies]
rmcp    = { version = "0.1", features = ["server", "transport-io"] }
tokio   = { version = "1", features = ["full"] }
serde   = { version = "1", features = ["derive"] }
serde_json = "1"
chrono  = { version = "0.4", features = ["serde"] }
uuid    = { version = "1", features = ["v4"] }
anyhow  = "1"
```

---

## Resources

| What | Link | When to Use |
|------|------|-------------|
| MCP Specification | [modelcontextprotocol.io](https://modelcontextprotocol.io) | Day 1, keep open throughout |
| `rmcp` Rust SDK | [github.com/modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk) | Day 2 onwards |
| Claude Desktop Setup | [modelcontextprotocol.io/quickstart/user](https://modelcontextprotocol.io/quickstart/user) | Day 2 |
| `serde_json` docs | [docs.rs/serde_json](https://docs.rs/serde_json) | Day 3–4 |
| `chrono` docs | [docs.rs/chrono](https://docs.rs/chrono) | Day 3–4, date filtering |
| Anthropic Tool Use Guide | [docs.anthropic.com/en/docs/build-with-claude/tool-use](https://docs.anthropic.com/en/docs/build-with-claude/tool-use) | Day 1, mental model |
| Jon Gjengset's Rust Streams | [youtube.com/@jonhoo](https://youtube.com/@jonhoo) | If you hit ownership/async confusion |

---

## What You'll Have at the End

- A working Rust binary that speaks MCP over stdio
- 5 tools covering the full UPI intelligence surface
- Claude Desktop as a zero-UI conversational layer
- A GitHub repo with a clear NPCI narrative
- Something genuinely portfolio-worthy — not a CRUD app, an *intelligence layer on a payment rail*

---

*Come back and course-correct at the end of each week based on where you are.*
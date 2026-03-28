---
name: transaction_json_schema_conventions
description: Exact schema, ID format, recurring patterns, and merchant conventions for transaction.json
type: project
---

## JSON Schema (exact field set)

Every record has exactly these 8 fields — no more, no less:

```json
{
    "id": "txn_1234567890",
    "amount": 300.00,
    "timestamp": "2024-01-01T12:00:00Z",
    "description": "Food",
    "category": "Food",
    "upi_id": "zomato@axis",
    "transaction_type": "Debit",
    "merchant_name": "Zomato"
}
```

- `merchant_name` is nullable — use `null` (not empty string) for peer-to-peer transfers (Other category)
- `amount` is a float with no currency symbol
- `timestamp` format: `YYYY-MM-DDTHH:MM:SSZ`
- `transaction_type`: exactly `"Credit"` or `"Debit"`

## Valid Categories

`Food`, `Grocery`, `Utilities`, `Entertainment`, `Transportation`, `Healthcare`, `Rental`, `Salary`, `Investment`, `Other`

## ID Format and Current State

- Format: `txn_` followed by a sequential integer
- Original data started at `txn_1234567890`
- After the Mar-Apr 2024 expansion (202 total transactions), highest ID is `txn_1234568091`
- Next available ID: `txn_1234568092`

## Recurring Payment Patterns (embedded in data)

| Merchant | Amount | Day of month | UPI ID | Category |
|---|---|---|---|---|
| NatPay Corp (Salary) | 60000.00 | 1st | natpaycor@hdfc | Salary / Credit |
| Landlord (Rent) | 12000.00 | 5th | landlord@sbi | Rental |
| Netflix | 649.00 | 10th | netflix@icici | Entertainment |
| Spotify | 199.00 | 10th | spotify@icici | Entertainment |
| YouTube Premium | 299.00 | 3rd | google@okaxis | Entertainment |
| Zerodha SIP | 10000.00 | 25th | zerodha@kotak | Investment |
| BESCOM (electricity) | ~950-980 | 15th-22nd | bescom@upi | Utilities |
| Jio Fiber | 799.00 | 18th-23rd | jio@jiopay | Utilities |
| Airtel (mobile) | 799.00 | last day | airtel@upi | Utilities |

## Merchant Name Conventions

- Food delivery: `Zomato` (zomato@axis), `Swiggy` (swiggy@icici)
- Grocery: `BigBasket` (bigbasket@hdfcbank), `Blinkit` (blinkit@razorpay)
- Rides: `Ola` (ola@okaxis), `Uber` (uber@razorpay)
- Travel: `IRCTC` (irctc@upi), `IndiGo` (indigo@axis)
- Healthcare: `Apollo Clinic` (apolloclinic@hdfcbank), `MedPlus` (medplus@upi), `Cult.fit` (cultfit@razorpay), `DentCare Clinic` (dentcare@hdfcbank)
- Peer-to-peer (Other): personal UPI IDs like `rahul@oksbi`, `priya@okhdfc`, merchant_name = null
- Refunds: same UPI ID as original merchant, `transaction_type: Credit`

## Amount Ranges by Category

| Category | Typical range |
|---|---|
| Food (meal) | 300–700 per meal; weekends skew higher (650–750+) |
| Grocery (BigBasket) | 1800–2800 per shop |
| Grocery (Blinkit) | 650–1800 |
| Transportation (cab) | 85–130 weekday; 250–290 longer trips |
| Transportation (flight) | 6500–8500 |
| Utilities (electricity) | 950–980 |
| Utilities (broadband/mobile) | 799 fixed |
| Entertainment (streaming) | 199–649 fixed |
| Healthcare | 1200–4500 |
| Investment (SIP) | 10000 fixed |
| Salary (Credit) | 60000 fixed |
| Rental | 12000 fixed |
| Other (P2P transfer) | 3000–5500 |

## Data Distribution (as of 202 transactions, Jan-Apr 2024)

- Jan: 36 | Feb: 39 | Mar: 64 | Apr: 63
- Food: 82 | Transportation: 51 | Grocery: 17 | Entertainment: 13
- Utilities: 12 | Other: 9 | Healthcare: 6 | Salary: 4 | Rental: 4 | Investment: 4

**Why:** This is a key project dataset compiled into the binary via `include_str!`. All expansion must preserve schema exactly.

**How to apply:** Always read current file first to get the actual highest ID before generating new IDs. Do not rely on this memory for the ID counter — verify it.

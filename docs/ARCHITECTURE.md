# System Architecture & Rust Module Layout

## 1. Repository Directory Tree

```plaintext
procureops-ai/
├── CLAUDE.md
├── AGENTS.md
├── .env.example
├── README.md
├── docs/
│   ├── PRD.md
│   ├── TECH_STACK.md
│   ├── ARCHITECTURE.md
│   ├── API_SPEC.md
│   ├── RULES.md
│   ├── UI_UX.md
│   └── TASKS.md
├── backend/
│   ├── Cargo.toml
│   ├── data/
│   │   ├── catalog.json         # Mock B2B supplier catalog (12+ items)
│   │   └── transactions.json    # Pre-seeded 25 realistic procurement records
│   └── src/
│       ├── main.rs              # Axum router, CORS, shared Arc<RwLock<AppState>>, server bind on :8000
│       ├── models.rs            # Serde structs: Transaction, CatalogItem, MandateRequest, KPIs, etc.
│       ├── guardrails.rs        # Typestate `ApprovedMandate` & `validate()` policy engine
│       ├── paypal.rs            # Async reqwest PayPal OAuth2, Orders v2, Payouts v1 + Mock fallback
│       ├── agent.rs             # LLM / deterministic mandate parser & catalog matcher
│       └── analytics.rs         # NL-to-Typed-Filter engine & KPI aggregator
└── frontend/
    ├── package.json
    ├── tsconfig.json
    ├── tailwind.config.ts
    └── src/
        ├── types/index.ts
        ├── lib/api.ts
        ├── components/
        │   ├── KpiBanner.tsx
        │   ├── MandatePanel.tsx
        │   ├── QueryBar.tsx
        │   └── CockpitGrid.tsx
        └── app/
            ├── layout.tsx
            └── page.tsx
```

## 2. Shared Application State (AppState)

In `backend/src/main.rs`, hold state in an `Arc<tokio::sync::RwLock<AppState>>` containing:
* `transactions`: `Vec<Transaction>` (loaded from `data/transactions.json` on startup and persisted on mutation)
* `catalog`: `Vec<CatalogItem>` (loaded from `data/catalog.json`)
* `paypal_client`: `PayPalClient`

## 3. Core Data Schema (Transaction)

Every record stored in `backend/data/transactions.json` and rendered in AG Grid conforms to:

```json
{
  "order_id": "ORD-2026-1025",
  "paypal_order_id": "8XY1294810293841B",
  "timestamp": "2026-10-25T10:15:00Z",
  "vendor": "DigiKey",
  "vendor_email": "settlements-sb@digikey-mock.com",
  "category": "Embedded Hardware",
  "item_name": "Raspberry Pi 5 (8GB) Single Board Computer",
  "quantity": 10,
  "unit_price": 78.50,
  "total_amount": 785.00,
  "authorized_budget": 900.00,
  "currency": "USD",
  "delivery_days": 2,
  "vendor_rating": 4.9,
  "status": "COMPLETED",
  "risk_score": 0.08,
  "guardrail_passed": true,
  "guardrail_reason": "All policy checks passed (Budget OK, Whitelisted Vendor, Rating >= 4.5)",
  "agent_reasoning": "Selected DigiKey over Mouser ($82/unit) for lowest unit cost ($78.50) and 2-day express SLA."
}
```

Allowed `status` enum values:
* `COMPLETED` (Captured via PayPal Orders API)
* `SETTLED` (Disbursed via PayPal Payouts API)
* `BLOCKED_BY_GUARDRAIL` (Rejected by Rust Typestate Guardrail before PayPal API)
* `APPROVED_PENDING_SETTLEMENT` (Eligible for one-click payout)


## 4. Current payment lifecycle (2026-10-26)

Order creation no longer captures immediately. States are AWAITING_APPROVAL -> PAYMENT_PENDING -> COMPLETED, followed optionally by PAYOUT_PENDING -> SETTLED. Errors/cancellation use PAYMENT_FAILED, PAYOUT_FAILED and CANCELLED. BLOCKED_BY_GUARDRAIL remains the policy rejection state. Older APPROVED_PENDING_SETTLEMENT rows are historical seed data and cannot initiate payouts.

Transactions additionally persist payment_mode, parser_mode, capture_id, approval_url, payment_error, payout_error, payout_sender_batch_id and request_id. Missing execution provenance on historical rows defaults to seed. See API_SPEC.md for current contracts.

AppState has a shared operation mutex for serialized mutations and an RwLock for readers. Mutations write a synchronized temporary file and atomically replace the ledger before publishing state. Capture/payout reservations precede external money-moving requests. Stable provider IDs and repeated authoritative status checks reconcile retries. Run one backend process per data directory.

The Next.js SDK v6 session calls the server capture endpoint only after buyer approval. Captured funds arrive in the configured ProcureOps Sandbox merchant, and a separate payout goes to an owned Sandbox test receiver. The illustrative catalog does not establish seller onboarding or authorize payments to the named suppliers.

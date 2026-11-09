# Product Requirements Document (PRD) — ProcureOps AI (Rust Edition)

## 1. Executive Summary
**ProcureOps AI** solves autonomous agentic commerce safety by combining:
1. **Typestate-Guarded Rust Procurement Engine:** Users submit plain-English purchasing mandates. The Rust backend parses the mandate, queries supplier catalogs, and enforces **compile-time and runtime policy guardrails** via Rust's type system (`ApprovedMandate`) before executing payment through the **PayPal Orders v2 API**.
2. **Executive Treasury Cockpit (AG Grid):** Finance managers audit every autonomous purchase in real time inside an **AG Grid** cockpit, run natural-language queries over spend logs, and trigger instant vendor disbursements via the **PayPal Payouts v1 API**.

## 2. Target Hackathon Tracks
- **Best Use of Agentic Commerce ($5,000):** Autonomous mandate parsing, multi-vendor comparison, Rust typestate guardrail enforcement, and PayPal checkout execution.
- **Best Use of PayPal + AI ($5,000):** End-to-end integration of PayPal Orders v2 and Payouts v1 APIs with AI mandate reasoning and natural language financial analytics.
- **Best Use of AG Grid ($5,000 Sponsor Track):** High-density financial grid featuring custom status badge renderers, risk gauges, dynamic NL filtering, sorting, and one-click PayPal Payout action buttons.

## 3. Core User Flows

### Flow 1: Autonomous Procurement Mandate
1. User enters a mandate in the Mandate Console:
   *"Buy 10 units of Raspberry Pi 5 (8GB) under $900 total with delivery under 4 days from an approved vendor."*
2. User sets the hard budget ceiling slider/input (e.g., `$900.00`) and clicks **Execute Mandate**.
3. Backend Agent performs a 4-step visible reasoning pipeline:
   - **Step 1 (Mandate Extraction):** Extracts item, quantity (10), target budget ($900), max delivery SLA (4 days).
   - **Step 2 (Catalog Lookup):** Queries mock B2B supplier catalogs (DigiKey, Mouser, SparkFun, UnauthorizedCloneShop) and selects the best compliant quote.
   - **Step 3 (Typestate Guardrail Validation):** Rust `guardrails::validate()` verifies total cost <= budget ceiling, total cost <= hard limit ($2,000), vendor is whitelisted, and vendor rating >= 4.5. On success, yields an `ApprovedMandate`.
   - **Step 4 (PayPal Order Execution):** Passes `&ApprovedMandate` to `PayPalClient::execute_order()`, calling PayPal REST v2 (`POST /v2/checkout/orders` and capture). Records the transaction and streams to AG Grid. If validation fails, logs `BLOCKED_BY_GUARDRAIL` with violation details.

### Flow 2: Conversational Treasury Analytics ("Ask-Your-Spend")
1. User types a natural language query above the AG Grid table:
   *"Show all blocked orders or orders over $500"*
2. Backend parses the query into a typed Rust iterator predicate / filter AST and generates an executive summary.
3. AG Grid updates dynamically to display matching transactions, active filter badge, and summary KPI insight.

### Flow 3: One-Click Vendor Settlement (PayPal Payouts)
1. In the AG Grid table, compliant rows (`COMPLETED` / `APPROVED_PENDING_SETTLEMENT`) feature an interactive **"PayPal Payout"** action button.
2. Clicking triggers `POST /api/paypal/payout`, executing the PayPal Payouts v1 API (`POST /v1/payments/payouts`), updating status to `SETTLED`, and recording the Payout Batch ID.


## 4. Integration clarification (2026-10-26)

The supported payment flow requires buyer approval through PayPal JavaScript SDK v6 between order creation and capture. A prepared order is AWAITING_APPROVAL; it is not a completed payment. Only verified captures become COMPLETED. A separate vendor payout is PAYOUT_PENDING until a matching recipient item succeeds. Historical seed records are read-only for payment purposes.

This prototype collects the buyer's payment into the ProcureOps Sandbox merchant and pays an owned Sandbox test receiver. Its catalog and supplier identities are illustrative, with no claim that the suppliers are connected merchants or that physical goods were purchased. Unattended external procurement is future scope requiring a suitable authorized payment arrangement and real supplier integrations.

Explicit simulation preserves a no-credential demo while real API failures remain errors or unresolved pending outcomes. No production PayPal endpoints are supported. The actual parser used is shown per operation; real model inference needs provider credentials. See API_SPEC.md and VERIFICATION.md for the implemented lifecycle and its verification limits.

The shared server model client supports OpenRouter and direct OpenAI. LLM_PROVIDER selects the provider, and the UI records its returned model metadata for extraction and treasury queries. OpenRouter account verification is tracked separately from local contract tests; see AI_SETUP.md.

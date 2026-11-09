# Step-by-Step Implementation Checklist (TASKS.md)
Execute these phases sequentially and mark [x] as each task is verified.

## Phase 1: Rust Cargo Workspace & Seed Data
* [x] **1.1:** Create /backend/Cargo.toml and .env.example matching docs/TECH_STACK.md.
* [x] **1.2:** Create /backend/data/catalog.json (12+ items across approved and unapproved vendors) and /backend/data/transactions.json (25 realistic historical procurement records).
* [x] **1.3:** Create /backend/src/models.rs with all Serde request/response structs and Transaction model.

## Phase 2: Rust Typestate Guardrails & PayPal Client
* [x] **2.1:** Implement /backend/src/guardrails.rs featuring the private-field ApprovedMandate struct and validate() function enforcing budget ceiling, $2,000 hard cap, vendor whitelist, >= 4.5 rating, and SLA days.
* [x] **2.2:** Implement /backend/src/paypal.rs (PayPalClient) requiring &ApprovedMandate, supporting PayPal OAuth2, Orders v2, and Payouts v1 via reqwest, with explicit simulation and preserved real API errors.

## Phase 3: Rust Procurement Agent & Analytics Engine
* [x] **3.1:** Implement /backend/src/agent.rs to parse mandates (OpenRouter or direct OpenAI with deterministic Rust regex/keyword fallback), select best catalog item, validate via guardrails::validate, execute via PayPalClient, and return the 4-step trace.
* [x] **3.2:** Implement /backend/src/analytics.rs to translate natural language queries into typed Rust iterator predicates over Vec<Transaction> and compute KPIs.

## Phase 4: Axum Server & Route Wiring
* [x] **4.1:** Implement /backend/src/main.rs with configured CORS origins, shared Arc<RwLock<AppState>>, serialized ledger mutations, and transaction/mandate/analytics/checkout/capture/cancel/payout/status/webhook routes.
* [x] **4.2:** Run cargo check and cargo test (or curl against localhost:8000) to confirm clean compilation and valid JSON responses.

## Phase 5: Next.js + AG Grid Frontend Cockpit
* [x] **5.1:** Scaffold /frontend with Next.js (App Router, TypeScript, Tailwind CSS), ag-grid-react, ag-grid-community, and lucide-react.
* [x] **5.2:** Implement src/types/index.ts, src/lib/api.ts, KpiBanner.tsx, MandatePanel.tsx, QueryBar.tsx, and CockpitGrid.tsx.
* [x] **5.3:** Assemble the Executive Cockpit and verify npm run build. Verified 2026-10-26 using webpack and official Tailwind PostCSS; includes TypeScript checking.

## Phase 6: Documentation & Demo Readiness
* [x] **6.1:** Create root README.md highlighting the **Rust Typestate Financial Guardrail** architecture, PayPal REST v2 integration, and AG Grid Executive Cockpit.
* [x] **6.2:** Resolve frontend lint errors/warnings and verify lint and production build. Verified 2026-10-26.
* [x] **6.3a:** Implement SDK v6 buyer approval, cancellation/error handling, and server-side Orders v2 create/capture. Verify simulated workflow and SDK callback contract in Chrome.
* [x] **6.3b1:** Authenticate the owner-configured Sandbox app, load the real hosted SDK v6, create a genuine USD order, open PayPal checkout, and verify the close/cancellation callback. Verified 2026-10-26.
* [x] **6.3b2:** Verify an actual COMPLETED capture. On 2026-10-27, after owner-confirmed Sandbox review settings were disabled, a new 78.50 USD order completed through actual SDK approval and server capture. PayPal API and merchant activity confirm capture 67A362971J280231P. The earlier capture 7DY8396289087864E remains PAYMENT_PENDING and is preserved. See SANDBOX_VERIFICATION.md.
* [x] **6.4:** Persist/display actual execution mode; real errors never become simulated success. Verify failure paths and update docs/RULES.md.
* [x] **6.5a:** Expose actual parser provenance for mandates and treasury queries; preserve clearly identified deterministic fallback.
* [x] **6.5b:** Verify actual OpenRouter calls with an owner-configured OPENROUTER_API_KEY for a non-preset mandate and treasury query. Passed 2026-10-27: model-extracted quantity two with independent guardrail rejection, correct four-record treasury filter via API and UI, visible returned-model provenance, and unchanged original payment ledger. See AI_VERIFICATION.md.
* [x] **6.5c:** Add the shared OpenRouter/direct OpenAI client and returned-model provenance for both workflows. Verify fixed provider endpoints, separate credentials, JSON-mode requests, actual returned model metadata, malformed/truncated output and provider failures using local contract tests. Include current pending/payment statuses in typed filters and reject partially invalid trees.
* [x] **6.6:** Correct quote selection, vendor/stock/quantity constraints, SLA-aware pricing and tighter prompt budgets; verify with Rust tests, including zero budgets and delivery-number parsing.
* [x] **6.7a:** Implement reserved stable payout IDs, concurrent duplicate protection, owned-receiver configuration, pending state and recipient-item reconciliation; verify with provider fixtures and Chrome.
* [x] **6.7b:** Configure the actual owned Sandbox recipient and Payouts capability; verify receipt in its Sandbox account. Verified 2026-10-27: batch W6A2KVLEXRW8N, matching item SUCCESS, local SETTLED, and +78.50 USD receipt in the owned Personal account. A repeated request returned the existing batch.
* [x] **6.8:** Verify 34 Rust tests including API lifecycles, provider errors, amount mismatch, webhook verification/reconciliation, retries, concurrent ledger mutations and model-provider contracts. Use serialized atomic ledger writes and unique order IDs. Analytics fixtures use immutable seed records, separate from actual payment history.
* [x] **6.9a:** Add MIT LICENSE, root ignore rules, separate environment examples, Rust setup instructions, API docs, resource inventory and Postman collection; identify legacy Python as unsupported reference code.
* [ ] **6.9b:** Publish the source repository under the owner account and add its actual URL to the submission. This workspace is not a Git checkout.
* [x] **6.10a:** Prepare local Devpost description, accurate integration/tools disclosure, owner-field checklist and 170-second demo script. See SUBMISSION_DRAFT.md and DEMO_SCRIPT.md. Save screenshots of actual completed Sandbox checkout, merchant activity, settlement and recipient receipt under .verification.
* [ ] **6.10b:** Use the verified AI/payment evidence, finalize owner fields, publish the public YouTube demo under three minutes, provide functional free judge access, and complete Devpost; confirm registration and team details. The local draft is not a submission.
* [x] **6.11a:** Verify 12 Chrome workflow checks against production output with an isolated ledger, including desktop/phone payment flows, native dialogs, export, navigation and responsive layouts; preserve screenshots/results under .verification.
* [ ] **6.11b:** Rehearse from the eventual published checkout and maintain judge access. Before public multi-user hosting, implement authentication and deployment-grade durable storage.
* [x] **6.12:** Complete the owner-requested UI redesign using website references and the GitHub frontend-design skill. Verify production build, lint, 12 browser checks, responsive screenshots and an unchanged original Sandbox ledger. See UI_REDESIGN.md and UI_UX.md.
* [x] **6.13:** Replace the generic package logo with an original SVG purchase-loop P and approval check, refine the wordmark, and add matching browser icon and reusable light/dark SVG exports. Verify build/lint, desktop/compact/mobile fit, favicon loading and unchanged payment history. Evidence: .verification/brand-results.json.
* [x] **6.14:** Rehearse the actual submission workflow, fix noncontiguous product-keyword matching while retaining exact model/capacity tokens, and repair the malformed optional npm lock entry. Verify 35 Rust tests from clean source, clean npm ci, lint, production build and 12 browser checks. Record a new completed Sandbox capture, reconciled payout, actual AI budget rejection and actual treasury query. Preserve earlier payment history. See RELEASE_REPORT.md.

Read [HACKATHON_READINESS.md](HACKATHON_READINESS.md) for the 2026-10-26 assessment, evidence, official links, and optional improvements.

## Resource review
* [x] **R1:** Review all 34 main entries on the hackathon Resources page, including every sponsor and recording link; record unavailable video transcripts and current replacements for inaccessible AI documentation in RESOURCE_INTEGRATION_MATRIX.md.
* [x] **R2:** Implement the relevant PayPal Orders, SDK v6, REST OAuth, Payouts and webhook paths. Optional financing/subscriptions/invoicing/donations and sponsor runtime products remain documented possibilities, not claimed integrations.

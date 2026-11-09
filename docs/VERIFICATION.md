# Verification — updated 2026-10-27

## Current result

The local integration, actual PayPal Sandbox checkout/payout and actual OpenRouter inference are verified. After the owner disabled Sandbox review settings, one new 78.50 USD purchase completed through the actual hosted SDK v6 and server capture endpoint. PayPal's API and merchant activity confirm capture 67A362971J280231P. One payout batch W6A2KVLEXRW8N reached SUCCESS with a matching successful recipient item; the owned Personal Sandbox account confirms receipt. The application record is SETTLED. The earlier review-held capture remains PAYMENT_PENDING and was preserved. OpenRouter passed a non-preset mandate and typed treasury query through the API and running UI; model verification left the payment ledger unchanged. See SANDBOX_VERIFICATION.md and AI_VERIFICATION.md. The catalog is illustrative; no physical supplier purchase occurred.

| Check | Result |
| --- | --- |
| Rust `cargo check --locked --offline` | Passed |
| Rust `cargo build --locked --offline` | Passed |
| Rust `cargo test --locked --offline` | 35 passed, including OpenRouter contract, typed model-output tests and noncontiguous product keywords with exact model/capacity checks |
| Frontend `npm run lint` | Passed, zero errors/warnings |
| Frontend `npm run build` | Passed, production build and TypeScript checks |
| `npm run verify:browser` | 12 Chrome checks passed after the UI redesign |
| Postman collection | Valid JSON with 12 local API requests; no credentials included |
| Local source ledger | Local fixture/browser tests used isolated copies; original 25 seed records and four actual Sandbox attempt records are preserved |
| Actual PayPal Sandbox authentication / SDK / order creation | Passed on 2026-10-26 with owner credentials |
| Actual PayPal buyer approval / capture request | Passed with the US pair on 2026-10-27; matching merchant/payer IDs and merchant activity verified |
| Actual PayPal completed capture | Passed 2026-10-27; 78.50 USD, COMPLETED, matching API and merchant activity |
| Actual PayPal payout / recipient receipt | Passed 2026-10-27; matching item SUCCESS, local SETTLED, receipt confirmed in recipient account |
| Actual repeated settled payout request | Returned the same batch without another payout |
| Actual OpenRouter calls | Passed 2026-10-27; non-preset mandate and treasury query via API/UI with actual returned model provenance |

## Rust coverage

Includes deterministic extraction, analytics predicates/KPIs, finite monetary values, hard/user/prompt budgets, vendor trust, delivery SLA and eligible quote selection. Route tests exercise prepare/approve/capture/cancel/payout/reconcile, rejection of historical payouts, duplicate request IDs, concurrent captures and payouts, simultaneous unique mandates, durable ledger contents, provider errors without simulated fallback, capture amount mismatch, unclaimed payout items, and unsigned/verified/replayed webhooks.

PayPal HTTP fixtures run on local ephemeral ports. Their passing status establishes application behavior against those fixtures, not PayPal account connectivity. Signature-verification tests mock PayPal's verification response; cryptographic acceptance of a genuine PayPal event remains an external acceptance check.

The shared model client has local contract coverage for the OpenRouter authorization header, JSON response mode, required-parameter routing and resolved model metadata. Tests reject malformed, non-object and truncated output, keep selected-provider keys separate, and verify that HTTP error bodies/keys are excluded from diagnostics. Typed analytics now accepts current payment states and rejects an entire composite filter when any child is invalid. Analytics fixtures read seed_transactions.json rather than the owner's mutable Sandbox ledger. Actual OpenRouter inference was subsequently verified separately; see AI_VERIFICATION.md.

## Chrome coverage

The verifier starts the built Rust binary in simulation with copied data, plus Next.js production output. It uses installed Google Chrome in headless mode and checks:

1. Seed ledger loads, simulation label is visible, and seed rows have no actionable payout.
2. A new mandate waits for approval; explicit simulated approval produces a capture ID.
3. A payout stays pending until checked, then becomes settled.
4. Budget breach and untrusted vendor requests are blocked before PayPal creation.
5. A treasury query filters the ledger and identifies its parser.
6. No uncaught browser errors occur during the workflow.
7. A separately intercepted SDK v6 contract test initializes with the public client ID and calls the server capture endpoint from `onApprove` with the correct local order reference.
8. Purchase views, keyboard ledger tabs, the policy panel and native purchase inspection work.
9. CSV export includes the current records and payment provenance.
10. Phone, tablet and compact desktop layouts contain page overflow at 390px, 768px and 1024px.
11. Mobile navigation closes correctly and New purchase focuses the request field.
12. Phone purchase cards support recent-first order, full-reference search, capture, payout, inspection and pagination.

The seventh check uses a stub SDK script and intercepted API responses. It does not log into PayPal or verify the real hosted SDK against a merchant account.

Artifacts:

- `.verification/browser-results.json`
- `.verification/screenshots/cockpit.png`
- `.verification/screenshots/verified-workflow.png`
- `.verification/screenshots/redesign-desktop.png`
- `.verification/screenshots/redesign-mobile.png`

Browser testing found and fixed an actual defect: a payout button's click also triggered AG Grid row selection and opened the audit dialog. Mouse handling is now suppressed for that action cell, and the regression check verifies that the dialog stays closed.

## Build environment

Node 24.12.0; Next.js 16.4.3; React 19.3.0; Tailwind 4/PostCSS; AG Grid Community 36.2.x; Rust stable GNU toolchain. Windows sandbox loopback restrictions required the HTTP integration/browser checks to run outside the sandbox. The production frontend uses supported webpack after Turbopack subprocess startup failed. Manrope is now hosted locally with its OFL license, avoiding external font requests during builds. The redesigned UI and retained payment history are documented in UI_REDESIGN.md.

`cargo fmt` through the toolchain dispatcher was unavailable, so the installed rustfmt executable was used directly. Rust sources were formatted. A transient sandbox EPERM on generated Next.js cache files was resolved by rebuilding outside the sandbox after stopping verification servers.

## Owner acceptance checks still required

- Reconcile the earlier review-held capture if its provider status changes. Completed checkout and payout have passed with the US pair; earlier India-based attempts remain cancelled locally.
- Register an HTTPS webhook and verify genuine app events if using webhooks in the demo.
- Publish the repository and complete the Devpost/video/judge-access deliverables.

The 34-resource review and optional integration decisions are in RESOURCE_INTEGRATION_MATRIX.md. Implementation checkboxes reflect completed local work separately from these external acceptance steps.

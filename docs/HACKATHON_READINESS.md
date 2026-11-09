# ProcureOps AI initial readiness review — 2026-10-26

> Historical assessment before the integration changes. See [VERIFICATION.md](VERIFICATION.md), [TASKS.md](TASKS.md), and [PAYPAL_SETUP.md](PAYPAL_SETUP.md) for the current implementation, verification results, and account steps.

The core prototype exists, but submission readiness is not established. The original checklist omitted payment approval, real integration evidence, and submission deliverables. This review inspected all seven project specifications, the Rust backend, the Next.js frontend, local configuration presence, and the official hackathon pages. No payment was initiated and no credentials were exposed.

## Official requirements and timing

Submit by **November 12, 2026, noon PST**, which is **November 13, 2026, 1:30 a.m. IST**. Use PayPal Sandbox plus meaningful AI, supply a public open-source repository with a license, a functional build with complete setup instructions or hosting, an English description and testing instructions, and a public YouTube demo under three minutes. Maintain free judge access through December 15. Registration, team eligibility, external repository visibility, and an existing Devpost submission were not verified from this workspace. [Official rules](https://paypalaihackathon.devpost.com/rules)

Hosting is optional. The official overview names the five equally weighted criteria: technological implementation, design, potential impact, innovation, and presentation. The target awards in the PRD exist, although the event describes them as awards rather than mandatory project tracks. [Hackathon overview](https://paypalaihackathon.devpost.com/)

## Verification performed

| Check | Result |
| --- | --- |
| `cargo check --locked --offline` | Passed |
| `cargo test --locked --offline` | 14 passed: parser/quote selection, analytics, guardrails |
| `npm run build` | Failed: Turbopack CSS-loader subprocess exits before connection while processing AG Grid styles; same failure outside the sandbox |
| `npx tsc --noEmit --incremental false` | Passed |
| `npm run lint` | Failed: 2 errors, 21 warnings |
| Catalog and ledger | 16 catalog entries; 25 transaction records |
| `backend/.env` | PayPal and AI values are placeholders or empty; this checks that file, not external account provisioning |
| Root repository files | No `.git`, root `.gitignore`, or `LICENSE`; README clone URL is a placeholder |
| Runtime/external validation | No active API responded on port 8000. A temporary prebuilt backend process did not become reachable on the requested test port. Real PayPal/AI calls and browser interactions remain unverified |

The existing Python test script exercises FastAPI and references `pandas_expr`; it does not validate the Rust API contract. Rust tests passing do not establish that the payment lifecycle works.

## Finish these first

### 1. Restore a reproducible frontend build

Investigate the custom Tailwind/Turbopack CSS processing in `frontend/next.config.ts` using the installed Next.js guides. The build failure identifies the AG Grid CSS transformation path; its exact cause has not been isolated. Resolve the ESLint errors at `frontend/src/app/page.tsx:103` and `frontend/src/types/index.ts:54`, then review the remaining warnings, particularly the stale callback dependency in `CockpitGrid.tsx`. Acceptance: a clean install, lint, and production build succeed.

### 2. Finish the real PayPal payment journey

`backend/src/paypal.rs:252` creates an order and immediately calls capture. There is no buyer approval callback or redirect flow. The create payload supplies neither an approved payment source nor a vendor payee. Define the fund flow: who is the buyer, who is the merchant receiving the capture, and why a later vendor payout is needed. Do not assume capturing into the configured merchant account pays the catalog supplier.

For standard PayPal Checkout, creation returns an approval URL, the buyer approves, and capture follows. Implement that lifecycle using owned Sandbox test accounts and store verifiable order/capture IDs. Unattended purchasing would need an appropriate, explicitly authorized payment arrangement rather than skipping buyer approval. [PayPal Orders integration](https://developer.paypal.com/api/rest/integration/orders-api)

### 3. Separate simulation from actual payment outcomes

`paypal.rs` substitutes simulated responses after OAuth, create, capture, or payout failures. `mode()` reports configuration, so even simulated fallback can be labeled `live_sandbox`. `agent.rs` then treats a passed guardrail as a successful completed payment. The frontend repeats capture/disbursement success messages.

Use explicit simulation selection and persist the actual execution source per transaction. Real-mode errors must remain errors; timeouts may require reconciliation before retry. Keep offline demos available with clear labels. Update the automatic-fallback instruction in `docs/RULES.md` when implementing this change.

### 4. Verify and expose actual AI use

LLM calls for mandate extraction and analytics exist. Their execution has not been demonstrated with a real provider here. Configure a server-side key, verify non-preset requests, and record the model/provider in submission materials. Show the parser actually used on each request; a configured-key flag alone cannot establish that inference succeeded. Keep deterministic fallback clearly identified.

### 5. Correct product and quote selection

`backend/src/agent.rs:226` chooses a low-price approved vendor before enforcing all constraints. It can:

- Fall back to the first three catalog entries for an unknown product.
- Choose another product from a named vendor or another vendor when the requested vendor has no match.
- Select the cheapest quote and block it for delivery SLA, even if a slightly more expensive compliant quote exists. For example, one Jetson delivered within two days selects DigiKey at $495/three days and blocks, although Mouser has $499/two days.
- Ignore `in_stock`, `min_quantity`, and `max_quantity`.
- Ignore the budget written in the prompt; only the separate authorized-budget field is enforced.

Reject unknown/ambiguous items, constrain vendor and quantity, evaluate eligibility before optimizing price, and enforce the tighter applicable budget. Return the competing quotes and rejection reasons to make the choice inspectable.

### 6. Make payouts and persistence reliable

`backend/src/main.rs:152` marks a payout `SETTLED` when a batch ID is available without checking item completion. The catalog contains illustrative recipient emails rather than verified test recipients. Requests can race: two callers can both see an unsettled row, send payouts, and only then update it. Random batch IDs on each attempt cannot deduplicate a logical retry. Mandate IDs are similarly calculated before the asynchronous execution, and JSON snapshots are written after releasing the lock, allowing conflicting IDs or stale writes.

Use stable per-operation retry IDs, atomic state transitions, and durable serialized writes or a transactional database. Reconcile payout items with polling or verified webhooks. PayPal initially accepts payouts for processing; even batch success requires inspecting individual items for holds or unclaimed funds. [Payout statuses](https://developer.paypal.com/api/payments.payouts-batch/v1/definitions/payout_batch_header/), [Payout API integration](https://developer.paypal.com/api/payouts)

### 7. Package a judge-ready submission

The README already has a walkthrough, but its clone URL is fictional and the MIT badge has no corresponding license file. Add the actual license and repository URL. Add root ignore rules before publishing: `backend/.env` exists and is outside `frontend/.gitignore` coverage. Document Rust as the supported backend and clarify or archive the leftover Python implementation. Align documented package versions and Node prerequisites with the installed frontend package requirements. Put the frontend API URL in its own supported environment file; Next.js does not automatically load a parent repository `.env`.

Create the submission description, screenshots, and video URL. Hosted access was not found in the local documentation; a locally runnable submission is also possible. Rehearse setup using a clean checkout without personal credentials. If hosting, replace the localhost API URL, support an externally reachable backend bind address, use durable storage, and control access to payout/reset routes, which are currently unauthenticated.

## Suggestions after the blockers

- **Focus the pitch:** a procurement assistant for small hardware teams that compares quotes and enforces company purchasing policy. Demonstrate one legitimate purchase, one rejected unsafe request, and a treasury question. Measure time saved with an actual timed comparison; label seeded data and estimated benefits.
- **Strengthen AG Grid deliberately:** the sponsor guidance emphasizes AG Studio, custom widgets, theming/layout, and the Studio Agent Framework. One useful extension would turn “show spend by vendor and blocked attempts” into an interactive dashboard with a linked ledger. These are competitive suggestions; the prize table allows any AG Grid tool. Community Grid already provides a foundation. [AG Grid sponsor guidance](https://paypalaihackathon.devpost.com/details/aggrid)
- **Present accurate product claims:** label Community Grid accurately instead of “Enterprise,” call the visible steps an execution trace unless streaming is implemented, and explain typestate precisely: a private Rust type forces the order path through runtime validation. It is not cryptographic authorization and does not establish that the policy or external payment result is correct.
- **Improve financial labels:** “blocked attempted spend” is supportable from this ledger. “Capital protected” or realized savings needs assumptions; repeated blocked attempts do not necessarily represent additional money saved.
- **Use a concise video structure:** problem and audience (20 seconds), real Sandbox purchase with approval (65 seconds), rejected request and trace (35 seconds), treasury query/dashboard (30 seconds), value and architecture (20 seconds). Total: 170 seconds. Keep payouts in the video only if their lifecycle is verified.

Use the reopened task 5.3 and added tasks 6.2–6.11 in [TASKS.md](TASKS.md) as the completion queue. Prioritize build, real payment plus honest outcomes, AI evidence, selection correctness, payout reliability, and submission packaging before optional sponsor features.

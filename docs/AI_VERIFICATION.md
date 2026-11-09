# Actual OpenRouter verification — 2026-10-27

Final submission recording: recursive/flint-v1.0:free extracted the one-board purchase that completed through PayPal Sandbox. nvidia/nemotron-3.5-super-vl:free parsed the two-board, $50 request that Rust policy blocked. The final read-only treasury recording used dots-studio/dots-3-note-preview:free and returned four independently checked Mouser-below-$600 records. One prior recording request returned HTTP 429 and truthful fallback; the successful retry supplies the final footage. See RELEASE_REPORT.md and evidence/ai-query.json. Payment operations performed for the demo are separate from the earlier read-only model checks below.

Actual inference passed using the owner's locally saved OPENROUTER_API_KEY, LLM_PROVIDER=openrouter and LLM_MODEL=openrouter/free. This is separate from the local provider-contract fixtures. The key was not printed or saved in evidence files.

## Model-backed mandate

An isolated Rust server used a copied seed ledger and simulation payments. Request:

> Please source two Raspberry Pi 5 boards from DigiKey, total spend under $50 and delivery within three days.

The response identified quantity **2** and DigiKey, with parser provenance **openrouter:nvidia/nemotron-3.5-super-vl:free**. Deterministic policy rejected the underfunded purchase as BLOCKED_BY_GUARDRAIL. No PayPal order ID was created. This verifies actual model extraction while retaining independent server policy checks.

## Model-backed treasury query

Request:

> Find Mouser orders costing less than $600.

| Path | Actual returned model | Result |
| --- | --- | --- |
| Isolated API | dots-studio/dots-3-note-preview:free | Typed predicate `(vendor == "Mouser" && total_amount < 600.00)`; four matching records |
| Running application UI | xingchen-agi/xing4.0-29b-a4b:free | Same predicate and four records; OpenRouter/model parser label visible |

Both record sets were checked against independently calculated filters over their source ledgers. The running application stayed in **PayPal Sandbox** mode. Its completed/settled payment record and all 29 payment-history rows were unchanged by the query.

The different model IDs are expected router selections; they came from each provider response's model field. The router alias is not presented as the model that executed.

## Local regression checks

- cargo check --locked --offline: passed.
- cargo test --locked --offline: 34 passed.
- cargo build --locked --offline: passed.
- npm run lint: passed.
- npm run build: passed, including TypeScript checking.
- npm run verify:browser: seven passed with both model keys explicitly cleared.

Contract fixtures verify bearer authentication, JSON mode, provider parameter requirements, separate credentials, resolved-model metadata and unusable/error output handling. Analytics tests use immutable seed data and reject partially invalid predicate trees.

## Artifacts and limits

- `.verification/openrouter-live-results.json`: actual API checks, resolved models, guardrail result and unchanged original-ledger hash comparison.
- `.verification/openrouter-ui-results.json`: actual UI query, parser label, result check and unchanged payment history.
- `.verification/screenshots/openrouter-treasury-live.png`: running UI with the actual parser label and filtered ledger.
- `AI_SETUP.md`: configuration and reproduction instructions.

These checks establish the demonstrated requests, not universal extraction accuracy, guaranteed free-model availability, or production service reliability. PayPal capture/payout evidence is separate in SANDBOX_VERIFICATION.md. No new payment was initiated for the model verification.

After the UI redesign on the same date, actual treasury filtering was verified again. The successful retry used dots-studio/dots-3-note-preview:free, displayed the resolved model in the new interface and returned the same four independently checked records. The original payment history and settled purchase were unchanged. See UI_REDESIGN.md for the initial unverified attempt and the successful retry evidence.

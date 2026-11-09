# ProcureOps AI — Devpost draft

Prepared 2026-10-27. This is a local draft, not a submitted entry. Actual PayPal and OpenRouter verification have passed. Replace the remaining owner fields below before submission.

## Project name and tagline

**ProcureOps AI**

Policy-checked purchasing and auditable PayPal settlement for small hardware teams.

## Inspiration

A small engineering team needs components quickly, while its finance team needs purchasing limits, approved suppliers and a clear payment history. Natural-language requests make procurement easier to start, but each proposed purchase still needs independent policy checks and an accurate account of where its payment stands.

## What it does

A buyer describes a component, quantity, budget and delivery deadline. ProcureOps compares an illustrative supplier catalog and validates stock, quantity, delivery, supplier rating, vendor approval and spending limits. Rejected requests show the policy reason before an order is created.

Approved requests become PayPal Sandbox checkout orders. The buyer approves through PayPal JavaScript SDK v6, and the Rust backend verifies and captures the order. The AG Grid cockpit shows the ledger, payment states and natural-language treasury filters. A separate payout action sends a captured amount to an owned Sandbox receiver and reconciles the recipient item before displaying settlement.

The current prototype demonstrates the payment workflow using test accounts and catalog data. Supplier names do not imply connected sellers, and no physical products are purchased.

## How it is built

Rust, Axum, Tokio, Reqwest and Serde implement the backend. A private `ApprovedMandate` type requires the order-creation path to pass through runtime policy validation. Amounts and supplier policy remain server decisions. Stable operation identifiers, durable ledger reservations and recipient reconciliation protect payment retries.

Next.js, React, TypeScript and Tailwind provide the interface. AG Grid Community supplies the auditable ledger. PayPal OAuth, Orders v2, JavaScript SDK v6 and Payouts v1 form the verified payment flow. Webhook signature-verification code is implemented; genuine webhook delivery has not yet been verified.

## AI and development-tool disclosure

OpenRouter extracts mandate fields and translates treasury questions into a closed filter representation. With openrouter/free, actual verification selected nvidia/nemotron-3.5-super-vl:free for a non-preset purchasing request, dots-studio/dots-3-note-preview:free for an API treasury query and xingchen-agi/xing4.0-29b-a4b:free for the UI query. The request for two Raspberry Pi boards under $50 was parsed by the model and independently blocked by budget policy. The Mouser-below-$600 query returned the same four records as an independent filter calculation. See AI_VERIFICATION.md.

The final recorded purchase used recursive/flint-v1.0:free; the recorded budget rejection used nvidia/nemotron-3.5-super-vl:free; and the final treasury query used dots-studio/dots-3-note-preview:free. The application exposes the provider and returned model actually used. Deterministic parsing and clearly labeled fallback keep local demonstrations usable without credentials. Direct OpenAI is supported as an alternative but was not used for this verification.

Codex assisted with implementation, resource research, documentation and testing. The owner should add any other tools actually used. PayPal AI Toolkit, MCP, and the other sponsor runtimes are not claimed as integrated dependencies.

## Challenges and accomplishments

The payment lifecycle needed a visible separation between prepared orders, buyer approval, pending captures and confirmed payments. Sandbox account-region and review-setting behavior also required actual account testing. The current implementation preserves provider errors and pending outcomes.

Verification includes 35 passing Rust tests, 12 Chrome workflow checks, frontend lint and a clean production build. The interface was redesigned and verified on desktop, tablet and phone. On 2026-10-27, an actual 78.50 USD Sandbox capture completed and a matching payout was independently confirmed in the owned recipient account. A repeated payout request returned its existing batch. A second actual purchase and settlement were recorded for the demo. One earlier review-held capture remains pending and is recorded separately. See VERIFICATION.md, UI_REDESIGN.md and SANDBOX_VERIFICATION.md.

## What's next

Integrate authorized supplier discovery, verify real webhook delivery, and add authentication and transactional storage before public multi-user deployment. Real supplier purchasing needs seller integration and the appropriate payment authorization.

## Owner fields before submission

| Field | Required owner result |
| --- | --- |
| Public source | Actual repository URL with MIT LICENSE and complete setup instructions |
| AI evidence | Verified in AI_VERIFICATION.md; use the actual model labels from the final recorded demonstration |
| Judge access | A working local build or hosted application, with concrete free access instructions and any necessary dedicated test access |
| Demo video | Public YouTube URL; use DEMO_SCRIPT.md |
| Team | Confirm registration, eligibility, members and representative |
| Existing-project history | Describe actual work completed during the event, if applicable |

The owner must confirm that judges can run the demonstrated AI/payment experience without buying access; do not leave required credentials or access arrangements unspecified. Keep general-purpose secrets out of the public repository and video.

Submission is due November 12, 2026 at noon PST (November 13 at 1:30 a.m. IST). Maintain free judge access through December 15. A runnable repository can satisfy demo access; public hosting is optional. [Official rules](https://paypalaihackathon.devpost.com/rules), [submission requirements](https://paypalaihackathon.devpost.com/)

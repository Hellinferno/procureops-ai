# Judge guide — ProcureOps AI

ProcureOps AI turns a hardware purchasing request into a policy-checked PayPal workflow, with a treasury ledger for review. The catalog and 25 historical purchases are illustrative. No physical goods are ordered. All connected payments use PayPal Sandbox.

## Start the working build

Install current stable Rust and Node.js 24 LTS. From the repository root:

```powershell
# Terminal 1
Copy-Item backend/.env.example backend/.env
cd backend
cargo run --locked
```

```powershell
# Terminal 2, from the repository root
Copy-Item frontend/.env.example frontend/.env.local
cd frontend
npm ci
npm run dev
```

Open http://localhost:3000. Preserve existing environment files if you already configured the app. The default is explicitly labeled **Simulation**: no account, payment credentials or model key is required for the local workflow. Deterministic parsing is identified in the interface.

## Five-minute tour

1. In Create a purchase, use **Compliant order**, then **Prepare checkout**. Inspect the supplier comparison and policy decision. In simulation, explicitly approve the simulated payment. The ledger records the simulated capture.
2. Inspect the new purchase and use **PayPal Payout**. Check the payout again to reconcile it. Simulation references start with `SIM-`; they are distinct from genuine Sandbox evidence.
3. Use **Over budget** and prepare the request. The policy rejects it before a PayPal order is created. Repeat with **Unapproved supplier**.
4. In treasury, ask `Find Mouser orders costing less than $600.` Review the matched rows, parser label, filter and totals. An actual model appears only when an API key is configured and valid structured output is returned.
5. Search and sort the ledger, inspect an order, export CSV and try the phone layout. Historical samples have no payment actions.

## Reproduce the connected demonstration

The default build demonstrates the complete local workflow. Genuine model inference and genuine Sandbox approval require the separate credentials described below; simulation is not evidence of either service.

- Follow [PayPal setup](PAYPAL_SETUP.md). Set `PAYPAL_MODE=sandbox` and a Sandbox REST client ID/secret in `backend/.env`. Use a US Business test merchant with USD enabled, an owned Personal test buyer/receiver, and normal-success testing settings. Configure `PAYPAL_PAYOUT_RECEIVER_EMAIL` to that owned receiver. Restart the backend.
- Follow [AI setup](AI_SETUP.md). Set `LLM_PROVIDER=openrouter`, `OPENROUTER_API_KEY` and `LLM_MODEL=openrouter/free` on the server, then restart it. Free-router models and capacity can change; the UI displays the returned model or a clearly identified fallback.
- Request `Please source one Raspberry Pi 5 board from DigiKey, total spend under $100 and delivery within three days.` Set the form budget to 100. Prepare checkout, approve in the actual PayPal Sandbox window, then inspect the confirmed capture. Initiate the separate owned-recipient payout and check its status. Pending outcomes remain pending until confirmed.
- For an actual model policy test, request two Raspberry Pi boards under $50, with a form budget of 50. The server should reject the purchase independently of model interpretation.
- Ask the treasury question from the tour and inspect the actual provider/model provenance.

The owner must provide any dedicated judge test access through the submission's private testing instructions and keep it working free of charge through December 15, 2026. The public source deliberately contains no API keys or account passwords. If dedicated access has not been supplied, do not assume the connected demonstration is preconfigured. This remains an owner submission requirement.

## Evidence and limits

The working prototype was verified on Windows with Node 24.12.0, stable Rust, Chrome, and production Next.js output. See [verification](VERIFICATION.md), [actual Sandbox verification](SANDBOX_VERIFICATION.md), and [actual model verification](AI_VERIFICATION.md). A completed 78.50 USD capture and a successful owned-recipient payout were independently confirmed. An earlier review-held payment is preserved separately.

Orders v2, SDK v6 and Payouts are central integrations. Catalog suppliers are examples, not connected sellers. A payout is a separate disbursement from the test merchant to the owned test receiver. Webhook code is implemented, but genuine webhook delivery is unverified; manual reconciliation is available.

The ledger is a local JSON file. Run one backend per ledger and retain it across restarts. Public multi-user hosting requires authentication, authorization and transactional storage. Local judge use is the supported delivery for this submission.

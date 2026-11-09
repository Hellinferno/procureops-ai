# ProcureOps API contracts

The supported API is the Rust Axum backend, normally http://localhost:8000. Monetary amounts are calculated from server-side catalog quotes and validated against the tighter prompt/authorized budget. PayPal secrets and model keys remain server-side.

## Read and query

| Method / route | Contract |
| --- | --- |
| `GET /api/health` | Service identity, actual configured payment mode, Sandbox credential presence and model-key presence. Presence is not connectivity proof. |
| `GET /api/paypal/config` | `mode`, `configured`, public `client_id`, Sandbox `sdk_url`, `payouts_enabled`, `webhook_configured`, user-facing message. Never a secret or OAuth token. |
| `GET /api/catalog` | Illustrative catalog entries. |
| `GET /api/transactions` | `{transactions, kpis}`. Historical rows default to `payment_mode=seed`, `parser_mode=seed`. |
| `POST /api/analytics/query` | Request `{query}`; response `{query, filter_expr, summary, records, parser_mode}`. Executes a closed typed predicate AST. |

## Prepare an order

`POST /api/mandate`

```json
{
  "prompt": "Buy 10 Raspberry Pi 5 under $900 within 3 days",
  "authorized_budget": 900,
  "request_id": "d5de427c-963b-4cb1-8fe1-e33bb3b16047"
}
```

`request_id` is optional, but clients should generate one UUID per intended purchase and reuse it for retries. The server serializes operations and stores the key; a duplicate returns the existing transaction.

Response: `{success, transaction, steps, paypal_raw}`. `success=true` means a checkout order was prepared; it does not mean money was captured. Eligible orders become `AWAITING_APPROVAL`; a rule violation becomes `BLOCKED_BY_GUARDRAIL`. No matching available item returns `success=false` with `transaction=null`. A real order-creation error is persisted as `PAYMENT_FAILED` with its error and no fabricated PayPal ID.

## Buyer approval, capture and cancellation

The frontend initializes the Sandbox JavaScript SDK v6 using the public client ID, checks PayPal eligibility, and starts a one-time payment session with the stored PayPal order ID. The SDK `onApprove` callback submits:

`POST /api/paypal/capture` with `{order_id: LOCAL_ORDER_ID}`.

The server derives the PayPal ID and amount from the ledger, checks that the stored order is approved at PayPal, uses a stable `PayPal-Request-Id`, and verifies the returned custom reference, amount, currency and capture status. Response: `{success, transaction, paypal_raw, message}`. Confirmed captures become `COMPLETED`; pending or ambiguous outcomes remain `PAYMENT_PENDING`. Retrying a completed capture returns the existing result without a second charge.

- `POST /api/paypal/payment/status` with `{order_id}` re-fetches provider details without initiating another payment.
- `POST /api/paypal/cancel` with `{order_id}` marks an uncaptured local checkout `CANCELLED`. Captured/blocked/cancelled orders cannot be captured again through this path.

Simulation uses the same ledger lifecycle with an explicitly labeled simulated approval button; it never calls PayPal.

## Vendor payout

`POST /api/paypal/payout` with `{order_id}`. Only a newly captured, guardrail-approved order in the matching execution mode is eligible. Seed records cannot initiate payouts. Sandbox requires `PAYPAL_PAYOUT_RECEIVER_EMAIL` to identify an owned test recipient.

The operation reserves and persists a stable sender batch ID before calling PayPal. Duplicate/concurrent requests do not create additional payouts. A batch ID produces `PAYOUT_PENDING`, not `SETTLED`. The response is `{success, order_id, payout_batch_id, new_status, message}`; `success` is true only for settlement confirmed through reconciliation.

`POST /api/paypal/payout/status` with `{order_id}` retrieves the known batch and checks the matching item. `SUCCESS` with matching sender reference, amount and currency yields `SETTLED`; `UNCLAIMED`, held, and processing items stay pending. Failed item statuses become `PAYOUT_FAILED`. Do not start a new payout to resolve an unknown outcome; retry/reconcile the reserved operation.

## Webhooks

`POST /api/paypal/webhook` accepts PayPal event JSON and its five signature headers. Sandbox credentials and `PAYPAL_WEBHOOK_ID` are required. The backend asks PayPal to verify the signature; unverified events receive 401. Capture/payout notifications trigger authoritative re-fetch and reference/amount checks. Repeated notifications are safe because updates are idempotent and never initiate another capture or payout.

## Reset, errors and operational constraints

`POST /api/reset` restores sample data only when no Sandbox records would be erased. Sandbox mode refuses reset.

Errors use `{detail}` with 400 (invalid request), 404 (unknown order), 409 (ineligible lifecycle/mode), 502 (provider failure/unconfirmed result), or 500 (persistence failure). Capture/payout reservations are durably saved before money-moving requests. Ledger mutations use one operation lock and atomic file replacement; run one process per ledger file. Production PayPal URLs are rejected. CORS permits configured `FRONTEND_ORIGINS`; default backend binding is loopback.

The local demo is not an authenticated public multi-user service. Add authorization and deployment-grade storage before public exposure.

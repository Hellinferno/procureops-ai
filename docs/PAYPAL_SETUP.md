# PayPal Sandbox setup and verification

Finish the local build first. Then follow these steps; no coding or real payment is required to obtain test credentials. Keep the client secret and test-account passwords in local files/password storage, not in chat, screenshots, or the public repository.

## 1. Open the Developer Dashboard

Visit [PayPal Developer](https://developer.paypal.com/) and choose **Log in to Dashboard**. Use your normal PayPal login here. Select **Sandbox**, not Live. [Official credential instructions](https://developer.paypal.com/api/rest/)

## 2. Find or create the Sandbox app

Open **Apps & Credentials**. Select the default Sandbox REST app, or choose **Create App** and name it `ProcureOps AI`. Associate it with your Sandbox Business merchant. Copy **Client ID** and reveal/copy **Secret**. These are application credentials, not the buyer's email/password.

## 3. Put the two values in the backend file

Open `D:\paypal\backend\.env`. If it does not exist, copy `backend/.env.example` to `backend/.env`. Set:

```ini
PAYPAL_MODE=sandbox
PAYPAL_CLIENT_ID=your_actual_sandbox_app_client_id
PAYPAL_CLIENT_SECRET=your_actual_sandbox_app_secret
PAYPAL_BASE_URL=https://api-m.sandbox.paypal.com
```

Replace those two example values with the dashboard values. Keep both in the backend file. The browser gets only the public client ID from the backend configuration endpoint.

Restart `cargo run --locked` from `D:\paypal\backend`. Open http://localhost:3000. The header should say **PayPal Sandbox** and the checkout panel should say buyer approval is required. Configuration is not proof of a successful payment.

## 4. Get the test buyer login

In the dashboard, open **Testing Tools -> Sandbox Accounts** (the navigation may label this **Sandbox -> Accounts**). Find the generated **Personal** account and use its **View/Edit Account** menu to obtain its email and password. Ensure it has enough test USD balance for the $785 demo. The **Business** account receives the capture. Generated test logins work on sandbox.paypal.com, not your normal PayPal account. [Official Sandbox walkthrough](https://github.com/paypaldev/getting-started-with-paypal-sandbox)

## 5. Verify checkout in the app

1. Select **Preset 1: Compliant Order** and execute it.
2. Check that the row starts at `AWAITING_APPROVAL`, not `COMPLETED`.
3. In **Buyer approval & payment**, click the official PayPal button. Log in with the Sandbox Personal buyer and approve.
4. Expect `COMPLETED` plus a capture ID. Open the Sandbox Business account's activity page and confirm the same amount/capture reference.
5. Repeat a purchase and cancel in the PayPal window. It should become `CANCELLED` and have no capture ID.
6. Run the budget-breach and rogue-supplier presets. They must be blocked without a PayPal order ID.

The buyer pays the ProcureOps test merchant. Supplier names/catalog quotes are illustrative; this does not buy goods from DigiKey or other listed companies. Unattended purchasing would require a separately authorized payment arrangement.

## 6. Optional: verify vendor payouts

Create another owned Sandbox Personal or Business account as a test recipient. Set its generated test email as `PAYPAL_PAYOUT_RECEIVER_EMAIL` in `backend/.env` and restart the backend. Confirm Payouts access is available for the Sandbox business/app in the dashboard; an authorization error needs account capability setup, not simulation fallback.

On a newly captured Sandbox row, click **PayPal Payout**. Expect `PAYOUT_PENDING`. Click **Check payout** until the recipient item succeeds, then expect `SETTLED`. Confirm receipt using the recipient's Sandbox activity page. Repeated clicks must not create additional payouts. All illustrative vendor emails are ignored for real API payouts; the configured owned test receiver is used.

## 7. Optional: register webhooks

When an HTTPS listener is available, register `https://YOUR-HOST/api/paypal/webhook` under the Sandbox app. Subscribe to `PAYMENT.CAPTURE.COMPLETED`, `PAYMENT.CAPTURE.PENDING`, and relevant `PAYMENT.PAYOUTS-ITEM`/`PAYMENT.PAYOUTSBATCH` events. Copy its webhook ID into `PAYPAL_WEBHOOK_ID` and restart. Webhooks are signature-verified and reconciled against PayPal. Dashboard mock webhook simulator events are not supported by PayPal's postback-verification method; use actual Sandbox app events for this acceptance check. [Official webhook guide](https://developer.paypal.com/api/rest/webhooks/rest/)

Local manual refresh works without webhooks. Public deployment requires authenticated access to payout/reset actions before exposing this demo to multiple users.

## Troubleshooting

- **Missing credentials:** confirm `backend/.env`, Sandbox mode, and restart; the Rust process loads that file explicitly.
- **Invalid client:** verify the app is a Sandbox app and both values come from the same app.
- **UNSUPPORTED_PAYEE_CURRENCY:** Authentication and order creation may succeed, but the linked Business Sandbox merchant cannot receive the order currency. This app uses USD. Enable USD receiving on that test merchant. If USD is unavailable for its region, create a US Business Sandbox test account and a Sandbox REST app linked to it, then update the app credentials locally. The Personal buyer account is separate; changing its password does not resolve a merchant currency restriction. [PayPal explanation](https://developer.paypal.com/api/errors/unsupported-payee-currency/)
- **No PayPal button:** check SDK loading, browser network/popup restrictions, and Sandbox client ID.
- **ORDER_NOT_APPROVED:** finish buyer approval before capture. Do not repeatedly submit another purchase.
- **PENDING_REVIEW:** a capture ID can exist while the capture itself remains pending, even when the enclosing order says COMPLETED. Check the Sandbox Business account in Developer Dashboard -> Testing Tools -> Sandbox Accounts -> View/Edit Account -> Settings. For an ordinary success test, turn off Payment Review and Negative Testing if enabled. Reconcile the existing capture first; changing test settings does not establish that an existing hold has cleared. The application keeps PAYMENT_PENDING until PayPal confirms capture completion. [PayPal technical-support guidance](https://www.paypal-community.com/t5/Sandbox-Environment/Sandbox-Smart-Button-Your-payment-is-on-hold/td-p/3159217)
- **Declined payment:** inspect the buyer's test balance/funding and PayPal debug ID. Errors never become simulated success.
- **Pending capture/payout:** refresh the same order's status; retain its IDs for reconciliation.
- **Payout unclaimed:** log in to the configured owned recipient and inspect its Sandbox account/activity.

Real Sandbox verification is complete only when the capture and, if used, recipient payout are visible in the relevant Sandbox accounts. Local mock-provider tests are separate evidence.

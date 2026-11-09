# Credentialed PayPal Sandbox verification

This is an actual Sandbox connection attempt, separate from the local HTTP fixtures and stub SDK test. User-provided account secrets were kept out of source files, logs, and this report. The buyer password was supplied to a private browser process in memory with terminal echo disabled.

An initial recheck on 2026-10-27 restarted the local services and confirmed OAuth with the previously saved credentials. The completed US-account test below supersedes that initial connection check. An owned US Personal Sandbox payout recipient is configured. OpenRouter was subsequently configured and verified separately; see AI_VERIFICATION.md.

## Completed US checkout and payout — 2026-10-27

An additional actual transaction was recorded for the final submission video: local order ORD-4ed59f73c71b4caf9404, confirmed 78.50 USD capture 8V126807U41618146, payout batch XXU6Y4X7G6DP8, final state SETTLED after successful recipient-item reconciliation. The actual approval popup and workspace footage show this same purchase. Earlier verification and payment history were retained. See RELEASE_REPORT.md and evidence/sandbox-rehearsal.json.

The owner confirmed that Sandbox Payment Review and Negative Testing had been turned off. A status check showed that the earlier capture was still pending, so it was preserved and one new 78.50 USD test order was created. The generated Sandbox Visa funding option was selected. Actual SDK buyer approval and the server-side capture completed successfully. A direct PayPal API read verified capture status COMPLETED, the amount/currency, the US merchant ID and the US payer ID. The merchant transaction page separately confirmed the same capture, local order reference and amount.

The owned US Personal Sandbox account was configured as PAYPAL_PAYOUT_RECEIVER_EMAIL. One payout was initiated from the corresponding captured row in AG Grid. Its first state was PAYOUT_PENDING; **Check payout** reconciled the successful matching recipient item and changed the application record to SETTLED. The Personal account's transaction page confirms receipt of 78.50 USD and the note containing the same local order ID. This uses the owner's Personal test account as an illustrative supplier receiver; no catalog supplier is connected.

| Check | Verified result |
| --- | --- |
| Local order | ORD-6796f90b944d465f86e2 |
| PayPal order | 0J805805DU264953K |
| Completed capture | 67A362971J280231P |
| Captured amount | 78.50 USD |
| Merchant activity | Completed, matching capture and local order reference |
| Payout batch | W6A2KVLEXRW8N |
| Payout batch / recipient item | SUCCESS / SUCCESS; receiver, sender reference, amount and currency matched |
| Provider payout transaction | 0S237586MD5476646 |
| Recipient account transaction | 2DY273088N826983V; Payment received, +78.50 USD, matching settlement note |
| Final application state | SETTLED |
| Repeat payout request | Returned the existing settled batch; no new payout was initiated |
| Earlier reviewed capture | 7DY8396289087864E remains PAYMENT_PENDING; it was not recaptured, cancelled or marked complete |

Sanitized evidence is in `.verification/sandbox-us-completed-results.json`. Screenshots are `.verification/screenshots/sandbox-us-completed.png`, `sandbox-merchant-completed.png`, `sandbox-us-settled.png` and `sandbox-recipient-received.png`. Payment credentials remain in the ignored backend/.env; login passwords and OAuth tokens were not persisted. Local services remain available at localhost:3000 and localhost:8000. Genuine webhook delivery remains unverified. Actual model-provider inference was subsequently verified separately in AI_VERIFICATION.md.

## Earlier US account test with review enabled — 2026-10-27

The owner supplied additional US Business and Personal test accounts and Sandbox REST app `ttyy`. Its credentials were saved only in the ignored backend/.env, and the backend was restarted. OAuth succeeded with HTTP 200. The provider order's merchant ID matched the supplied Business account, and the payer ID matched the supplied Personal account. Account countries were confirmed in the owner's screenshots.

The real SDK v6 opened checkout for **78.50 USD**. PayPal initially displayed a generic `RETRY` error after login; its own Try again link reached the approval screen for the same order. Buyer approval then invoked the application's server-side capture endpoint.

| Check | Result |
| --- | --- |
| Local order | ORD-06c6a4cc1569458c8949 |
| PayPal order | 2YP83719W7208831L |
| PayPal capture | 7DY8396289087864E |
| Provider order status | COMPLETED |
| Capture status | PENDING, reason PENDING_REVIEW |
| Local payment status | PAYMENT_PENDING |
| Merchant activity | Matching capture ID, custom local order ID and 78.50 USD, marked Pending / On hold |
| Completed capture | Not yet verified; order COMPLETED does not override a pending capture |
| Payout | Not initiated; capture is not eligible |
| Pending-payment payout guard | Verified: HTTP 409, no reserved sender ID and no payout batch |

The merchant transaction page did not expose an accept/review control. The owner was asked to check the Sandbox Business account's **Payment Review** and **Negative Testing** settings in the Developer Dashboard and turn them off for the ordinary success test if enabled. The subsequent confirmation and completed test are recorded above. The existing hold was not cleared by that settings change; it remains pending and must not be marked completed locally.

Sanitized evidence: `.verification/sandbox-us-results.json`. Screenshots: `.verification/screenshots/sandbox-us-pending.png` and `.verification/screenshots/sandbox-merchant-pending.png`. App secrets, passwords and OAuth tokens are excluded.

## Historical India account attempts — 2026-10-26

| Check | Result |
| --- | --- |
| Existing app credentials | Both values present; preserved locally in backend/.env |
| Execution mode | Set PAYPAL_MODE=sandbox; the old USE_PAYPAL_SANDBOX=false flag had left the app in simulation |
| OAuth client authentication | HTTP 200 with an access token; token not printed or persisted |
| Payout capability scope | Advertised in the OAuth scope; this does not establish payout-account eligibility |
| Local backend and frontend | Healthy; available at localhost:8000 and localhost:3000 |
| Real hosted JavaScript SDK | Loaded; initialized and rendered its actual PayPal button |
| Real Orders v2 create | Accepted; order 40112858GP1867134, amount 78.50 USD |
| Local transaction | ORD-aa7778b8f71c458da250 |
| Hosted PayPal checkout | Opened; submitted the provided Personal Sandbox login |
| Approval | PayPal redirected to its generic error page; decoded error was UNSUPPORTED_PAYEE_CURRENCY |
| Capture | Not performed or confirmed; provider order remained CREATED, with no captures |
| Close/cancel callback | Verified against the real SDK; closing the failed popup changed the local checkout to CANCELLED |
| Vendor payout | Not initiated; no eligible capture and no configured owned recipient |
| Business test-account login | Verified with the owner-provided Sandbox login |
| Personal test-account login | Verified; account profile displays India |
| Business test-account country | IN in the merchant dashboard metadata |
| Business USD balance | Already present; this is not proof that the checkout corridor is supported |
| Receiving preference | ASK_ME. Attempted YES/automatic conversion, but reload showed that the change did not persist |
| Second actual checkout attempt | Order 90K31182GY9649349, 78.50 USD; same UNSUPPORTED_PAYEE_CURRENCY, no capture |

## Historical account diagnosis and recommendation

Both supplied Sandbox accounts were inspected and are India-based. The Business account already holds USD. Two actual checkout attempts returned UNSUPPORTED_PAYEE_CURRENCY. The receiving-preference page did not persist a change from ASK_ME to automatic acceptance; no successful settings change is claimed. The exact provider-side restriction has not been established beyond this currency error and the observed account regions.

PayPal supports international payments for India; an India buyer/India merchant pair is therefore unsuitable for a domestic test. [PayPal India payment scope](https://www.paypal.com/in/digital-wallet/paypal-consumer-fees), [PayPal currency error explanation](https://developer.paypal.com/api/errors/unsupported-payee-currency/)

For a predictable USD hackathon demo, create two **additional Sandbox test accounts** under Developer Dashboard -> Testing Tools -> Sandbox Accounts: Business / United States and Personal / United States. Create a **Sandbox** REST app linked to that new Business test account, replace the app Client ID and Secret in backend/.env, and keep PAYPAL_MODE=sandbox. This does not change the owner's real account country or use live money. Creating accounts/apps in the Developer Dashboard requires the owner's real Developer Dashboard login; generated Sandbox logins do not provide that administrative access. [Sandbox account management](https://developer.paypal.com/sandbox-testing/accounts)

After the keys are updated, restart the backend, repeat buyer approval/capture, and verify the capture ID. An owned compatible recipient can then be used to test payouts. Failed test orders should stay uncaptured/cancelled; do not mark them completed.

Sanitized machine-readable evidence is saved in `.verification/sandbox-connection.json` and `.verification/sandbox-live-results.json`. No password or access token is included. No production PayPal endpoint was used.

# ProcureOps AI — 170-second demo plan

The finished local video is `submission/media/ProcureOps-AI-demo.mp4`, 159.55 seconds (about 2:40), with English synthetic narration and matching captions. It shows genuine application and PayPal Sandbox footage; waiting time is shortened and frames are held for explanation. See RELEASE_REPORT.md for the recorded capture, settlement, actual AI models and verification limits. The public YouTube upload remains an owner step. The original rehearsal plan below is retained for an optional new recording.

| Time | Screen and narration |
| --- | --- |
| 0:00–0:20 | Cockpit. "Small hardware teams need components quickly, while finance needs approved suppliers and spending limits. ProcureOps turns a purchasing request into a policy-checked, auditable payment workflow." |
| 0:20–0:45 | Enter a non-preset mandate for one Raspberry Pi under $100 with a three-day deadline. Show the actual model parser label, extracted fields, supplier comparison and Rust policy result. Explain that model interpretation is independently validated. |
| 0:45–1:20 | Show AWAITING_APPROVAL, the real PayPal Sandbox approval window, then COMPLETED and the capture reference. Explain the buyer-to-test-merchant flow. Cut waiting time if needed; retain the approval action and outcome. |
| 1:20–1:45 | Initiate the owned-recipient payout, show PAYOUT_PENDING, select Check payout, then show SETTLED. Show the recipient's matching amount and settlement note. This is a separate disbursement, not a purchase from a catalog supplier. |
| 1:45–2:05 | Execute a budget-breach or unapproved-supplier request. Show the policy rejection and absence of a PayPal order ID. |
| 2:05–2:30 | Ask "Show all blocked orders or orders over $500." Show actual AI parser provenance, the typed filter, matching ledger rows and summary. |
| 2:30–2:50 | Show the order audit details and summarize: Rust policy validation before order creation, buyer approval, verified captures and recipient reconciliation. Explain that catalog data and historical rows are illustrative. |

Rehearse once with the final published build and actual model credentials before recording. If PayPal leaves an operation pending, show that status honestly and use a separate verified completed example for the success demonstration. Preserve existing payment records.

After recording, check that the video is under three minutes, upload it publicly to YouTube and add its actual URL to Devpost. Use English narration or provide an English translation. [Official video requirements](https://paypalaihackathon.devpost.com/rules)

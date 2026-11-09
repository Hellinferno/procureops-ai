# Hackathon resource integration matrix

Reviewed 2026-10-26 against [the supplied resource index](https://paypalaihackathon.devpost.com/resources). This inventory covers its 34 primary entries: seven core resources, eight payment capabilities, five AI resources, ten sponsors, and four recordings. It distinguishes implementation from optional possibilities; listing a resource does not mean it was integrated. Inspiration categories (agents, money movement, business tools, consumer experiences, social impact) were also reviewed; ProcureOps fits procurement and business automation.

## Core resources supplied by the user

| Resource | Decision and implementation |
| --- | --- |
| [Developer portal](https://developer.paypal.com/) | Account/app management. The tracking-parameter URL failed to open; the canonical portal was accessible. Owner login is required. |
| [Payments](https://developer.paypal.com/payments/) | Product selection reference; this product needs one-time Checkout and separate payouts. |
| [Checkout](https://developer.paypal.com/checkout/) | Implemented create -> buyer approval -> capture. Reviewed the linked [quickstart](https://developer.paypal.com/checkout/integrate/). |
| [JavaScript SDK v6](https://developer.paypal.com/sdk/js/set-up) | Implemented client-ID initialization, eligibility, payment session, approval, cancellation and error callbacks. |
| [REST APIs](https://developer.paypal.com/api/rest/) | Server-only OAuth/client secret, Orders v2 and Payouts v1. Real failures are preserved. |
| [Developer Dashboard](https://developer.paypal.com/dashboard/) | Needed for app credentials, test accounts and webhook registration. No account access was claimed. |
| [Sandbox walkthrough](https://github.com/paypaldev/getting-started-with-paypal-sandbox) | Reviewed buyer/seller roles, test balances, dashboard vs Sandbox logins. Setup guide is in PAYPAL_SETUP.md. |

## Other payment capabilities linked in Resources

| Resource | Applicability |
| --- | --- |
| [Low-code/no-code](https://developer.paypal.com/payment-links-buttons/overview) | Payment links/buttons and invoices are alternatives; the custom approval flow uses REST and SDK v6. |
| [Pay Later](https://developer.paypal.com/pay-later/overview) | Optional financing; not needed for policy-guarded one-time procurement. No eligibility claims or financing integration added. |
| [Subscriptions](https://developer.paypal.com/platforms/subscriptions) | Relevant to recurring vendor contracts later; outside this one-time purchasing workflow. |
| [Invoicing](https://developer.paypal.com/invoicing/about) | Useful future accounts-payable workflow. Not represented as implemented. |
| [Payouts](https://developer.paypal.com/payouts/overview) | Implemented stable sender-batch IDs, pending state and recipient-item reconciliation. Requires an owned Sandbox receiver. |
| [Disputes](https://developer.paypal.com/platforms/disputes/overview) | Potential post-purchase operations. Current capture IDs provide references; no dispute automation was added. |
| [Donations](https://developer.paypal.com/sdk/donate/) | Reviewed; not relevant to purchasing. |
| [Webhooks](https://developer.paypal.com/api/webhooks/overview) | Implemented authenticated callback handling with authoritative re-fetch. Reviewed [signature verification](https://developer.paypal.com/api/rest/webhooks/rest/). |

## AI and agent resources

The original docs.paypal.ai links redirected to inaccessible pages in this browsing environment. The corresponding current official developer guides were located and reviewed instead.

| Resource | Applicability |
| --- | --- |
| [PayPal AI Toolkit](https://github.com/paypal/AI-Toolkit) | Developer tooling for API/MCP context. Reviewed; not installed or claimed as a runtime dependency. |
| [AI documentation](https://developer.paypal.com/ai-tools/get-started) | Reviewed current agentic integration options. Existing model extraction plus guarded Rust REST calls is the application architecture. |
| [Agent Toolkit quickstart](https://developer.paypal.com/ai-tools/toolkit/) | TypeScript/Python tool wrappers are optional. Calling the Orders API through Rust preserves the existing private ApprovedMandate boundary. |
| [MCP quickstart](https://developer.paypal.com/ai-tools/mcp-server) | Optional connection for MCP clients. No unguarded payment tools were exposed to the model. |
| [Agent tools reference](https://developer.paypal.com/ai-tools/agent-tools/) | Tool catalog reviewed through the official guide/reference link. Future invoice/dispute assistants could use it; not a prerequisite for the current REST integration. |

## All ten sponsors

| Sponsor page | Product fit and decision |
| --- | --- |
| [AG Grid](https://paypalaihackathon.devpost.com/details/aggrid) | Community ledger already integrated. Added lifecycle-aware actions, seed restrictions, accurate edition labels, and corrected action/row event behavior. Sponsor guidance highlights AG Studio and custom agent dashboards as a possible next enhancement. |
| [APIMatic](https://paypalaihackathon.devpost.com/details/apimatic) | API-context development tooling; useful to developers, not required at runtime. Resource-info and offer links are on its sponsor page. Not installed or claimed. |
| [Astropods](https://paypalaihackathon.devpost.com/details/astropods) | Agent hosting/runtime and observability. Optional alternative deployment architecture; existing Rust runtime retained. |
| [Bryntum](https://paypalaihackathon.devpost.com/details/bryntum) | Scheduling/Gantt/calendar. Could show delivery planning later; not needed for checkout. |
| [Channel3](https://paypalaihackathon.devpost.com/details/channel3) | Most relevant future supplier discovery integration: real product search and availability. Needs a separate API key, product-to-policy mapping and seller payment arrangements. Current catalog remains explicitly illustrative. |
| [Elastic](https://paypalaihackathon.devpost.com/details/elastic) | Search/retrieval and agent tools could support a larger catalog. Current in-memory dataset does not require a search cluster. |
| [KERNEL](https://paypalaihackathon.devpost.com/details/kernel) | Cloud agent browsers. Not used to automate payment consent; approval uses PayPal's supported SDK. Browser verification uses local Chrome. |
| [Postman](https://paypalaihackathon.devpost.com/details/postman) | Added an importable local API collection for reproducible testing. No external workspace was modified. |
| [Render](https://paypalaihackathon.devpost.com/details/render) | Optional hosting. Local bind, allowed origins and data directory are configurable; public hosting still needs authentication and durable deployment storage. |
| [Zapier](https://paypalaihackathon.devpost.com/details/zapier) | Optional business-app automation. Not needed to complete the checkout flow; no outbound third-party actions configured. |

## Four recordings

| Listed recording | Review status |
| --- | --- |
| [October 6: Sandbox and AI Toolkit walkthrough](https://www.youtube.com/watch?v=FFJfCW9z3Do) | Resolved redirect and checked metadata. Video/transcript was not accessible; used the written official Sandbox/AI guides. |
| [October 7: APIMatic Context Plugins](https://www.youtube.com/watch?v=zRWixN68tDg) | Metadata accessible; no readable transcript. Reviewed APIMatic sponsor instructions instead. |
| [October 12: Payments dashboard](https://www.youtube.com/watch?v=LGfG44i1Fj8) | Fetch failed. Reviewed AG Grid sponsor guidance; recording contents were not verified. |
| [October 13: Start building with PayPal](https://www.youtube.com/watch?v=ck_zyB66p_Y) | Metadata accessible; no readable transcript. Checkout SDK/API guides supply the integration contract. |

## What this means for completion

Core implementation: SDK v6 approval/cancel/error handling, server-side Orders v2, explicit simulation, payout reconciliation, webhook verification, policy enforcement, parser provenance and local API/browser checks. Actual Sandbox buyer approval, completed capture, successful payout and recipient receipt were verified on 2026-10-27; see SANDBOX_VERIFICATION.md. Actual OpenRouter extraction and treasury queries also passed; see AI_VERIFICATION.md. Genuine webhook delivery remains unverified. Optional sponsor products and unrelated PayPal capabilities are not part of the acceptance gate.

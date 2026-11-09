# ProcureOps workspace UI

Updated 2026-10-27 after the owner's request for a complete visual redesign. This replaces the original dark cockpit specification.

## Design direction

A light purchasing workspace with a navy navigation rail, cobalt actions, locally hosted Manrope variable type and restrained financial status colors. The main work is preparing purchases and understanding their payment state. Technical references belong in inspection details.

Branding uses an original angular P with an approval check and a routed negative-space counter. ProcureMark.tsx renders the scalable sidebar mark; app/icon.svg supplies the matching browser icon. Reusable primary/inverse SVG assets are in public/brand. The wordmark fits the full rail, while the mark alone identifies the compact rail.

Reference research: [Ramp spend management](https://ramp.com/spend-management) for financial hierarchy and [Linear](https://linear.app/) for navigation and spacing. Applied [Anthropic's frontend-design skill](https://github.com/anthropics/skills/tree/main/skills/frontend-design), downloaded into an ignored workspace folder for this task. The interface uses original components and assets. Manrope's OFL license is included alongside its local font file, so builds make no external font requests.

## Workspace structure

- Sidebar: Overview, All purchases, Needs approval and Payouts operate on the actual ledger. Policy controls, Connections and Help open useful native dialogs.
- Topbar: current view, actual Sandbox/simulation/disconnected mode, help and workspace controls.
- Summary: one metric strip for captured spend, blocked attempted spend, policy-approved records and average risk. No invented growth percentages or decorative sparklines.
- Workbench: real spend-by-date chart next to a compact natural-language purchase composer. Chart values come from captured records, including pending/failed disbursements of captured funds; sample history is disclosed.
- Ledger: full-width AG Grid with supplier, item/reference, amount and budget, lifecycle status, policy decision, date and actions. Latest records sort first. Search includes complete order references as well as product/supplier values.
- Inspector: native side drawer with order details, authoritative policy reason, individual checks, mode, model and payment references. Historical sample rows remain read-only for payment actions.

## Purchase flow

Examples fill the request without executing it. **Prepare checkout** runs the existing extraction, catalog and policy flow. A request outcome exposes a collapsible decision trail and optional raw payment response.

The payment review panel appears for an actual selected/prepared non-seed transaction. Buyer approval, cancellation, capture/status refresh, PayPal SDK v6 initialization and provider-error handling retain their API contracts. Simulation approval is explicitly labeled. A payout stays pending until provider reconciliation confirms its recipient item; repeated actions use the same operation.

## Ledger and analytics

View tabs filter the loaded lifecycle states. Treasury search uses the existing typed model-filter endpoint and displays actual parser provenance. The interpreted predicate is inspectable without occupying the default view. CSV export includes payment mode and transaction references. Refresh reloads data; restoring samples is available only from the simulation workspace's Connections panel.

Status wording distinguishes Captured, Settled, Policy blocked, Needs approval, Payment pending, Payment failed, Cancelled, Payout pending and Payout failed. The original enum is retained in status metadata and exports. A passed policy check is not presented as completed payment.

## Responsive and accessible behavior

Desktop uses a full rail; compact desktop uses named icon controls. Phone/tablet navigation opens from a labeled menu. The purchase form comes before the chart on a phone. Screens up to 900px use purchase cards with search, pagination, detail inspection and payment actions; larger screens use AG Grid with internal scrolling when needed. Both views show recent purchases first.

Use visible keyboard focus, associated form labels, keyboard-operable ledger tabs, Enter-to-inspect grid cells, native dialog focus/escape behavior, reduced-motion support and helpful loading/error/empty states. External credential forms and secrets are outside the application UI.

Financial totals include historical examples where disclosed. Catalog suppliers remain illustrative and are not connected merchants. The mode badge reflects actual runtime configuration; a configured provider key is not a guarantee of provider availability.

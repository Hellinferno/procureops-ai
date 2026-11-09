# Technology Stack & Dependencies (Rust + Next.js)

## 1. Backend Stack (`/backend` — Rust 2021 Edition)
- **Web Framework:** `axum` (`0.7`) + `tokio` (`1`, full features) + `tower-http` (`0.5`, `cors`, `trace`)
- **HTTP Client:** `reqwest` (`0.12`, features: `["json", "rustls-tls"]`)
- **Serialization:** `serde` (`1.0`, derive) + `serde_json` (`1.0`)
- **Date/Time & IDs:** `chrono` (`0.4`, serde) + `uuid` (`1`, v4, serde)
- **Config & Env:** `dotenvy` (`0.15`) + `tracing` + `tracing-subscriber`
- **Deterministic Parsing:** `regex` (`1`)

### `backend/Cargo.toml`
```toml
[package]
name = "procureops-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1.38", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.10", features = ["v4", "serde"] }
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
regex = "1"
```

## 2. Frontend Stack (/frontend)
* **Framework:** Next.js 16.4.3 (App Router, TypeScript)
* **Styling:** Tailwind CSS 4 + @tailwindcss/postcss
* **Data Grid:** ag-grid-react (^36.2.0) and ag-grid-community (^36.2.0)
* **Icons:** lucide-react

## 3. Configuration and verification

Use `backend/.env.example` for server values and `frontend/.env.example` for `frontend/.env.local`. The Rust server loads `backend/.env` explicitly. Frontend API URLs are compiled from `NEXT_PUBLIC_API_BASE_URL`; changing that value requires rebuilding production output.

Actual frontend: Next.js 16.4.3, React 19.3.0, AG Grid Community/React 36.2.x, Tailwind 4 with the official PostCSS plugin, and Playwright for verification. Node >=20.9.0 is required by the installed Next.js package; Node 24.12.0 was used here. `npm run dev` and `npm run build` use the supported webpack path because Turbopack subprocess startup failed on this Windows environment. The Manrope variable font is hosted locally with its OFL license and has no build-time network dependency.

PayPal uses JavaScript SDK v6 in the browser and Reqwest REST calls from Rust. `PAYPAL_MODE=simulation|sandbox` controls execution. Production PayPal endpoints are not accepted. `PAYPAL_PAYOUT_RECEIVER_EMAIL` selects an owned test receiver; `PAYPAL_WEBHOOK_ID` enables signature verification for registered Sandbox events.

The shared llm.rs client supports OpenRouter and direct OpenAI. Select LLM_PROVIDER, its server-only API key and LLM_MODEL; OpenRouter defaults to openrouter/free. Both extraction and analytics record returned model metadata and preserve deterministic fallback. See AI_SETUP.md.

`npm run verify:browser` requires installed Google Chrome and a built Rust binary. It starts isolated simulation servers, tests the UI and SDK callback contract, and writes artifacts under `.verification/`.

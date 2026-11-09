# OpenRouter setup and model verification

ProcureOps supports OpenRouter for both purchasing-intent extraction and natural-language treasury filters. The Rust backend owns the provider connection. Its policy checks remain independent of model output.

## Configure the provider

1. Sign in to [OpenRouter](https://openrouter.ai/) and create a key on [API Keys](https://openrouter.ai/settings/keys).
2. In `D:\paypal\backend\.env`, set:

```ini
LLM_PROVIDER=openrouter
OPENROUTER_API_KEY=your_actual_openrouter_key
LLM_MODEL=openrouter/free
```

3. Keep the key in this ignored backend file and restart the backend. Do not add it to a frontend file, public repository or screenshot.

`openrouter/free` selects an available free model with the requested capabilities. Availability, capacity and rate limits can affect a call. The application records the model returned by OpenRouter, rather than presenting the router alias as the model that executed. You can set LLM_MODEL to another OpenRouter model slug if desired; inspect that model's availability and cost in your account first. [Free router documentation](https://openrouter.ai/docs/guides/routing/routers/free-router)

The implementation sends JSON-mode chat requests to `https://openrouter.ai/api/v1/chat/completions`, uses bearer authentication and requires a provider that supports the requested parameters. Both mandate and analytics requests use the same provider client. Requests have a 45-second timeout, and OpenRouter output is capped at 4,096 tokens. [API quickstart](https://openrouter.ai/docs/quickstart), [provider routing](https://openrouter.ai/docs/guides/routing/provider-selection)

## Verify actual use

First test the model with simulation payments and a separate data directory. For example, request two Raspberry Pi boards within three days under $200, then ask a non-preset treasury question. Successful provider responses should show `openrouter:<returned-model>` in the mandate trace and query result. A true health configuration flag only indicates that a key is present; it does not establish inference success.

For an API-only production-ledger check without creating a PayPal order, use a deliberately underfunded mandate such as two Raspberry Pi boards with a $50 ceiling. Verify the model parser label and extracted quantity, followed by a guardrail rejection and no PayPal order ID. Verify treasury filtering against the actual returned records. Never approve another payment just to test parsing.

No key yields `deterministic`. HTTP failures, rate limits, timeouts, invalid/truncated JSON and invalid domain output yield `deterministic_fallback` when the selected provider is configured. The app does not claim AI execution for a fallback. PayPal mode and payment outcomes are separate from parser mode.

Actual OpenRouter inference passed on 2026-10-27 for a non-preset mandate and treasury queries through the API and running UI. See [AI_VERIFICATION.md](AI_VERIFICATION.md). Local provider-contract tests remain separate fixture evidence.

## Existing direct OpenAI configuration

Direct OpenAI remains supported by setting LLM_PROVIDER=openai, OPENAI_API_KEY and the desired LLM_MODEL. When LLM_PROVIDER is omitted, existing OpenAI-only configuration keeps working; an OpenRouter-only key selects OpenRouter. An explicitly selected provider uses only its own key. The production model endpoints are fixed and cannot be redirected through an environment URL.

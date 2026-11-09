# Rust Engineering Rules & Guardrail Constraints

## Rule 1: Compile-Time Typestate Guardrail (ApprovedMandate)

In `backend/src/guardrails.rs`:

* Define `pub struct ApprovedMandate` with private fields so no module outside `guardrails` can instantiate it directly.
* Expose `pub fn validate(...) -> Result<ApprovedMandate, String>`.
* `PayPalClient::execute_order(&self, mandate: &ApprovedMandate)` MUST require `&ApprovedMandate`. This gives you a killer talking point for the hackathon judges: *"The order creation path requires a private ApprovedMandate value produced by deterministic runtime policy validation. This does not provide cryptographic payment authorization."*

## Rule 2: Explicit Simulation and Confirmed Sandbox Outcomes

Updated after the 2026-10-26 integration review. `PAYPAL_MODE=simulation` is an explicitly labeled offline demonstration. `PAYPAL_MODE=sandbox` uses only PayPal's official Sandbox endpoint. Without an explicit mode, real non-placeholder credentials select Sandbox unless USE_PAYPAL_SANDBOX=false; otherwise simulation is selected.

Never replace a real OAuth/API failure with simulated success. Persist the actual payment mode per transaction. Order creation yields AWAITING_APPROVAL. SDK v6 buyer approval precedes capture; verify the capture reference, amount, currency, ID and status. Keep unresolved capture/payout outcomes pending and reconcile using stable operation identifiers. Only a successful matching payout item yields SETTLED.

## Rule 3: Model Provider and Deterministic Rust Parser

* Use LLM_PROVIDER=openrouter with OPENROUTER_API_KEY and an OpenRouter LLM_MODEL, or LLM_PROVIDER=openai with OPENAI_API_KEY. Both operations share the server-only model client in llm.rs.
* An explicit provider must use its own key. Existing OpenAI-only configuration works without LLM_PROVIDER; an OpenRouter-only key also selects that provider.
* Record the returned provider/model provenance on successful structured output. Model responses remain untrusted input to deterministic policy and typed-filter validation.
* Without a configured key, use the deterministic Rust parser. Provider or output failures remain clearly labeled deterministic_fallback; they must not be presented as actual model execution.

## Rule 4: Memory-Safe Analytics Without Arbitrary Code Execution

* Never execute dynamic code or shell scripts for user analytics queries.
* Parse analytics queries into structured Rust AST predicates or match against typed transaction fields with standard iterator filters (`iter().filter(...)`), preventing injection vulnerabilities.

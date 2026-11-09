# AGENTS.md — Google Antigravity & Autonomous Agent Instructions (Rust Edition)

## Overview
You are building **ProcureOps AI** using a **Rust (Axum + Tokio + Reqwest + Serde)** backend and a **Next.js (TypeScript + Tailwind + AG Grid)** frontend for the **PayPal AI Hackathon 2026**.

## Context Discovery
Inspect `/docs` before generating code:
- `docs/PRD.md` — Functional scope & user flows
- `docs/TECH_STACK.md` — `Cargo.toml` dependencies & frontend packages
- `docs/ARCHITECTURE.md` — Directory layout & `Transaction` schema
- `docs/API_SPEC.md` — Axum endpoint contracts & PayPal REST v2 payloads
- `docs/RULES.md` — Typestate guardrail pattern & dual-mode fallback rules
- `docs/UI_UX.md` — Next.js + AG Grid cockpit layout specifications
- `docs/TASKS.md` — Step-by-step implementation checklist

## Execution Workflow
1. Follow `docs/TASKS.md` sequentially from Phase 1 to Phase 6.
2. Run `cargo check` after backend changes and `npm run build` after frontend changes to guarantee zero compiler errors.
3. Update `docs/TASKS.md` checkboxes (`[x]`) as each milestone passes verification.

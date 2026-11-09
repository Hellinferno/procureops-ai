"use client";
import { useState } from "react";
import { ArrowUpRight, Check, ChevronDown, CircleAlert, Loader2, ShieldCheck, Sparkles } from "lucide-react";
import { MandateResponse } from "../types";
import StatusBadge from "./StatusBadge";
interface Props { onExecute: (prompt: string, budget: number) => Promise<MandateResponse>; isExecuting: boolean; lastResponse: MandateResponse | null }
const presets = [
  { label: "Compliant order", prompt: "Buy 10 units of Raspberry Pi 5 (8GB) under $900 within 3 days", budget: 900 },
  { label: "Over budget", prompt: "Buy 15x NVIDIA Jetson Orin Nano under $1,200", budget: 1200 },
  { label: "Unapproved supplier", prompt: "Buy 20x ESP32 Bulk Pack from ShadowPartsShenzhen under $400", budget: 400 },
];
export default function MandatePanel({ onExecute, isExecuting, lastResponse }: Props) {
  const [prompt, setPrompt] = useState(presets[0].prompt); const [budget, setBudget] = useState(900);
  return <section className="panel composer-panel" id="new-purchase" aria-label="Create a purchase">
    <div className="panel-heading"><div className="heading-with-icon"><span className="icon-tile blue"><Sparkles size={17} /></span><div><h2>Create a purchase</h2><p>Tell your agent what you need.</p></div></div><span className="mini-label">AI assisted</span></div>
    <form onSubmit={async e => { e.preventDefault(); if (isExecuting || !prompt.trim() || budget <= 0) return; try { await onExecute(prompt, budget); } catch { /* Page displays the operation error. */ } }}>
      <label className="field-label" htmlFor="purchase-prompt">What are we buying?</label><textarea id="purchase-prompt" rows={3} value={prompt} onChange={e => setPrompt(e.target.value)} disabled={isExecuting} placeholder="e.g. Buy 10 Raspberry Pi boards under $900, delivered in 3 days" />
      <div className="preset-buttons" aria-label="Example purchase requests">{presets.map(p => <button type="button" key={p.label} disabled={isExecuting} onClick={() => { setPrompt(p.prompt); setBudget(p.budget); }}>{p.label}</button>)}</div>
      <div className="budget-row"><label htmlFor="purchase-budget">Budget ceiling <span>USD</span></label><div className="money-input"><span>$</span><input id="purchase-budget" type="number" min="0.01" step="0.01" value={budget} disabled={isExecuting} onChange={e => setBudget(Number(e.target.value))} /></div></div>
      <button className="button primary full-width" type="submit" disabled={isExecuting || !prompt.trim() || budget <= 0}>{isExecuting ? <><Loader2 className="spin" size={16} />Checking your request…</> : <>Prepare checkout <ArrowUpRight size={16} /></>}</button><p className="composer-note"><ShieldCheck size={13} />Policy checked first. You approve the payment.</p>
    </form>
    {lastResponse && <div className="execution-result" aria-live="polite"><div className="result-heading"><span>Request outcome</span>{lastResponse.transaction ? <StatusBadge status={lastResponse.transaction.status} /> : <span className="status-badge danger">No matching product</span>}</div>
      <details className="execution-details"><summary>View decision trail <ChevronDown size={14} /></summary><ol>{lastResponse.steps.map(step => <li key={step.step}><span className={`step-icon ${step.status === "blocked" || step.status === "failed" ? "danger" : ""}`}>{step.status === "blocked" || step.status === "failed" ? <CircleAlert size={13} /> : step.status === "pending" ? <span>{step.step}</span> : <Check size={13} />}</span><div><strong>Stage {step.step}: {step.title}</strong><p>{step.detail}</p></div></li>)}</ol><div className="parser-note">Parser: {lastResponse.transaction?.parser_mode || "Unavailable"}</div></details>
      {lastResponse.paypal_raw && <details className="wire-details"><summary>View payment response</summary><pre>{JSON.stringify(lastResponse.paypal_raw, null, 2)}</pre></details>}
    </div>}
  </section>;
}

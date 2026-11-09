"use client";
import { useEffect, useRef, useState } from "react";
import { ArrowUpRight, Check, Copy, FileText, Loader2, Package, ShieldCheck, ShieldX, X } from "lucide-react";
import type { Transaction } from "../types";
import StatusBadge from "./StatusBadge";
interface Props { transaction: Transaction | null; isOpen: boolean; onClose: () => void; onPayout: (id: string) => Promise<void>; isSettling?: boolean; onReviewPayment?: () => void }
const approvedVendors = ["digikey", "mouser", "adafruit", "sparkfun", "aws marketplace"];

export default function TransactionModal({ transaction: t, isOpen, onClose, onPayout, isSettling, onReviewPayment }: Props) {
  const ref = useRef<HTMLDialogElement>(null); const [copied, setCopied] = useState(false);
  useEffect(() => { const dialog = ref.current; if (!dialog) return; if (isOpen && !dialog.open) dialog.showModal(); else if (!isOpen && dialog.open) dialog.close(); }, [isOpen, t?.order_id]);
  if (!t) return null;
  const canPayout = t.payment_mode !== "seed" && !!t.capture_id && (t.status === "COMPLETED" || t.status === "PAYOUT_PENDING" || (!!t.payout_batch_id && t.status === "PAYOUT_FAILED"));
  const checks = [{ label: "Authorized budget", ok: t.total_amount <= t.authorized_budget, value: `$${t.authorized_budget.toLocaleString("en-US")} ceiling` }, { label: "Transaction limit", ok: t.total_amount <= 2000, value: "$2,000 maximum" }, { label: "Approved supplier", ok: approvedVendors.includes(t.vendor.trim().toLowerCase()), value: t.vendor }, { label: "Supplier quality", ok: t.vendor_rating >= 4.5, value: `${t.vendor_rating.toFixed(1)} / 5 rating` }];
  return <dialog ref={ref} className="audit-dialog" aria-labelledby="audit-title" onCancel={e => { e.preventDefault(); onClose(); }} onClick={e => { if (e.target === ref.current) onClose(); }}>
    <div className="audit-content"><div className="drawer-header"><div className="heading-with-icon"><span className="icon-tile blue"><FileText size={18} /></span><div><h2 id="audit-title">Purchase details</h2><p>{new Date(t.timestamp).toLocaleString("en-US", { month: "short", day: "numeric", year: "numeric", hour: "numeric", minute: "2-digit" })}</p></div></div><button className="icon-button" aria-label="Close purchase details" onClick={onClose}><X size={18} /></button></div>
      <div className="drawer-scroll"><div className="audit-amount"><div><span>Purchase amount</span><strong>{t.total_amount.toLocaleString("en-US", { style: "currency", currency: t.currency })}</strong></div><StatusBadge status={t.status} /></div>
        <div className="order-reference"><span>{t.order_id}</span><button className="icon-button" aria-label="Copy order reference" onClick={() => { void navigator.clipboard.writeText(t.order_id).then(() => { setCopied(true); setTimeout(() => setCopied(false), 2000); }); }}>{copied ? <Check size={14} /> : <Copy size={14} />}</button></div>
        <div className="audit-section"><h3><Package size={16} />Order information</h3><p className="audit-item-name">{t.item_name}</p><dl className="audit-facts"><div><dt>Supplier</dt><dd>{t.vendor}</dd></div><div><dt>Quantity</dt><dd>{t.quantity} units</dd></div><div><dt>Unit price</dt><dd>${t.unit_price.toFixed(2)}</dd></div><div><dt>Quoted delivery</dt><dd>{t.delivery_days} days</dd></div><div><dt>Execution mode</dt><dd>{t.payment_mode === "seed" ? "Historical sample" : t.payment_mode === "sandbox" ? "PayPal Sandbox" : "Simulation"}</dd></div><div><dt>Risk score</dt><dd>{Math.round(t.risk_score * 100)}%</dd></div></dl></div>
        <div className="audit-section"><h3><ShieldCheck size={16} />Policy decision</h3><div className={`policy-message ${t.guardrail_passed ? "success" : "danger"}`}>{t.guardrail_reason}</div><div className="policy-check-list">{checks.map(check => <div key={check.label}><span className={check.ok ? "approved" : "blocked"}>{check.ok ? <ShieldCheck size={15} /> : <ShieldX size={15} />}{check.label}</span><span>{check.value}</span></div>)}</div></div>
        <div className="audit-section"><h3>Agent decision</h3><p className="audit-reasoning">{t.agent_reasoning || "This historical record does not include an execution trace."}</p></div>
        <details className="audit-references"><summary>Model and payment references</summary><dl>{[["Parser", t.parser_mode], ["PayPal order", t.paypal_order_id], ["Capture", t.capture_id], ["Payout batch", t.payout_batch_id]].map(([name, value]) => <div key={name}><dt>{name}</dt><dd>{value || "Not available"}</dd></div>)}</dl></details>
        {(t.payment_error || t.payout_error) && <p className="notice warning" role="alert">{t.payment_error || t.payout_error}</p>}
        {t.payment_mode === "seed" && <p className="sample-notice">This is an illustrative historical record. Payment actions are available only for new, verified purchases.</p>}
      </div><div className="drawer-footer"><button className="button secondary" onClick={onClose}>Close</button>{(t.status === "AWAITING_APPROVAL" || t.status === "PAYMENT_PENDING") && t.payment_mode !== "seed" && <button className="button primary" onClick={onReviewPayment}>Review payment <ArrowUpRight size={15} /></button>}{canPayout && <button className="button primary" disabled={isSettling} onClick={() => void onPayout(t.order_id)}>{isSettling ? <Loader2 className="spin" size={15} /> : <ArrowUpRight size={15} />}{t.payout_batch_id ? "Check payout status" : "PayPal Payout"}</button>}</div>
    </div>
  </dialog>;
}

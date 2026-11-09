"use client";
import { ArrowUpRight, ShieldCheck, CheckCheck, Gauge } from "lucide-react";
import { KpiSummary } from "../types";
const money = (value: number) => value.toLocaleString("en-US", { style: "currency", currency: "USD" });
export default function KpiBanner({ kpis, loading }: { kpis: KpiSummary | null; loading?: boolean }) {
  const risk = Math.round((kpis?.avg_risk_score || 0) * 100);
  const metrics = [
    { label: "Captured spend", value: money(kpis?.total_spend || 0), note: "Captured and settled purchases", icon: ArrowUpRight, tone: "blue" },
    { label: "Blocked attempted spend", value: money(kpis?.money_saved_by_guardrails || 0), note: `${kpis?.blocked_count || 0} requests stopped by policy`, icon: ShieldCheck, tone: "green" },
    { label: "Policy-approved orders", value: String(kpis?.completed_count || 0).padStart(2, "0"), note: "Across the loaded purchase history", icon: CheckCheck, tone: "blue" },
    { label: "Average risk score", value: `${risk}%`, note: risk < 25 ? "Low risk across the portfolio" : "Review higher-risk purchases", icon: Gauge, tone: "green" },
  ];
  return <section className="metric-strip" aria-label="Purchase summary">{metrics.map(({ label, value, note, icon: Icon, tone }) => <div className="metric" key={label}><div className="metric-label"><span>{label}</span><Icon size={16} aria-hidden="true" /></div><p className={`metric-value ${loading ? "skeleton-text" : ""}`}>{loading ? "—" : value}</p><span className="metric-note"><span className={`tiny-dot ${tone}`} />{note}</span></div>)}</section>;
}

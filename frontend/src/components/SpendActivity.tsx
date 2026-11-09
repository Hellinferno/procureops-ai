"use client";
import { useMemo } from "react";
import { CalendarDays, ArrowUpRight } from "lucide-react";
import type { Transaction } from "../types";
export default function SpendActivity({ transactions, onViewAll }: { transactions: Transaction[]; onViewAll: () => void }) {
  const activity = useMemo(() => {
    const dated = transactions.filter(t => ["COMPLETED", "SETTLED", "PAYOUT_PENDING", "PAYOUT_FAILED"].includes(t.status));
    const latest = Math.max(...transactions.map(t => new Date(t.timestamp).getTime()).filter(Number.isFinite));
    if (!Number.isFinite(latest)) return [];
    const last = new Date(latest); last.setUTCHours(0, 0, 0, 0);
    return Array.from({ length: 7 }, (_, index) => {
      const date = new Date(last.getTime() - (6 - index) * 86400000); const key = date.toISOString().slice(0, 10);
      return { date: key, label: date.toLocaleDateString("en-US", { month: "short", day: "numeric", timeZone: "UTC" }), amount: dated.filter(t => t.timestamp.startsWith(key)).reduce((total, t) => total + t.total_amount, 0) };
    });
  }, [transactions]);
  const total = activity.reduce((sum, day) => sum + day.amount, 0);
  const ceiling = Math.max(100, Math.ceil(Math.max(...activity.map(d => d.amount), 0) / 500) * 500);
  return <section className="panel activity-panel" aria-label="Spend activity">
    <div className="panel-heading"><div><h2>Spend activity</h2><p>Captured purchases over the latest seven days</p></div><span className="date-label"><CalendarDays size={14} />{activity.length ? `${activity[0].label} – ${activity.at(-1)?.label}` : "No history yet"}</span></div>
    <div className="activity-summary"><strong>{total.toLocaleString("en-US", { style: "currency", currency: "USD" })}</strong><span><span className="tiny-dot blue" />Captured & settled</span></div>
    <div className="spend-chart" role="img" aria-label={"Daily captured spend. " + activity.map(day => day.label + ": $" + day.amount.toFixed(2)).join("; ")}>
      <div className="chart-y-axis" aria-hidden="true">{[ceiling, ceiling / 2, 0].map(value => <span key={value}>{value >= 1000 ? "$" + (value / 1000).toLocaleString("en-US") + "k" : "$" + value.toLocaleString("en-US")}</span>)}</div>
      <div className="chart-plot" aria-hidden="true"><div className="chart-lines"><span /><span /><span /></div><div className="chart-bars">{activity.map((day, index) => <div className="chart-column" key={day.date} title={day.label + ": $" + day.amount.toFixed(2)}><div className={"chart-bar" + (index === activity.length - 1 ? " latest" : "")} style={{ height: Math.max(day.amount > 0 ? 2 : 0.5, day.amount / ceiling * 100) + "%" }} /><span>{day.label}</span></div>)}</div></div>
    </div>
    <div className="activity-footer"><span>Includes illustrative historical purchases</span><button className="text-button" onClick={onViewAll}>View transactions <ArrowUpRight size={14} /></button></div>
  </section>;
}

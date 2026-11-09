"use client";
import { useState } from "react";
import { ArrowUpRight, Download, RefreshCw, Sparkles, X } from "lucide-react";
import type { AnalyticsQueryResponse, Transaction } from "../types";
interface Props { onSearch: (query: string) => Promise<void>; onRefresh: () => Promise<void>; isLoading: boolean; activeQueryResult: AnalyticsQueryResponse | null; onClearFilter: () => void; displayedRecords?: Transaction[] }
export default function QueryBar({ onSearch, onRefresh, isLoading, activeQueryResult, onClearFilter, displayedRecords = [] }: Props) {
  const [input, setInput] = useState("");
  const exportCsv = () => {
    const rows = [["Order ID", "Date", "Supplier", "Item", "Quantity", "Amount", "Currency", "Budget", "Status", "Execution mode", "PayPal order", "Capture", "Payout batch"], ...displayedRecords.map(t => [t.order_id, t.timestamp, t.vendor, t.item_name, t.quantity, t.total_amount, t.currency, t.authorized_budget, t.status, t.payment_mode, t.paypal_order_id || "", t.capture_id || "", t.payout_batch_id || ""])];
    const csv = rows.map(row => row.map(value => `"${String(value).replaceAll('"', '""')}"`).join(",")).join("\r\n");
    const url = URL.createObjectURL(new Blob(["\ufeff", csv], { type: "text/csv;charset=utf-8;" })); const link = document.createElement("a"); link.href = url; link.download = "procureops-purchases.csv"; link.click(); URL.revokeObjectURL(url);
  };
  return <div className="query-bar"><form className="query-form" onSubmit={e => { e.preventDefault(); if (input.trim() && !isLoading) void onSearch(input); }}><div className="query-input"><Sparkles size={16} /><input aria-label="Ask your treasury" placeholder="Ask your treasury in plain English…" value={input} disabled={isLoading} onChange={e => setInput(e.target.value)} />{input && <button type="button" aria-label="Clear treasury query" onClick={() => { setInput(""); onClearFilter(); }}><X size={14} /></button>}</div><button type="submit" className="button subtle" disabled={isLoading || !input.trim()}>Ask Treasury <ArrowUpRight size={14} /></button></form>
    <div className="ledger-tools"><button className="button secondary export-button" type="button" onClick={exportCsv} disabled={isLoading || !displayedRecords.length}><Download size={15} />Export CSV</button><button className="icon-button" type="button" aria-label="Refresh purchases" onClick={() => void onRefresh()} disabled={isLoading}><RefreshCw size={15} className={isLoading ? "spin" : ""} /></button></div>
    {activeQueryResult && <div className="query-result" aria-live="polite"><div className="query-result-copy"><Sparkles size={16} /><div><strong>{activeQueryResult.summary}</strong><span>Parser: {activeQueryResult.parser_mode}</span><details><summary>View interpreted filter</summary><code>{activeQueryResult.filter_expr}</code></details></div></div><button className="text-button" onClick={() => { setInput(""); onClearFilter(); }}>Clear filter <X size={13} /></button></div>}
  </div>;
}

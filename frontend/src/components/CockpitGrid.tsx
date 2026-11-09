"use client";
import { useCallback, useMemo, useRef, useState, useSyncExternalStore } from "react";
import { AgGridReact } from "ag-grid-react";
import { AllCommunityModule, ColDef, ICellRendererParams, ModuleRegistry, RowClickedEvent } from "ag-grid-community";
import "ag-grid-community/styles/ag-grid.css";
import "ag-grid-community/styles/ag-theme-quartz.css";
import { ArrowUpRight, Check, ChevronRight, Loader2, Search, ShieldCheck, ShieldX } from "lucide-react";
import type { Transaction } from "../types";
import StatusBadge, { statusLabel } from "./StatusBadge";

ModuleRegistry.registerModules([AllCommunityModule]);
const subscribeCompact = (onChange: () => void) => { const media = window.matchMedia("(max-width: 900px)"); media.addEventListener("change", onChange); return () => media.removeEventListener("change", onChange); };
const compactSnapshot = () => window.matchMedia("(max-width: 900px)").matches;
const serverSnapshot = () => false;
const payoutEligible = (t: Transaction) => t.payment_mode !== "seed" && !!t.capture_id && (t.status === "COMPLETED" || t.status === "PAYOUT_PENDING" || (!!t.payout_batch_id && t.status === "PAYOUT_FAILED"));
interface Props { rowData: Transaction[]; onPayout: (orderId: string) => Promise<void>; isLoading?: boolean; onSelectTransaction?: (transaction: Transaction) => void }
export default function CockpitGrid({ rowData, onPayout, isLoading, onSelectTransaction }: Props) {
  const [payoutLoadingId, setPayoutLoadingId] = useState<string | null>(null);
  const [quickFilter, setQuickFilter] = useState("");
  const gridRef = useRef<AgGridReact<Transaction>>(null);
  const compact = useSyncExternalStore(subscribeCompact, compactSnapshot, serverSnapshot);
  const [mobilePage, setMobilePage] = useState(0);
  const mobileRows = rowData.filter(t => [t.order_id, t.vendor, t.item_name, t.category, t.payment_mode, t.status, statusLabel(t.status), String(t.total_amount)].join(" ").toLowerCase().includes(quickFilter.toLowerCase())).sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
  const mobilePages = Math.max(1, Math.ceil(mobileRows.length / 3));
  const safePage = Math.min(mobilePage, mobilePages - 1);
  const settle = useCallback(async (id: string, event: React.MouseEvent) => { event.stopPropagation(); setPayoutLoadingId(id); try { await onPayout(id); } finally { setPayoutLoadingId(null); } }, [onPayout]);
  const onRowClicked = useCallback((event: RowClickedEvent<Transaction>) => { if (!event.isEventHandlingSuppressed && !(event.event?.target as HTMLElement | null)?.closest("button") && event.data) onSelectTransaction?.(event.data); }, [onSelectTransaction]);
  const columns = useMemo<ColDef<Transaction>[]>(() => [
    { headerName: "Supplier", field: "vendor", minWidth: 158, flex: 1, cellRenderer: ({ data: t }: ICellRendererParams<Transaction>) => t && <div className="supplier-cell"><span className={`supplier-avatar supplier-${t.vendor.length % 4}`}>{t.vendor.split(" ").map(s => s[0]).join("").slice(0, 2).toUpperCase()}</span><div><strong>{t.vendor}</strong><span>{t.vendor_rating.toFixed(1)} supplier rating</span></div></div> },
    { headerName: "Purchase", field: "item_name", getQuickFilterText: ({ data }) => data ? [data.order_id, data.item_name, data.category, data.payment_mode].join(" ") : "", minWidth: 230, flex: 1.7, cellRenderer: ({ data: t }: ICellRendererParams<Transaction>) => t && <div className="purchase-cell"><strong title={t.item_name}>{t.item_name}</strong><span><span title={t.order_id}>{t.order_id.length > 20 ? `…${t.order_id.slice(-10)}` : t.order_id}</span><span>{t.quantity} {t.quantity === 1 ? "unit" : "units"}</span>{t.payment_mode === "seed" && <span className="sample-label">Sample</span>}</span></div> },
    { headerName: "Amount", field: "total_amount", width: 122, type: "rightAligned", cellRenderer: ({ data: t }: ICellRendererParams<Transaction>) => t && <div className="amount-cell"><strong>{t.total_amount.toLocaleString("en-US", { style: "currency", currency: t.currency })}</strong><span>of ${t.authorized_budget.toLocaleString("en-US")} budget</span></div> },
    { headerName: "Status", field: "status", getQuickFilterText: ({ data }) => data ? data.status + " " + statusLabel(data.status) : "", filterValueGetter: ({ data }) => data ? statusLabel(data.status) : "", width: 174, cellRenderer: ({ data: t }: ICellRendererParams<Transaction>) => t && <div className="center-cell"><StatusBadge status={t.status} /></div> },
    { headerName: "Policy", field: "guardrail_passed", width: 95, cellRenderer: ({ data: t }: ICellRendererParams<Transaction>) => t && <div className={`policy-cell ${t.guardrail_passed ? "approved" : "blocked"}`} title={t.guardrail_reason}>{t.guardrail_passed ? <ShieldCheck size={14} /> : <ShieldX size={14} />}{t.guardrail_passed ? "Passed" : "Blocked"}</div> },
    { headerName: "Date", field: "timestamp", width: 100, sort: "desc", cellRenderer: ({ value }: ICellRendererParams<Transaction>) => <div className="center-cell date-cell">{new Date(value).toLocaleDateString("en-US", { month: "short", day: "numeric" })}</div> },
    { headerName: "", colId: "actions", width: 133, sortable: false, filter: false, resizable: false, cellRendererParams: { suppressMouseEventHandling: () => true }, cellRenderer: ({ data: t }: ICellRendererParams<Transaction>) => {
      if (!t) return null;
      const eligible = payoutEligible(t);
      return <div className="center-cell action-cell">{eligible ? <button className="row-payout" disabled={payoutLoadingId === t.order_id} onClick={e => void settle(t.order_id, e)}>{payoutLoadingId === t.order_id ? <Loader2 className="spin" size={13} /> : <ArrowUpRight size={13} />}{t.payout_batch_id ? "Check payout" : "PayPal Payout"}</button> : <button className="row-detail" aria-label={`Inspect ${t.order_id}`} onClick={e => { e.stopPropagation(); onSelectTransaction?.(t); }}>View <ChevronRight size={14} /></button>}</div>;
    } },
  ], [payoutLoadingId, settle, onSelectTransaction]);
  const defaults = useMemo<ColDef<Transaction>>(() => ({ sortable: true, filter: true, resizable: true }), []);
  const total = rowData.reduce((sum, t) => sum + t.total_amount, 0);
  return <div className="ledger-body">
    <div className="table-filter-row"><div className="table-filter"><Search size={15} /><input aria-label="Filter purchase table" placeholder="Search purchases..." value={quickFilter} onChange={e => { setQuickFilter(e.target.value); setMobilePage(0); gridRef.current?.api.setGridOption("quickFilterText", e.target.value); }} /></div><span className="table-hint">Select a purchase to inspect its details</span></div>
    {compact ? <div className="mobile-purchase-list" aria-label="Purchase cards">
      {mobileRows.slice(safePage * 3, safePage * 3 + 3).map(t => <article className="purchase-card" key={t.order_id}>
        <button className="purchase-card-main" aria-label={"Inspect " + t.order_id} onClick={() => onSelectTransaction?.(t)}><span className={"supplier-avatar supplier-" + t.vendor.length % 4}>{t.vendor.split(" ").map(p => p[0]).join("").slice(0, 2).toUpperCase()}</span><span><strong>{t.vendor}</strong><span>{t.item_name}</span></span><b>{t.total_amount.toLocaleString("en-US", { style: "currency", currency: t.currency })}</b></button>
        <div className="purchase-card-meta"><span>{t.quantity} units</span><span>{new Date(t.timestamp).toLocaleDateString("en-US", { month: "short", day: "numeric" })}</span>{t.payment_mode === "seed" && <span className="sample-label">Sample</span>}</div>
        <div className="purchase-card-footer"><StatusBadge status={t.status} />{payoutEligible(t) ? <button className="row-payout" disabled={payoutLoadingId === t.order_id} onClick={e => void settle(t.order_id, e)}>{payoutLoadingId === t.order_id && <Loader2 className="spin" size={12} />}{t.payout_batch_id ? "Check payout" : "PayPal Payout"}</button> : <button className="row-detail" onClick={() => onSelectTransaction?.(t)}>View details <ChevronRight size={13} /></button>}</div>
      </article>)}
      {!mobileRows.length && <div className="grid-empty">No purchases match this view. Try another filter or create a purchase.</div>}
      <div className="mobile-pagination"><span>{mobileRows.length ? safePage * 3 + 1 : 0}–{Math.min((safePage + 1) * 3, mobileRows.length)} of {mobileRows.length}</span><div><button className="icon-button" aria-label="Previous purchases" disabled={safePage === 0} onClick={() => setMobilePage(safePage - 1)}><ChevronRight className="rotate-back" size={17} /></button><span>{safePage + 1} / {mobilePages}</span><button className="icon-button" aria-label="Next purchases" disabled={safePage === mobilePages - 1} onClick={() => setMobilePage(safePage + 1)}><ChevronRight size={17} /></button></div></div>
    </div> : <div className="ag-theme-quartz procure-grid"><AgGridReact<Transaction> ref={gridRef} theme="legacy" quickFilterText={quickFilter} rowData={rowData} loading={isLoading} getRowId={p => p.data.order_id} columnDefs={columns} defaultColDef={defaults} pagination paginationPageSize={5} paginationPageSizeSelector={[5, 10, 20, 50]} rowHeight={66} headerHeight={43} animateRows={false} onRowClicked={onRowClicked} onCellKeyDown={event => { if ((event.event as KeyboardEvent | undefined)?.key === "Enter" && !(event.event?.target as HTMLElement | null)?.closest("button") && event.data) onSelectTransaction?.(event.data); }} overlayNoRowsTemplate={'<div class="grid-empty">No purchases match this view. Try another filter or create a purchase.</div>'} /></div>}
    <div className="ledger-footer"><span><Check size={13} />{rowData.length} records <span className="footer-divider" />{total.toLocaleString("en-US", { style: "currency", currency: "USD" })} total order value</span><span>Historical samples are labeled in each row.</span></div>
  </div>;
}

"use client";

import React, { useState, useEffect, useCallback, useMemo, useRef } from "react";
import {
  Transaction,
  KpiSummary,
  MandateResponse,
  AnalyticsQueryResponse,
  HealthResponse,
  CheckoutConfig,
  PaymentResponse,
} from "../types";
import {
  fetchTransactions,
  executeMandate,
  queryAnalytics,
  settlePayout,
  resetDatabase,
  fetchHealth,
  fetchCheckoutConfig,
  refreshPayout,
} from "../lib/api";

import PayPalCheckout from "../components/PayPalCheckout";
import KpiBanner from "../components/KpiBanner";
import SpendActivity from "../components/SpendActivity";
import MandatePanel from "../components/MandatePanel";
import QueryBar from "../components/QueryBar";
import CockpitGrid from "../components/CockpitGrid";
import TransactionModal from "../components/TransactionModal";
import ProcureMark from "../components/ProcureMark";

import { ArrowUpRight, CheckCircle2, AlertTriangle, CreditCard, ChevronDown, ChevronRight, Clock3, LayoutDashboard, LifeBuoy, ListChecks, Menu, Plug, Plus, ShieldCheck, X } from "lucide-react";

interface ToastNotification {
  id: string;
  type: "success" | "warning" | "info";
  title: string;
  message: string;
}

export default function Home() {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [displayedTransactions, setDisplayedTransactions] = useState<Transaction[]>([]);
  const [kpis, setKpis] = useState<KpiSummary | null>(null);
  const [health, setHealth] = useState<HealthResponse | null>(null);

  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [isExecutingMandate, setIsExecutingMandate] = useState<boolean>(false);
  const [isSearching, setIsSearching] = useState<boolean>(false);
  const [lastMandateResponse, setLastMandateResponse] = useState<MandateResponse | null>(null);
  const [activeQuery, setActiveQuery] = useState<AnalyticsQueryResponse | null>(null);

  const [checkoutConfig, setCheckoutConfig] = useState<CheckoutConfig | null>(null);
  const [dataError, setDataError] = useState<string | null>(null);

  const [activeView, setActiveView] = useState("overview");
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [workspacePanel, setWorkspacePanel] = useState<"policy" | "connections" | "help" | null>(null);
  const workspaceDialog = useRef<HTMLDialogElement>(null);
  const approvalCount = transactions.filter(t => t.status === "AWAITING_APPROVAL" || t.status === "PAYMENT_PENDING").length;
  const visibleTransactions = useMemo(() => displayedTransactions.filter(t => {
    if (activeView === "approvals") return t.status === "AWAITING_APPROVAL" || t.status === "PAYMENT_PENDING";
    if (activeView === "payouts") return ["COMPLETED", "SETTLED", "APPROVED_PENDING_SETTLEMENT", "PAYOUT_PENDING", "PAYOUT_FAILED"].includes(t.status);
    if (activeView === "blocked") return t.status === "BLOCKED_BY_GUARDRAIL";
    return true;
  }), [displayedTransactions, activeView]);
  const navigate = (view: string, scroll = true) => {
    if (sidebarOpen) document.querySelector<HTMLButtonElement>('[aria-label="Open navigation"]')?.focus();
    setActiveView(view); setSidebarOpen(false); setActiveQuery(null); setDisplayedTransactions(transactions);
    if (scroll) document.getElementById(view === "overview" ? "workspace-top" : "purchase-ledger")?.scrollIntoView({ behavior: "smooth", block: "start" });
  };
  const newPurchase = () => {
    setSidebarOpen(false);
    document.getElementById("new-purchase")?.scrollIntoView({ behavior: "smooth", block: "center" });
    document.getElementById("purchase-prompt")?.focus({ preventScroll: true });
  };
  useEffect(() => {
    const dialog = workspaceDialog.current;
    if (!dialog) return;
    if (workspacePanel && !dialog.open) dialog.showModal();
    else if (!workspacePanel && dialog.open) dialog.close();
  }, [workspacePanel]);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => { if (event.key === "Escape" && sidebarOpen) { setSidebarOpen(false); document.querySelector<HTMLButtonElement>('[aria-label="Open navigation"]')?.focus(); } };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [sidebarOpen]);

  // Modal inspection state
  const [selectedTransaction, setSelectedTransaction] = useState<Transaction | null>(null);
  const [isModalOpen, setIsModalOpen] = useState<boolean>(false);
  const [isSettlingModal, setIsSettlingModal] = useState<boolean>(false);

  // Toast notifications state
  const [toasts, setToasts] = useState<ToastNotification[]>([]);

  const showToast = useCallback((type: "success" | "warning" | "info", title: string, message: string) => {
    const id = `${Date.now()}-${Math.random().toString(36).substring(2, 6)}`;
    setToasts((prev) => [...prev, { id, type, title, message }]);
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 4500);
  }, []);

  const removeToast = (id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  };

  const loadData = useCallback(async () => {
    try {
      const [txData, healthData, checkoutData] = await Promise.all([
        fetchTransactions(),
        fetchHealth(),
        fetchCheckoutConfig(),
      ]);

      setCheckoutConfig(checkoutData);
      setDataError(null);
      if (txData) {
        setTransactions(txData.transactions);
        setDisplayedTransactions(txData.transactions);
        setKpis(txData.kpis);
      }
      if (healthData) {
        setHealth(healthData);
      }
    } catch (err) {
      setDataError(`Could not connect to the service. ${String(err)}`);
      setHealth(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    let active = true;
    Promise.all([fetchTransactions(), fetchHealth(), fetchCheckoutConfig()])
      .then(([ledger, service, configuration]) => {
        if (!active) return;
        setTransactions(ledger.transactions);
        setDisplayedTransactions(ledger.transactions);
        setKpis(ledger.kpis);
        setHealth(service);
        setCheckoutConfig(configuration);
      })
      .catch((error) => { if (active) setDataError(`Could not connect to the service. ${String(error)}`); })
      .finally(() => { if (active) setIsLoading(false); });
    return () => { active = false; };
  }, []);

  // Handle Mandate execution
  const handleExecuteMandate = async (
    prompt: string,
    budget: number
  ): Promise<MandateResponse> => {
    try {
      setIsExecutingMandate(true);
      const res = await executeMandate(prompt, budget);
      setLastMandateResponse(res);
      setActiveView("overview");
      setSelectedTransaction(null);

      // Refresh transactions and KPIs
      const refreshed = await fetchTransactions();
      setTransactions(refreshed.transactions);
      setDisplayedTransactions(refreshed.transactions);
      setKpis(refreshed.kpis);
      setActiveQuery(null);

      if (res.success) {
        showToast(
          "success",
          "Order ready for buyer approval",
          "The order passed policy checks. Approve payment in the checkout panel to capture."
        );
      } else {
        showToast(
          "warning",
          "Mandate could not proceed",
          res.transaction?.payment_error || res.transaction?.guardrail_reason || res.steps.at(-1)?.detail || "No matching product found."
        );
      }

      return res;
    } catch (err) {
      showToast("warning", "Mandate Execution Failed", String(err));
      throw err;
    } finally {
      setIsExecutingMandate(false);
    }
  };

  // Handle Conversational Analytics search
  const handleSearch = async (queryStr: string) => {
    try {
      setIsSearching(true);
      setActiveView("all");
      const res = await queryAnalytics(queryStr);
      setActiveQuery(res);
      setDisplayedTransactions(res.records);
    } catch (err) {
      console.error("Search query failed:", err);
      showToast("warning", "Treasury Search Failed", "Could not interpret natural language query.");
    } finally {
      setIsSearching(false);
    }
  };

  const handleClearFilter = () => {
    setActiveQuery(null);
    setDisplayedTransactions(transactions);
  };

  // Handle PayPal Payout settlement from AG Grid row or Modal
  const handlePayout = async (orderId: string) => {
    try {
      setIsSettlingModal(true);
      const current = transactions.find((t) => t.order_id === orderId);
      const result = current?.payout_batch_id ? await refreshPayout(orderId) : await settlePayout(orderId);
      const refreshed = await fetchTransactions();
      setTransactions(refreshed.transactions);
      setKpis(refreshed.kpis);

      // Update selected transaction if open in modal
      const updatedItem = refreshed.transactions.find((t) => t.order_id === orderId);
      if (updatedItem) {
        setSelectedTransaction(updatedItem);
        setLastMandateResponse(previous => previous?.transaction?.order_id === orderId ? { ...previous, transaction: updatedItem } : previous);
      }

      if (activeQuery) {
        const updatedQuery = await queryAnalytics(activeQuery.query);
        setActiveQuery(updatedQuery);
        setDisplayedTransactions(updatedQuery.records);
      } else {
        setDisplayedTransactions(refreshed.transactions);
      }

      showToast(
        "success",
        result.new_status === "SETTLED" ? "Payout completion confirmed" : "Payout status updated",
        result.message
      );
    } catch (err) {
      showToast("warning", "Payout operation failed", String(err));
      await loadData();
    } finally {
      setIsSettlingModal(false);
    }
  };

  // Handle database reset
  const handleReset = async () => {
    try {
      setIsLoading(true);
      await resetDatabase();
      await loadData();
      setActiveQuery(null);
      setLastMandateResponse(null);
      setSelectedTransaction(null);
      setIsModalOpen(false);
      showToast("info", "Ledger Restored", "Reset to 25 sample historical procurement records.");
    } catch (error) {
      showToast("warning", "Reset unavailable", String(error));
    } finally {
      setIsLoading(false);
    }
  };

  const handlePaymentUpdate = useCallback(async (response: PaymentResponse) => {
    setLastMandateResponse((previous) => previous?.transaction?.order_id === response.transaction.order_id ? {
      ...previous, transaction: response.transaction, paypal_raw: response.paypal_raw,
      steps: previous.steps.map((step) => step.step === 4 ? {
        ...step, title: "Payment result", status: response.success ? "done" : "pending", detail: response.message,
      } : step),
    } : previous);
    setSelectedTransaction((previous) => previous?.order_id === response.transaction.order_id ? response.transaction : previous);
    showToast("info", response.transaction.payment_mode === "simulation" ? "Simulation result" : "PayPal Sandbox result", response.message);
    setActiveQuery(null);
    await loadData();
  }, [loadData, showToast]);

  const handleSelectTransaction = (tx: Transaction) => {
    setSelectedTransaction(tx);
    setIsModalOpen(true);
  };

  const viewNames: Record<string, string> = { overview: "Overview", all: "All purchases", approvals: "Needs approval", payouts: "Payouts", blocked: "Policy blocked" };
  const paymentLabel = health?.paypal_mode === "sandbox" ? "PayPal Sandbox" : health ? "Simulation · no money moves" : dataError ? "Service disconnected" : "Connecting…";
  return (
    <div className="app-shell">
      <a className="skip-link" href="#workspace-top">Skip to workspace</a>
      {sidebarOpen && <button className="nav-scrim" aria-label="Close navigation" onClick={() => setSidebarOpen(false)} />}
      <aside className={"app-sidebar" + (sidebarOpen ? " open" : "")} id="workspace-navigation">
        <a className="brand" aria-label="ProcureOps home" href="#workspace-top" onClick={() => navigate("overview", false)}><span className="brand-mark"><ProcureMark /></span><span className="brand-wordmark">ProcureOps<span className="brand-suffix">AI</span></span></a>
        <button className="workspace-switch" onClick={() => setWorkspacePanel("connections")}><span className="workspace-icon">P</span><span><strong>ProcureOps workspace</strong><small>{health?.paypal_mode === "sandbox" ? "Sandbox environment" : health ? "Simulation environment" : "Local environment"}</small></span><ChevronDown size={14} /></button>
        <div className="nav-group-label">Workspace</div>
        <nav aria-label="Main navigation">
          {([{ id: "overview", label: "Overview", icon: LayoutDashboard }, { id: "all", label: "All purchases", icon: ListChecks }, { id: "approvals", label: "Needs approval", icon: Clock3 }, { id: "payouts", label: "Payouts", icon: ArrowUpRight }]).map(({ id, label, icon: Icon }) => <button key={id} aria-label={label} title={label} className={"nav-item" + (activeView === id ? " active" : "")} aria-current={activeView === id ? "page" : undefined} onClick={() => navigate(id)}><Icon size={18} /><span>{label}</span>{id === "approvals" && approvalCount > 0 && <span className="nav-count">{approvalCount}</span>}</button>)}
        </nav>
        <div className="nav-group-label secondary-group">Manage</div>
        <nav aria-label="Workspace settings"><button className="nav-item" aria-label="Policy controls" title="Policy controls" onClick={() => { setWorkspacePanel("policy"); setSidebarOpen(false); }}><ShieldCheck size={18} /><span>Policy controls</span></button><button className="nav-item" aria-label="Connections" title="Connections" onClick={() => { setWorkspacePanel("connections"); setSidebarOpen(false); }}><Plug size={18} /><span>Connections</span></button></nav>
        <div className="sidebar-bottom"><div className="sidebar-policy"><span className="sidebar-shield"><ShieldCheck size={19} /></span><strong>Every purchase, checked.</strong><p>Your budget and supplier policy stay in control.</p><button onClick={() => setWorkspacePanel("policy")}>View purchase policy <ArrowUpRight size={13} /></button></div><button aria-label="Help and getting started" title="Help and getting started" className="nav-item help-nav" onClick={() => setWorkspacePanel("help")}><LifeBuoy size={18} /><span>Help & getting started</span></button><div className="sidebar-profile"><span className="profile-avatar">PO</span><div><strong>ProcureOps</strong><span><span className={"tiny-dot " + (health ? "green" : "muted")} />{health ? "Service connected" : "Connecting to service"}</span></div></div></div>
      </aside>
      <div className="app-main">
        <header className="topbar"><div className="breadcrumbs"><button className="icon-button mobile-menu" aria-label="Open navigation" aria-expanded={sidebarOpen} aria-controls="workspace-navigation" onClick={() => setSidebarOpen(!sidebarOpen)}><Menu size={20} /></button><span>Workspace</span><ChevronRight size={14} /><strong>{viewNames[activeView]}</strong></div><div className="topbar-actions"><span className={"environment-pill" + (!health ? " disconnected" : "")}><span className="tiny-dot" />{paymentLabel}</span><span className="topbar-divider" /><button className="icon-button" aria-label="Open help" onClick={() => setWorkspacePanel("help")}><LifeBuoy size={18} /></button><button className="topbar-avatar" aria-label="Workspace connections" onClick={() => setWorkspacePanel("connections")}>PO</button></div></header>
        <main className="workspace-main" id="workspace-top">
          <div className="page-heading"><div><h1>Purchasing overview</h1><p>Track spend, prepare purchases, and review every decision.</p></div><button className="button primary new-purchase-button" onClick={newPurchase}><Plus size={17} />New purchase</button></div>
          {dataError && <div role="alert" className="notice danger"><AlertTriangle size={17} /><div><strong>We couldn&apos;t load your workspace.</strong><p>{dataError}</p></div><button className="button secondary" onClick={() => void loadData()}>Try again</button></div>}
          <KpiBanner kpis={kpis} loading={isLoading} />
          <div className="workbench"><SpendActivity transactions={transactions} onViewAll={() => navigate("all")} /><div className="purchase-workflow"><MandatePanel onExecute={handleExecuteMandate} isExecuting={isExecutingMandate} lastResponse={lastMandateResponse} /><PayPalCheckout config={checkoutConfig} transaction={selectedTransaction ?? lastMandateResponse?.transaction ?? null} onUpdate={handlePaymentUpdate} /></div></div>
          <section className="panel ledger-panel" id="purchase-ledger" aria-label="Purchase ledger"><div className="ledger-heading"><div><h2>Purchases</h2><span className="record-count">{visibleTransactions.length} records</span></div><span className="ledger-edition">Powered by AG Grid Community</span></div><div className="ledger-tabs" role="tablist" aria-label="Purchase views">{[{ id: "all", label: "All purchases" }, { id: "approvals", label: "Needs approval" }, { id: "payouts", label: "Payouts" }, { id: "blocked", label: "Policy blocked" }].map(tab => <button key={tab.id} role="tab" onKeyDown={event => { const directions = ["ArrowRight", "ArrowLeft", "Home", "End"]; if (!directions.includes(event.key)) return; event.preventDefault(); const tabs = Array.from(event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>("[role=tab]") || []); const index = tabs.indexOf(event.currentTarget); const next = event.key === "Home" ? 0 : event.key === "End" ? tabs.length - 1 : (index + (event.key === "ArrowRight" ? 1 : -1) + tabs.length) % tabs.length; tabs[next]?.focus(); tabs[next]?.click(); }} tabIndex={activeView === tab.id || (activeView === "overview" && tab.id === "all") ? 0 : -1} aria-selected={activeView === tab.id || (activeView === "overview" && tab.id === "all")} onClick={() => navigate(tab.id, false)}>{tab.label}{tab.id === "approvals" && approvalCount > 0 && <span>{approvalCount}</span>}</button>)}</div>
            <QueryBar onSearch={handleSearch} onRefresh={async () => { setActiveQuery(null); await loadData(); }} isLoading={isSearching || isLoading} activeQueryResult={activeQuery} onClearFilter={handleClearFilter} displayedRecords={visibleTransactions} />
            <CockpitGrid rowData={visibleTransactions} onPayout={handlePayout} isLoading={isLoading} onSelectTransaction={handleSelectTransaction} />
          </section>
          <footer className="workspace-footer"><span>Historical seed rows are sample data. New purchases identify Sandbox or simulation execution.</span><span><ShieldCheck size={13} />Buyer approval stays in your hands.</span></footer>
        </main>
      </div>
      <TransactionModal transaction={selectedTransaction} isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} onPayout={handlePayout} isSettling={isSettlingModal} onReviewPayment={() => { setIsModalOpen(false); document.getElementById("payment-review")?.scrollIntoView({ behavior: "smooth", block: "center" }); }} />
      <dialog ref={workspaceDialog} className="info-dialog" aria-labelledby="workspace-dialog-title" onCancel={e => { e.preventDefault(); setWorkspacePanel(null); }} onClick={e => { if (e.target === workspaceDialog.current) setWorkspacePanel(null); }}>
        <div className="drawer-header"><div><h2 id="workspace-dialog-title">{workspacePanel === "policy" ? "Your purchasing policy" : workspacePanel === "connections" ? "Workspace connections" : "Getting started"}</h2><p>{workspacePanel === "policy" ? "Checks run before a payment can be prepared." : workspacePanel === "connections" ? "Configuration for this local workspace." : "A clear path from request to settlement."}</p></div><button className="icon-button" aria-label="Close workspace panel" onClick={() => setWorkspacePanel(null)}><X size={18} /></button></div>
        <div className="info-dialog-body">{workspacePanel === "policy" ? <><div className="policy-limit"><ShieldCheck size={22} /><div><span>Maximum per purchase</span><strong>$2,000.00</strong></div></div><ul className="help-list"><li>The tighter of your request budget and authorized ceiling applies.</li><li>Suppliers must be approved and rated at least 4.5 out of 5.</li><li>Stock, order quantity, and delivery deadline must match.</li><li>Buyer approval is required before a payment is captured.</li></ul><h3>Approved suppliers</h3><div className="supplier-tags">{["DigiKey", "Mouser", "Adafruit", "SparkFun", "AWS Marketplace"].map(name => <span key={name}>{name}</span>)}</div></> : workspacePanel === "connections" ? <><div className="connection-row"><span className="icon-tile blue"><CreditCard size={20} /></span><div><strong>PayPal payments</strong><span>{checkoutConfig?.message || "Payment service unavailable"}</span></div><span className={"status-badge " + (checkoutConfig?.configured ? "success" : "warning")}>{checkoutConfig?.configured ? "Configured" : "Setup needed"}</span></div><div className="connection-row"><span className="icon-tile green"><ShieldCheck size={20} /></span><div><strong>AI interpretation</strong><span>The actual model is shown with each request.</span></div><span className={"status-badge " + (health?.llm_configured ? "success" : "neutral")}>{health?.llm_configured ? "Key configured" : "Deterministic mode"}</span></div><div className="connection-row"><span className="icon-tile blue"><ArrowUpRight size={20} /></span><div><strong>Vendor payouts</strong><span>Uses the owned test recipient configured on the server.</span></div><span className={"status-badge " + (checkoutConfig?.payouts_enabled ? "success" : "neutral")}>{checkoutConfig?.payouts_enabled ? "Enabled" : "Not configured"}</span></div><p className="sample-notice">Configuration does not guarantee provider availability. Payment and model outcomes are verified per operation.</p>{checkoutConfig?.mode === "simulation" && <button className="button secondary" onClick={() => { void handleReset(); setWorkspacePanel(null); }}>Restore sample ledger</button>}</> : <><ol className="help-list"><li>Describe the item, quantity, budget and delivery deadline in Create a purchase.</li><li>Prepare checkout. Review the supplier choice and the policy decision.</li><li>Approve in PayPal, then check the capture status. A pending payment is still unresolved.</li><li>Use the ledger to inspect a purchase, ask a treasury question, export records or reconcile a payout.</li></ol><p className="sample-notice">This prototype uses an illustrative supplier catalog and test payments. It does not place physical orders with the named suppliers.</p></>}</div><div className="drawer-footer"><button className="button secondary" onClick={() => setWorkspacePanel(null)}>Done</button></div>
      </dialog>
      <div className="toast-container" aria-live="polite">{toasts.map(toast => <div className={"toast " + toast.type} key={toast.id}><span className="toast-icon">{toast.type === "warning" ? <AlertTriangle size={18} /> : <CheckCircle2 size={18} />}</span><div><strong>{toast.title}</strong><p>{toast.message}</p></div><button className="icon-button" aria-label="Dismiss notification" onClick={() => removeToast(toast.id)}><X size={15} /></button></div>)}</div>
    </div>
  );
}

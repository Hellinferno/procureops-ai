"use client";

import Script from "next/script";
import { useCallback, useEffect, useRef, useState } from "react";
import { CheckoutConfig, PaymentResponse, Transaction } from "../types";
import { capturePayment, cancelPayment, refreshPayment } from "../lib/api";

interface PayPalSession { start(options: { presentationMode: string }, order: Promise<{ orderId: string }>): Promise<void> }
interface PayPalInstance {
  findEligibleMethods(): Promise<{ isEligible(method: string): boolean }>;
  createPayPalOneTimePaymentSession(callbacks: {
    onApprove(data: { orderId: string }): Promise<void>;
    onCancel(): Promise<void>;
    onError(error: unknown): void;
  }): Promise<PayPalSession>;
}
declare global {
  interface Window { paypal?: { createInstance(options: { clientId: string; components: string[] }): Promise<PayPalInstance> } }
}

export default function PayPalCheckout({ config, transaction, onUpdate }: {
  config: CheckoutConfig | null;
  transaction: Transaction | null;
  onUpdate: (response: PaymentResponse) => Promise<void>;
}) {
  const container = useRef<HTMLDivElement>(null);
  const [ready, setReady] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const transactionId = transaction?.order_id;
  const paypalId = transaction?.paypal_order_id;
  const awaiting = transaction?.status === "AWAITING_APPROVAL";
  const approve = useCallback(async () => {
    if (!transactionId) return;
    setBusy(true); setError(null);
    try { await onUpdate(await capturePayment(transactionId)); }
    catch (error) { setError(String(error)); }
    finally { setBusy(false); }
  }, [transactionId, onUpdate]);
  const cancel = useCallback(async () => {
    if (!transactionId) return;
    setBusy(true); setError(null);
    try { await onUpdate(await cancelPayment(transactionId)); }
    catch (error) { setError(String(error)); }
    finally { setBusy(false); }
  }, [transactionId, onUpdate]);

  useEffect(() => {
    const element = container.current;
    if (!ready || !element || !awaiting || config?.mode !== "sandbox" || !config.client_id || !paypalId) return;
    let disposed = false;
    let button: HTMLElement | undefined;
    async function initialize() {
      try {
        if (!window.paypal || !config?.client_id) throw new Error("PayPal SDK did not load");
        const sdk = await window.paypal.createInstance({ clientId: config.client_id, components: ["paypal-payments"] });
        const eligible = await sdk.findEligibleMethods();
        if (!eligible.isEligible("paypal")) throw new Error("PayPal is unavailable for this browser or account");
        const session = await sdk.createPayPalOneTimePaymentSession({
          onApprove: async (data) => {
            if (data.orderId !== paypalId) { setError("PayPal returned a different order. No capture was requested."); return; }
            await approve();
          },
          onCancel: cancel,
          onError: (error) => setError(String(error)),
        });
        if (disposed || !element) return;
        button = document.createElement("paypal-button");
        button.setAttribute("aria-label", "Approve PayPal Sandbox payment");
        button.addEventListener("click", () => {
          setError(null);
          // Start in the click handler to preserve browser popup activation.
          void session.start({ presentationMode: "auto" }, Promise.resolve({ orderId: paypalId! }))
            .catch((error) => setError(String(error)));
        });
        element.replaceChildren(button);
      } catch (error) { if (!disposed) setError(String(error)); }
    }
    void initialize();
    return () => { disposed = true; button?.remove(); };
  }, [ready, awaiting, config, paypalId, approve, cancel]);

  const refresh = async () => {
    if (!transactionId) return;
    setBusy(true); setError(null);
    try { await onUpdate(await refreshPayment(transactionId)); }
    catch (error) { setError(String(error)); }
    finally { setBusy(false); }
  };
  if (!transaction || transaction.payment_mode === "seed" || transaction.status === "BLOCKED_BY_GUARDRAIL") return null;
  return (
    <section className="panel checkout-panel" id="payment-review" aria-label="PayPal checkout">
      <div className="checkout-heading"><h2>Payment review</h2><span className="mini-label">{transaction.payment_mode === "sandbox" ? "Sandbox" : "Simulation"}</span></div>
      <p className="checkout-description">{config?.message || "Connecting to the payment service…"}</p>
      {config?.mode === "sandbox" && config.configured && <Script id="paypal-sdk-v6" src={config.sdk_url} strategy="afterInteractive" onReady={() => setReady(true)} onError={() => setError("Unable to load PayPal. Check your connection and retry.")} />}
      <p className="payment-order-reference">{transaction.order_id} <span>{transaction.status.replaceAll("_", " ").toLowerCase()}</span></p>
      {awaiting && transaction.payment_mode === config?.mode && <div className="approval-actions"><p>Approve <strong>{"$"}{transaction.total_amount.toFixed(2)} {transaction.currency}</strong> to pay the ProcureOps test merchant. A vendor payout is a separate action.</p>{config.mode === "simulation" ? <button type="button" disabled={busy} onClick={approve} className="button primary full-width">Simulate buyer approval & capture</button> : <div ref={container} className="paypal-button-container" />}<button type="button" onClick={cancel} disabled={busy} className="text-button cancel-checkout">Cancel checkout</button></div>}
      <p className="payment-reference">Capture ID: <code>{transaction.capture_id || "Not captured"}</code></p>
      <details className="payment-parser"><summary>Interpretation details</summary><p>Parser: {transaction.parser_mode}</p></details>
      {transaction.paypal_order_id && <button type="button" disabled={busy} onClick={refresh} className="text-button">Refresh payment status</button>}
      {transaction.payment_error && <p className="notice warning">{transaction.payment_error}</p>}
      {error && <p role="alert" className="notice danger">{error}</p>}
    </section>
  );
}

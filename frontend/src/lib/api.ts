import {
  TransactionsResponse,
  MandateResponse,
  AnalyticsQueryResponse,
  PayoutResponse,
  HealthResponse,
  CheckoutConfig,
  PaymentResponse,
} from "../types";

const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:8000";

export async function fetchHealth(): Promise<HealthResponse> {
  const res = await fetch(`${API_BASE_URL}/api/health`, { cache: "no-store" });
  if (!res.ok) {
    throw new Error(`Failed to fetch health: ${res.statusText}`);
  }
  return res.json();
}

export async function fetchTransactions(): Promise<TransactionsResponse> {
  const res = await fetch(`${API_BASE_URL}/api/transactions`, {
    cache: "no-store",
  });
  if (!res.ok) {
    throw new Error(`Failed to fetch transactions: ${res.statusText}`);
  }
  return res.json();
}

export async function executeMandate(
  prompt: string,
  authorized_budget: number
): Promise<MandateResponse> {
  const res = await fetch(`${API_BASE_URL}/api/mandate`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ prompt, authorized_budget, request_id: crypto.randomUUID() }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ detail: res.statusText }));
    throw new Error(err.detail || "Failed to execute procurement mandate");
  }
  return res.json();
}

export async function fetchCheckoutConfig(): Promise<CheckoutConfig> {
  const response = await fetch(`${API_BASE_URL}/api/paypal/config`, { cache: "no-store" });
  if (!response.ok) throw new Error("Payment configuration unavailable");
  return response.json();
}
async function paymentAction<T>(path: string, order_id: string): Promise<T> {
  const response = await fetch(`${API_BASE_URL}/api/paypal/${path}`, {
    method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ order_id }),
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ detail: response.statusText }));
    throw new Error(error.detail || "Payment operation failed");
  }
  return response.json();
}
export const capturePayment = (id: string) => paymentAction<PaymentResponse>("capture", id);
export const cancelPayment = (id: string) => paymentAction<PaymentResponse>("cancel", id);
export const refreshPayment = (id: string) => paymentAction<PaymentResponse>("payment/status", id);
export const refreshPayout = (id: string) => paymentAction<PayoutResponse>("payout/status", id);

export async function queryAnalytics(
  query: string
): Promise<AnalyticsQueryResponse> {
  const res = await fetch(`${API_BASE_URL}/api/analytics/query`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ query }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ detail: res.statusText }));
    throw new Error(err.detail || "Failed to query treasury analytics");
  }
  return res.json();
}

export async function settlePayout(order_id: string): Promise<PayoutResponse> {
  const res = await fetch(`${API_BASE_URL}/api/paypal/payout`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ order_id }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ detail: res.statusText }));
    throw new Error(err.detail || "Failed to settle PayPal payout");
  }
  return res.json();
}

export async function resetDatabase(): Promise<{ success: boolean; message: string }> {
  const res = await fetch(`${API_BASE_URL}/api/reset`, {
    method: "POST",
  });
  if (!res.ok) {
    throw new Error("Failed to reset transactions");
  }
  return res.json();
}

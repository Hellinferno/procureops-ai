export type TransactionStatus =
  | "COMPLETED"
  | "SETTLED"
  | "BLOCKED_BY_GUARDRAIL"
  | "APPROVED_PENDING_SETTLEMENT"
  | "AWAITING_APPROVAL" | "PAYMENT_PENDING" | "PAYMENT_FAILED"
  | "CANCELLED" | "PAYOUT_PENDING" | "PAYOUT_FAILED";

export interface Transaction {
  order_id: string;
  paypal_order_id: string | null;
  timestamp: string;
  vendor: string;
  vendor_email: string;
  category: string;
  item_name: string;
  quantity: number;
  unit_price: number;
  total_amount: number;
  authorized_budget: number;
  currency: string;
  delivery_days: number;
  vendor_rating: number;
  status: TransactionStatus;
  risk_score: number;
  guardrail_passed: boolean;
  guardrail_reason: string;
  agent_reasoning: string;
  payout_batch_id: string | null;
  payment_mode: "seed" | "simulation" | "sandbox";
  parser_mode: string;
  capture_id: string | null;
  approval_url: string | null;
  payment_error: string | null;
  payout_error: string | null;
  payout_sender_batch_id: string | null;
}

export interface KpiSummary {
  total_spend: number;
  completed_count: number;
  blocked_count: number;
  money_saved_by_guardrails: number;
  avg_risk_score: number;
}

export interface TransactionsResponse {
  transactions: Transaction[];
  kpis: KpiSummary;
}

export interface MandateStep {
  step: number;
  title: string;
  status: "done" | "blocked" | "failed" | "pending";
  detail: string;
}

export interface MandateResponse {
  success: boolean;
  transaction: Transaction | null;
  steps: MandateStep[];
  paypal_raw?: Record<string, unknown> | null;
}

export interface AnalyticsQueryResponse {
  query: string;
  filter_expr: string;
  summary: string;
  records: Transaction[];
  parser_mode: string;
}

export interface PayoutResponse {
  success: boolean;
  order_id: string;
  payout_batch_id: string | null;
  new_status: string;
  message: string;
}

export interface HealthResponse {
  status: string;
  service: string;
  paypal_live_configured: boolean;
  paypal_mode: string;
  llm_configured: boolean;
}

export interface CheckoutConfig {
  mode: "simulation" | "sandbox";
  configured: boolean;
  client_id: string | null;
  sdk_url: string;
  payouts_enabled: boolean;
  webhook_configured: boolean;
  message: string;
}
export interface PaymentResponse {
  success: boolean;
  transaction: Transaction;
  paypal_raw: Record<string, unknown> | null;
  message: string;
}

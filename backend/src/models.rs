use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Completed,
    Settled,
    BlockedByGuardrail,
    ApprovedPendingSettlement,
    AwaitingApproval,
    PaymentPending,
    PaymentFailed,
    Cancelled,
    PayoutPending,
    PayoutFailed,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::Completed => "COMPLETED",
            TransactionStatus::Settled => "SETTLED",
            TransactionStatus::BlockedByGuardrail => "BLOCKED_BY_GUARDRAIL",
            TransactionStatus::ApprovedPendingSettlement => "APPROVED_PENDING_SETTLEMENT",
            TransactionStatus::AwaitingApproval => "AWAITING_APPROVAL",
            TransactionStatus::PaymentPending => "PAYMENT_PENDING",
            TransactionStatus::PaymentFailed => "PAYMENT_FAILED",
            TransactionStatus::Cancelled => "CANCELLED",
            TransactionStatus::PayoutPending => "PAYOUT_PENDING",
            TransactionStatus::PayoutFailed => "PAYOUT_FAILED",
        }
    }
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub order_id: String,
    pub paypal_order_id: Option<String>,
    pub timestamp: String,
    pub vendor: String,
    pub vendor_email: String,
    pub category: String,
    pub item_name: String,
    pub quantity: u32,
    pub unit_price: f64,
    pub total_amount: f64,
    pub authorized_budget: f64,
    pub currency: String,
    pub delivery_days: u32,
    pub vendor_rating: f64,
    pub status: TransactionStatus,
    pub risk_score: f64,
    pub guardrail_passed: bool,
    pub guardrail_reason: String,
    pub agent_reasoning: String,
    pub payout_batch_id: Option<String>,
    #[serde(default = "legacy_mode")]
    pub payment_mode: String,
    #[serde(default = "legacy_mode")]
    pub parser_mode: String,
    #[serde(default)]
    pub capture_id: Option<String>,
    #[serde(default)]
    pub approval_url: Option<String>,
    #[serde(default)]
    pub payment_error: Option<String>,
    #[serde(default)]
    pub payout_error: Option<String>,
    #[serde(default)]
    pub payout_sender_batch_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
}

fn legacy_mode() -> String {
    "seed".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogItem {
    pub id: String,
    pub item_name: String,
    pub category: String,
    pub vendor: String,
    pub vendor_email: String,
    pub unit_price: f64,
    pub min_quantity: u32,
    pub max_quantity: u32,
    pub delivery_days: u32,
    pub vendor_rating: f64,
    pub in_stock: bool,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct MandateRequest {
    pub prompt: String,
    pub authorized_budget: f64,
    #[serde(default)]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Done,
    Blocked,
    Failed,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MandateStep {
    pub step: u8,
    pub title: String,
    pub status: StepStatus,
    pub detail: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MandateResponse {
    pub success: bool,
    pub transaction: Option<Transaction>,
    pub steps: Vec<MandateStep>,
    pub paypal_raw: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsQueryRequest {
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsQueryResponse {
    pub query: String,
    pub filter_expr: String,
    pub summary: String,
    pub records: Vec<Transaction>,
    pub parser_mode: String,
}

#[derive(Debug, Deserialize)]
pub struct PayoutRequest {
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayoutResponse {
    pub success: bool,
    pub order_id: String,
    pub payout_batch_id: Option<String>,
    pub new_status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiSummary {
    pub total_spend: f64,
    pub completed_count: usize,
    pub blocked_count: usize,
    pub money_saved_by_guardrails: f64,
    pub avg_risk_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionsResponse {
    pub transactions: Vec<Transaction>,
    pub kpis: KpiSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub paypal_live_configured: bool,
    pub paypal_mode: String,
    pub llm_configured: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckoutConfig {
    pub mode: String,
    pub configured: bool,
    pub client_id: Option<String>,
    pub sdk_url: String,
    pub payouts_enabled: bool,
    pub webhook_configured: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub success: bool,
    pub transaction: Transaction,
    pub paypal_raw: Option<Value>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub detail: String,
}

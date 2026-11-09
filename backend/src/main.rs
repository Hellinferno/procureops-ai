mod agent;
mod analytics;
mod guardrails;
#[cfg(test)]
mod integration_tests;
mod llm;
mod models;
mod paypal;

use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use models::*;
use paypal::PayPalClient;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{Mutex, RwLock};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

type SharedState = Arc<RwLock<AppState>>;
struct AppState {
    transactions: Vec<Transaction>,
    catalog: Vec<CatalogItem>,
    paypal: PayPalClient,
    transactions_path: PathBuf,
    seed_path: PathBuf,
    operations: Arc<Mutex<()>>,
}
fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}
fn error_response(status: StatusCode, detail: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorBody {
            detail: detail.into(),
        }),
    )
        .into_response()
}
// All callers hold operations: serialize mutation + durable write + publication.
async fn commit(state: &SharedState, transactions: Vec<Transaction>) -> Result<(), String> {
    let path = state.read().await.transactions_path.clone();
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let payload = serde_json::to_vec_pretty(&transactions).map_err(|e| e.to_string())?;
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::File::create(&temporary)
        .await
        .map_err(|e| e.to_string())?;
    file.write_all(&payload).await.map_err(|e| e.to_string())?;
    file.sync_all().await.map_err(|e| e.to_string())?;
    drop(file);
    tokio::fs::rename(&temporary, &path)
        .await
        .map_err(|e| format!("Ledger write failed: {e}"))?;
    state.write().await.transactions = transactions;
    Ok(())
}
async fn save_transaction(state: &SharedState, transaction: Transaction) -> Result<(), String> {
    let mut records = state.read().await.transactions.clone();
    match records
        .iter()
        .position(|t| t.order_id == transaction.order_id)
    {
        Some(i) => records[i] = transaction,
        None => records.push(transaction),
    };
    commit(state, records).await
}
async fn find_transaction(state: &SharedState, id: &str) -> Option<Transaction> {
    state
        .read()
        .await
        .transactions
        .iter()
        .find(|t| t.order_id == id)
        .cloned()
}
async fn health(State(state): State<SharedState>) -> Json<HealthResponse> {
    let state = state.read().await;
    Json(HealthResponse {
        status: "healthy".into(),
        service: "ProcureOps AI (Rust)".into(),
        paypal_mode: state.paypal.mode().into(),
        paypal_live_configured: state.paypal.is_live_configured(),
        llm_configured: agent::llm_configured(),
    })
}
async fn checkout_config(State(state): State<SharedState>) -> Json<CheckoutConfig> {
    Json(state.read().await.paypal.config())
}
async fn get_catalog(State(state): State<SharedState>) -> Json<Vec<CatalogItem>> {
    Json(state.read().await.catalog.clone())
}
async fn get_transactions(State(state): State<SharedState>) -> Json<TransactionsResponse> {
    let state = state.read().await;
    Json(TransactionsResponse {
        transactions: state.transactions.clone(),
        kpis: analytics::compute_kpis(&state.transactions),
    })
}
async fn execute_mandate(
    State(state): State<SharedState>,
    Json(request): Json<MandateRequest>,
) -> Response {
    if request.prompt.trim().is_empty()
        || request.prompt.len() > 8000
        || !request.authorized_budget.is_finite()
        || request.authorized_budget <= 0.0
    {
        return error_response(
            StatusCode::BAD_REQUEST,
            "Enter a mandate and a positive budget (maximum 8,000 characters)",
        );
    }
    let request_id = request
        .request_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    if uuid::Uuid::parse_str(&request_id).is_err() {
        return error_response(StatusCode::BAD_REQUEST, "request_id must be a UUID");
    }
    let operations = state.read().await.operations.clone();
    let _operation = operations.lock().await;
    if let Some(existing) = state
        .read()
        .await
        .transactions
        .iter()
        .find(|t| t.request_id.as_deref() == Some(&request_id))
        .cloned()
    {
        return Json(MandateResponse {
            success: existing.guardrail_passed && existing.paypal_order_id.is_some(),
            transaction: Some(existing),
            steps: vec![],
            paypal_raw: None,
        })
        .into_response();
    }
    let (catalog, paypal) = {
        let state = state.read().await;
        (state.catalog.clone(), state.paypal.clone())
    };
    let id = format!(
        "ORD-{}",
        &uuid::Uuid::parse_str(&request_id)
            .unwrap()
            .simple()
            .to_string()[..20]
    );
    let mut response = agent::execute_mandate(
        request.prompt.trim(),
        request.authorized_budget,
        &catalog,
        &paypal,
        id,
    )
    .await;
    if let Some(transaction) = response.transaction.as_mut() {
        transaction.request_id = Some(request_id);
        if let Err(e) = save_transaction(&state, transaction.clone()).await {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, e);
        }
    }
    Json(response).into_response()
}
async fn query_analytics(
    State(state): State<SharedState>,
    Json(request): Json<AnalyticsQueryRequest>,
) -> Response {
    if request.query.trim().is_empty() || request.query.len() > 4000 {
        return error_response(
            StatusCode::BAD_REQUEST,
            "Enter a treasury query of at most 4,000 characters",
        );
    }
    let transactions = state.read().await.transactions.clone();
    Json(analytics::execute_query(&request.query, &transactions).await).into_response()
}
fn payment_response(
    transaction: Transaction,
    raw: Option<Value>,
    message: impl Into<String>,
) -> Response {
    let success = matches!(
        transaction.status,
        TransactionStatus::Completed
            | TransactionStatus::Settled
            | TransactionStatus::PayoutPending
            | TransactionStatus::PayoutFailed
    );
    Json(PaymentResponse {
        success,
        transaction,
        paypal_raw: raw,
        message: message.into(),
    })
    .into_response()
}
async fn capture_payment(
    State(state): State<SharedState>,
    Json(request): Json<PayoutRequest>,
) -> Response {
    let operations = state.read().await.operations.clone();
    let _operation = operations.lock().await;
    let Some(mut transaction) = find_transaction(&state, &request.order_id).await else {
        return error_response(StatusCode::NOT_FOUND, "Order not found");
    };
    if matches!(
        transaction.status,
        TransactionStatus::Completed
            | TransactionStatus::Settled
            | TransactionStatus::PayoutPending
            | TransactionStatus::PayoutFailed
    ) {
        return payment_response(
            transaction,
            None,
            "Payment was already captured; no additional charge was made",
        );
    }
    if !transaction.guardrail_passed
        || !matches!(
            transaction.status,
            TransactionStatus::AwaitingApproval | TransactionStatus::PaymentPending
        )
    {
        return error_response(
            StatusCode::CONFLICT,
            "This order is not eligible for capture",
        );
    }
    let paypal = state.read().await.paypal.clone();
    if transaction.payment_mode != paypal.mode() {
        return error_response(
            StatusCode::CONFLICT,
            "Restore the original payment mode to capture this order",
        );
    }
    transaction.status = TransactionStatus::PaymentPending;
    if let Err(e) = save_transaction(&state, transaction.clone()).await {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, e);
    }
    match paypal.capture_order(&transaction).await {
        Ok(raw) => match paypal::capture_details(&raw, &transaction) {
            Ok((id, completed)) => {
                transaction.capture_id = Some(id);
                transaction.payment_error = None;
                transaction.status = if completed {
                    TransactionStatus::Completed
                } else {
                    TransactionStatus::PaymentPending
                };
                if let Err(e) = save_transaction(&state, transaction.clone()).await {
                    return error_response(StatusCode::INTERNAL_SERVER_ERROR, e);
                }
                payment_response(
                    transaction,
                    Some(raw),
                    if completed {
                        "Capture confirmed"
                    } else {
                        "Capture pending; refresh payment status"
                    },
                )
            }
            Err(e) => {
                transaction.payment_error = Some(e.to_string());
                if let Err(err) = save_transaction(&state, transaction).await {
                    return error_response(StatusCode::INTERNAL_SERVER_ERROR, err);
                }
                error_response(StatusCode::BAD_GATEWAY, e.to_string())
            }
        },
        Err(e) => {
            if !e.uncertain && e.message.contains("ORDER_NOT_APPROVED") {
                transaction.status = TransactionStatus::AwaitingApproval;
            } else if !e.uncertain {
                transaction.status = TransactionStatus::PaymentFailed;
            }
            transaction.payment_error = Some(e.to_string());
            if let Err(err) = save_transaction(&state, transaction).await {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, err);
            }
            error_response(StatusCode::BAD_GATEWAY, e.to_string())
        }
    }
}
async fn payment_status(
    State(state): State<SharedState>,
    Json(request): Json<PayoutRequest>,
) -> Response {
    let operations = state.read().await.operations.clone();
    let _operation = operations.lock().await;
    let Some(mut transaction) = find_transaction(&state, &request.order_id).await else {
        return error_response(StatusCode::NOT_FOUND, "Order not found");
    };
    let paypal = state.read().await.paypal.clone();
    if transaction.payment_mode != paypal.mode() {
        return error_response(
            StatusCode::CONFLICT,
            "Restore the original payment mode to check this payment",
        );
    }
    if transaction.payment_mode == "simulation"
        || !matches!(
            transaction.status,
            TransactionStatus::AwaitingApproval
                | TransactionStatus::PaymentPending
                | TransactionStatus::PaymentFailed
        )
    {
        return payment_response(transaction, None, "Ledger status loaded");
    }
    let Some(id) = transaction.paypal_order_id.as_deref() else {
        return payment_response(transaction, None, "No PayPal order was created");
    };
    match paypal.get_order(id).await {
        Ok(raw) => {
            if raw.pointer("/purchase_units/0/payments/captures").is_some() {
                match paypal::capture_details(&raw, &transaction) {
                    Ok((id, done)) => {
                        transaction.capture_id = Some(id);
                        transaction.status = if done {
                            TransactionStatus::Completed
                        } else {
                            TransactionStatus::PaymentPending
                        };
                        transaction.payment_error = None;
                    }
                    Err(e) => return error_response(StatusCode::BAD_GATEWAY, e.to_string()),
                }
                if let Err(e) = save_transaction(&state, transaction.clone()).await {
                    return error_response(StatusCode::INTERNAL_SERVER_ERROR, e);
                }
            }
            payment_response(
                transaction,
                Some(raw),
                "Payment status retrieved from PayPal",
            )
        }
        Err(e) => error_response(StatusCode::BAD_GATEWAY, e.to_string()),
    }
}
async fn cancel_payment(
    State(state): State<SharedState>,
    Json(request): Json<PayoutRequest>,
) -> Response {
    let operations = state.read().await.operations.clone();
    let _operation = operations.lock().await;
    let Some(mut transaction) = find_transaction(&state, &request.order_id).await else {
        return error_response(StatusCode::NOT_FOUND, "Order not found");
    };
    if transaction.status != TransactionStatus::AwaitingApproval {
        return error_response(
            StatusCode::CONFLICT,
            "Only an uncaptured order awaiting approval can be cancelled",
        );
    }
    transaction.status = TransactionStatus::Cancelled;
    if let Err(e) = save_transaction(&state, transaction.clone()).await {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, e);
    }
    payment_response(
        transaction,
        None,
        "Local checkout cancelled; no capture was requested",
    )
}
fn payout_response(transaction: &Transaction, message: impl Into<String>) -> Response {
    Json(PayoutResponse {
        success: transaction.status == TransactionStatus::Settled,
        order_id: transaction.order_id.clone(),
        payout_batch_id: transaction.payout_batch_id.clone(),
        new_status: transaction.status.to_string(),
        message: message.into(),
    })
    .into_response()
}
async fn trigger_payout(
    State(state): State<SharedState>,
    Json(request): Json<PayoutRequest>,
) -> Response {
    let operations = state.read().await.operations.clone();
    let _operation = operations.lock().await;
    let Some(mut transaction) = find_transaction(&state, &request.order_id).await else {
        return error_response(StatusCode::NOT_FOUND, "Order not found");
    };
    if transaction.status == TransactionStatus::Settled {
        return payout_response(&transaction, "Already settled; no additional payout sent");
    }
    if transaction.payout_batch_id.is_some() {
        return payout_response(
            &transaction,
            "Payout exists. Refresh its status; no additional payout sent",
        );
    }
    if !transaction.guardrail_passed
        || transaction.capture_id.is_none()
        || !matches!(
            transaction.status,
            TransactionStatus::Completed | TransactionStatus::PayoutPending
        )
    {
        return error_response(StatusCode::CONFLICT,"Only captured, policy-approved purchases can be paid out. Historical seed records are read-only.");
    }
    let paypal = state.read().await.paypal.clone();
    if transaction.payment_mode != paypal.mode() || !paypal.config().payouts_enabled {
        return error_response(
            StatusCode::CONFLICT,
            "Configure the matching payment mode and an owned Sandbox payout receiver",
        );
    }
    transaction
        .payout_sender_batch_id
        .get_or_insert_with(|| format!("payout-{}", transaction.order_id));
    transaction.status = TransactionStatus::PayoutPending;
    if let Err(e) = save_transaction(&state, transaction.clone()).await {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, e);
    }
    match paypal.create_payout(&transaction).await {
        Ok(raw) => {
            let Some(batch) = paypal::payout_batch_id(&raw) else {
                return error_response(StatusCode::BAD_GATEWAY,"PayPal returned no payout batch ID; retain the reserved payout for reconciliation");
            };
            transaction.payout_batch_id = Some(batch);
            transaction.payout_error = None;
            if let Err(e) = save_transaction(&state, transaction.clone()).await {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, e);
            }
            payout_response(
                &transaction,
                format!(
                    "{} payout accepted. Refresh status to confirm recipient completion.",
                    transaction.payment_mode
                ),
            )
        }
        Err(e) => {
            if !e.uncertain {
                transaction.status = TransactionStatus::PayoutFailed;
            }
            transaction.payout_error = Some(e.to_string());
            if let Err(err) = save_transaction(&state, transaction).await {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, err);
            }
            error_response(StatusCode::BAD_GATEWAY, e.to_string())
        }
    }
}
async fn reconcile_payout(
    state: &SharedState,
    mut transaction: Transaction,
) -> Result<Transaction, String> {
    let paypal = state.read().await.paypal.clone();
    let raw = paypal
        .get_payout(&transaction)
        .await
        .map_err(|e| e.to_string())?;
    if paypal::payout_batch_id(&raw) != transaction.payout_batch_id {
        return Err("Payout batch ID mismatch".into());
    }
    transaction.status =
        match paypal::payout_state(&raw, &transaction).map_err(|e| e.to_string())? {
            "settled" => TransactionStatus::Settled,
            "failed" => TransactionStatus::PayoutFailed,
            _ => TransactionStatus::PayoutPending,
        };
    transaction.payout_error = if transaction.status == TransactionStatus::PayoutFailed {
        Some("PayPal payout was not completed; inspect the batch in the Sandbox dashboard".into())
    } else {
        None
    };
    save_transaction(state, transaction.clone()).await?;
    Ok(transaction)
}
async fn payout_status(
    State(state): State<SharedState>,
    Json(request): Json<PayoutRequest>,
) -> Response {
    let operations = state.read().await.operations.clone();
    let _operation = operations.lock().await;
    let Some(transaction) = find_transaction(&state, &request.order_id).await else {
        return error_response(StatusCode::NOT_FOUND, "Order not found");
    };
    match reconcile_payout(&state, transaction).await {
        Ok(t) => payout_response(&t, format!("Verified payout status: {}", t.status)),
        Err(e) => error_response(StatusCode::BAD_GATEWAY, e),
    }
}
async fn webhook(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(event): Json<Value>,
) -> Response {
    let paypal = state.read().await.paypal.clone();
    if let Err(e) = paypal.verify_webhook(&headers, &event).await {
        return error_response(StatusCode::UNAUTHORIZED, e.to_string());
    }
    let operations = state.read().await.operations.clone();
    let _operation = operations.lock().await;
    let kind = event
        .get("event_type")
        .and_then(Value::as_str)
        .unwrap_or("");
    if kind == "PAYMENT.CAPTURE.COMPLETED" || kind == "PAYMENT.CAPTURE.PENDING" {
        let order = event
            .pointer("/resource/supplementary_data/related_ids/order_id")
            .and_then(Value::as_str);
        let transaction = state
            .read()
            .await
            .transactions
            .iter()
            .find(|t| t.paypal_order_id.as_deref() == order && order.is_some())
            .cloned();
        if let Some(mut t) = transaction {
            if !matches!(
                t.status,
                TransactionStatus::PayoutPending
                    | TransactionStatus::PayoutFailed
                    | TransactionStatus::Settled
            ) {
                let raw = match paypal
                    .get_order(t.paypal_order_id.as_deref().unwrap())
                    .await
                {
                    Ok(raw) => raw,
                    Err(e) => return error_response(StatusCode::BAD_GATEWAY, e.to_string()),
                };
                let (id, done) = match paypal::capture_details(&raw, &t) {
                    Ok(d) => d,
                    Err(e) => return error_response(StatusCode::BAD_GATEWAY, e.to_string()),
                };
                t.capture_id = Some(id);
                t.status = if done {
                    TransactionStatus::Completed
                } else {
                    TransactionStatus::PaymentPending
                };
                t.payment_error = None;
                if let Err(e) = save_transaction(&state, t).await {
                    return error_response(StatusCode::INTERNAL_SERVER_ERROR, e);
                }
            }
        }
    } else if kind.starts_with("PAYMENT.PAYOUTS-") {
        let batch = event
            .pointer("/resource/batch_header/payout_batch_id")
            .or_else(|| event.pointer("/resource/payout_batch_id"))
            .and_then(Value::as_str);
        let transaction = state
            .read()
            .await
            .transactions
            .iter()
            .find(|t| t.payout_batch_id.as_deref() == batch && batch.is_some())
            .cloned();
        if let Some(t) = transaction {
            if let Err(e) = reconcile_payout(&state, t).await {
                return error_response(StatusCode::BAD_GATEWAY, e);
            }
        }
    }
    Json(json!({"received":true})).into_response()
}
async fn reset_data(State(state): State<SharedState>) -> Response {
    let operations = state.read().await.operations.clone();
    let _operation = operations.lock().await;
    let state_guard = state.read().await;
    if state_guard.paypal.mode() == "sandbox"
        || state_guard
            .transactions
            .iter()
            .any(|t| t.payment_mode == "sandbox")
    {
        return error_response(
            StatusCode::CONFLICT,
            "Sandbox payment records cannot be cleared by demo reset",
        );
    }
    let path = state_guard.seed_path.clone();
    drop(state_guard);
    let seed = match load_json::<Vec<Transaction>>(&path) {
        Ok(seed) => seed,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };
    if let Err(e) = commit(&state, seed).await {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, e);
    }
    Json(ResetResponse {
        success: true,
        message: "Simulation ledger restored to historical seed data".into(),
    })
    .into_response()
}
fn build_app(state: SharedState) -> Router {
    let origins = std::env::var("FRONTEND_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://127.0.0.1:3000".into());
    let origins: Vec<HeaderValue> = origins
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    Router::new()
        .route("/api/health", get(health))
        .route("/api/catalog", get(get_catalog))
        .route("/api/transactions", get(get_transactions))
        .route("/api/mandate", post(execute_mandate))
        .route("/api/analytics/query", post(query_analytics))
        .route("/api/paypal/config", get(checkout_config))
        .route("/api/paypal/capture", post(capture_payment))
        .route("/api/paypal/payment/status", post(payment_status))
        .route("/api/paypal/cancel", post(cancel_payment))
        .route("/api/paypal/payout", post(trigger_payout))
        .route("/api/paypal/payout/status", post(payout_status))
        .route("/api/paypal/webhook", post(webhook))
        .route("/api/reset", post(reset_data))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([axum::http::header::CONTENT_TYPE]),
        )
        .with_state(state)
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::from_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env")).ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "procureops_backend=info,tower_http=info".into()),
        )
        .init();
    let data = std::env::var("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"));
    let transactions_path = data.join("transactions.json");
    let seed_path = data.join("seed_transactions.json");
    let state = Arc::new(RwLock::new(AppState {
        transactions: load_json(&transactions_path)?,
        catalog: load_json(&data.join("catalog.json"))?,
        paypal: PayPalClient::from_env()?,
        transactions_path,
        seed_path,
        operations: Arc::new(Mutex::new(())),
    }));
    let host = std::env::var("BIND_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".into());
    let address: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(
        "ProcureOps Rust API: http://{}; PayPal mode: {}",
        listener.local_addr()?,
        state.read().await.paypal.mode()
    );
    axum::serve(listener, build_app(state)).await?;
    Ok(())
}

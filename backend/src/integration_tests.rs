use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::task::JoinHandle;

struct Harness {
    url: String,
    client: reqwest::Client,
    task: JoinHandle<()>,
    dir: PathBuf,
}
impl Drop for Harness {
    fn drop(&mut self) {
        self.task.abort();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
impl Harness {
    async fn post(&self, path: &str, body: Value) -> (StatusCode, Value) {
        let response = self
            .client
            .post(format!("{}{path}", self.url))
            .json(&body)
            .send()
            .await
            .unwrap();
        (response.status(), response.json().await.unwrap())
    }
    async fn prepare(&self) -> Value {
        let (status,body)=self.post("/api/mandate",json!({"prompt":"Buy 10 Raspberry Pi 5 under $900 within 3 days","authorized_budget":900})).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body
    }
}
async fn serve(paypal: PayPalClient) -> Harness {
    let dir = std::env::temp_dir().join(format!("procureops-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let seed = include_str!("../data/seed_transactions.json");
    std::fs::write(dir.join("transactions.json"), seed).unwrap();
    std::fs::write(dir.join("seed_transactions.json"), seed).unwrap();
    let state = Arc::new(RwLock::new(AppState {
        transactions: serde_json::from_str(seed).unwrap(),
        catalog: serde_json::from_str(include_str!("../data/catalog.json")).unwrap(),
        paypal,
        transactions_path: dir.join("transactions.json"),
        seed_path: dir.join("seed_transactions.json"),
        operations: Arc::new(Mutex::new(())),
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move { axum::serve(listener, build_app(state)).await.unwrap() });
    Harness {
        url,
        task,
        dir,
        client: reqwest::Client::builder().no_proxy().build().unwrap(),
    }
}
struct Mock {
    url: String,
    task: JoinHandle<()>,
    captures: Arc<AtomicUsize>,
    payouts: Arc<AtomicUsize>,
}
impl Drop for Mock {
    fn drop(&mut self) {
        self.task.abort();
    }
}
#[derive(Clone)]
struct MockState {
    order: Arc<RwLock<Value>>,
    captures: Arc<AtomicUsize>,
    payouts: Arc<AtomicUsize>,
    scenario: &'static str,
}
async fn provider(scenario: &'static str) -> Mock {
    let state = MockState {
        order: Arc::new(RwLock::new(json!({}))),
        captures: Arc::new(AtomicUsize::new(0)),
        payouts: Arc::new(AtomicUsize::new(0)),
        scenario,
    };
    let app=Router::new()
        .route("/v1/oauth2/token",post(||async{Json(json!({"access_token":"test-token"}))}))
        .route("/v2/checkout/orders",post(|State(s):State<MockState>,headers:HeaderMap,Json(body):Json<Value>|async move{
            assert!(headers.contains_key("paypal-request-id"));
            if s.scenario=="create_error" {return (StatusCode::UNPROCESSABLE_ENTITY,Json(json!({"name":"UNPROCESSABLE_ENTITY","details":[{"issue":"INSTRUMENT_DECLINED"}]})));}
            *s.order.write().await=body;
            (StatusCode::CREATED,Json(json!({"id":"TEST-PAYPAL-ORDER","status":"CREATED"})))
        }))
        .route("/v2/checkout/orders/:id",get(|State(s):State<MockState>|async move{
            let body=s.order.read().await.clone();
            if s.captures.load(Ordering::SeqCst)>0 {
                Json(capture_payload(&body,s.scenario))
            } else {Json(json!({"id":"TEST-PAYPAL-ORDER","status":if s.scenario=="unapproved"{"CREATED"}else{"APPROVED"},"purchase_units":body["purchase_units"]}))}
        }))
        .route("/v2/checkout/orders/:id/capture",post(|State(s):State<MockState>,headers:HeaderMap|async move{
            assert!(headers.get("paypal-request-id").unwrap().to_str().unwrap().starts_with("capture-ORD-"));
            s.captures.fetch_add(1,Ordering::SeqCst);
            if s.scenario=="capture_error" {return (StatusCode::UNPROCESSABLE_ENTITY,Json(json!({"name":"UNPROCESSABLE_ENTITY","details":[{"issue":"INSTRUMENT_DECLINED"}]})));}
            (StatusCode::CREATED,Json(capture_payload(&*s.order.read().await,s.scenario)))
        }))
        .route("/v1/payments/payouts",post(|State(s):State<MockState>,headers:HeaderMap,Json(body):Json<Value>|async move{
            assert_eq!(headers.get("paypal-request-id").unwrap().to_str().unwrap(),body["sender_batch_header"]["sender_batch_id"].as_str().unwrap());
            assert_eq!(body["items"][0]["receiver"],"owned-recipient@example.test");
            s.payouts.fetch_add(1,Ordering::SeqCst);
            (StatusCode::CREATED,Json(json!({"batch_header":{"payout_batch_id":"TEST-BATCH","batch_status":"PENDING"}})))
        }))
        .route("/v1/payments/payouts/:id",get(|State(s):State<MockState>|async move{
            let body=s.order.read().await.clone();
            Json(json!({"batch_header":{"payout_batch_id":"TEST-BATCH","batch_status":"SUCCESS"},"items":[{
                "transaction_status":if s.scenario=="unclaimed"{"UNCLAIMED"}else{"SUCCESS"},
                "payout_item":{"sender_item_id":body["purchase_units"][0]["custom_id"],"amount":{"currency":"USD","value":body["purchase_units"][0]["amount"]["value"]}}
            }]}))
        }))
        .route("/v1/notifications/verify-webhook-signature",post(|Json(body):Json<Value>|async move{
            Json(json!({"verification_status":if body["transmission_sig"]=="valid"{"SUCCESS"}else{"FAILURE"}}))
        })).with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    Mock {
        url,
        task,
        captures: state.captures,
        payouts: state.payouts,
    }
}
fn capture_payload(body: &Value, scenario: &str) -> Value {
    json!({"id":"TEST-PAYPAL-ORDER","status":"COMPLETED","purchase_units":[{
        "custom_id":body["purchase_units"][0]["custom_id"],"payments":{"captures":[{
            "id":"TEST-CAPTURE","status":"COMPLETED","amount":{"currency_code":"USD","value":if scenario=="amount_mismatch"{json!("1.00")}else{body["purchase_units"][0]["amount"]["value"].clone()}}
        }]}
    }]})
}
#[tokio::test]
async fn simulation_lifecycle_is_labelled_and_seed_cannot_be_paid() {
    let h = serve(PayPalClient::for_test(String::new(), false)).await;
    let purchase = h.prepare().await;
    assert_eq!(purchase["transaction"]["status"], "AWAITING_APPROVAL");
    assert_eq!(purchase["transaction"]["payment_mode"], "simulation");
    let id = purchase["transaction"]["order_id"].clone();
    let (_, captured) = h.post("/api/paypal/capture", json!({"order_id":id})).await;
    assert_eq!(captured["transaction"]["status"], "COMPLETED");
    let (_, payout) = h.post("/api/paypal/payout", json!({"order_id":id})).await;
    assert_eq!(payout["new_status"], "PAYOUT_PENDING");
    assert_eq!(payout["success"], false);
    let (_, payout) = h
        .post("/api/paypal/payout/status", json!({"order_id":id}))
        .await;
    assert_eq!(payout["new_status"], "SETTLED");
    let (_, again) = h.post("/api/paypal/payout", json!({"order_id":id})).await;
    assert_eq!(again["new_status"], "SETTLED");
    assert_eq!(
        h.post("/api/paypal/payout", json!({"order_id":"ORD-2026-1002"}))
            .await
            .0,
        StatusCode::CONFLICT
    );
}
#[tokio::test]
async fn cancellation_and_unknown_products_cannot_be_captured() {
    let h = serve(PayPalClient::for_test(String::new(), false)).await;
    let p = h.prepare().await;
    let id = p["transaction"]["order_id"].clone();
    assert_eq!(
        h.post("/api/paypal/cancel", json!({"order_id":id})).await.1["transaction"]["status"],
        "CANCELLED"
    );
    assert_eq!(
        h.post("/api/paypal/capture", json!({"order_id":id}))
            .await
            .0,
        StatusCode::CONFLICT
    );
    let (_, unknown) = h
        .post(
            "/api/mandate",
            json!({"prompt":"Buy 5 unicorn plushies under $900","authorized_budget":900}),
        )
        .await;
    assert!(unknown["transaction"].is_null());
    assert_eq!(unknown["success"], false);
}
#[tokio::test]
async fn prompt_budget_is_a_hard_ceiling() {
    let h = serve(PayPalClient::for_test(String::new(), false)).await;
    let (_, body) = h
        .post(
            "/api/mandate",
            json!({"prompt":"Buy 10 Raspberry Pi 5 under $500","authorized_budget":900}),
        )
        .await;
    assert_eq!(body["transaction"]["authorized_budget"], 500.0);
    assert_eq!(body["transaction"]["status"], "BLOCKED_BY_GUARDRAIL");
    assert!(body["paypal_raw"].is_null());
}
#[tokio::test]
async fn sandbox_flow_uses_approval_and_captures_once_under_concurrency() {
    let mock = provider("success").await;
    let h = serve(PayPalClient::for_test(mock.url.clone(), true)).await;
    let p = h.prepare().await;
    assert_eq!(p["transaction"]["status"], "AWAITING_APPROVAL");
    assert_eq!(mock.captures.load(Ordering::SeqCst), 0);
    let id = p["transaction"]["order_id"].clone();
    let (a, b) = tokio::join!(
        h.post("/api/paypal/capture", json!({"order_id":id})),
        h.post("/api/paypal/capture", json!({"order_id":id}))
    );
    assert_eq!(a.0, StatusCode::OK);
    assert_eq!(b.0, StatusCode::OK);
    assert_eq!(mock.captures.load(Ordering::SeqCst), 1);
    let (a, b) = tokio::join!(
        h.post("/api/paypal/payout", json!({"order_id":id})),
        h.post("/api/paypal/payout", json!({"order_id":id}))
    );
    assert_eq!(a.1["new_status"], "PAYOUT_PENDING");
    assert_eq!(b.1["new_status"], "PAYOUT_PENDING");
    assert_eq!(mock.payouts.load(Ordering::SeqCst), 1);
    assert_eq!(
        h.post("/api/paypal/payout/status", json!({"order_id":id}))
            .await
            .1["new_status"],
        "SETTLED"
    );
    assert_eq!(
        h.post("/api/reset", json!({})).await.0,
        StatusCode::CONFLICT
    );
}
#[tokio::test]
async fn unapproved_order_never_reaches_capture_api() {
    let mock = provider("unapproved").await;
    let h = serve(PayPalClient::for_test(mock.url.clone(), true)).await;
    let p = h.prepare().await;
    assert_eq!(
        h.post(
            "/api/paypal/capture",
            json!({"order_id":p["transaction"]["order_id"]})
        )
        .await
        .0,
        StatusCode::BAD_GATEWAY
    );
    assert_eq!(mock.captures.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn real_provider_errors_do_not_fall_back_to_simulation() {
    for scenario in ["create_error", "capture_error", "amount_mismatch"] {
        let mock = provider(scenario).await;
        let h = serve(PayPalClient::for_test(mock.url.clone(), true)).await;
        let p = h.prepare().await;
        assert_eq!(p["transaction"]["payment_mode"], "sandbox");
        if scenario == "create_error" {
            assert_eq!(p["transaction"]["status"], "PAYMENT_FAILED");
            assert!(p["transaction"]["paypal_order_id"].is_null());
        } else {
            let (code, _) = h
                .post(
                    "/api/paypal/capture",
                    json!({"order_id":p["transaction"]["order_id"]}),
                )
                .await;
            assert_eq!(code, StatusCode::BAD_GATEWAY);
            let ledger: Value =
                serde_json::from_slice(&std::fs::read(h.dir.join("transactions.json")).unwrap())
                    .unwrap();
            assert_ne!(
                ledger.as_array().unwrap().last().unwrap()["status"],
                "COMPLETED"
            );
        }
    }
}
#[tokio::test]
async fn request_retries_do_not_create_duplicate_records() {
    let h = serve(PayPalClient::for_test(String::new(), false)).await;
    let body = json!({"prompt":"Buy 1 Raspberry Pi 5 under $100","authorized_budget":100,"request_id":uuid::Uuid::new_v4().to_string()});
    let (a, b) = tokio::join!(
        h.post("/api/mandate", body.clone()),
        h.post("/api/mandate", body)
    );
    assert_eq!(
        a.1["transaction"]["order_id"],
        b.1["transaction"]["order_id"]
    );
    let ledger: Vec<Transaction> = load_json(&h.dir.join("transactions.json")).unwrap();
    assert_eq!(ledger.len(), 26);
}
#[tokio::test]
async fn unclaimed_payout_remains_pending_even_when_batch_succeeds() {
    let mock = provider("unclaimed").await;
    let h = serve(PayPalClient::for_test(mock.url.clone(), true)).await;
    let p = h.prepare().await;
    let id = p["transaction"]["order_id"].clone();
    h.post("/api/paypal/capture", json!({"order_id":id})).await;
    h.post("/api/paypal/payout", json!({"order_id":id})).await;
    assert_eq!(
        h.post("/api/paypal/payout/status", json!({"order_id":id}))
            .await
            .1["new_status"],
        "PAYOUT_PENDING"
    );
}
#[tokio::test]
async fn unsigned_webhooks_are_rejected() {
    let mock = provider("success").await;
    let h = serve(PayPalClient::for_test(mock.url.clone(), true)).await;
    assert_eq!(
        h.post(
            "/api/paypal/webhook",
            json!({"event_type":"PAYMENT.CAPTURE.COMPLETED"})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
}
#[test]
fn quote_selection_applies_sla_stock_and_exact_vendor() {
    let mut catalog: Vec<CatalogItem> =
        serde_json::from_str(include_str!("../data/catalog.json")).unwrap();
    let intent = agent::parse_mandate_deterministic("Buy 1 NVIDIA Jetson under $600 within 2 days");
    assert_eq!(
        agent::select_quote(&catalog, &intent, 600.0)
            .0
            .unwrap()
            .vendor,
        "Mouser"
    );
    for item in &mut catalog {
        item.in_stock = false;
    }
    assert!(agent::select_quote(&catalog, &intent, 600.0).0.is_none());
    let catalog: Vec<CatalogItem> =
        serde_json::from_str(include_str!("../data/catalog.json")).unwrap();
    let intent =
        agent::parse_mandate_deterministic("Buy 1 Raspberry Pi 5 from ImaginaryVendor under $100");
    assert!(agent::select_quote(&catalog, &intent, 100.0).0.is_none());
}
#[test]
fn guardrails_reject_nonfinite_and_nonpositive_money() {
    for amount in [f64::NAN, f64::INFINITY, -1.0, 0.0] {
        assert!(guardrails::validate("DigiKey", 4.9, amount, 900.0, 2, 3, "test".into()).is_err());
    }
}

#[test]
fn delivery_numbers_are_not_budgets_and_zero_budgets_are_not_ignored() {
    let intent =
        agent::parse_mandate_deterministic("Buy 1 Raspberry Pi 5 with delivery under 3 days");
    assert_eq!(intent.target_budget, None);
    let intent = agent::parse_mandate_deterministic("Buy 1 Raspberry Pi 5 under $0");
    assert_eq!(intent.target_budget, Some(0.0));
    assert!(guardrails::validate("DigiKey", 4.9, 78.5, 100.0, 2, 0, "test".into()).is_err());
}

#[tokio::test]
async fn verified_webhook_reconciles_and_rejects_invalid_signatures() {
    let mock = provider("success").await;
    let h = serve(PayPalClient::for_test(mock.url.clone(), true)).await;
    let p = h.prepare().await;
    mock.captures.store(1, Ordering::SeqCst); // Provider completed payment before callback arrived.
    let event = json!({"id":"TEST-EVENT","event_type":"PAYMENT.CAPTURE.COMPLETED","resource":{"supplementary_data":{"related_ids":{"order_id":"TEST-PAYPAL-ORDER"}}}});
    for signature in ["invalid", "valid", "valid"] {
        let response = h
            .client
            .post(format!("{}/api/paypal/webhook", h.url))
            .header("paypal-transmission-id", "test-transmission")
            .header("paypal-transmission-time", "2026-10-26T00:00:00Z")
            .header("paypal-auth-algo", "SHA256withRSA")
            .header(
                "paypal-cert-url",
                "https://api.sandbox.paypal.com/v1/notifications/certs/TEST",
            )
            .header("paypal-transmission-sig", signature)
            .json(&event)
            .send()
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            if signature == "valid" {
                StatusCode::OK
            } else {
                StatusCode::UNAUTHORIZED
            }
        );
    }
    let ledger: Vec<Transaction> = load_json(&h.dir.join("transactions.json")).unwrap();
    let t = ledger
        .iter()
        .find(|t| Some(t.order_id.as_str()) == p["transaction"]["order_id"].as_str())
        .unwrap();
    assert_eq!(t.status, TransactionStatus::Completed);
    assert_eq!(t.capture_id.as_deref(), Some("TEST-CAPTURE"));
    assert_eq!(mock.captures.load(Ordering::SeqCst), 1); // Callback verification never captures again.
}

#[tokio::test]
async fn different_concurrent_requests_keep_unique_ids_and_both_durable_records() {
    let h = serve(PayPalClient::for_test(String::new(), false)).await;
    let (a, b) = tokio::join!(
        h.post(
            "/api/mandate",
            json!({"prompt":"Buy 1 Raspberry Pi 5 under $100","authorized_budget":100})
        ),
        h.post(
            "/api/mandate",
            json!({"prompt":"Buy 2 Raspberry Pi 5 under $200","authorized_budget":200})
        )
    );
    assert_ne!(
        a.1["transaction"]["order_id"],
        b.1["transaction"]["order_id"]
    );
    let ledger: Vec<Transaction> = load_json(&h.dir.join("transactions.json")).unwrap();
    assert_eq!(ledger.len(), 27);
    assert_eq!(
        ledger
            .iter()
            .map(|t| &t.order_id)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        27
    );
}

//! Sandbox PayPal REST client. Real errors never become simulated successes.
use crate::{
    guardrails::ApprovedMandate,
    models::{CheckoutConfig, Transaction},
};
use axum::http::HeaderMap;
use reqwest::{Client, Method};
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PayPalError {
    pub message: String,
    pub uncertain: bool,
    recovery_url: Option<String>,
}
impl PayPalError {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            uncertain: false,
            recovery_url: None,
        }
    }
}
impl std::fmt::Display for PayPalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
#[derive(Clone)]
pub struct PayPalClient {
    client_id: String,
    client_secret: String,
    base_url: String,
    sandbox: bool,
    recipient: Option<String>,
    webhook_id: Option<String>,
    http: Client,
}
fn configured(value: &str) -> bool {
    !value.trim().is_empty() && !value.starts_with("your_") && !value.starts_with("your-")
}
fn env_value(key: &str) -> String {
    std::env::var(key).unwrap_or_default().trim().to_string()
}
impl PayPalClient {
    pub fn from_env() -> Result<Self, String> {
        let client_id = env_value("PAYPAL_CLIENT_ID");
        let client_secret = env_value("PAYPAL_CLIENT_SECRET");
        let sandbox = match env_value("PAYPAL_MODE").to_lowercase().as_str() {
            "sandbox" => true,
            "simulation" => false,
            "" => {
                configured(&client_id)
                    && configured(&client_secret)
                    && env_value("USE_PAYPAL_SANDBOX") != "false"
            }
            _ => {
                return Err(
                    "PAYPAL_MODE must be sandbox or simulation; live payments are disabled".into(),
                )
            }
        };
        let base = env_value("PAYPAL_BASE_URL");
        let base_url: String = if base.is_empty() {
            "https://api-m.sandbox.paypal.com".into()
        } else {
            base.trim_end_matches('/').into()
        };
        if sandbox && base_url != "https://api-m.sandbox.paypal.com" {
            return Err("Only the official PayPal Sandbox endpoint is supported".into());
        }
        let recipient = env_value("PAYPAL_PAYOUT_RECEIVER_EMAIL");
        let webhook_id = env_value("PAYPAL_WEBHOOK_ID");
        Ok(Self {
            client_id,
            client_secret,
            base_url,
            sandbox,
            recipient: configured(&recipient).then_some(recipient),
            webhook_id: configured(&webhook_id).then_some(webhook_id),
            http: Client::builder()
                .timeout(Duration::from_secs(20))
                .build()
                .map_err(|e| e.to_string())?,
        })
    }
    pub fn is_live_configured(&self) -> bool {
        self.sandbox && configured(&self.client_id) && configured(&self.client_secret)
    }
    pub fn mode(&self) -> &'static str {
        if self.sandbox {
            "sandbox"
        } else {
            "simulation"
        }
    }
    pub fn config(&self) -> CheckoutConfig {
        CheckoutConfig {
            mode: self.mode().into(),
            configured: self.is_live_configured(),
            client_id: self.is_live_configured().then(|| self.client_id.clone()),
            sdk_url: "https://www.sandbox.paypal.com/web-sdk/v6/core".into(),
            payouts_enabled: !self.sandbox || self.recipient.is_some(),
            webhook_configured: self.webhook_id.is_some(),
            message: if !self.sandbox {
                "Simulation: no money moves. Configure Sandbox credentials to test PayPal."
            } else if !self.is_live_configured() {
                "Sandbox selected but client ID or secret is missing. Add them in backend/.env."
            } else {
                "PayPal Sandbox: buyer approval is required before capture. No live money moves."
            }
            .into(),
        }
    }
    async fn access_token(&self) -> Result<String, PayPalError> {
        if !self.is_live_configured() {
            return Err(PayPalError::invalid(
                "PayPal Sandbox credentials are missing",
            ));
        }
        let response = self
            .http
            .post(format!("{}/v1/oauth2/token", self.base_url))
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .map_err(|_| {
                PayPalError::invalid(
                    "Could not authenticate with PayPal Sandbox; check connectivity",
                )
            })?;
        let body = Self::response(response).await?;
        body.get("access_token")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| PayPalError::invalid("PayPal OAuth response has no access token"))
    }
    async fn response(response: reqwest::Response) -> Result<Value, PayPalError> {
        let status = response.status();
        let body: Value = response.json().await.map_err(|_| PayPalError {
            message: "PayPal returned an unreadable response; reconcile before retrying".into(),
            uncertain: true,
            recovery_url: None,
        })?;
        if status.is_success() {
            return Ok(body);
        }
        let issue = body
            .pointer("/details/0/issue")
            .or_else(|| body.get("name"))
            .or_else(|| body.get("error"))
            .and_then(Value::as_str)
            .unwrap_or("PAYPAL_API_ERROR");
        let debug = body
            .get("debug_id")
            .and_then(Value::as_str)
            .unwrap_or("unavailable");
        Err(PayPalError {
            message: format!("PayPal {status}: {issue} (debug ID: {debug})"),
            uncertain: status.is_server_error()
                || issue.contains("BATCH_ALREADY")
                || issue.contains("DUPLICATE_BATCH"),
            recovery_url: payout_recovery_url(&body),
        })
    }
    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        request_id: Option<&str>,
    ) -> Result<Value, PayPalError> {
        let token = self.access_token().await?;
        let mut request = self
            .http
            .request(method, format!("{}{path}", self.base_url))
            .bearer_auth(token)
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation");
        if let Some(id) = request_id {
            request = request.header("PayPal-Request-Id", id);
        }
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.map_err(|_| PayPalError { message: "PayPal request outcome is unknown. Refresh status or retry the same operation; do not create another payment.".into(), uncertain: true, recovery_url: None })?;
        Self::response(response).await
    }
    /// Creation requires a validated mandate. Capture is a separate operation after buyer approval.
    pub async fn execute_order(
        &self,
        mandate: &ApprovedMandate,
        local_id: &str,
    ) -> Result<Value, PayPalError> {
        if !self.sandbox {
            return Ok(
                json!({"id":format!("SIM-{local_id}"),"status":"CREATED","mode":"simulation"}),
            );
        }
        self.request(
            Method::POST,
            "/v2/checkout/orders",
            Some(json!({"intent":"CAPTURE","purchase_units":[{
                "reference_id":local_id,"custom_id":local_id,"description":mandate.description(),
                "amount":{"currency_code":"USD","value":format!("{:.2}",mandate.total_amount())}
            }]})),
            Some(&format!("create-{local_id}")),
        )
        .await
    }
    pub async fn capture_order(&self, transaction: &Transaction) -> Result<Value, PayPalError> {
        let id = transaction
            .paypal_order_id
            .as_deref()
            .ok_or_else(|| PayPalError::invalid("No PayPal order is linked"))?;
        if transaction.payment_mode != self.mode() {
            return Err(PayPalError::invalid(
                "Restore the original payment mode to continue this order",
            ));
        }
        if !self.sandbox {
            return Ok(
                json!({"id":id,"status":"COMPLETED","mode":"simulation","purchase_units":[{
                    "custom_id":transaction.order_id,"payments":{"captures":[{"id":format!("SIM-CAP-{}",transaction.order_id),"status":"COMPLETED",
                    "amount":{"currency_code":transaction.currency,"value":format!("{:.2}",transaction.total_amount)}}]}
                }]}),
            );
        }
        let current = self.get_order(id).await?;
        match current.get("status").and_then(Value::as_str) {
            Some("COMPLETED") => Ok(current),
            Some("APPROVED") => {
                self.request(
                    Method::POST,
                    &format!("/v2/checkout/orders/{id}/capture"),
                    Some(json!({})),
                    Some(&format!("capture-{}", transaction.order_id)),
                )
                .await
            }
            _ => Err(PayPalError::invalid(
                "ORDER_NOT_APPROVED: approve this payment with the PayPal button before capturing",
            )),
        }
    }
    pub async fn get_order(&self, id: &str) -> Result<Value, PayPalError> {
        self.request(
            Method::GET,
            &format!("/v2/checkout/orders/{id}"),
            None,
            None,
        )
        .await
    }
    pub async fn create_payout(&self, transaction: &Transaction) -> Result<Value, PayPalError> {
        let sender_id = transaction
            .payout_sender_batch_id
            .as_deref()
            .ok_or_else(|| PayPalError::invalid("Payout was not reserved"))?;
        if transaction.payment_mode != self.mode() {
            return Err(PayPalError::invalid(
                "Seed records or payments from another mode cannot be paid out",
            ));
        }
        if !self.sandbox {
            return Ok(
                json!({"mode":"simulation","batch_header":{"payout_batch_id":format!("SIM-{sender_id}"),"batch_status":"PENDING"}}),
            );
        }
        let recipient = self.recipient.as_deref().ok_or_else(|| PayPalError::invalid("Set PAYPAL_PAYOUT_RECEIVER_EMAIL to an owned Sandbox recipient before testing payouts"))?;
        let result = self.request(Method::POST, "/v1/payments/payouts", Some(json!({
            "sender_batch_header":{"sender_batch_id":sender_id,"email_subject":"ProcureOps Sandbox vendor settlement"},
            "items":[{"recipient_type":"EMAIL","amount":{"value":format!("{:.2}",transaction.total_amount),"currency":transaction.currency},
                "receiver":recipient,"note":format!("Sandbox settlement for {}",transaction.order_id),"sender_item_id":transaction.order_id}]
        })), Some(sender_id)).await;
        match result {
            Err(error) if error.recovery_url.is_some() => {
                let url = error.recovery_url.as_deref().unwrap();
                let path = url
                    .strip_prefix(&self.base_url)
                    .filter(|p| p.starts_with("/v1/payments/payouts/"));
                match path {
                    Some(path) => self.request(Method::GET, path, None, None).await,
                    None => Err(error),
                }
            }
            other => other,
        }
    }
    pub async fn get_payout(&self, transaction: &Transaction) -> Result<Value, PayPalError> {
        let id = transaction.payout_batch_id.as_deref().ok_or_else(|| {
            PayPalError::invalid("Payout batch is not known yet; retry the reserved payout")
        })?;
        if transaction.payment_mode != self.mode() {
            return Err(PayPalError::invalid(
                "Restore the original payment mode to check this payout",
            ));
        }
        if !self.sandbox {
            return Ok(
                json!({"mode":"simulation","batch_header":{"payout_batch_id":id,"batch_status":"SUCCESS"},"items":[{
                    "transaction_status":"SUCCESS","payout_item":{"sender_item_id":transaction.order_id,"amount":{"currency":transaction.currency,"value":format!("{:.2}",transaction.total_amount)}}
                }]}),
            );
        }
        self.request(
            Method::GET,
            &format!("/v1/payments/payouts/{id}?page_size=1000"),
            None,
            None,
        )
        .await
    }
    pub async fn verify_webhook(
        &self,
        headers: &HeaderMap,
        event: &Value,
    ) -> Result<(), PayPalError> {
        if !self.sandbox {
            return Err(PayPalError::invalid(
                "Webhooks require configured Sandbox mode",
            ));
        }
        let webhook_id = self
            .webhook_id
            .as_deref()
            .ok_or_else(|| PayPalError::invalid("PAYPAL_WEBHOOK_ID is not configured"))?;
        let header = |name: &str| -> Result<String, PayPalError> {
            headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .filter(|v| !v.is_empty())
                .map(str::to_string)
                .ok_or_else(|| PayPalError::invalid(format!("Missing webhook header: {name}")))
        };
        let cert = header("paypal-cert-url")?;
        let parsed = reqwest::Url::parse(&cert)
            .map_err(|_| PayPalError::invalid("Invalid certificate URL"))?;
        if parsed.scheme() != "https"
            || !parsed
                .host_str()
                .is_some_and(|h| h == "paypal.com" || h.ends_with(".paypal.com"))
        {
            return Err(PayPalError::invalid("Invalid certificate host"));
        }
        let response = self.request(Method::POST, "/v1/notifications/verify-webhook-signature", Some(json!({
            "auth_algo":header("paypal-auth-algo")?,"cert_url":cert,"transmission_id":header("paypal-transmission-id")?,
            "transmission_sig":header("paypal-transmission-sig")?,"transmission_time":header("paypal-transmission-time")?,
            "webhook_id":webhook_id,"webhook_event":event
        })), None).await?;
        if response.get("verification_status").and_then(Value::as_str) == Some("SUCCESS") {
            Ok(())
        } else {
            Err(PayPalError::invalid(
                "PayPal webhook signature verification failed",
            ))
        }
    }
    #[cfg(test)]
    pub fn for_test(base_url: String, sandbox: bool) -> Self {
        Self {
            client_id: "test-id".into(),
            client_secret: "test-secret".into(),
            base_url,
            sandbox,
            recipient: Some("owned-recipient@example.test".into()),
            webhook_id: Some("test-webhook".into()),
            http: Client::builder()
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap(),
        }
    }
}
pub fn capture_details(
    raw: &Value,
    transaction: &Transaction,
) -> Result<(String, bool), PayPalError> {
    if raw.get("id").and_then(Value::as_str) != transaction.paypal_order_id.as_deref() {
        return Err(PayPalError::invalid("PayPal order ID does not match"));
    }
    let units = raw
        .get("purchase_units")
        .and_then(Value::as_array)
        .ok_or_else(|| PayPalError::invalid("Missing purchase units"))?;
    if units.len() != 1 {
        return Err(PayPalError::invalid("Unexpected purchase units"));
    }
    let unit = &units[0];
    if unit.get("custom_id").and_then(Value::as_str) != Some(transaction.order_id.as_str()) {
        return Err(PayPalError::invalid(
            "PayPal reference does not match approved mandate",
        ));
    }
    let captures = unit
        .pointer("/payments/captures")
        .and_then(Value::as_array)
        .ok_or_else(|| PayPalError::invalid("No capture confirmed by PayPal"))?;
    if captures.len() != 1 {
        return Err(PayPalError::invalid("Unexpected number of captures"));
    }
    let capture = &captures[0];
    let amount = capture
        .pointer("/amount/value")
        .and_then(Value::as_str)
        .and_then(|v| v.parse::<f64>().ok());
    if amount.is_none_or(|v| !v.is_finite() || (v - transaction.total_amount).abs() > 0.001)
        || capture
            .pointer("/amount/currency_code")
            .and_then(Value::as_str)
            != Some(transaction.currency.as_str())
    {
        return Err(PayPalError::invalid(
            "Capture amount or currency does not match mandate",
        ));
    }
    let id = capture
        .get("id")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| PayPalError::invalid("Missing capture ID"))?;
    match capture.get("status").and_then(Value::as_str) {
        Some("COMPLETED") => Ok((id.into(), true)),
        Some("PENDING") => Ok((id.into(), false)),
        _ => Err(PayPalError::invalid(
            "PayPal did not complete or accept capture",
        )),
    }
}
pub fn payout_batch_id(raw: &Value) -> Option<String> {
    raw.pointer("/batch_header/payout_batch_id")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}
pub fn payout_state(raw: &Value, transaction: &Transaction) -> Result<&'static str, PayPalError> {
    if matches!(
        raw.pointer("/batch_header/batch_status")
            .and_then(Value::as_str),
        Some("DENIED" | "CANCELED")
    ) {
        return Ok("failed");
    }
    let item = raw
        .get("items")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find(|i| {
                i.pointer("/payout_item/sender_item_id")
                    .and_then(Value::as_str)
                    == Some(transaction.order_id.as_str())
            })
        });
    let Some(item) = item else {
        return Ok("pending");
    };
    let amount = item
        .pointer("/payout_item/amount/value")
        .and_then(Value::as_str)
        .and_then(|v| v.parse::<f64>().ok());
    if amount.is_none_or(|v| !v.is_finite() || (v - transaction.total_amount).abs() > 0.001)
        || item
            .pointer("/payout_item/amount/currency")
            .and_then(Value::as_str)
            != Some(transaction.currency.as_str())
    {
        return Err(PayPalError::invalid(
            "Payout amount or currency does not match ledger",
        ));
    }
    Ok(
        match item.get("transaction_status").and_then(Value::as_str) {
            Some("SUCCESS") => "settled",
            Some("FAILED" | "DENIED" | "RETURNED" | "REFUNDED" | "CANCELED") => "failed",
            _ => "pending",
        },
    )
}

// Duplicate sender-batch responses link to the existing payout. Follow only a
// path on the configured provider, never an arbitrary response URL.
fn payout_recovery_url(value: &Value) -> Option<String> {
    match value {
        Value::Object(object) => {
            if let Some(href) = object.get("href").and_then(Value::as_str) {
                if href.contains("/v1/payments/payouts/") {
                    return Some(href.to_string());
                }
            }
            object.values().find_map(payout_recovery_url)
        }
        Value::Array(array) => array.iter().find_map(payout_recovery_url),
        _ => None,
    }
}

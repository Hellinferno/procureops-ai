//! Autonomous procurement agent: intent parsing, quote selection, guardrail
//! validation and PayPal execution, returned as a four-stage trace.

use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use tracing::warn;

use crate::guardrails::{self, BLOCKED_RISK_SCORE, DEFAULT_MAX_DELIVERY_DAYS};
use crate::models::{
    CatalogItem, MandateResponse, MandateStep, StepStatus, Transaction, TransactionStatus,
};
use crate::paypal::PayPalClient;

static QUANTITY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(\d+)\s*(?:x\b|units?\b|pcs?\b|pieces?\b|packs?\b)")
        .expect("valid quantity regex")
});
static COMMAND_QTY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:buy|procure|purchase|order)\s+(\d+)\b").expect("valid command regex")
});
static SLA_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:under|within|in|less than)\s*(\d+)\s*(?:business\s+)?days?")
        .expect("valid SLA regex")
});

#[derive(Debug, Clone)]
pub struct MandateIntent {
    pub item_query: String,
    pub quantity: u32,
    pub max_delivery_days: Option<u32>,
    pub preferred_vendor: Option<String>,
    pub target_budget: Option<f64>,
    pub parser_mode: String,
}

#[derive(Debug, Deserialize)]
struct LlmIntent {
    item_query: Option<String>,
    quantity: Option<u32>,
    max_delivery_days: Option<u32>,
    preferred_vendor: Option<String>,
}

pub fn llm_configured() -> bool {
    crate::llm::configured()
}

pub fn parse_mandate_deterministic(prompt: &str) -> MandateIntent {
    let text = prompt.to_lowercase();

    let quantity = QUANTITY_RE
        .captures(&text)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .or_else(|| {
            COMMAND_QTY_RE
                .captures(&text)
                .and_then(|caps| caps.get(1))
                .and_then(|m| m.as_str().parse::<u32>().ok())
        })
        .unwrap_or(1);
    // A zero quantity is rejected instead of silently becoming a purchase.

    let max_delivery_days = SLA_RE
        .captures(&text)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok());

    let preferred_vendor = [
        "shadowpartsshenzhen",
        "greymarkettech",
        "digikey",
        "mouser",
        "adafruit",
        "sparkfun",
        "aws marketplace",
    ]
    .iter()
    .find(|vendor| text.contains(**vendor))
    .map(|vendor| (*vendor).to_string())
    .or_else(|| {
        Regex::new(r"(?i)\bfrom\s+([a-z][a-z0-9 -]*?)(?:\s+(?:under|within|with|for|below)\b|$)")
            .unwrap()
            .captures(&text)
            .map(|c| c[1].trim().to_string())
    });

    let item_query = if text.contains("raspberry") || text.contains("pi 5") {
        "raspberry pi 5"
    } else if text.contains("jetson") || text.contains("orin") || text.contains("nvidia") {
        "nvidia jetson"
    } else if text.contains("esp32") || text.contains("wroom") {
        "esp32"
    } else if text.contains("gpu")
        || text.contains("graviton")
        || text.contains("aws")
        || text.contains("ec2")
    {
        "aws"
    } else if text.contains("bedrock") || text.contains("sagemaker") {
        "sagemaker"
    } else if text.contains("5g") || text.contains("quectel") || text.contains("modem") {
        "5g"
    } else if text.contains("nordic") || text.contains("nrf") || text.contains("ble") {
        "nordic"
    } else if text.contains("coral") || text.contains("tpu") {
        "coral"
    } else if text.contains("portenta") || text.contains("arduino") {
        "portenta"
    } else if text.contains("ddr5") || text.contains("memory") || text.contains("ram") {
        "ddr5"
    } else {
        text.as_str()
    };

    MandateIntent {
        item_query: item_query.to_string(),
        quantity,
        max_delivery_days,
        preferred_vendor,
        target_budget: prompt_budget(prompt),
        parser_mode: "deterministic".into(),
    }
}

async fn parse_mandate_with_llm(prompt: &str) -> Option<MandateIntent> {
    let system = "You are a structured procurement intent extractor. Return ONLY JSON with keys: \
        \"item_query\" (short product keyword string), \"quantity\" (integer >= 1), \
        \"max_delivery_days\" (integer or null), \"preferred_vendor\" (string or null). \
        Do not invent a vendor that is not named in the request.";

    let output = crate::llm::structured_json(system, prompt).await?;
    let intent: LlmIntent = match serde_json::from_value(output.value) {
        Ok(parsed) => parsed,
        Err(_) => {
            warn!("Model mandate output did not match the intent schema");
            return None;
        }
    };

    let item_query = intent
        .item_query
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "hardware".to_string());

    Some(MandateIntent {
        item_query,
        quantity: intent.quantity.unwrap_or(1),
        max_delivery_days: intent.max_delivery_days,
        preferred_vendor: intent
            .preferred_vendor
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty()),
        target_budget: prompt_budget(prompt),
        parser_mode: output.parser_mode,
    })
}

pub async fn parse_mandate(prompt: &str) -> MandateIntent {
    if llm_configured() {
        if let Some(intent) = parse_mandate_with_llm(prompt).await {
            return intent;
        }
        warn!("Falling back to deterministic mandate parser");
    }
    let mut intent = parse_mandate_deterministic(prompt);
    if llm_configured() {
        intent.parser_mode = "deterministic_fallback".into();
    }
    intent
}

fn prompt_budget(prompt: &str) -> Option<f64> {
    Regex::new(r"(?i)(?:under|below|budget(?:\s+of)?|up to|at most|less than)\s*\$?\s*([-+]?[0-9][0-9,]*(?:\.[0-9]+)?)")
        .unwrap().captures_iter(prompt)
        .filter(|c| {
            let suffix = prompt[c.get(0).unwrap().end()..].trim_start().to_lowercase();
            !suffix.starts_with("day") && !suffix.starts_with("business day")
        })
        .filter_map(|c| c[1].replace(',', "").parse::<f64>().ok())
        .filter(|v| v.is_finite()).min_by(f64::total_cmp)
}

fn matches_item(item: &CatalogItem, item_query: &str) -> bool {
    let query = item_query.to_lowercase();
    let keywords: Vec<&str> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    if keywords.is_empty() {
        return false;
    }
    // Model keywords need not form a contiguous catalog phrase. Keep every
    // token, including model numbers and capacities, as an exact requirement.
    [&item.item_name, &item.category, &item.description]
        .iter()
        .any(|field| {
            let normalized = field.to_lowercase();
            let words: Vec<&str> = normalized
                .split(|c: char| !c.is_alphanumeric())
                .filter(|word| !word.is_empty())
                .collect();
            keywords.iter().all(|keyword| words.contains(keyword))
        })
}

/// Returns the selected quote and how many candidates were evaluated.
pub fn select_quote(
    catalog: &[CatalogItem],
    intent: &MandateIntent,
    budget: f64,
) -> (Option<CatalogItem>, usize) {
    let item_query = intent.item_query.to_lowercase();
    let preferred = intent.preferred_vendor.as_deref();

    let candidates: Vec<CatalogItem> = catalog
        .iter()
        .filter(|item| {
            matches_item(item, &item_query)
                && item.in_stock
                && intent.quantity >= item.min_quantity
                && intent.quantity <= item.max_quantity
                && preferred
                    .map(|vendor| item.vendor.to_lowercase() == vendor)
                    .unwrap_or(true)
        })
        .cloned()
        .collect();

    if candidates.is_empty() {
        return (None, 0);
    }

    let candidate_count = candidates.len();
    let by_price = |a: &CatalogItem, b: &CatalogItem| {
        a.unit_price
            .total_cmp(&b.unit_price)
            .then(a.delivery_days.cmp(&b.delivery_days))
    };

    let mut approved: Vec<CatalogItem> = candidates
        .iter()
        .filter(|item| {
            guardrails::validate(
                &item.vendor,
                item.vendor_rating,
                round2(item.unit_price * intent.quantity as f64),
                budget,
                item.delivery_days,
                intent
                    .max_delivery_days
                    .unwrap_or(DEFAULT_MAX_DELIVERY_DAYS),
                item.item_name.clone(),
            )
            .is_ok()
        })
        .cloned()
        .collect();
    approved.sort_by(by_price);

    if let Some(quote) = approved.first().cloned() {
        return (Some(quote), candidate_count);
    }

    let mut all = candidates;
    all.sort_by(by_price);
    (all.first().cloned(), candidate_count)
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Prepare a purchase; a separate buyer-approved capture completes payment.
pub async fn execute_mandate(
    prompt: &str,
    authorized_budget: f64,
    catalog: &[CatalogItem],
    paypal: &PayPalClient,
    order_id: String,
) -> MandateResponse {
    let intent = parse_mandate(prompt).await;
    let budget = intent
        .target_budget
        .map(|b| b.min(authorized_budget))
        .unwrap_or(authorized_budget);
    let quantity = intent.quantity;
    let sla = intent
        .max_delivery_days
        .unwrap_or(DEFAULT_MAX_DELIVERY_DAYS);
    let mut steps = vec![MandateStep {
        step: 1,
        title: "Mandate extracted".into(),
        status: StepStatus::Done,
        detail: format!(
            "{} | Qty {quantity} | Effective ceiling ${budget:.2} | SLA {sla} days | {}",
            intent.item_query, intent.parser_mode
        ),
    }];
    let (quote, evaluated) = select_quote(catalog, &intent, budget);
    let Some(quote) = quote.filter(|_| quantity > 0) else {
        steps.push(MandateStep { step:2,title:"Supplier catalog".into(),status:StepStatus::Failed,
            detail:"No matching in-stock product satisfies the requested vendor and quantity. Nothing was sent to PayPal.".into() });
        return MandateResponse {
            success: false,
            transaction: None,
            steps,
            paypal_raw: None,
        };
    };
    let amount = round2(quote.unit_price * quantity as f64);
    steps.push(MandateStep {
        step: 2,
        title: "Supplier comparison".into(),
        status: StepStatus::Done,
        detail: format!(
            "Compared {evaluated} available quotes. {}: ${:.2}, {}-day delivery, rating {:.1}.",
            quote.vendor, amount, quote.delivery_days, quote.vendor_rating
        ),
    });
    let validation = guardrails::validate(
        &quote.vendor,
        quote.vendor_rating,
        amount,
        budget,
        quote.delivery_days,
        sla,
        format!(
            "ProcureOps: {quantity} x {} from {}",
            quote.item_name, quote.vendor
        ),
    );
    let mut raw = None;
    let mut approval_url = None;
    let mut payment_error = None;
    let (status, risk_score, guardrail_passed, reason, paypal_order_id) = match validation {
        Err(reason) => {
            steps.push(MandateStep {
                step: 3,
                title: "Policy validation".into(),
                status: StepStatus::Blocked,
                detail: reason.clone(),
            });
            steps.push(MandateStep {
                step: 4,
                title: "PayPal checkout".into(),
                status: StepStatus::Blocked,
                detail: "No PayPal request made.".into(),
            });
            (
                TransactionStatus::BlockedByGuardrail,
                BLOCKED_RISK_SCORE,
                false,
                reason,
                None,
            )
        }
        Ok(approved) => {
            steps.push(MandateStep {step:3,title:"Policy validation".into(),status:StepStatus::Done,
                detail:format!("Budget, $2,000 hard cap, vendor {}, rating and {}-day SLA passed. ApprovedMandate created.",approved.vendor(),approved.delivery_days())});
            let result = paypal.execute_order(&approved, &order_id).await;
            match result {
                Ok(body) if body.get("id").and_then(Value::as_str).is_some() => {
                    let id = body["id"].as_str().unwrap().to_string();
                    approval_url = body
                        .get("links")
                        .and_then(Value::as_array)
                        .and_then(|links| {
                            links.iter().find(|l| {
                                matches!(l["rel"].as_str(), Some("approve" | "payer-action"))
                            })
                        })
                        .and_then(|l| l["href"].as_str())
                        .map(str::to_string);
                    raw = Some(body);
                    steps.push(MandateStep {step:4,title:"Buyer approval required".into(),status:StepStatus::Pending,
                        detail:format!("{} order {id} created. Use the checkout button to approve; no payment has been captured.",paypal.mode())});
                    (
                        TransactionStatus::AwaitingApproval,
                        approved.risk_score(),
                        true,
                        "All purchasing policies passed".into(),
                        Some(id),
                    )
                }
                other => {
                    let message = match other {
                        Err(e) => e.to_string(),
                        _ => "PayPal returned no order ID".into(),
                    };
                    payment_error = Some(message.clone());
                    steps.push(MandateStep {
                        step: 4,
                        title: "PayPal order creation failed".into(),
                        status: StepStatus::Failed,
                        detail: message,
                    });
                    (
                        TransactionStatus::PaymentFailed,
                        approved.risk_score(),
                        true,
                        "Policy passed; PayPal did not confirm an order".into(),
                        None,
                    )
                }
            }
        }
    };
    let success = status == TransactionStatus::AwaitingApproval;
    let transaction = Transaction {
        order_id,
        paypal_order_id,
        timestamp: chrono::Utc::now().to_rfc3339(),
        vendor: quote.vendor,
        vendor_email: quote.vendor_email,
        category: quote.category,
        item_name: quote.item_name,
        quantity,
        unit_price: quote.unit_price,
        total_amount: amount,
        authorized_budget: budget,
        currency: "USD".into(),
        delivery_days: quote.delivery_days,
        vendor_rating: quote.vendor_rating,
        status,
        risk_score,
        guardrail_passed,
        guardrail_reason: reason,
        agent_reasoning: format!(
            "Selected from {evaluated} available quotes; validated before order creation."
        ),
        payout_batch_id: None,
        payment_mode: paypal.mode().into(),
        parser_mode: intent.parser_mode,
        capture_id: None,
        approval_url,
        payment_error,
        payout_error: None,
        payout_sender_batch_id: None,
        request_id: None,
    };
    MandateResponse {
        success,
        transaction: Some(transaction),
        steps,
        paypal_raw: raw,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> Vec<CatalogItem> {
        serde_json::from_str(include_str!("../data/catalog.json")).expect("catalog fixture")
    }

    #[test]
    fn parses_compliant_preset() {
        let intent = parse_mandate_deterministic(
            "Buy 10 units of Raspberry Pi 5 (8GB) under $900 within 3 days",
        );
        assert_eq!(intent.quantity, 10);
        assert_eq!(intent.item_query, "raspberry pi 5");
        assert_eq!(intent.max_delivery_days, Some(3));
        assert!(intent.preferred_vendor.is_none());
    }

    #[test]
    fn parses_vendor_and_quantity() {
        let intent = parse_mandate_deterministic(
            "Buy 20x ESP32 Bulk Pack from ShadowPartsShenzhen under $400",
        );
        assert_eq!(intent.quantity, 20);
        assert_eq!(intent.item_query, "esp32");
        assert_eq!(
            intent.preferred_vendor.as_deref(),
            Some("shadowpartsshenzhen")
        );
    }

    #[test]
    fn selects_cheapest_approved_quote() {
        let intent = parse_mandate_deterministic(
            "Buy 10 units of Raspberry Pi 5 (8GB) under $900 within 3 days",
        );
        let (quote, evaluated) = select_quote(&catalog(), &intent, 900.0);
        let quote = quote.expect("a quote must be selected");
        assert_eq!(quote.vendor, "DigiKey");
        assert_eq!(quote.unit_price, 78.50);
        assert!(evaluated >= 3);
    }

    #[test]
    fn model_keywords_match_long_catalog_names_without_changing_product_specs() {
        let mut intent = parse_mandate_deterministic(
            "Buy 1 unit of Raspberry Pi 5 from DigiKey under $100 within 3 days",
        );
        intent.item_query = "raspberry pi 5 board".into();
        let quote = select_quote(&catalog(), &intent, 100.0).0.unwrap();
        assert_eq!(quote.vendor, "DigiKey");
        assert_eq!(quote.unit_price, 78.50);
        assert!(matches_item(&quote, "Raspberry Pi 5 (8GB)"));
        for wrong in ["raspberry pi 4 board", "raspberry pi 5 4gb", "raspberry pi 50", "!!!"] {
            assert!(!matches_item(&quote, wrong), "must reject {wrong}");
        }
    }

    #[test]
    fn honors_untrusted_vendor_request() {
        let intent = parse_mandate_deterministic(
            "Buy 20x ESP32 Bulk Pack from ShadowPartsShenzhen under $400",
        );
        let (quote, _) = select_quote(&catalog(), &intent, 900.0);
        let quote = quote.expect("a quote must be selected");
        assert_eq!(quote.vendor, "ShadowPartsShenzhen");
        assert_eq!(quote.item_name.contains("ESP32"), true);
    }
}

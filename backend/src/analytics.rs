//! Natural-language treasury analytics translated into typed Rust predicates.
//!
//! Queries are never evaluated as code. They map to a closed [`Predicate`] AST
//! that can only compare known transaction fields.

use regex::Regex;
use serde_json::Value;

use crate::models::{AnalyticsQueryResponse, KpiSummary, Transaction, TransactionStatus};

#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    All,
    Status(TransactionStatus),
    AmountAbove(f64),
    AmountBelow(f64),
    Vendor(String),
    HighRisk(f64),
    LowRisk(f64),
    Contains(String),
    And(Vec<Predicate>),
    Or(Vec<Predicate>),
}

impl Predicate {
    pub fn matches(&self, transaction: &Transaction) -> bool {
        match self {
            Predicate::All => true,
            Predicate::Status(status) => transaction.status == *status,
            Predicate::AmountAbove(threshold) => transaction.total_amount > *threshold,
            Predicate::AmountBelow(threshold) => transaction.total_amount < *threshold,
            Predicate::Vendor(vendor) => transaction
                .vendor
                .to_lowercase()
                .contains(&vendor.to_lowercase()),
            Predicate::HighRisk(threshold) => transaction.risk_score >= *threshold,
            Predicate::LowRisk(threshold) => transaction.risk_score < *threshold,
            Predicate::Contains(needle) => {
                let needle = needle.to_lowercase();
                [
                    &transaction.order_id,
                    &transaction.vendor,
                    &transaction.item_name,
                    &transaction.category,
                    &transaction.status.to_string(),
                ]
                .iter()
                .any(|field| field.to_lowercase().contains(&needle))
            }
            Predicate::And(predicates) => predicates.iter().all(|p| p.matches(transaction)),
            Predicate::Or(predicates) => predicates.iter().any(|p| p.matches(transaction)),
        }
    }

    pub fn expr(&self) -> String {
        match self {
            Predicate::All => "true".to_string(),
            Predicate::Status(status) => format!("status == TransactionStatus::{:?}", status),
            Predicate::AmountAbove(threshold) => format!("total_amount > {threshold:.2}"),
            Predicate::AmountBelow(threshold) => format!("total_amount < {threshold:.2}"),
            Predicate::Vendor(vendor) => format!("vendor == \"{vendor}\""),
            Predicate::HighRisk(threshold) => format!("risk_score >= {threshold:.2}"),
            Predicate::LowRisk(threshold) => format!("risk_score < {threshold:.2}"),
            Predicate::Contains(needle) => format!("search_all(\"{needle}\")"),
            Predicate::And(predicates) => format!(
                "({})",
                predicates
                    .iter()
                    .map(Predicate::expr)
                    .collect::<Vec<_>>()
                    .join(" && ")
            ),
            Predicate::Or(predicates) => format!(
                "({})",
                predicates
                    .iter()
                    .map(Predicate::expr)
                    .collect::<Vec<_>>()
                    .join(" || ")
            ),
        }
    }
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub fn compute_kpis(transactions: &[Transaction]) -> KpiSummary {
    if transactions.is_empty() {
        return KpiSummary {
            total_spend: 0.0,
            completed_count: 0,
            blocked_count: 0,
            money_saved_by_guardrails: 0.0,
            avg_risk_score: 0.0,
        };
    }

    let total_spend: f64 = transactions
        .iter()
        .filter(|t| {
            matches!(
                t.status,
                TransactionStatus::Completed
                    | TransactionStatus::Settled
                    | TransactionStatus::PayoutPending
                    | TransactionStatus::PayoutFailed
            )
        })
        .map(|t| t.total_amount)
        .sum();

    let completed_count = transactions
        .iter()
        .filter(|t| {
            matches!(
                t.status,
                TransactionStatus::Completed
                    | TransactionStatus::Settled
                    | TransactionStatus::ApprovedPendingSettlement
                    | TransactionStatus::PayoutPending
                    | TransactionStatus::PayoutFailed
            )
        })
        .count();

    let blocked: Vec<&Transaction> = transactions
        .iter()
        .filter(|t| t.status == TransactionStatus::BlockedByGuardrail)
        .collect();

    let avg_risk: f64 =
        transactions.iter().map(|t| t.risk_score).sum::<f64>() / transactions.len() as f64;

    KpiSummary {
        total_spend: round2(total_spend),
        completed_count,
        blocked_count: blocked.len(),
        money_saved_by_guardrails: round2(blocked.iter().map(|t| t.total_amount).sum()),
        avg_risk_score: round2(avg_risk),
    }
}

pub fn translate_query(query: &str) -> Predicate {
    let q = query.trim().to_lowercase();

    if q.is_empty()
        || q == "all"
        || q.contains("show all transactions")
        || q.contains("all transactions")
        || q.contains("show all orders")
        || q.contains("everything")
    {
        return Predicate::All;
    }

    let blocked = q.contains("blocked")
        || q.contains("guardrail")
        || q.contains("rejected")
        || q.contains("halted");

    let pending = q.contains("pending") || q.contains("payout") || q.contains("settlement");
    let settled = q.contains("settled");
    let completed = q.contains("completed");

    let amount_above =
        Regex::new(r"(?:over|above|greater than|more than|>)\s*\$?\s*(\d+(?:\.\d+)?)")
            .ok()
            .and_then(|re| re.captures(&q))
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse::<f64>().ok());
    let amount_below = Regex::new(r"(?:under|below|less than|<)\s*\$?\s*(\d+(?:\.\d+)?)")
        .ok()
        .and_then(|re| re.captures(&q))
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok());

    let high_risk = q.contains("high risk") || q.contains("risky");
    let low_risk = q.contains("low risk");

    let mut vendors: Vec<Predicate> = Vec::new();
    for (needle, label) in [
        ("digikey", "DigiKey"),
        ("mouser", "Mouser"),
        ("adafruit", "Adafruit"),
        ("sparkfun", "SparkFun"),
        ("aws marketplace", "AWS Marketplace"),
        ("shadowpartsshenzhen", "ShadowPartsShenzhen"),
        ("greymarkettech", "GreyMarketTech"),
    ] {
        if q.contains(needle) {
            vendors.push(Predicate::Vendor(label.to_string()));
        }
    }

    // "blocked orders or orders over $500" is an OR, not an AND.
    if blocked && (amount_above.is_some() || amount_below.is_some()) && q.contains(" or ") {
        let mut clauses = vec![Predicate::Status(TransactionStatus::BlockedByGuardrail)];
        if let Some(threshold) = amount_above {
            clauses.push(Predicate::AmountAbove(threshold));
        }
        if let Some(threshold) = amount_below {
            clauses.push(Predicate::AmountBelow(threshold));
        }
        return Predicate::Or(clauses);
    }

    let mut conditions: Vec<Predicate> = Vec::new();

    if blocked {
        conditions.push(Predicate::Status(TransactionStatus::BlockedByGuardrail));
    } else if pending {
        conditions.push(Predicate::Status(
            TransactionStatus::ApprovedPendingSettlement,
        ));
    } else if settled {
        conditions.push(Predicate::Status(TransactionStatus::Settled));
    } else if completed {
        conditions.push(Predicate::Status(TransactionStatus::Completed));
    }

    if let Some(threshold) = amount_above {
        conditions.push(Predicate::AmountAbove(threshold));
    }
    if let Some(threshold) = amount_below {
        conditions.push(Predicate::AmountBelow(threshold));
    }

    if vendors.len() == 1 {
        conditions.push(vendors.remove(0));
    } else if vendors.len() > 1 {
        conditions.push(Predicate::Or(vendors));
    }

    if high_risk {
        conditions.push(Predicate::HighRisk(0.5));
    }
    if low_risk {
        conditions.push(Predicate::LowRisk(0.25));
    }

    match conditions.len() {
        0 => Predicate::Contains(q),
        1 => conditions.remove(0),
        _ => Predicate::And(conditions),
    }
}

fn predicate_from_json(value: &Value, depth: u8) -> Option<Predicate> {
    if depth > 3 {
        return None;
    }

    let kind = value.get("kind")?.as_str()?;
    let number = value.get("value").and_then(Value::as_f64);

    match kind {
        "all" => Some(Predicate::All),
        "blocked" => Some(Predicate::Status(TransactionStatus::BlockedByGuardrail)),
        "status" => {
            let status = value.get("value")?.as_str()?.to_uppercase();
            serde_json::from_value::<TransactionStatus>(Value::String(status))
                .ok()
                .map(Predicate::Status)
        }
        "amount_above" => number.map(Predicate::AmountAbove),
        "amount_below" => number.map(Predicate::AmountBelow),
        "vendor" => value
            .get("value")?
            .as_str()
            .map(|vendor| Predicate::Vendor(vendor.to_string())),
        "high_risk" => Some(Predicate::HighRisk(number.unwrap_or(0.5))),
        "low_risk" => Some(Predicate::LowRisk(number.unwrap_or(0.25))),
        "contains" => value
            .get("value")?
            .as_str()
            .map(|needle| Predicate::Contains(needle.to_string())),
        "and" | "or" => {
            let items = value.get("items")?.as_array()?;
            let predicates: Vec<Predicate> = items
                .iter()
                .map(|item| predicate_from_json(item, depth + 1))
                .collect::<Option<Vec<_>>>()?;
            if predicates.is_empty() {
                None
            } else if kind == "and" {
                Some(Predicate::And(predicates))
            } else {
                Some(Predicate::Or(predicates))
            }
        }
        _ => None,
    }
}

async fn translate_query_with_llm(query: &str) -> Option<(Predicate, String)> {
    let system = "Translate the treasury question into a JSON predicate tree. Return ONLY JSON. \
        Allowed kinds: {\"kind\":\"all\"}, {\"kind\":\"blocked\"}, \
        {\"kind\":\"status\",\"value\":\"COMPLETED|SETTLED|BLOCKED_BY_GUARDRAIL|APPROVED_PENDING_SETTLEMENT|AWAITING_APPROVAL|PAYMENT_PENDING|PAYMENT_FAILED|CANCELLED|PAYOUT_PENDING|PAYOUT_FAILED\"}, \
        {\"kind\":\"amount_above\",\"value\":NUMBER}, {\"kind\":\"amount_below\",\"value\":NUMBER}, \
        {\"kind\":\"vendor\",\"value\":\"DigiKey\"}, {\"kind\":\"high_risk\",\"value\":0.5}, \
        {\"kind\":\"low_risk\",\"value\":0.25}, {\"kind\":\"contains\",\"value\":\"text\"}, \
        or composite {\"kind\":\"and|or\",\"items\":[...]}. \
        Columns: vendor, category, item_name, total_amount, risk_score, status. Use \"or\" for \"or\" wording.";

    let output = crate::llm::structured_json(system, query).await?;
    let predicate = predicate_from_json(&output.value, 0)?;
    Some((predicate, output.parser_mode))
}

fn summarize(records: &[Transaction]) -> String {
    if records.is_empty() {
        return "No transactions matched the query.".to_string();
    }

    let total: f64 = records.iter().map(|t| t.total_amount).sum();
    let blocked = records
        .iter()
        .filter(|t| t.status == TransactionStatus::BlockedByGuardrail)
        .count();
    let mut vendors: Vec<String> = records.iter().map(|t| t.vendor.clone()).collect();
    vendors.sort();
    vendors.dedup();

    if blocked > 0 {
        format!(
            "Found {} matching record(s) totaling ${:.2} (including {} guardrail-blocked transaction{}).",
            records.len(),
            total,
            blocked,
            if blocked == 1 { "" } else { "s" }
        )
    } else {
        format!(
            "Found {} matching record(s) totaling ${:.2} across {} vendor(s).",
            records.len(),
            total,
            vendors.len()
        )
    }
}

pub async fn execute_query(query: &str, transactions: &[Transaction]) -> AnalyticsQueryResponse {
    let (predicate, parser_mode) = match translate_query_with_llm(query).await {
        Some(result) => result,
        None => (
            translate_query(query),
            if crate::agent::llm_configured() {
                "deterministic_fallback"
            } else {
                "deterministic"
            }
            .to_string(),
        ),
    };

    let records: Vec<Transaction> = transactions
        .iter()
        .filter(|transaction| predicate.matches(transaction))
        .cloned()
        .collect();

    AnalyticsQueryResponse {
        query: query.to_string(),
        filter_expr: predicate.expr(),
        summary: summarize(&records),
        records,
        parser_mode: parser_mode.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ledger() -> Vec<Transaction> {
        serde_json::from_str(include_str!("../data/seed_transactions.json"))
            .expect("ledger fixture")
    }

    #[test]
    fn kpis_match_seed_ledger() {
        let kpis = compute_kpis(&ledger());
        assert_eq!(kpis.blocked_count, 4);
        assert_eq!(kpis.completed_count, 21);
        assert!(kpis.total_spend > 0.0);
        assert!(kpis.money_saved_by_guardrails > 0.0);
        assert!((0.0..=1.0).contains(&kpis.avg_risk_score));
    }

    #[test]
    fn blocked_or_high_value_is_an_or() {
        let predicate = translate_query("Show all blocked orders or orders over $500");
        assert!(matches!(predicate, Predicate::Or(_)));
        assert_eq!(
            predicate.expr(),
            "(status == TransactionStatus::BlockedByGuardrail || total_amount > 500.00)"
        );
    }

    #[test]
    fn vendor_or_maps_to_vendor_predicates() {
        let predicate = translate_query("Orders from DigiKey or Mouser");
        assert!(matches!(predicate, Predicate::Or(_)));
        let txs = ledger();
        let matches: Vec<_> = txs.iter().filter(|t| predicate.matches(t)).collect();
        assert!(matches
            .iter()
            .all(|t| t.vendor == "DigiKey" || t.vendor == "Mouser"));
    }

    #[test]
    fn pending_payout_filter_matches_seed_rows() {
        let predicate = translate_query("Show orders pending payout settlement");
        assert_eq!(
            predicate,
            Predicate::Status(TransactionStatus::ApprovedPendingSettlement)
        );
        assert_eq!(ledger().iter().filter(|t| predicate.matches(t)).count(), 4);
    }

    #[test]
    fn unknown_query_falls_back_to_text_search() {
        let predicate = translate_query("ESP32");
        assert!(matches!(predicate, Predicate::Contains(_)));
        assert!(ledger().iter().any(|t| predicate.matches(t)));
    }

    #[test]
    fn model_filter_supports_pending_captures_and_rejects_partial_trees() {
        let pending = serde_json::json!({"kind":"status","value":"PAYMENT_PENDING"});
        assert_eq!(
            predicate_from_json(&pending, 0),
            Some(Predicate::Status(TransactionStatus::PaymentPending))
        );
        let invalid = serde_json::json!({"kind":"and","items":[pending,{"kind":"execute","value":"unknown"}]});
        assert!(predicate_from_json(&invalid, 0).is_none());
    }
}

//! Compile-time typestate financial guardrails.
//!
//! `ApprovedMandate` has private fields and can only be constructed by
//! [`validate`] inside this module. The PayPal client requires a reference to
//! it, so no other module can execute a payment without passing every policy
//! check first.

pub const MAX_TRANSACTION_AMOUNT: f64 = 2_000.00;
pub const MIN_VENDOR_RATING: f64 = 4.5;
pub const DEFAULT_MAX_DELIVERY_DAYS: u32 = 14;
pub const BLOCKED_RISK_SCORE: f64 = 0.92;

pub const APPROVED_VENDORS: [&str; 5] = [
    "digikey",
    "mouser",
    "adafruit",
    "sparkfun",
    "aws marketplace",
];

/// An unforgeable token proving a purchase passed every financial policy check.
///
/// Fields are private to this module: outside code can only obtain one through
/// [`validate`], never by struct literal.
#[derive(Debug)]
pub struct ApprovedMandate {
    vendor: String,
    total_amount: f64,
    description: String,
    delivery_days: u32,
    risk_score: f64,
}

impl ApprovedMandate {
    pub fn vendor(&self) -> &str {
        &self.vendor
    }

    pub fn total_amount(&self) -> f64 {
        self.total_amount
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn delivery_days(&self) -> u32 {
        self.delivery_days
    }

    pub fn risk_score(&self) -> f64 {
        self.risk_score
    }
}

pub fn is_approved_vendor(vendor: &str) -> bool {
    APPROVED_VENDORS.contains(&vendor.trim().to_lowercase().as_str())
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn approved_risk_score(total_amount: f64, authorized_budget: f64) -> f64 {
    let utilization = if authorized_budget > 0.0 {
        total_amount / authorized_budget
    } else {
        1.0
    };
    round2((0.05 + 0.10 * (utilization - 0.70)).clamp(0.04, 0.20))
}

/// Enforce all five deterministic policies and, only on success, mint an
/// `ApprovedMandate`.
#[allow(clippy::too_many_arguments)]
pub fn validate(
    vendor: &str,
    vendor_rating: f64,
    total_amount: f64,
    authorized_budget: f64,
    delivery_days: u32,
    max_delivery_days: u32,
    description: String,
) -> Result<ApprovedMandate, String> {
    let mut violations: Vec<String> = Vec::new();

    if !total_amount.is_finite()
        || total_amount <= 0.0
        || !authorized_budget.is_finite()
        || authorized_budget <= 0.0
        || !vendor_rating.is_finite()
    {
        return Err(
            "VIOLATION: Amount, budget, and vendor rating must be finite; amounts must be positive"
                .into(),
        );
    }

    if total_amount > authorized_budget {
        violations.push(format!(
            "Budget violation: Total ${:.2} exceeds user authorized budget of ${:.2} (+${:.2})",
            total_amount,
            authorized_budget,
            total_amount - authorized_budget
        ));
    }

    if total_amount > MAX_TRANSACTION_AMOUNT {
        violations.push(format!(
            "Enterprise policy violation: Total ${:.2} exceeds org hard ceiling of ${:.2} (+${:.2})",
            total_amount,
            MAX_TRANSACTION_AMOUNT,
            total_amount - MAX_TRANSACTION_AMOUNT
        ));
    }

    if !is_approved_vendor(vendor) {
        violations.push(format!(
            "Untrusted supplier violation: Vendor '{}' is not in approved corporate procurement whitelist",
            vendor
        ));
    }

    if vendor_rating < MIN_VENDOR_RATING {
        violations.push(format!(
            "Supplier quality violation: Vendor rating ({:.1}/5.0) is below required minimum of {:.1}",
            vendor_rating, MIN_VENDOR_RATING
        ));
    }

    if max_delivery_days == 0 {
        return Err("VIOLATION: Delivery SLA must be at least one day".into());
    }
    let sla_cap = max_delivery_days;
    if delivery_days > sla_cap {
        violations.push(format!(
            "Delivery SLA violation: Estimated delivery ({} days) exceeds maximum mandate SLA ({} days)",
            delivery_days, sla_cap
        ));
    }

    if !violations.is_empty() {
        return Err(format!("VIOLATION: {}", violations.join("; ")));
    }

    Ok(ApprovedMandate {
        vendor: vendor.to_string(),
        total_amount,
        description,
        delivery_days,
        risk_score: approved_risk_score(total_amount, authorized_budget),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> Result<ApprovedMandate, String> {
        validate(
            "DigiKey",
            4.9,
            785.00,
            900.00,
            2,
            3,
            "10x Raspberry Pi 5".to_string(),
        )
    }

    #[test]
    fn approves_compliant_mandate() {
        let mandate = valid().expect("compliant mandate must pass");
        assert_eq!(mandate.vendor(), "DigiKey");
        assert_eq!(mandate.total_amount(), 785.00);
        assert!(mandate.risk_score() >= 0.04 && mandate.risk_score() <= 0.20);
    }

    #[test]
    fn blocks_budget_ceiling_breach() {
        let err = validate("DigiKey", 4.9, 1_000.00, 900.00, 2, 3, "x".into()).unwrap_err();
        assert!(err.contains("authorized budget"));
    }

    #[test]
    fn blocks_hard_cap_even_with_large_budget() {
        let err = validate("DigiKey", 4.9, 2_500.00, 5_000.00, 2, 3, "x".into()).unwrap_err();
        assert!(err.contains("hard ceiling"));
    }

    #[test]
    fn blocks_untrusted_vendor_and_low_rating() {
        let err = validate(
            "ShadowPartsShenzhen",
            2.9,
            360.00,
            400.00,
            21,
            14,
            "x".into(),
        )
        .unwrap_err();
        assert!(err.contains("ShadowPartsShenzhen"));
        assert!(err.contains("Supplier quality"));
    }

    #[test]
    fn blocks_sla_breach() {
        let err = validate("DigiKey", 4.9, 785.00, 900.00, 5, 3, "x".into()).unwrap_err();
        assert!(err.contains("Delivery SLA"));
    }
}

use std::collections::BTreeMap;

use sdkwork_contract_service::{CommerceMoney, CommerceServiceError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RechargePackageItem {
    pub id: String,
    pub price_amount: CommerceMoney,
    pub currency_code: String,
    pub bonus_points: i64,
    pub grant_amount: i64,
    pub points: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatePointsRechargeOrderOutcome {
    pub success: bool,
    pub order_id: String,
    pub order_no: String,
    pub out_trade_no: String,
    pub amount: CommerceMoney,
    pub currency_code: String,
    pub points: i64,
    pub provider_code: String,
    pub payment_method: String,
    pub payment_product: String,
    pub status: String,
    pub next_action: String,
    pub cashier_url: String,
    pub qr_code_payload: String,
    pub request_payment_payload: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RechargeGrantPreview {
    pub grant_amount: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RechargeSettingsSnapshot {
    pub base_currency_code: String,
    pub base_points_per_cny: String,
    pub currency_to_cny_rates: BTreeMap<String, String>,
    pub preview_examples: BTreeMap<String, BTreeMap<String, RechargeGrantPreview>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckoutStatusSnapshot {
    pub order_id: String,
    pub order_no: String,
    pub out_trade_no: String,
    pub amount: CommerceMoney,
    pub currency_code: String,
    pub points: i64,
    pub provider_code: String,
    pub payment_method: String,
    pub payment_product: String,
    pub order_status: String,
    pub payment_status: String,
    pub recharge_status: String,
    pub status: String,
    pub created_at: String,
    pub expires_at: String,
    pub paid_at: String,
    pub next_action: String,
    pub cashier_url: String,
    pub qr_code_payload: String,
    pub request_payment_payload: Option<String>,
}

impl RechargePackageItem {
    pub fn new(
        id: &str,
        price_amount: CommerceMoney,
        currency_code: &str,
        bonus_points: i64,
        grant_amount: i64,
        points: i64,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("id", id)?;
        crate::validation::require_non_empty("currency_code", currency_code)?;
        if bonus_points < 0 {
            return Err(CommerceServiceError::validation(
                "recharge bonus points must be non-negative",
            ));
        }
        if grant_amount < 0 {
            return Err(CommerceServiceError::validation(
                "recharge grant amount must be non-negative",
            ));
        }
        if points < 0 {
            return Err(CommerceServiceError::validation(
                "recharge points must be non-negative",
            ));
        }

        Ok(Self {
            id: id.trim().to_string(),
            price_amount,
            currency_code: currency_code.trim().to_ascii_uppercase(),
            bonus_points,
            grant_amount,
            points,
        })
    }
}

impl RechargeSettingsSnapshot {
    pub fn new(
        base_currency_code: &str,
        base_points_per_cny: &str,
        currency_to_cny_rates: BTreeMap<String, String>,
        preview_examples: BTreeMap<String, BTreeMap<String, RechargeGrantPreview>>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("base_currency_code", base_currency_code)?;
        crate::validation::require_non_empty("base_points_per_cny", base_points_per_cny)?;
        if currency_to_cny_rates.is_empty() {
            return Err(CommerceServiceError::validation(
                "recharge currency to CNY rates must not be empty",
            ));
        }
        if !currency_to_cny_rates
            .keys()
            .any(|currency_code| currency_code == &base_currency_code.trim().to_ascii_uppercase())
        {
            return Err(CommerceServiceError::validation(
                "recharge base currency rate must be configured",
            ));
        }

        Ok(Self {
            base_currency_code: base_currency_code.trim().to_ascii_uppercase(),
            base_points_per_cny: base_points_per_cny.trim().to_string(),
            currency_to_cny_rates,
            preview_examples,
        })
    }
}

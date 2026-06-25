use sdkwork_commerce_contract_service::{CommerceMoney, CommerceServiceError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckoutLineInput {
    pub sku_id: String,
    pub quantity: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateCheckoutSessionCommand {
    pub currency_code: String,
    pub idempotency_key: String,
    pub lines: Vec<CheckoutLineInput>,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub request_no: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckoutSessionDetailQuery {
    pub checkout_session_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateCheckoutQuoteCommand {
    pub checkout_session_id: String,
    pub idempotency_key: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub request_no: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckoutSessionView {
    pub checkout_session_id: String,
    pub currency_code: String,
    pub discount_amount: CommerceMoney,
    pub original_amount: CommerceMoney,
    pub payable_amount: CommerceMoney,
    pub quote_id: Option<String>,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckoutQuoteView {
    pub checkout_session_id: String,
    pub currency_code: String,
    pub discount_amount: CommerceMoney,
    pub original_amount: CommerceMoney,
    pub payable_amount: CommerceMoney,
    pub quote_id: String,
}

impl CheckoutLineInput {
    pub fn new(sku_id: &str, quantity: i64) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("sku_id", sku_id)?;
        if quantity <= 0 {
            return Err(CommerceServiceError::validation(
                "checkout line quantity must be greater than zero",
            ));
        }
        Ok(Self {
            sku_id: sku_id.trim().to_string(),
            quantity,
        })
    }
}

impl CreateCheckoutSessionCommand {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        currency_code: &str,
        lines: Vec<CheckoutLineInput>,
        request_no: &str,
        idempotency_key: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("currency_code", currency_code)?;
        crate::validation::require_non_empty("request_no", request_no)?;
        crate::validation::require_non_empty("idempotency_key", idempotency_key)?;
        if lines.is_empty() {
            return Err(CommerceServiceError::validation(
                "checkout session requires at least one line",
            ));
        }

        Ok(Self {
            currency_code: currency_code.trim().to_ascii_uppercase(),
            idempotency_key: idempotency_key.trim().to_string(),
            lines,
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            request_no: request_no.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
        })
    }
}

impl CheckoutSessionDetailQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        checkout_session_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("checkout_session_id", checkout_session_id)?;

        Ok(Self {
            checkout_session_id: checkout_session_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
        })
    }
}

impl CreateCheckoutQuoteCommand {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        checkout_session_id: &str,
        request_no: &str,
        idempotency_key: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("checkout_session_id", checkout_session_id)?;
        crate::validation::require_non_empty("request_no", request_no)?;
        crate::validation::require_non_empty("idempotency_key", idempotency_key)?;

        Ok(Self {
            checkout_session_id: checkout_session_id.trim().to_string(),
            idempotency_key: idempotency_key.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            request_no: request_no.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
        })
    }
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

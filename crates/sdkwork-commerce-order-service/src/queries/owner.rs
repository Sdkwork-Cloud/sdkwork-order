use std::collections::BTreeMap;

use sdkwork_commerce_contract_service::{CommerceMoney, CommerceServiceError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderOwnerListQuery {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub status: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderOwnerDetailQuery {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub order_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderOwnerSummary {
    pub order_id: String,
    pub order_sn: String,
    pub status: String,
    pub subject: String,
    pub total_amount: CommerceMoney,
    pub paid_amount: Option<CommerceMoney>,
    pub discount_amount: Option<CommerceMoney>,
    pub quantity: i64,
    pub created_at: String,
    pub pay_time: Option<String>,
    pub expire_time: Option<String>,
    pub payment_method: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderOwnerItem {
    pub id: String,
    pub product_name: String,
    pub quantity: i64,
    pub unit_price: CommerceMoney,
    pub total_amount: CommerceMoney,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderOwnerDetail {
    pub summary: OrderOwnerSummary,
    pub items: Vec<OrderOwnerItem>,
    pub out_trade_no: Option<String>,
    pub transaction_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderOwnerStatistics {
    pub total_orders: i64,
    pub pending_payment: i64,
    pub pending_shipment: i64,
    pub pending_receipt: i64,
    pub completed: i64,
    pub total_amount: CommerceMoney,
}

impl OrderOwnerListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        status: Option<&str>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;

        Ok(Self {
            tenant_id: tenant_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            status: optional_text(status),
            page: page.unwrap_or(1).max(1),
            page_size: page_size.unwrap_or(20).clamp(1, 100),
        })
    }

    pub fn limit(&self) -> i64 {
        self.page_size
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }
}

impl OrderOwnerDetailQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        order_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("order_id", order_id)?;

        Ok(Self {
            tenant_id: tenant_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            order_id: order_id.trim().to_string(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancelOwnerOrderCommand {
    pub cancel_reason: Option<String>,
    pub order_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayOwnerOrderCommand {
    pub order_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub payment_method: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayOwnerOrderOutcome {
    pub amount: CommerceMoney,
    pub order_id: String,
    pub out_trade_no: String,
    pub payment_id: String,
    pub payment_method: String,
    pub payment_params: BTreeMap<String, String>,
}

impl CancelOwnerOrderCommand {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        order_id: &str,
        cancel_reason: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("order_id", order_id)?;

        Ok(Self {
            cancel_reason: optional_text(cancel_reason),
            order_id: order_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateOwnerOrderCommand {
    pub checkout_session_id: String,
    pub idempotency_key: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub request_no: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateOwnerOrderOutcome {
    pub order_id: String,
    pub order_sn: String,
    pub status: String,
    pub total_amount: CommerceMoney,
}

impl CreateOwnerOrderCommand {
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

impl PayOwnerOrderCommand {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        order_id: &str,
        payment_method: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("order_id", order_id)?;
        crate::validation::require_non_empty("payment_method", payment_method)?;

        Ok(Self {
            order_id: order_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            payment_method: payment_method.trim().to_ascii_lowercase(),
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

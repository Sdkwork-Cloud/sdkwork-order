use sdkwork_contract_service::{CommerceMoney, CommerceServiceError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AfterSalesRequestDetailQuery {
    pub after_sales_request_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AfterSalesEventListQuery {
    pub after_sales_request_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateAfterSalesRequestCommand {
    pub after_sales_type: String,
    pub description: Option<String>,
    pub idempotency_key: String,
    pub order_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub reason_code: String,
    pub request_no: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateAfterSalesReturnShipmentCommand {
    pub after_sales_request_id: String,
    pub carrier_code: Option<String>,
    pub idempotency_key: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub request_no: String,
    pub tenant_id: String,
    pub tracking_no: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateAfterSalesRequestCommand {
    pub after_sales_request_id: String,
    pub approved_amount: Option<String>,
    pub currency_code: Option<String>,
    pub description: Option<String>,
    pub idempotency_key: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub reason_code: Option<String>,
    pub request_no: String,
    pub requested_amount: Option<String>,
    pub reviewer_note: Option<String>,
    pub status: Option<String>,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AfterSalesRequestView {
    pub after_sales_no: String,
    pub after_sales_request_id: String,
    pub after_sales_type: String,
    pub currency_code: String,
    pub order_id: String,
    pub reason_code: String,
    pub requested_amount: CommerceMoney,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AfterSalesReturnShipmentView {
    pub after_sales_request_id: String,
    pub return_shipment_id: String,
    pub return_shipment_no: String,
    pub status: String,
    pub tracking_no: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AfterSalesEventView {
    pub after_sales_request_id: String,
    pub event_id: String,
    pub event_no: String,
    pub event_type: String,
    pub to_status: String,
}

impl AfterSalesRequestDetailQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        after_sales_request_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("after_sales_request_id", after_sales_request_id)?;

        Ok(Self {
            after_sales_request_id: after_sales_request_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
        })
    }
}

impl AfterSalesEventListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        after_sales_request_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        AfterSalesRequestDetailQuery::new(
            tenant_id,
            organization_id,
            owner_user_id,
            after_sales_request_id,
        )
        .map(|query| Self {
            after_sales_request_id: query.after_sales_request_id,
            organization_id: query.organization_id,
            owner_user_id: query.owner_user_id,
            tenant_id: query.tenant_id,
        })
    }
}

impl CreateAfterSalesRequestCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        order_id: &str,
        reason_code: &str,
        after_sales_type: &str,
        description: Option<&str>,
        request_no: &str,
        idempotency_key: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("order_id", order_id)?;
        crate::validation::require_non_empty("reason_code", reason_code)?;
        crate::validation::require_non_empty("after_sales_type", after_sales_type)?;
        crate::validation::require_non_empty("request_no", request_no)?;
        crate::validation::require_non_empty("idempotency_key", idempotency_key)?;

        Ok(Self {
            after_sales_type: after_sales_type.trim().to_ascii_lowercase(),
            description: optional_text(description),
            idempotency_key: idempotency_key.trim().to_string(),
            order_id: order_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            reason_code: reason_code.trim().to_string(),
            request_no: request_no.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
        })
    }
}

impl CreateAfterSalesReturnShipmentCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        after_sales_request_id: &str,
        tracking_no: Option<&str>,
        carrier_code: Option<&str>,
        request_no: &str,
        idempotency_key: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("after_sales_request_id", after_sales_request_id)?;
        crate::validation::require_non_empty("request_no", request_no)?;
        crate::validation::require_non_empty("idempotency_key", idempotency_key)?;

        Ok(Self {
            after_sales_request_id: after_sales_request_id.trim().to_string(),
            carrier_code: optional_text(carrier_code),
            idempotency_key: idempotency_key.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            request_no: request_no.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
            tracking_no: optional_text(tracking_no),
        })
    }
}

impl UpdateAfterSalesRequestCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        after_sales_request_id: &str,
        status: Option<&str>,
        reason_code: Option<&str>,
        description: Option<&str>,
        requested_amount: Option<&str>,
        approved_amount: Option<&str>,
        currency_code: Option<&str>,
        reviewer_note: Option<&str>,
        request_no: &str,
        idempotency_key: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("after_sales_request_id", after_sales_request_id)?;
        crate::validation::require_non_empty("request_no", request_no)?;
        crate::validation::require_non_empty("idempotency_key", idempotency_key)?;

        let command = Self {
            after_sales_request_id: after_sales_request_id.trim().to_string(),
            approved_amount: optional_text(approved_amount),
            currency_code: optional_text(currency_code),
            description: merge_optional_text(description, reviewer_note),
            idempotency_key: idempotency_key.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            reason_code: optional_text(reason_code),
            request_no: request_no.trim().to_string(),
            requested_amount: optional_text(requested_amount),
            reviewer_note: optional_text(reviewer_note),
            status: optional_text(status),
            tenant_id: tenant_id.trim().to_string(),
        };
        if !command.has_updates() {
            return Err(CommerceServiceError::validation(
                "at least one after-sales request field must be provided",
            ));
        }
        Ok(command)
    }

    pub fn has_updates(&self) -> bool {
        self.status.is_some()
            || self.reason_code.is_some()
            || self.description.is_some()
            || self.requested_amount.is_some()
            || self.approved_amount.is_some()
            || self.currency_code.is_some()
            || self.reviewer_note.is_some()
    }
}

fn merge_optional_text(primary: Option<&str>, secondary: Option<&str>) -> Option<String> {
    optional_text(primary).or_else(|| optional_text(secondary))
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

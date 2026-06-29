use sdkwork_contract_service::CommerceServiceError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FulfillmentListQuery {
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub order_id: Option<String>,
    pub status: Option<String>,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FulfillmentDetailQuery {
    pub fulfillment_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FulfillmentView {
    pub fulfillment_id: String,
    pub fulfillment_no: String,
    pub fulfillment_type: String,
    pub order_id: String,
    pub status: String,
}

impl FulfillmentListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        order_id: Option<&str>,
        status: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;

        Ok(Self {
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            order_id: optional_text(order_id),
            status: optional_text(status),
            tenant_id: tenant_id.trim().to_string(),
        })
    }
}

impl FulfillmentDetailQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        fulfillment_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("fulfillment_id", fulfillment_id)?;

        Ok(Self {
            fulfillment_id: fulfillment_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
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

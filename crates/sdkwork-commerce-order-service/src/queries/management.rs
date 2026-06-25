use sdkwork_commerce_contract_service::CommerceServiceError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderManagementListQuery {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub status: Option<String>,
    pub q: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderManagementDetailQuery {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub order_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancelManagementOrderCommand {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub order_id: String,
    pub cancel_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseManagementOrderCommand {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub order_id: String,
    pub close_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderManagementEventListQuery {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub order_id: String,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderCancellationListQuery {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub status: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderManagementEventView {
    pub id: String,
    pub event_type: String,
    pub from_status: Option<String>,
    pub to_status: String,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub message: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderCancellationView {
    pub id: String,
    pub order_id: String,
    pub status: String,
    pub reason_code: String,
    pub reason_message: Option<String>,
    pub created_at: String,
}

impl OrderManagementListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        status: Option<&str>,
        q: Option<&str>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        Ok(Self {
            tenant_id: tenant_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            status: optional_text(status),
            q: optional_text(q),
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

impl OrderManagementDetailQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        order_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("order_id", order_id)?;
        Ok(Self {
            tenant_id: tenant_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            order_id: order_id.trim().to_string(),
        })
    }
}

impl CancelManagementOrderCommand {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        order_id: &str,
        cancel_reason: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("order_id", order_id)?;
        Ok(Self {
            tenant_id: tenant_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            order_id: order_id.trim().to_string(),
            cancel_reason: optional_text(cancel_reason),
        })
    }
}

impl CloseManagementOrderCommand {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        order_id: &str,
        close_reason: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("order_id", order_id)?;
        Ok(Self {
            tenant_id: tenant_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            order_id: order_id.trim().to_string(),
            close_reason: optional_text(close_reason),
        })
    }
}

impl OrderManagementEventListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        order_id: &str,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("order_id", order_id)?;
        Ok(Self {
            tenant_id: tenant_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            order_id: order_id.trim().to_string(),
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

impl OrderCancellationListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        status: Option<&str>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        Ok(Self {
            tenant_id: tenant_id.trim().to_string(),
            organization_id: optional_text(organization_id),
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

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

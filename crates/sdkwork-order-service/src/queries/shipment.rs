use sdkwork_contract_service::CommerceServiceError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentDetailQuery {
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub shipment_id: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentPackageListQuery {
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub shipment_id: String,
    pub tenant_id: String,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentTrackingEventListQuery {
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub shipment_id: String,
    pub tenant_id: String,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentView {
    pub carrier_code: String,
    pub fulfillment_id: String,
    pub shipment_id: String,
    pub shipment_no: String,
    pub status: String,
    pub tracking_no: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentPackageView {
    pub package_id: String,
    pub package_no: String,
    pub package_type: String,
    pub shipment_id: String,
    pub status: String,
    pub tracking_no: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentTrackingEventView {
    pub event_id: String,
    pub event_status: Option<String>,
    pub event_time: String,
    pub event_type: String,
    pub location_text: Option<String>,
    pub shipment_id: String,
    pub tracking_event_no: String,
}

/// 物流包裹分页结果。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentPackagePage {
    pub items: Vec<ShipmentPackageView>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

/// 物流轨迹事件分页结果。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentTrackingEventPage {
    pub items: Vec<ShipmentTrackingEventView>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

impl ShipmentPackagePage {
    pub fn has_more(&self) -> bool {
        self.page.saturating_mul(self.page_size) < self.total
    }

    pub fn total_pages(&self) -> i64 {
        if self.page_size <= 0 {
            return 0;
        }
        (self.total + self.page_size - 1) / self.page_size
    }
}

impl ShipmentTrackingEventPage {
    pub fn has_more(&self) -> bool {
        self.page.saturating_mul(self.page_size) < self.total
    }

    pub fn total_pages(&self) -> i64 {
        if self.page_size <= 0 {
            return 0;
        }
        (self.total + self.page_size - 1) / self.page_size
    }
}

impl ShipmentDetailQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        shipment_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("shipment_id", shipment_id)?;

        Ok(Self {
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            shipment_id: shipment_id.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
        })
    }
}

impl ShipmentPackageListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        shipment_id: &str,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("shipment_id", shipment_id)?;
        let (page, page_size) = crate::validation::offset_list_params(page, page_size)?;
        Ok(Self {
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            shipment_id: shipment_id.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
            page,
            page_size,
        })
    }

    pub fn limit(&self) -> i64 {
        self.page_size
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }
}

impl ShipmentTrackingEventListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        shipment_id: &str,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("shipment_id", shipment_id)?;
        let (page, page_size) = crate::validation::offset_list_params(page, page_size)?;
        Ok(Self {
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            shipment_id: shipment_id.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
            page,
            page_size,
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

/// 运营端物流列表查询。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentManagementListQuery {
    pub fulfillment_id: Option<String>,
    pub order_id: Option<String>,
    pub organization_id: Option<String>,
    pub page: i64,
    pub page_size: i64,
    pub status: Option<String>,
    pub tenant_id: String,
}

/// 运营端物流详情查询。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentManagementDetailQuery {
    pub organization_id: Option<String>,
    pub shipment_id: String,
    pub tenant_id: String,
}

/// 运营端物流包裹列表查询。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentPackageManagementListQuery {
    pub organization_id: Option<String>,
    pub page: i64,
    pub page_size: i64,
    pub shipment_id: String,
    pub tenant_id: String,
}

/// 运营端创建物流包裹命令。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateShipmentPackageCommand {
    pub idempotency_key: String,
    pub organization_id: Option<String>,
    pub package_no: Option<String>,
    pub package_type: String,
    pub request_no: String,
    pub shipment_id: String,
    pub status: Option<String>,
    pub tenant_id: String,
    pub tracking_no: Option<String>,
}

/// 运营端更新物流包裹命令。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateShipmentPackageCommand {
    pub idempotency_key: String,
    pub organization_id: Option<String>,
    pub package_id: String,
    pub package_type: Option<String>,
    pub request_no: String,
    pub shipment_id: String,
    pub status: Option<String>,
    pub tenant_id: String,
    pub tracking_no: Option<String>,
}

/// 运营端物流分页结果。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentManagementListPage {
    pub items: Vec<ShipmentView>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

impl ShipmentManagementListPage {
    pub fn has_more(&self) -> bool {
        self.page.saturating_mul(self.page_size) < self.total
    }

    pub fn total_pages(&self) -> i64 {
        if self.page_size <= 0 {
            return 0;
        }
        (self.total + self.page_size - 1) / self.page_size
    }
}

impl ShipmentManagementListQuery {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        order_id: Option<&str>,
        fulfillment_id: Option<&str>,
        status: Option<&str>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        let (page, page_size) = crate::validation::offset_list_params(page, page_size)?;
        Ok(Self {
            fulfillment_id: optional_text(fulfillment_id),
            order_id: optional_text(order_id),
            organization_id: optional_text(organization_id),
            page,
            page_size,
            status: optional_text(status),
            tenant_id: tenant_id.trim().to_string(),
        })
    }

    pub fn limit(&self) -> i64 {
        self.page_size
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }
}

impl ShipmentManagementDetailQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        shipment_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("shipment_id", shipment_id)?;
        Ok(Self {
            organization_id: optional_text(organization_id),
            shipment_id: shipment_id.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
        })
    }
}

impl ShipmentPackageManagementListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        shipment_id: &str,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("shipment_id", shipment_id)?;
        let (page, page_size) = crate::validation::offset_list_params(page, page_size)?;
        Ok(Self {
            organization_id: optional_text(organization_id),
            page,
            page_size,
            shipment_id: shipment_id.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
        })
    }

    pub fn limit(&self) -> i64 {
        self.page_size
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }
}

impl CreateShipmentPackageCommand {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        shipment_id: &str,
        package_type: &str,
        request_no: &str,
        idempotency_key: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("shipment_id", shipment_id)?;
        crate::validation::require_non_empty("package_type", package_type)?;
        crate::validation::require_non_empty("request_no", request_no)?;
        crate::validation::require_non_empty("idempotency_key", idempotency_key)?;
        Ok(Self {
            idempotency_key: idempotency_key.trim().to_string(),
            organization_id: optional_text(organization_id),
            package_no: None,
            package_type: package_type.trim().to_string(),
            request_no: request_no.trim().to_string(),
            shipment_id: shipment_id.trim().to_string(),
            status: None,
            tenant_id: tenant_id.trim().to_string(),
            tracking_no: None,
        })
    }
}

impl UpdateShipmentPackageCommand {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        shipment_id: &str,
        package_id: &str,
        request_no: &str,
        idempotency_key: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("shipment_id", shipment_id)?;
        crate::validation::require_non_empty("package_id", package_id)?;
        crate::validation::require_non_empty("request_no", request_no)?;
        crate::validation::require_non_empty("idempotency_key", idempotency_key)?;
        Ok(Self {
            idempotency_key: idempotency_key.trim().to_string(),
            organization_id: optional_text(organization_id),
            package_id: package_id.trim().to_string(),
            package_type: None,
            request_no: request_no.trim().to_string(),
            shipment_id: shipment_id.trim().to_string(),
            status: None,
            tenant_id: tenant_id.trim().to_string(),
            tracking_no: None,
        })
    }
}

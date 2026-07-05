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

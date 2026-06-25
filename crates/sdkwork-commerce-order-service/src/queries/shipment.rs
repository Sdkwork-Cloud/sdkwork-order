use sdkwork_commerce_contract_service::CommerceServiceError;

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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentTrackingEventListQuery {
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub shipment_id: String,
    pub tenant_id: String,
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
    ) -> Result<Self, CommerceServiceError> {
        ShipmentDetailQuery::new(tenant_id, organization_id, owner_user_id, shipment_id).map(
            |query| Self {
                organization_id: query.organization_id,
                owner_user_id: query.owner_user_id,
                shipment_id: query.shipment_id,
                tenant_id: query.tenant_id,
            },
        )
    }
}

impl ShipmentTrackingEventListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        shipment_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        ShipmentDetailQuery::new(tenant_id, organization_id, owner_user_id, shipment_id).map(
            |query| Self {
                organization_id: query.organization_id,
                owner_user_id: query.owner_user_id,
                shipment_id: query.shipment_id,
                tenant_id: query.tenant_id,
            },
        )
    }
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

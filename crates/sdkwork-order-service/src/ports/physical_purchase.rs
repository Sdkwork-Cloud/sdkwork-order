use std::future::Future;
use std::pin::Pin;

use sdkwork_contract_service::{CommerceMoney, CommerceServiceError};

pub type PhysicalPurchaseFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShippingAddressSnapshot {
    pub receiver_name: String,
    pub receiver_phone: String,
    pub country_code: String,
    pub province: String,
    pub city: String,
    pub district: Option<String>,
    pub detail_address: String,
    pub postal_code: Option<String>,
}

impl ShippingAddressSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        receiver_name: &str,
        receiver_phone: &str,
        country_code: &str,
        province: &str,
        city: &str,
        district: Option<&str>,
        detail_address: &str,
        postal_code: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            receiver_name: required_text("receiver_name", receiver_name)?,
            receiver_phone: required_text("receiver_phone", receiver_phone)?,
            country_code: required_text("country_code", country_code)?.to_ascii_uppercase(),
            province: required_text("province", province)?,
            city: required_text("city", city)?,
            district: optional_text(district),
            detail_address: required_text("detail_address", detail_address)?,
            postal_code: optional_text(postal_code),
        })
    }

    pub fn snapshot_json(&self) -> Result<String, CommerceServiceError> {
        serde_json::to_string(self).map_err(|error| {
            CommerceServiceError::validation(format!(
                "shipping address snapshot is invalid: {error}"
            ))
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvePhysicalCheckoutLine {
    pub sku_id: String,
    pub quantity: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvePhysicalCheckoutRequest {
    pub tenant_id: String,
    pub owner_user_id: String,
    pub currency_code: String,
    pub lines: Vec<ResolvePhysicalCheckoutLine>,
    pub shipping_address: ShippingAddressSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedPhysicalCheckoutLine {
    pub sku_id: String,
    pub product_id: String,
    pub merchant_organization_id: String,
    pub shop_id: String,
    pub title: String,
    pub unit_price: CommerceMoney,
    pub currency_code: String,
    pub fulfillment_type: String,
    pub quantity: i64,
    pub inventory_tracking: String,
    pub sku_snapshot_json: String,
}

impl ResolvedPhysicalCheckoutLine {
    pub fn fulfillment_type_is_physical(&self) -> bool {
        matches!(
            self.fulfillment_type.trim().to_ascii_lowercase().as_str(),
            "physical" | "physical_shipment"
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedPhysicalCheckout {
    pub merchant_organization_id: String,
    pub shop_id: String,
    pub shop_snapshot_json: String,
    pub shipping_address: ShippingAddressSnapshot,
    pub lines: Vec<ResolvedPhysicalCheckoutLine>,
}

pub trait PhysicalCheckoutResolverPort: Send + Sync {
    fn resolve_physical_checkout<'a>(
        &'a self,
        request: ResolvePhysicalCheckoutRequest,
    ) -> PhysicalPurchaseFuture<'a, ResolvedPhysicalCheckout>;
}

#[derive(Default)]
pub struct UnavailablePhysicalCheckoutResolverPort;

impl PhysicalCheckoutResolverPort for UnavailablePhysicalCheckoutResolverPort {
    fn resolve_physical_checkout<'a>(
        &'a self,
        _request: ResolvePhysicalCheckoutRequest,
    ) -> PhysicalPurchaseFuture<'a, ResolvedPhysicalCheckout> {
        Box::pin(async move {
            Err(CommerceServiceError::provider_unavailable(
                "physical checkout resolver is not configured",
            ))
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalInventoryLine {
    pub sku_id: String,
    pub shop_id: String,
    pub quantity: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservePhysicalOrderInventoryRequest {
    pub tenant_id: String,
    pub merchant_organization_id: String,
    pub order_id: String,
    pub request_no: String,
    pub idempotency_key: String,
    pub lines: Vec<PhysicalInventoryLine>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleasePhysicalOrderInventoryRequest {
    pub tenant_id: String,
    pub order_id: String,
    pub reason_code: String,
    pub request_no: String,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalInventoryMutationOutcome {
    pub accepted: bool,
    pub replayed: bool,
}

pub trait PhysicalInventoryReservationPort: Send + Sync {
    fn reserve_physical_order_inventory<'a>(
        &'a self,
        request: ReservePhysicalOrderInventoryRequest,
    ) -> PhysicalPurchaseFuture<'a, PhysicalInventoryMutationOutcome>;

    fn release_physical_order_inventory<'a>(
        &'a self,
        request: ReleasePhysicalOrderInventoryRequest,
    ) -> PhysicalPurchaseFuture<'a, PhysicalInventoryMutationOutcome>;
}

#[derive(Default)]
pub struct UnavailablePhysicalInventoryReservationPort;

impl PhysicalInventoryReservationPort for UnavailablePhysicalInventoryReservationPort {
    fn reserve_physical_order_inventory<'a>(
        &'a self,
        _request: ReservePhysicalOrderInventoryRequest,
    ) -> PhysicalPurchaseFuture<'a, PhysicalInventoryMutationOutcome> {
        Box::pin(async move {
            Err(CommerceServiceError::provider_unavailable(
                "physical inventory reservation is not configured",
            ))
        })
    }

    fn release_physical_order_inventory<'a>(
        &'a self,
        _request: ReleasePhysicalOrderInventoryRequest,
    ) -> PhysicalPurchaseFuture<'a, PhysicalInventoryMutationOutcome> {
        Box::pin(async move {
            Err(CommerceServiceError::provider_unavailable(
                "physical inventory release is not configured",
            ))
        })
    }
}

pub fn physical_inventory_reserve_idempotency_key(order_id: &str) -> String {
    format!("physical-goods:reserve:{order_id}")
}

pub fn physical_inventory_release_idempotency_key(order_id: &str) -> String {
    format!("physical-goods:release:{order_id}")
}

fn required_text(field: &str, value: &str) -> Result<String, CommerceServiceError> {
    crate::validation::require_non_empty(field, value)?;
    Ok(value.trim().to_owned())
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

pub const PHYSICAL_CHECKOUT_RESOLVER_PORT: &str = "merchandise.physical_checkout.resolver";
pub const PHYSICAL_INVENTORY_RESERVATION_PORT: &str = "inventory.physical_order.reservation";

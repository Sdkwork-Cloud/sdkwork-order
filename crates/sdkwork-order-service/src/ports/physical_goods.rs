use std::future::Future;
use std::pin::Pin;

use sdkwork_contract_service::CommerceServiceError;

pub type PhysicalGoodsFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FulfillPaidPhysicalOrderRequest {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub order_id: String,
    pub paid_at: String,
    pub request_no: String,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalGoodsFulfillmentOutcome {
    pub accepted: bool,
    pub replayed: bool,
    pub fulfillment_status: String,
}

/// Cross-capability boundary for consuming reserved stock and opening shipment fulfillment.
/// Implementations must use the request idempotency key for both inventory and Order writes.
pub trait PhysicalGoodsFulfillmentPort: Send + Sync {
    fn fulfill_paid_physical_order<'a>(
        &'a self,
        request: FulfillPaidPhysicalOrderRequest,
    ) -> PhysicalGoodsFuture<'a, PhysicalGoodsFulfillmentOutcome>;
}

#[derive(Default)]
pub struct UnavailablePhysicalGoodsFulfillmentPort;

impl PhysicalGoodsFulfillmentPort for UnavailablePhysicalGoodsFulfillmentPort {
    fn fulfill_paid_physical_order<'a>(
        &'a self,
        _request: FulfillPaidPhysicalOrderRequest,
    ) -> PhysicalGoodsFuture<'a, PhysicalGoodsFulfillmentOutcome> {
        Box::pin(async move {
            Err(CommerceServiceError::provider_unavailable(
                "physical goods fulfillment is not configured",
            ))
        })
    }
}

pub fn physical_goods_fulfillment_idempotency_key(order_id: &str) -> String {
    format!("physical-goods:fulfill:{order_id}")
}

pub const PHYSICAL_GOODS_FULFILLMENT_PORT: &str = "inventory.physical_goods.fulfillment";

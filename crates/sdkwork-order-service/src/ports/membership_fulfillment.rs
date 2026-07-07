use std::future::Future;
use std::pin::Pin;

use sdkwork_contract_service::CommerceServiceError;

pub type MembershipPurchaseFulfillmentFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembershipPurchaseFulfillmentRequest {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub order_id: String,
    pub request_no: String,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembershipPurchaseFulfillmentOutcome {
    pub accepted: bool,
    pub replayed: bool,
    pub fulfillment_status: String,
}

pub trait MembershipPurchaseFulfillmentPort: Send + Sync {
    fn fulfill_membership_purchase<'a>(
        &'a self,
        request: MembershipPurchaseFulfillmentRequest,
    ) -> MembershipPurchaseFulfillmentFuture<'a, MembershipPurchaseFulfillmentOutcome>;
}

pub fn membership_purchase_fulfillment_idempotency_key(order_id: &str) -> String {
    format!("membership-purchase:fulfill:{order_id}")
}

pub const MEMBERSHIP_PURCHASE_FULFILLMENT_PORT: &str = "membership.purchase.fulfillment";

/// No-op adapter used when membership fulfillment is not wired at gateway assembly.
pub struct NoopMembershipPurchaseFulfillmentPort;

impl MembershipPurchaseFulfillmentPort for NoopMembershipPurchaseFulfillmentPort {
    fn fulfill_membership_purchase<'a>(
        &'a self,
        _request: MembershipPurchaseFulfillmentRequest,
    ) -> MembershipPurchaseFulfillmentFuture<'a, MembershipPurchaseFulfillmentOutcome> {
        Box::pin(async move {
            Ok(MembershipPurchaseFulfillmentOutcome {
                accepted: false,
                replayed: false,
                fulfillment_status: "awaiting_external_fulfillment".to_owned(),
            })
        })
    }
}

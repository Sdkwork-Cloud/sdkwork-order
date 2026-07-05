use std::future::Future;
use std::pin::Pin;

use sdkwork_contract_service::CommerceServiceError;

use crate::{
    FulfillPointsRechargeOrderCommand, FulfillPointsRechargeOrderOutcome,
    MarkPointsRechargePaymentSucceededCommand, PointsRechargeFulfillmentContext,
};

pub type PointsRechargeFulfillmentFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait PointsRechargeFulfillmentStore: Send + Sync {
    fn load_points_recharge_fulfillment_context<'a>(
        &'a self,
        command: &'a FulfillPointsRechargeOrderCommand,
    ) -> PointsRechargeFulfillmentFuture<'a, Option<PointsRechargeFulfillmentContext>>;

    fn commit_points_recharge_fulfillment<'a>(
        &'a self,
        command: FulfillPointsRechargeOrderCommand,
        context: &'a PointsRechargeFulfillmentContext,
    ) -> PointsRechargeFulfillmentFuture<'a, FulfillPointsRechargeOrderOutcome>;

    fn rollback_points_recharge_fulfillment<'a>(
        &'a self,
        command: &'a FulfillPointsRechargeOrderCommand,
        context: &'a PointsRechargeFulfillmentContext,
    ) -> PointsRechargeFulfillmentFuture<'a, ()>;

    fn mark_points_recharge_payment_succeeded<'a>(
        &'a self,
        command: MarkPointsRechargePaymentSucceededCommand,
    ) -> PointsRechargeFulfillmentFuture<'a, ()>;
}

pub const POINTS_RECHARGE_FULFILLMENT_STORE: &str = "order.points_recharge.fulfillment";

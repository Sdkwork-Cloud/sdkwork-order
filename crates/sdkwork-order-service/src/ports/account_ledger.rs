use std::future::Future;
use std::pin::Pin;

use sdkwork_contract_service::CommerceServiceError;

pub type AccountPointsCreditFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PointsRechargeCreditRequest {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub order_id: String,
    pub order_no: String,
    pub points: i64,
    pub request_no: String,
    pub idempotency_key: String,
    pub transaction_no: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PointsRechargeCreditOutcome {
    pub accepted: bool,
    pub replayed: bool,
}

pub trait AccountPointsCreditPort: Send + Sync {
    fn credit_points_recharge<'a>(
        &'a self,
        request: PointsRechargeCreditRequest,
    ) -> AccountPointsCreditFuture<'a, PointsRechargeCreditOutcome>;

    fn reverse_points_recharge_credit<'a>(
        &'a self,
        request: PointsRechargeCreditRequest,
    ) -> AccountPointsCreditFuture<'a, PointsRechargeCreditOutcome>;
}

pub fn points_recharge_compensation_idempotency_key(order_id: &str) -> String {
    format!("points-recharge:compensate:{order_id}")
}

pub fn points_recharge_compensation_transaction_no(order_id: &str) -> String {
    format!("points-recharge:compensate:{order_id}")
}

pub const ACCOUNT_POINTS_CREDIT_PORT: &str = "account.points.credit";

pub fn points_recharge_fulfillment_idempotency_key(order_id: &str) -> String {
    format!("points-recharge:fulfill:{order_id}")
}

pub fn points_recharge_fulfillment_transaction_no(order_id: &str) -> String {
    format!("points-recharge:{order_id}")
}

pub fn points_recharge_payment_success_idempotency_key(order_id: &str) -> String {
    format!("points-recharge:payment-success:{order_id}")
}

pub const POINTS_RECHARGE_LEDGER_BUSINESS_TYPE: &str = "points_recharge";

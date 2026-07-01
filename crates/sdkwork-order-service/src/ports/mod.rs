mod account_ledger;
mod points_recharge_fulfillment;

pub use account_ledger::{
    points_recharge_fulfillment_idempotency_key, points_recharge_fulfillment_transaction_no,
    points_recharge_payment_success_idempotency_key, AccountPointsCreditFuture,
    AccountPointsCreditPort, PointsRechargeCreditOutcome, PointsRechargeCreditRequest,
    ACCOUNT_POINTS_CREDIT_PORT, POINTS_RECHARGE_LEDGER_BUSINESS_TYPE,
};
pub use points_recharge_fulfillment::{
    PointsRechargeFulfillmentFuture, PointsRechargeFulfillmentStore,
    POINTS_RECHARGE_FULFILLMENT_STORE,
};

use crate::{CreateOrderCommand, OrderDetailQuery, OrderListQuery, PaidOrderReference};
use sdkwork_contract_service::CommerceServiceError;

pub trait OrderRepositoryPort {
    fn create_order(
        &self,
        command: &CreateOrderCommand,
    ) -> Result<PaidOrderReference, CommerceServiceError>;

    fn retrieve_order(
        &self,
        query: &OrderDetailQuery,
    ) -> Result<Option<PaidOrderReference>, CommerceServiceError>;

    fn list_orders(&self, query: &OrderListQuery) -> Result<Vec<String>, CommerceServiceError>;
}

pub const ORDER_REPOSITORY_PORT: &str = "order.repository";
pub const IDEMPOTENCY_REPOSITORY_PORT: &str = "idempotency.repository";

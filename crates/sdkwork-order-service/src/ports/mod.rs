mod account_ledger;
mod membership_fulfillment;
mod owner_order_payment;
mod points_recharge_fulfillment;

pub use owner_order_payment::{
    ConfirmOwnerOrderPaymentOutcome, OrderPaymentSettlementAttempt,
    OwnerOrderPaymentConfirmationFuture, OwnerOrderPaymentConfirmationPort,
    OWNER_ORDER_PAYMENT_CONFIRMATION_PORT,
};
pub use membership_fulfillment::{
    membership_purchase_fulfillment_idempotency_key, MembershipPurchaseFulfillmentFuture,
    MembershipPurchaseFulfillmentOutcome, MembershipPurchaseFulfillmentPort,
    MembershipPurchaseFulfillmentRequest, NoopMembershipPurchaseFulfillmentPort,
    MEMBERSHIP_PURCHASE_FULFILLMENT_PORT,
};
pub use account_ledger::{
    points_recharge_compensation_idempotency_key, points_recharge_compensation_transaction_no,
    points_recharge_fulfillment_idempotency_key, points_recharge_fulfillment_transaction_no,
    points_recharge_payment_success_idempotency_key, AccountPointsCreditFuture,
    AccountPointsCreditPort, PointsRechargeCreditOutcome, PointsRechargeCreditRequest,
    ACCOUNT_POINTS_CREDIT_PORT, POINTS_RECHARGE_LEDGER_BUSINESS_TYPE,
};
pub use points_recharge_fulfillment::{
    PointsRechargeFulfillmentFuture, PointsRechargeFulfillmentStore,
    POINTS_RECHARGE_FULFILLMENT_STORE,
};

/// 仓储端口标识符，用于 `CommerceServiceContract` 能力注册。
///
/// 实际仓储抽象由 `SqliteCommerceOrderStore` / `PostgresCommerceOrderStore` 通过
/// 路由层枚举适配器（`BackendOrderAdminStore` / `AppAfterSalesState`）提供，
/// 无需额外的同步 trait 抽象层。
pub const ORDER_REPOSITORY_PORT: &str = "order.repository";
pub const IDEMPOTENCY_REPOSITORY_PORT: &str = "idempotency.repository";

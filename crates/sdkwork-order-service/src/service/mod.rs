mod account_value_fulfillment;
mod account_value_request_execution;
mod order_payment_settlement;
mod points_recharge_fulfillment;

pub use account_value_fulfillment::{
    default_fulfill_account_value_order_command, fulfill_account_value_order,
};
pub use account_value_request_execution::execute_account_value_request_review;
pub use order_payment_settlement::{
    settle_owner_order_after_payment_success, stable_checkout_order_subject,
    stable_order_settlement_subject, OrderSubjectKind, OwnerOrderSettlementOutcome,
    OwnerOrderSettlementPorts,
};
pub use points_recharge_fulfillment::{
    default_fulfill_points_recharge_command, fulfill_points_recharge_order,
    ledger_business_type_for_points_recharge, mark_points_recharge_payment_succeeded,
};

use sdkwork_contract_service::CommerceServiceContract;

pub fn order_service_contract() -> CommerceServiceContract {
    CommerceServiceContract::new(
        "order",
        "commerce.order",
        vec![
            "checkout.sessions.create",
            "checkout.sessions.quotes.create",
            "checkout.sessions.orders.create",
            "afterSales.requests.create",
            "afterSales.requests.update",
            "afterSales.returnShipments.create",
            "afterSales.reviews.create",
            "orders.cancel",
            "orders.cancellations.create",
            "orders.payments.create",
            "recharges.orders.create",
            "recharges.orders.cancel",
            "orders.refundRequests.create",
            "withdrawals.requests.create",
            "backend.accountValuePackages.create",
            "backend.accountValuePackages.update",
            "backend.accountValuePackages.retire",
            "backend.tokenBankPlans.create",
            "backend.tokenBankPlans.update",
            "backend.tokenBankPlans.retire",
            "backend.refundRequests.approve",
            "backend.refundRequests.reject",
            "backend.refundRequests.retry",
            "backend.withdrawalRequests.approve",
            "backend.withdrawalRequests.reject",
            "backend.withdrawalRequests.retry",
            "memberships.orders.create",
            "orders.paymentConfirmations.create",
            "orders.admin.cancel",
            "orders.admin.close",
            "shipments.packages.create",
            "shipments.packages.update",
        ],
        vec![
            "checkout.sessions.retrieve",
            "orders.list",
            "orders.retrieve",
            "orders.events.list",
            "recharges.plans.list",
            "orders.refundRequests.list",
            "orders.refundRequests.retrieve",
            "withdrawals.requests.retrieve",
            "backend.accountValuePackages.list",
            "backend.tokenBankPlans.list",
            "backend.refundRequests.list",
            "backend.withdrawalRequests.list",
            "afterSales.requests.list",
            "afterSales.requests.retrieve",
            "afterSales.management.list",
            "afterSales.management.retrieve",
            "afterSales.returnShipments.list",
            "afterSales.events.list",
            "fulfillments.list",
            "fulfillments.retrieve",
            "shipments.list",
            "shipments.retrieve",
            "shipments.packages.list",
            "shipments.packages.management.list",
            "shipments.trackingEvents.list",
        ],
        vec![
            crate::ports::ORDER_REPOSITORY_PORT,
            crate::ports::IDEMPOTENCY_REPOSITORY_PORT,
            crate::ports::POINTS_RECHARGE_FULFILLMENT_STORE,
            crate::ports::ACCOUNT_POINTS_CREDIT_PORT,
            crate::ports::ACCOUNT_VALUE_LEDGER_PORT,
            crate::ports::PAYMENT_REFUND_EXECUTOR_PORT,
            crate::ports::PAYMENT_PAYOUT_EXECUTOR_PORT,
            crate::ports::COUPON_REDEMPTION_PORT,
            crate::ports::OWNER_ORDER_PAYMENT_CONFIRMATION_PORT,
            crate::ports::OWNER_ORDER_PAYMENT_STATE_PORT,
            crate::ports::MEMBERSHIP_PURCHASE_FULFILLMENT_PORT,
        ],
        true,
    )
}

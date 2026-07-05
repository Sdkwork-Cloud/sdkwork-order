mod order_payment_settlement;
mod points_recharge_fulfillment;

pub use order_payment_settlement::{
    settle_owner_order_after_payment_success, OwnerOrderSettlementOutcome,
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
            "afterSales.returnShipments.create",
            "afterSales.reviews.create",
            "orders.cancellations.create",
            "orders.pointsRecharge.fulfillments.create",
            "shipments.packages.create",
            "shipments.packages.update",
        ],
        vec![
            "checkout.sessions.retrieve",
            "orders.list",
            "orders.retrieve",
            "orders.events.list",
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
            "commerceReports.orderRevenue.list",
            "audit.commerceEvents.list",
        ],
        vec![
            crate::ports::ORDER_REPOSITORY_PORT,
            crate::ports::IDEMPOTENCY_REPOSITORY_PORT,
            crate::ports::POINTS_RECHARGE_FULFILLMENT_STORE,
            crate::ports::ACCOUNT_POINTS_CREDIT_PORT,
            crate::ports::OWNER_ORDER_PAYMENT_CONFIRMATION_PORT,
        ],
        true,
    )
}

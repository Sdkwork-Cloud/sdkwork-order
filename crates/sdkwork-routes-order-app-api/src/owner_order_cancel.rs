//! Owner-initiated order cancel orchestration (payments before order state).

use sdkwork_contract_service::CommerceServiceError;
use sdkwork_order_service::CancelOwnerOrderCommand;

use crate::order_router::{CommerceOrderStore, OwnerOrderPaymentStore};

/// Close payment intents first, then cancel the order.
///
/// Payment cancellation is attempted before mutating order status so a PSP
/// failure does not leave a cancelled order with still-open payment attempts.
pub async fn cancel_owner_order_with_payments(
    orders: &dyn CommerceOrderStore,
    payments: &dyn OwnerOrderPaymentStore,
    command: CancelOwnerOrderCommand,
) -> Result<(), CommerceServiceError> {
    payments
        .cancel_owner_order_payments(command.clone())
        .await?;
    orders.cancel_owner_order(command).await
}

/// Best-effort rollback when a recharge checkout create succeeded but pay failed.
pub async fn compensate_failed_recharge_pay(
    orders: &dyn CommerceOrderStore,
    payments: &dyn OwnerOrderPaymentStore,
    command: CancelOwnerOrderCommand,
) {
    let _ = cancel_owner_order_with_payments(orders, payments, command).await;
}

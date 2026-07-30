//! Owner-initiated order cancel orchestration (payments before order state).

use sdkwork_contract_service::CommerceServiceError;
use sdkwork_order_service::{
    physical_inventory_release_idempotency_key, CancelOwnerOrderCommand,
    PhysicalInventoryReservationPort, ReleasePhysicalOrderInventoryRequest,
};

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

pub async fn cancel_owner_order_with_payments_and_inventory(
    orders: &dyn CommerceOrderStore,
    payments: &dyn OwnerOrderPaymentStore,
    inventory: &dyn PhysicalInventoryReservationPort,
    command: CancelOwnerOrderCommand,
    request_no: &str,
) -> Result<(), CommerceServiceError> {
    let release = ReleasePhysicalOrderInventoryRequest {
        tenant_id: command.tenant_id.clone(),
        order_id: command.order_id.clone(),
        reason_code: command
            .cancel_type
            .clone()
            .unwrap_or_else(|| "buyer_cancelled".to_owned()),
        request_no: request_no.to_owned(),
        idempotency_key: physical_inventory_release_idempotency_key(&command.order_id),
    };
    cancel_owner_order_with_payments(orders, payments, command).await?;
    inventory.release_physical_order_inventory(release).await?;
    Ok(())
}

/// Best-effort rollback when a recharge checkout create succeeded but pay failed.
pub async fn compensate_failed_recharge_pay(
    orders: &dyn CommerceOrderStore,
    payments: &dyn OwnerOrderPaymentStore,
    command: CancelOwnerOrderCommand,
) {
    let _ = cancel_owner_order_with_payments(orders, payments, command).await;
}

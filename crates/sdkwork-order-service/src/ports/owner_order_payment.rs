pub use sdkwork_payment_service::{
    ConfirmOwnerOrderPaymentOutcome, OrderPaymentSettlementAttempt,
    OwnerOrderPaymentConfirmationFuture, OwnerOrderPaymentConfirmationPort,
    OWNER_ORDER_PAYMENT_CONFIRMATION_PORT,
};

/// Order-owned persistence boundary for the payment-success part of settlement.
///
/// Payment confirms the provider intent/attempt, while Order remains the only
/// owner allowed to advance `commerce_order` lifecycle state.
pub trait OwnerOrderPaymentStatePort: Send + Sync {
    fn mark_owner_order_payment_succeeded<'a>(
        &'a self,
        attempt: &'a OrderPaymentSettlementAttempt,
        paid_at: &'a str,
    ) -> OwnerOrderPaymentConfirmationFuture<'a, ()>;
}

pub const OWNER_ORDER_PAYMENT_STATE_PORT: &str = "order.owner_order_payment.state";

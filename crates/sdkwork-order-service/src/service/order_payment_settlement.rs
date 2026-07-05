use sdkwork_contract_service::CommerceServiceError;

use crate::{
    default_fulfill_points_recharge_command, fulfill_points_recharge_order,
    mark_points_recharge_payment_succeeded, points_recharge_payment_success_idempotency_key,
    AccountPointsCreditPort, MarkPointsRechargePaymentSucceededCommand,
    OrderPaymentSettlementAttempt, OwnerOrderPaymentConfirmationPort, PointsRechargeFulfillmentStore,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OwnerOrderSettlementOutcome {
    pub payment_confirmed: bool,
    pub payment_replayed: bool,
    pub fulfillment_accepted: bool,
    pub fulfillment_replayed: bool,
    pub order_id: String,
    pub points_credited: i64,
    pub fulfillment_status: String,
}

pub async fn settle_owner_order_after_payment_success<S, P, Payment>(
    payment_store: &Payment,
    recharge_store: &S,
    credit_port: &P,
    attempt: &OrderPaymentSettlementAttempt,
    order_subject: Option<&str>,
    request_no: &str,
) -> Result<OwnerOrderSettlementOutcome, CommerceServiceError>
where
    S: PointsRechargeFulfillmentStore,
    P: AccountPointsCreditPort + ?Sized,
    Payment: OwnerOrderPaymentConfirmationPort + ?Sized,
{
    let payment_outcome = payment_store
        .confirm_owner_order_payment(
            &attempt.tenant_id,
            attempt.organization_id.as_deref(),
            &attempt.owner_user_id,
            &attempt.order_id,
        )
        .await?;

    let mut fulfillment_accepted = false;
    let mut fulfillment_replayed = false;
    let mut points_credited = 0_i64;
    let mut fulfillment_status = String::new();

    if is_points_recharge_subject(order_subject) {
        let idempotency_key =
            points_recharge_payment_success_idempotency_key(&attempt.order_id);
        let payment_command = MarkPointsRechargePaymentSucceededCommand::new(
            &attempt.tenant_id,
            attempt.organization_id.as_deref(),
            &attempt.owner_user_id,
            &attempt.order_id,
            &payment_outcome.paid_at,
            request_no,
            &idempotency_key,
        )?;
        mark_points_recharge_payment_succeeded(recharge_store, payment_command).await?;

        let fulfill_command = default_fulfill_points_recharge_command(
            &attempt.tenant_id,
            attempt.organization_id.as_deref(),
            &attempt.owner_user_id,
            &attempt.order_id,
            request_no,
        )?;
        let fulfill_outcome =
            fulfill_points_recharge_order(recharge_store, credit_port, fulfill_command).await?;
        fulfillment_accepted = fulfill_outcome.accepted;
        fulfillment_replayed = fulfill_outcome.replayed;
        points_credited = fulfill_outcome.points_credited;
        fulfillment_status = fulfill_outcome.fulfillment_status;
    }

    Ok(OwnerOrderSettlementOutcome {
        payment_confirmed: true,
        payment_replayed: payment_outcome.replayed,
        fulfillment_accepted,
        fulfillment_replayed,
        order_id: attempt.order_id.clone(),
        points_credited,
        fulfillment_status,
    })
}

fn is_points_recharge_subject(subject: Option<&str>) -> bool {
    subject
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some_and(|value| value.eq_ignore_ascii_case("points_recharge"))
}

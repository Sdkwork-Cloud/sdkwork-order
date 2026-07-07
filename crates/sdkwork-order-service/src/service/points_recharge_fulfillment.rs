use sdkwork_contract_service::CommerceServiceError;

use crate::{
    points_recharge_compensation_idempotency_key, points_recharge_compensation_transaction_no,
    AccountPointsCreditPort, FulfillPointsRechargeOrderCommand, FulfillPointsRechargeOrderOutcome,
    MarkPointsRechargePaymentSucceededCommand, PointsRechargeCreditRequest,
    PointsRechargeFulfillmentContext, PointsRechargeFulfillmentStore,
    POINTS_RECHARGE_LEDGER_BUSINESS_TYPE, points_recharge_fulfillment_idempotency_key,
    points_recharge_fulfillment_transaction_no,
};

pub async fn mark_points_recharge_payment_succeeded<S>(
    store: &S,
    command: MarkPointsRechargePaymentSucceededCommand,
) -> Result<(), CommerceServiceError>
where
    S: PointsRechargeFulfillmentStore,
{
    store.mark_points_recharge_payment_succeeded(command).await
}

pub async fn fulfill_points_recharge_order<S, P>(
    store: &S,
    credit_port: &P,
    command: FulfillPointsRechargeOrderCommand,
) -> Result<FulfillPointsRechargeOrderOutcome, CommerceServiceError>
where
    S: PointsRechargeFulfillmentStore,
    P: AccountPointsCreditPort + ?Sized,
{
    let Some(context) = store
        .load_points_recharge_fulfillment_context(&command)
        .await?
    else {
        return Err(CommerceServiceError::not_found(
            "points recharge order was not found",
        ));
    };

    if context.already_fulfilled() {
        return Ok(FulfillPointsRechargeOrderOutcome::replayed(
            &context.order_id,
            &context.order_no,
            context.points,
        ));
    }

    context.validate_for_fulfillment()?;

    store
        .reserve_points_recharge_fulfillment(&command, &context)
        .await?;

    let credit_request = build_credit_request(&command, &context);
    let credit_outcome = match credit_port.credit_points_recharge(credit_request.clone()).await {
        Ok(outcome) => outcome,
        Err(error) => {
            let _ = store
                .release_points_recharge_fulfillment_reservation(&command, &context)
                .await;
            return Err(error);
        }
    };

    let mut outcome = match store
        .commit_points_recharge_fulfillment(command.clone(), &context)
        .await
    {
        Ok(outcome) => outcome,
        Err(error) => {
            tracing::error!(
                target = "order.fulfillment",
                order_id = %context.order_id,
                ?error,
                "points recharge fulfillment commit failed after account credit; compensating"
            );
            if !credit_outcome.replayed {
                let compensation = build_compensation_request(&credit_request);
                if let Err(compensation_error) =
                    credit_port.reverse_points_recharge_credit(compensation).await
                {
                    tracing::error!(
                        target = "order.fulfillment",
                        order_id = %context.order_id,
                        ?compensation_error,
                        "points recharge compensation debit failed; operator replay required"
                    );
                }
            }
            let _ = store
                .release_points_recharge_fulfillment_reservation(&command, &context)
                .await;
            return Err(error);
        }
    };

    if outcome.replayed {
        return Ok(outcome);
    }

    outcome.replayed = credit_outcome.replayed;
    Ok(outcome)
}

fn build_credit_request(
    command: &FulfillPointsRechargeOrderCommand,
    context: &PointsRechargeFulfillmentContext,
) -> PointsRechargeCreditRequest {
    PointsRechargeCreditRequest {
        tenant_id: command.tenant_id.clone(),
        organization_id: command.organization_id.clone(),
        owner_user_id: command.owner_user_id.clone(),
        order_id: context.order_id.clone(),
        order_no: context.order_no.clone(),
        points: context.points,
        request_no: command.request_no.clone(),
        idempotency_key: points_recharge_fulfillment_idempotency_key(&context.order_id),
        transaction_no: points_recharge_fulfillment_transaction_no(&context.order_id),
    }
}

fn build_compensation_request(
    credit_request: &PointsRechargeCreditRequest,
) -> PointsRechargeCreditRequest {
    PointsRechargeCreditRequest {
        tenant_id: credit_request.tenant_id.clone(),
        organization_id: credit_request.organization_id.clone(),
        owner_user_id: credit_request.owner_user_id.clone(),
        order_id: credit_request.order_id.clone(),
        order_no: credit_request.order_no.clone(),
        points: credit_request.points,
        request_no: format!("{}:compensate", credit_request.request_no),
        idempotency_key: points_recharge_compensation_idempotency_key(&credit_request.order_id),
        transaction_no: points_recharge_compensation_transaction_no(&credit_request.order_id),
    }
}

pub fn default_fulfill_points_recharge_command(
    tenant_id: &str,
    organization_id: Option<&str>,
    owner_user_id: &str,
    order_id: &str,
    request_no: &str,
) -> Result<FulfillPointsRechargeOrderCommand, CommerceServiceError> {
    FulfillPointsRechargeOrderCommand::new(
        tenant_id,
        organization_id,
        owner_user_id,
        order_id,
        request_no,
        &points_recharge_fulfillment_idempotency_key(order_id),
    )
}

#[allow(dead_code)]
pub fn ledger_business_type_for_points_recharge() -> &'static str {
    POINTS_RECHARGE_LEDGER_BUSINESS_TYPE
}

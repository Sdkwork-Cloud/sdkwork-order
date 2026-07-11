use sdkwork_contract_service::CommerceServiceError;

use crate::{
    AccountValueFulfillmentContext, AccountValueFulfillmentStore, AccountValueLedgerCommand,
    AccountValueLedgerPort, AccountValueOrderSubject, FulfillAccountValueOrderCommand,
    FulfillAccountValueOrderOutcome,
};

pub async fn fulfill_account_value_order<S, L>(
    store: &S,
    ledger_port: &L,
    command: FulfillAccountValueOrderCommand,
) -> Result<FulfillAccountValueOrderOutcome, CommerceServiceError>
where
    S: AccountValueFulfillmentStore,
    L: AccountValueLedgerPort + ?Sized,
{
    let Some(context) = store
        .load_account_value_fulfillment_context(&command)
        .await?
    else {
        return Err(CommerceServiceError::not_found(
            "account value order was not found",
        ));
    };

    if context.already_fulfilled() {
        return Ok(FulfillAccountValueOrderOutcome::replayed(&context));
    }

    context.validate_for_fulfillment()?;
    store
        .reserve_account_value_fulfillment(&command, &context)
        .await?;

    let credit_command = build_credit_command(&command, &context)?;
    let ledger_outcome = match ledger_port
        .apply_account_value_ledger_command(credit_command.clone())
        .await
    {
        Ok(outcome) => outcome,
        Err(error) => {
            let _ = store
                .release_account_value_fulfillment_reservation(&command, &context)
                .await;
            return Err(error);
        }
    };

    match store
        .commit_account_value_fulfillment(command.clone(), &context)
        .await
    {
        Ok(mut outcome) => {
            if !outcome.replayed {
                outcome.replayed = ledger_outcome.replayed;
            }
            Ok(outcome)
        }
        Err(error) => {
            tracing::error!(
                target = "order.account_value_fulfillment",
                order_id = %context.order_id,
                ?error,
                "account value fulfillment commit failed after account ledger credit; compensating"
            );
            if !ledger_outcome.replayed {
                let compensation = build_compensation_command(&command, &context)?;
                if let Err(compensation_error) = ledger_port
                    .apply_account_value_ledger_command(compensation)
                    .await
                {
                    tracing::error!(
                        target = "order.account_value_fulfillment",
                        order_id = %context.order_id,
                        ?compensation_error,
                        "account value fulfillment compensation failed; operator replay required"
                    );
                }
            }
            let _ = store
                .release_account_value_fulfillment_reservation(&command, &context)
                .await;
            Err(error)
        }
    }
}

pub fn default_fulfill_account_value_order_command(
    subject: AccountValueOrderSubject,
    tenant_id: &str,
    organization_id: Option<&str>,
    owner_user_id: &str,
    order_id: &str,
    request_no: &str,
) -> Result<FulfillAccountValueOrderCommand, CommerceServiceError> {
    FulfillAccountValueOrderCommand::new(
        tenant_id,
        organization_id,
        owner_user_id,
        order_id,
        request_no,
        &subject.fulfillment_idempotency_key(order_id)?,
    )
}

fn build_credit_command(
    command: &FulfillAccountValueOrderCommand,
    context: &AccountValueFulfillmentContext,
) -> Result<AccountValueLedgerCommand, CommerceServiceError> {
    let business_type = context
        .subject
        .fulfillment_business_type(context.target_asset);
    AccountValueLedgerCommand::credit(
        &command.tenant_id,
        command.organization_id.as_deref(),
        &command.owner_user_id,
        context.target_asset,
        context.grant_amount.clone(),
        ledger_unit_code(context)?,
        business_type,
        &context.order_id,
        &command.request_no,
        &context
            .subject
            .fulfillment_idempotency_key(&context.order_id)?,
    )
}

fn build_compensation_command(
    command: &FulfillAccountValueOrderCommand,
    context: &AccountValueFulfillmentContext,
) -> Result<AccountValueLedgerCommand, CommerceServiceError> {
    let business_type = context
        .subject
        .compensation_business_type(context.target_asset);
    AccountValueLedgerCommand::debit(
        &command.tenant_id,
        command.organization_id.as_deref(),
        &command.owner_user_id,
        context.target_asset,
        context.grant_amount.clone(),
        ledger_unit_code(context)?,
        business_type,
        &context.order_id,
        &format!("{}:compensate", command.request_no),
        &format!(
            "{}:compensate",
            context
                .subject
                .fulfillment_idempotency_key(&context.order_id)?
        ),
    )
}

fn ledger_unit_code(
    context: &AccountValueFulfillmentContext,
) -> Result<&str, CommerceServiceError> {
    let configured = context.asset_unit_code.trim();
    if !configured.is_empty() {
        return Ok(configured);
    }
    let default = context.target_asset.default_unit_code();
    if default.is_empty() {
        return Err(CommerceServiceError::validation(
            "account value fulfillment requires asset unit code",
        ));
    }
    Ok(default)
}

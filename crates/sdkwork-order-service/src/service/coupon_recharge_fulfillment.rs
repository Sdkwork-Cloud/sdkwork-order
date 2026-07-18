use sdkwork_contract_service::CommerceServiceError;

use crate::{
    coupon_recharge_redemption_idempotency_key, fulfill_account_value_order, AccountValueAssetCode,
    AccountValueFulfillmentStore, AccountValueLedgerPort, AccountValueOrderSubject,
    CouponRedemptionPort, CouponRedemptionRequest, FulfillAccountValueOrderCommand,
    FulfillAccountValueOrderOutcome,
};

pub async fn redeem_coupon_and_fulfill_account_value_order<S, C, L>(
    store: &S,
    coupon_port: &C,
    ledger_port: &L,
    command: FulfillAccountValueOrderCommand,
) -> Result<FulfillAccountValueOrderOutcome, CommerceServiceError>
where
    S: AccountValueFulfillmentStore + ?Sized,
    C: CouponRedemptionPort + ?Sized,
    L: AccountValueLedgerPort + ?Sized,
{
    let Some(context) = store
        .load_account_value_fulfillment_context(&command)
        .await?
    else {
        return Err(CommerceServiceError::not_found(
            "coupon recharge order was not found",
        ));
    };

    if context.subject != AccountValueOrderSubject::CouponRecharge
        || context.target_asset != AccountValueAssetCode::TokenBank
    {
        return Err(CommerceServiceError::validation(
            "coupon recharge fulfillment requires a Token Bank coupon order",
        ));
    }
    if context.already_fulfilled() {
        return Ok(FulfillAccountValueOrderOutcome::replayed(&context));
    }

    let coupon_code = context
        .coupon_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            CommerceServiceError::invalid_state("coupon recharge order has no coupon code")
        })?;
    let redemption = coupon_port
        .redeem_coupon(CouponRedemptionRequest {
            tenant_id: command.tenant_id.clone(),
            organization_id: command.organization_id.clone(),
            owner_user_id: command.owner_user_id.clone(),
            coupon_code: coupon_code.to_owned(),
            order_id: context.order_id.clone(),
            request_no: context.order_id.clone(),
            idempotency_key: coupon_recharge_redemption_idempotency_key(&context.order_id),
        })
        .await?;

    if !redemption.accepted
        || redemption.target_asset != context.target_asset
        || redemption.grant_amount != context.grant_amount
    {
        return Err(CommerceServiceError::conflict(
            "coupon benefit changed before Token Bank fulfillment",
        ));
    }

    fulfill_account_value_order(store, ledger_port, command).await
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sdkwork_contract_service::{CommerceLedgerBusinessType, CommerceMoney, CommerceServiceError};
use sdkwork_order_service::{
    default_fulfill_account_value_order_command, fulfill_account_value_order,
    settle_owner_order_after_payment_success, AccountPointsCreditFuture, AccountPointsCreditPort,
    AccountValueAssetCode, AccountValueFulfillmentContext, AccountValueFulfillmentFuture,
    AccountValueFulfillmentStore, AccountValueLedgerCommand, AccountValueLedgerOutcome,
    AccountValueLedgerPort, AccountValueOrderSubject, ConfirmOwnerOrderPaymentOutcome,
    FulfillAccountValueOrderCommand, FulfillAccountValueOrderOutcome,
    FulfillPointsRechargeOrderCommand, FulfillPointsRechargeOrderOutcome,
    MarkPointsRechargePaymentSucceededCommand, MembershipPurchaseFulfillmentFuture,
    MembershipPurchaseFulfillmentOutcome, MembershipPurchaseFulfillmentPort,
    MembershipPurchaseFulfillmentRequest, OrderPaymentSettlementAttempt,
    OwnerOrderPaymentConfirmationFuture, OwnerOrderPaymentConfirmationPort,
    PointsRechargeCreditOutcome, PointsRechargeCreditRequest, PointsRechargeFulfillmentContext,
    PointsRechargeFulfillmentFuture, PointsRechargeFulfillmentStore,
};

#[tokio::test]
async fn fulfill_token_bank_recharge_credits_account_value_ledger_then_commits_order() {
    let store = Arc::new(MockAccountValueFulfillmentStore::default());
    let ledger = Arc::new(MockAccountValueLedgerPort::default());

    store.seed_context(AccountValueFulfillmentContext {
        order_id: "order-token-1".to_owned(),
        order_no: "ORD-TOKEN-1".to_owned(),
        subject: AccountValueOrderSubject::TokenBankRecharge,
        target_asset: AccountValueAssetCode::TokenBank,
        order_status: "pending_payment".to_owned(),
        fulfillment_status: "unfulfilled".to_owned(),
        payment_status: "success".to_owned(),
        payment_attempt_status: "succeeded".to_owned(),
        grant_amount: CommerceMoney::new("30000").expect("grant"),
        asset_unit_code: "TOKEN_BANK".to_owned(),
    });

    let command = default_fulfill_account_value_order_command(
        AccountValueOrderSubject::TokenBankRecharge,
        "tenant-1",
        Some("org-1"),
        "user-1",
        "order-token-1",
        "req-token-fulfill-1",
    )
    .expect("command");

    let outcome = fulfill_account_value_order(store.as_ref(), ledger.as_ref(), command)
        .await
        .expect("fulfillment");

    assert!(outcome.accepted);
    assert!(!outcome.replayed);
    assert_eq!(outcome.target_asset, AccountValueAssetCode::TokenBank);
    assert_eq!(outcome.amount.as_str(), "30000");
    assert_eq!(store.reserve_calls(), 1);
    assert_eq!(store.commit_calls(), 1);

    let calls = ledger.commands();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].asset, AccountValueAssetCode::TokenBank);
    assert_eq!(calls[0].amount.as_str(), "30000");
    assert_eq!(calls[0].currency_code, "TOKEN_BANK");
    assert_eq!(
        calls[0].business_type,
        CommerceLedgerBusinessType::TOKEN_BANK_PURCHASE_CREDIT
    );
    assert_eq!(calls[0].resource_id, "order-token-1");
    assert_eq!(
        calls[0].idempotency_key,
        "token-bank-recharge:fulfill:order-token-1"
    );
}

#[tokio::test]
async fn fulfill_account_value_order_replays_without_duplicate_ledger_credit() {
    let store = Arc::new(MockAccountValueFulfillmentStore::default());
    let ledger = Arc::new(MockAccountValueLedgerPort::default());

    store.seed_context(AccountValueFulfillmentContext {
        order_id: "order-token-replayed".to_owned(),
        order_no: "ORD-TOKEN-REPLAYED".to_owned(),
        subject: AccountValueOrderSubject::TokenBankPlanPurchase,
        target_asset: AccountValueAssetCode::TokenBank,
        order_status: "fulfilled".to_owned(),
        fulfillment_status: "fulfilled".to_owned(),
        payment_status: "success".to_owned(),
        payment_attempt_status: "succeeded".to_owned(),
        grant_amount: CommerceMoney::new("50000").expect("grant"),
        asset_unit_code: "TOKEN_BANK".to_owned(),
    });

    let command = default_fulfill_account_value_order_command(
        AccountValueOrderSubject::TokenBankPlanPurchase,
        "tenant-1",
        None,
        "user-1",
        "order-token-replayed",
        "req-token-fulfill-2",
    )
    .expect("command");

    let outcome = fulfill_account_value_order(store.as_ref(), ledger.as_ref(), command)
        .await
        .expect("fulfillment replay");

    assert!(outcome.replayed);
    assert_eq!(ledger.commands().len(), 0);
    assert_eq!(store.reserve_calls(), 0);
    assert_eq!(store.commit_calls(), 0);
}

#[tokio::test]
async fn settlement_payment_success_dispatches_token_bank_recharge_to_account_value_ledger() {
    let payment_store = Arc::new(MockOwnerOrderPaymentStore::default());
    let account_value_store = Arc::new(MockAccountValueFulfillmentStore::default());
    let account_value_ledger = Arc::new(MockAccountValueLedgerPort::default());
    let points_store = Arc::new(UnsupportedPointsRechargeStore);
    let points_port = Arc::new(UnsupportedAccountPointsCreditPort);
    let membership_port = Arc::new(UnsupportedMembershipPurchaseFulfillmentPort);

    account_value_store.seed_context(AccountValueFulfillmentContext {
        order_id: "order-token-settle".to_owned(),
        order_no: "ORD-TOKEN-SETTLE".to_owned(),
        subject: AccountValueOrderSubject::TokenBankRecharge,
        target_asset: AccountValueAssetCode::TokenBank,
        order_status: "pending_payment".to_owned(),
        fulfillment_status: "unfulfilled".to_owned(),
        payment_status: "success".to_owned(),
        payment_attempt_status: "succeeded".to_owned(),
        grant_amount: CommerceMoney::new("120000").expect("grant"),
        asset_unit_code: "TOKEN_BANK".to_owned(),
    });

    let attempt = OrderPaymentSettlementAttempt {
        tenant_id: "tenant-1".to_owned(),
        organization_id: Some("org-1".to_owned()),
        owner_user_id: "user-1".to_owned(),
        order_id: "order-token-settle".to_owned(),
    };

    let outcome = settle_owner_order_after_payment_success(
        payment_store.as_ref(),
        points_store.as_ref(),
        account_value_store.as_ref(),
        points_port.as_ref(),
        account_value_ledger.as_ref(),
        membership_port.as_ref(),
        &attempt,
        Some("token_bank_recharge"),
        "req-token-settle-1",
    )
    .await
    .expect("token bank settlement");

    assert!(outcome.payment_confirmed);
    assert!(outcome.fulfillment_accepted);
    assert!(!outcome.fulfillment_replayed);
    assert_eq!(outcome.points_credited, 0);
    assert_eq!(outcome.fulfillment_status, "fulfilled");
    assert_eq!(payment_store.confirm_calls(), 1);
    assert_eq!(account_value_store.reserve_calls(), 1);
    assert_eq!(account_value_store.commit_calls(), 1);

    let commands = account_value_ledger.commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].asset, AccountValueAssetCode::TokenBank);
    assert_eq!(commands[0].amount.as_str(), "120000");
    assert_eq!(commands[0].currency_code, "TOKEN_BANK");
}

#[derive(Default)]
struct MockAccountValueFulfillmentStore {
    contexts: Mutex<HashMap<String, AccountValueFulfillmentContext>>,
    reserve_calls: Mutex<u32>,
    commit_calls: Mutex<u32>,
    release_calls: Mutex<u32>,
}

impl MockAccountValueFulfillmentStore {
    fn seed_context(&self, context: AccountValueFulfillmentContext) {
        self.contexts
            .lock()
            .expect("contexts lock")
            .insert(context.order_id.clone(), context);
    }

    fn reserve_calls(&self) -> u32 {
        *self.reserve_calls.lock().expect("reserve lock")
    }

    fn commit_calls(&self) -> u32 {
        *self.commit_calls.lock().expect("commit lock")
    }
}

#[derive(Default)]
struct MockOwnerOrderPaymentStore {
    confirm_calls: Mutex<u32>,
}

impl MockOwnerOrderPaymentStore {
    fn confirm_calls(&self) -> u32 {
        *self.confirm_calls.lock().expect("confirm lock")
    }
}

impl OwnerOrderPaymentConfirmationPort for MockOwnerOrderPaymentStore {
    fn confirm_owner_order_payment<'a>(
        &'a self,
        tenant_id: &'a str,
        organization_id: Option<&'a str>,
        owner_user_id: &'a str,
        order_id: &'a str,
    ) -> OwnerOrderPaymentConfirmationFuture<'a, ConfirmOwnerOrderPaymentOutcome> {
        *self.confirm_calls.lock().expect("confirm lock") += 1;
        Box::pin(async move {
            Ok(ConfirmOwnerOrderPaymentOutcome {
                tenant_id: tenant_id.to_owned(),
                organization_id: organization_id.map(str::to_owned),
                owner_user_id: owner_user_id.to_owned(),
                order_id: order_id.to_owned(),
                paid_at: "2026-07-08 00:00:00".to_owned(),
                replayed: false,
            })
        })
    }
}

struct UnsupportedPointsRechargeStore;

impl PointsRechargeFulfillmentStore for UnsupportedPointsRechargeStore {
    fn load_points_recharge_fulfillment_context<'a>(
        &'a self,
        _command: &'a FulfillPointsRechargeOrderCommand,
    ) -> PointsRechargeFulfillmentFuture<'a, Option<PointsRechargeFulfillmentContext>> {
        Box::pin(async {
            Err(CommerceServiceError::unsupported_capability(
                "points recharge store should not be called for token_bank_recharge",
            ))
        })
    }

    fn reserve_points_recharge_fulfillment<'a>(
        &'a self,
        _command: &'a FulfillPointsRechargeOrderCommand,
        _context: &'a PointsRechargeFulfillmentContext,
    ) -> PointsRechargeFulfillmentFuture<'a, ()> {
        Box::pin(async {
            Err(CommerceServiceError::unsupported_capability(
                "points recharge reservation should not be called for token_bank_recharge",
            ))
        })
    }

    fn release_points_recharge_fulfillment_reservation<'a>(
        &'a self,
        _command: &'a FulfillPointsRechargeOrderCommand,
        _context: &'a PointsRechargeFulfillmentContext,
    ) -> PointsRechargeFulfillmentFuture<'a, ()> {
        Box::pin(async {
            Err(CommerceServiceError::unsupported_capability(
                "points recharge release should not be called for token_bank_recharge",
            ))
        })
    }

    fn commit_points_recharge_fulfillment<'a>(
        &'a self,
        _command: FulfillPointsRechargeOrderCommand,
        _context: &'a PointsRechargeFulfillmentContext,
    ) -> PointsRechargeFulfillmentFuture<'a, FulfillPointsRechargeOrderOutcome> {
        Box::pin(async {
            Err(CommerceServiceError::unsupported_capability(
                "points recharge commit should not be called for token_bank_recharge",
            ))
        })
    }

    fn rollback_points_recharge_fulfillment<'a>(
        &'a self,
        _command: &'a FulfillPointsRechargeOrderCommand,
        _context: &'a PointsRechargeFulfillmentContext,
    ) -> PointsRechargeFulfillmentFuture<'a, ()> {
        Box::pin(async {
            Err(CommerceServiceError::unsupported_capability(
                "points recharge rollback should not be called for token_bank_recharge",
            ))
        })
    }

    fn mark_points_recharge_payment_succeeded<'a>(
        &'a self,
        _command: MarkPointsRechargePaymentSucceededCommand,
    ) -> PointsRechargeFulfillmentFuture<'a, ()> {
        Box::pin(async {
            Err(CommerceServiceError::unsupported_capability(
                "points recharge payment success should not be called for token_bank_recharge",
            ))
        })
    }
}

struct UnsupportedAccountPointsCreditPort;

impl AccountPointsCreditPort for UnsupportedAccountPointsCreditPort {
    fn credit_points_recharge<'a>(
        &'a self,
        _request: PointsRechargeCreditRequest,
    ) -> AccountPointsCreditFuture<'a, PointsRechargeCreditOutcome> {
        Box::pin(async {
            Err(CommerceServiceError::unsupported_capability(
                "points credit port should not be called for token_bank_recharge",
            ))
        })
    }

    fn reverse_points_recharge_credit<'a>(
        &'a self,
        _request: PointsRechargeCreditRequest,
    ) -> AccountPointsCreditFuture<'a, PointsRechargeCreditOutcome> {
        Box::pin(async {
            Err(CommerceServiceError::unsupported_capability(
                "points reverse port should not be called for token_bank_recharge",
            ))
        })
    }
}

struct UnsupportedMembershipPurchaseFulfillmentPort;

impl MembershipPurchaseFulfillmentPort for UnsupportedMembershipPurchaseFulfillmentPort {
    fn fulfill_membership_purchase<'a>(
        &'a self,
        _request: MembershipPurchaseFulfillmentRequest,
    ) -> MembershipPurchaseFulfillmentFuture<'a, MembershipPurchaseFulfillmentOutcome> {
        Box::pin(async {
            Err(CommerceServiceError::unsupported_capability(
                "membership port should not be called for token_bank_recharge",
            ))
        })
    }
}

impl AccountValueFulfillmentStore for MockAccountValueFulfillmentStore {
    fn load_account_value_fulfillment_context<'a>(
        &'a self,
        command: &'a FulfillAccountValueOrderCommand,
    ) -> AccountValueFulfillmentFuture<'a, Option<AccountValueFulfillmentContext>> {
        let context = self
            .contexts
            .lock()
            .expect("contexts lock")
            .get(&command.order_id)
            .cloned();
        Box::pin(async move { Ok(context) })
    }

    fn reserve_account_value_fulfillment<'a>(
        &'a self,
        _command: &'a FulfillAccountValueOrderCommand,
        _context: &'a AccountValueFulfillmentContext,
    ) -> AccountValueFulfillmentFuture<'a, ()> {
        *self.reserve_calls.lock().expect("reserve lock") += 1;
        Box::pin(async { Ok(()) })
    }

    fn release_account_value_fulfillment_reservation<'a>(
        &'a self,
        _command: &'a FulfillAccountValueOrderCommand,
        _context: &'a AccountValueFulfillmentContext,
    ) -> AccountValueFulfillmentFuture<'a, ()> {
        *self.release_calls.lock().expect("release lock") += 1;
        Box::pin(async { Ok(()) })
    }

    fn commit_account_value_fulfillment<'a>(
        &'a self,
        _command: FulfillAccountValueOrderCommand,
        context: &'a AccountValueFulfillmentContext,
    ) -> AccountValueFulfillmentFuture<'a, FulfillAccountValueOrderOutcome> {
        *self.commit_calls.lock().expect("commit lock") += 1;
        Box::pin(async move { Ok(FulfillAccountValueOrderOutcome::fulfilled(context)) })
    }
}

#[derive(Default)]
struct MockAccountValueLedgerPort {
    commands: Mutex<Vec<AccountValueLedgerCommand>>,
}

impl MockAccountValueLedgerPort {
    fn commands(&self) -> Vec<AccountValueLedgerCommand> {
        self.commands.lock().expect("commands lock").clone()
    }
}

impl AccountValueLedgerPort for MockAccountValueLedgerPort {
    fn apply_account_value_ledger_command<'a>(
        &'a self,
        command: AccountValueLedgerCommand,
    ) -> AccountValueFulfillmentFuture<'a, AccountValueLedgerOutcome> {
        self.commands.lock().expect("commands lock").push(command);
        Box::pin(async {
            Ok(AccountValueLedgerOutcome {
                accepted: true,
                replayed: false,
                ledger_entry_id: Some("ledger-1".to_owned()),
                account_effect_reference_id: None,
            })
        })
    }
}

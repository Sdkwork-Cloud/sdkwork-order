use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sdkwork_contract_service::{CommerceLedgerBusinessType, CommerceMoney, CommerceServiceError};
use sdkwork_order_service::{
    execute_account_value_request_review, AccountValueAssetCode, AccountValueFuture,
    AccountValueLedgerCommand, AccountValueLedgerOperation, AccountValueLedgerOutcome,
    AccountValueLedgerPort, AccountValueOrderSubject, AccountValueRequestExecutionStore,
    AccountValueRequestReviewAction, AccountValueRequestStatusCommand, AccountValueRequestView,
    PaymentExecutorOutcome, PaymentPayoutExecutionRequest, PaymentPayoutExecutorPort,
    PaymentRefundExecutionRequest, PaymentRefundExecutorPort, ReviewAccountValueRequestCommand,
};

#[tokio::test]
async fn approve_token_bank_refund_holds_account_value_then_refunds_and_settles_hold() {
    let store = Arc::new(MockAccountValueRequestStore::default());
    let ledger = Arc::new(MockAccountValueLedgerPort::default());
    let refunds = Arc::new(MockPaymentRefundExecutorPort::default());
    let payouts = Arc::new(MockPaymentPayoutExecutorPort::default());

    let request = request_view(
        "refund-token-1",
        Some("order-token-1"),
        AccountValueOrderSubject::RefundRequest,
        AccountValueAssetCode::TokenBank,
        "32000",
        "TOKEN_BANK",
        "requested",
    )
    .with_provider_amount("9900", "CNY");
    store.seed(request);

    let command = review_command(
        AccountValueOrderSubject::RefundRequest,
        "refund-token-1",
        AccountValueRequestReviewAction::Approve,
    );

    let outcome = execute_account_value_request_review(
        store.as_ref(),
        ledger.as_ref(),
        refunds.as_ref(),
        payouts.as_ref(),
        command,
    )
    .await
    .expect("refund execution");

    assert_eq!(outcome.status, "refunded");
    assert_eq!(
        outcome.provider_reference_id.as_deref(),
        Some("provider-refund-1")
    );
    assert_eq!(
        outcome.account_effect_reference_id.as_deref(),
        Some("account-hold-1")
    );

    let ledger_commands = ledger.commands();
    assert_eq!(ledger_commands.len(), 2);
    assert_eq!(
        ledger_commands[0].operation,
        AccountValueLedgerOperation::Hold
    );
    assert_eq!(ledger_commands[0].asset, AccountValueAssetCode::TokenBank);
    assert_eq!(ledger_commands[0].amount.as_str(), "32000");
    assert_eq!(
        ledger_commands[0].business_type,
        CommerceLedgerBusinessType::TOKEN_BANK_HOLD
    );
    assert_eq!(ledger_commands[0].resource_id, "refund-token-1");
    assert_eq!(
        ledger_commands[0].idempotency_key,
        "refund-request:account-hold:refund-token-1"
    );

    assert_eq!(
        ledger_commands[1].operation,
        AccountValueLedgerOperation::HoldSettle
    );
    assert_eq!(ledger_commands[1].resource_id, "account-hold-1");
    assert_eq!(
        ledger_commands[1].business_type,
        CommerceLedgerBusinessType::TOKEN_BANK_REVERSAL
    );
    assert_eq!(
        ledger_commands[1].idempotency_key,
        "refund-request:account-hold:refund-token-1:settle"
    );

    let refund_requests = refunds.requests();
    assert_eq!(refund_requests.len(), 1);
    assert_eq!(refund_requests[0].owner_user_id, "user-1");
    assert_eq!(refund_requests[0].refund_request_id, "refund-token-1");
    assert_eq!(refund_requests[0].original_order_id, "order-token-1");
    assert_eq!(refund_requests[0].amount.as_str(), "9900");
    assert_eq!(refund_requests[0].currency_code, "CNY");
    assert_eq!(
        refund_requests[0].idempotency_key,
        "refund-request:payment-refund:refund-token-1"
    );
    assert!(payouts.requests().is_empty());

    let statuses = store.status_commands();
    assert_eq!(statuses.len(), 3);
    assert_eq!(statuses[0].status, "account_reversal_held");
    assert_eq!(
        statuses[0].account_effect_reference_id.as_deref(),
        Some("account-hold-1")
    );
    assert_eq!(statuses[1].status, "provider_refund_processing");
    assert_eq!(statuses[2].status, "refunded");
}

#[tokio::test]
async fn approve_cash_withdrawal_holds_cash_then_payouts_and_settles_hold() {
    let store = Arc::new(MockAccountValueRequestStore::default());
    let ledger = Arc::new(MockAccountValueLedgerPort::default());
    let refunds = Arc::new(MockPaymentRefundExecutorPort::default());
    let payouts = Arc::new(MockPaymentPayoutExecutorPort::default());

    let request = request_view(
        "withdrawal-cash-1",
        None,
        AccountValueOrderSubject::CashWithdrawal,
        AccountValueAssetCode::Cash,
        "8800",
        "CNY",
        "requested",
    )
    .with_provider_amount("8800", "CNY");
    store.seed(request);

    let command = review_command(
        AccountValueOrderSubject::CashWithdrawal,
        "withdrawal-cash-1",
        AccountValueRequestReviewAction::Approve,
    );

    let outcome = execute_account_value_request_review(
        store.as_ref(),
        ledger.as_ref(),
        refunds.as_ref(),
        payouts.as_ref(),
        command,
    )
    .await
    .expect("withdrawal execution");

    assert_eq!(outcome.status, "paid_out");
    assert_eq!(
        outcome.provider_reference_id.as_deref(),
        Some("provider-payout-1")
    );

    let ledger_commands = ledger.commands();
    assert_eq!(ledger_commands.len(), 2);
    assert_eq!(
        ledger_commands[0].operation,
        AccountValueLedgerOperation::Hold
    );
    assert_eq!(ledger_commands[0].asset, AccountValueAssetCode::Cash);
    assert_eq!(ledger_commands[0].amount.as_str(), "8800");
    assert_eq!(
        ledger_commands[0].business_type,
        CommerceLedgerBusinessType::CASH_ADJUSTMENT
    );
    assert_eq!(
        ledger_commands[0].idempotency_key,
        "withdrawal:account-hold:withdrawal-cash-1"
    );
    assert_eq!(
        ledger_commands[1].operation,
        AccountValueLedgerOperation::HoldSettle
    );
    assert_eq!(ledger_commands[1].resource_id, "account-hold-1");
    assert_eq!(
        ledger_commands[1].idempotency_key,
        "withdrawal:account-hold:withdrawal-cash-1:settle"
    );

    let payout_requests = payouts.requests();
    assert_eq!(payout_requests.len(), 1);
    assert_eq!(
        payout_requests[0].withdrawal_request_id,
        "withdrawal-cash-1"
    );
    assert_eq!(payout_requests[0].amount.as_str(), "8800");
    assert_eq!(
        payout_requests[0].idempotency_key,
        "withdrawal:payment-payout:withdrawal-cash-1"
    );
    assert!(refunds.requests().is_empty());

    let statuses = store.status_commands();
    assert_eq!(statuses.len(), 3);
    assert_eq!(statuses[0].status, "account_cash_held");
    assert_eq!(statuses[1].status, "provider_payout_processing");
    assert_eq!(statuses[2].status, "paid_out");
}

#[tokio::test]
async fn reject_refund_request_only_persists_rejected_status() {
    let store = Arc::new(MockAccountValueRequestStore::default());
    let ledger = Arc::new(MockAccountValueLedgerPort::default());
    let refunds = Arc::new(MockPaymentRefundExecutorPort::default());
    let payouts = Arc::new(MockPaymentPayoutExecutorPort::default());

    store.seed(request_view(
        "refund-reject-1",
        Some("order-1"),
        AccountValueOrderSubject::RefundRequest,
        AccountValueAssetCode::Points,
        "100",
        "POINT",
        "requested",
    ));

    let command = review_command(
        AccountValueOrderSubject::RefundRequest,
        "refund-reject-1",
        AccountValueRequestReviewAction::Reject,
    );

    let outcome = execute_account_value_request_review(
        store.as_ref(),
        ledger.as_ref(),
        refunds.as_ref(),
        payouts.as_ref(),
        command,
    )
    .await
    .expect("reject refund");

    assert_eq!(outcome.status, "rejected");
    assert!(ledger.commands().is_empty());
    assert!(refunds.requests().is_empty());
    assert!(payouts.requests().is_empty());
    assert_eq!(store.status_commands().len(), 1);
    assert_eq!(store.status_commands()[0].status, "rejected");
}

#[tokio::test]
async fn refund_provider_failure_releases_account_hold_and_marks_failure() {
    let store = Arc::new(MockAccountValueRequestStore::default());
    let ledger = Arc::new(MockAccountValueLedgerPort::default());
    let refunds = Arc::new(MockPaymentRefundExecutorPort::with_failure());
    let payouts = Arc::new(MockPaymentPayoutExecutorPort::default());

    let request = request_view(
        "refund-fail-1",
        Some("order-fail-1"),
        AccountValueOrderSubject::RefundRequest,
        AccountValueAssetCode::TokenBank,
        "1200",
        "TOKEN_BANK",
        "requested",
    )
    .with_provider_amount("100", "USD");
    store.seed(request);

    let command = review_command(
        AccountValueOrderSubject::RefundRequest,
        "refund-fail-1",
        AccountValueRequestReviewAction::Approve,
    );

    let error = execute_account_value_request_review(
        store.as_ref(),
        ledger.as_ref(),
        refunds.as_ref(),
        payouts.as_ref(),
        command,
    )
    .await
    .expect_err("provider refund failure");

    assert_eq!(error.message(), "provider refund failed");
    let ledger_commands = ledger.commands();
    assert_eq!(ledger_commands.len(), 2);
    assert_eq!(
        ledger_commands[0].operation,
        AccountValueLedgerOperation::Hold
    );
    assert_eq!(
        ledger_commands[1].operation,
        AccountValueLedgerOperation::HoldRelease
    );
    assert_eq!(ledger_commands[1].resource_id, "account-hold-1");
    assert_eq!(
        ledger_commands[1].idempotency_key,
        "refund-request:account-hold:refund-fail-1:release"
    );
    assert_eq!(
        store
            .status_commands()
            .last()
            .expect("failure status")
            .status,
        "provider_refund_failed"
    );
}

#[derive(Default)]
struct MockAccountValueRequestStore {
    requests: Mutex<HashMap<String, AccountValueRequestView>>,
    status_commands: Mutex<Vec<AccountValueRequestStatusCommand>>,
}

impl MockAccountValueRequestStore {
    fn seed(&self, view: AccountValueRequestView) {
        self.requests
            .lock()
            .expect("requests lock")
            .insert(view.request_id.clone(), view);
    }

    fn status_commands(&self) -> Vec<AccountValueRequestStatusCommand> {
        self.status_commands.lock().expect("status lock").clone()
    }
}

impl AccountValueRequestExecutionStore for MockAccountValueRequestStore {
    fn load_account_value_request_for_execution<'a>(
        &'a self,
        command: &'a ReviewAccountValueRequestCommand,
    ) -> AccountValueFuture<'a, Option<AccountValueRequestView>> {
        let request = self
            .requests
            .lock()
            .expect("requests lock")
            .get(&command.request_id)
            .cloned()
            .filter(|request| request.subject == command.subject);
        Box::pin(async move { Ok(request) })
    }

    fn mark_account_value_request_status<'a>(
        &'a self,
        command: AccountValueRequestStatusCommand,
    ) -> AccountValueFuture<'a, AccountValueRequestView> {
        self.status_commands
            .lock()
            .expect("status lock")
            .push(command.clone());
        let mut requests = self.requests.lock().expect("requests lock");
        let request = requests
            .get_mut(&command.request_id)
            .expect("seeded request");
        request.status = command.status.clone();
        if command.provider_reference_id.is_some() {
            request.provider_reference_id = command.provider_reference_id.clone();
        }
        if command.account_effect_reference_id.is_some() {
            request.account_effect_reference_id = command.account_effect_reference_id.clone();
        }
        request.updated_at = "2026-07-08 00:01:00".to_owned();
        let view = request.clone();
        Box::pin(async move { Ok(view) })
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
    ) -> AccountValueFuture<'a, AccountValueLedgerOutcome> {
        self.commands.lock().expect("commands lock").push(command);
        Box::pin(async {
            Ok(AccountValueLedgerOutcome {
                accepted: true,
                replayed: false,
                ledger_entry_id: Some("ledger-1".to_owned()),
                account_effect_reference_id: Some("account-hold-1".to_owned()),
            })
        })
    }
}

#[derive(Default)]
struct MockPaymentRefundExecutorPort {
    requests: Mutex<Vec<PaymentRefundExecutionRequest>>,
    fail: bool,
}

impl MockPaymentRefundExecutorPort {
    fn with_failure() -> Self {
        Self {
            requests: Mutex::new(Vec::new()),
            fail: true,
        }
    }

    fn requests(&self) -> Vec<PaymentRefundExecutionRequest> {
        self.requests.lock().expect("refund lock").clone()
    }
}

impl PaymentRefundExecutorPort for MockPaymentRefundExecutorPort {
    fn execute_provider_refund<'a>(
        &'a self,
        request: PaymentRefundExecutionRequest,
    ) -> AccountValueFuture<'a, PaymentExecutorOutcome> {
        self.requests.lock().expect("refund lock").push(request);
        Box::pin(async move {
            if self.fail {
                return Err(CommerceServiceError::storage("provider refund failed"));
            }
            Ok(PaymentExecutorOutcome {
                accepted: true,
                replayed: false,
                provider_reference_id: Some("provider-refund-1".to_owned()),
                status: "succeeded".to_owned(),
            })
        })
    }
}

#[derive(Default)]
struct MockPaymentPayoutExecutorPort {
    requests: Mutex<Vec<PaymentPayoutExecutionRequest>>,
}

impl MockPaymentPayoutExecutorPort {
    fn requests(&self) -> Vec<PaymentPayoutExecutionRequest> {
        self.requests.lock().expect("payout lock").clone()
    }
}

impl PaymentPayoutExecutorPort for MockPaymentPayoutExecutorPort {
    fn execute_provider_payout<'a>(
        &'a self,
        request: PaymentPayoutExecutionRequest,
    ) -> AccountValueFuture<'a, PaymentExecutorOutcome> {
        self.requests.lock().expect("payout lock").push(request);
        Box::pin(async {
            Ok(PaymentExecutorOutcome {
                accepted: true,
                replayed: false,
                provider_reference_id: Some("provider-payout-1".to_owned()),
                status: "succeeded".to_owned(),
            })
        })
    }
}

fn review_command(
    subject: AccountValueOrderSubject,
    request_id: &str,
    action: AccountValueRequestReviewAction,
) -> ReviewAccountValueRequestCommand {
    ReviewAccountValueRequestCommand::new(
        "tenant-1",
        Some("org-1"),
        subject,
        request_id,
        action,
        Some("business-review"),
        Some("reviewed"),
        "review-request-no",
        &format!("review:{request_id}:{}", action.as_str()),
    )
    .expect("review command")
}

fn request_view(
    request_id: &str,
    original_order_id: Option<&str>,
    subject: AccountValueOrderSubject,
    target_asset: AccountValueAssetCode,
    account_amount: &str,
    account_unit_code: &str,
    status: &str,
) -> AccountValueRequestView {
    AccountValueRequestView::new(
        request_id,
        request_id,
        original_order_id,
        "user-1",
        subject,
        target_asset,
        CommerceMoney::new(account_amount).expect("amount"),
        account_unit_code,
        status,
        None,
        "2026-07-08 00:00:00",
        "2026-07-08 00:00:00",
    )
    .expect("request view")
}

trait RequestProviderAmountExt {
    fn with_provider_amount(self, amount: &str, currency_code: &str) -> Self;
}

impl RequestProviderAmountExt for AccountValueRequestView {
    fn with_provider_amount(mut self, amount: &str, currency_code: &str) -> Self {
        self.provider_amount = Some(CommerceMoney::new(amount).expect("provider amount"));
        self.provider_currency_code = Some(currency_code.to_owned());
        self
    }
}

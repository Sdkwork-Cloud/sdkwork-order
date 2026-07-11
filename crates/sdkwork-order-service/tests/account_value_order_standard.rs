use sdkwork_contract_service::CommerceMoney;
use sdkwork_order_service::{
    account_package_fulfillment_idempotency_key, coupon_recharge_fulfillment_idempotency_key,
    order_service_contract, refund_account_hold_idempotency_key,
    refund_payment_execution_idempotency_key, token_bank_plan_purchase_idempotency_key,
    token_bank_plan_renewal_idempotency_key, token_bank_recharge_fulfillment_idempotency_key,
    withdrawal_account_hold_idempotency_key, withdrawal_payment_execution_idempotency_key,
    AccountValueAssetCode, AccountValueLedgerCommand, AccountValueLedgerPort,
    AccountValueOrderSubject, AccountValueRequestReviewAction, CreateAccountRechargeOrderCommand,
    CreateCashWithdrawalRequestCommand, CreateCouponRechargeOrderCommand,
    CreateOrderRefundRequestCommand, NoopAccountValueLedgerPort, NoopPaymentPayoutExecutorPort,
    OrderSubjectKind, PaymentPayoutExecutionRequest, PaymentPayoutExecutorPort,
    RetireAccountValuePackageCommand, RetireTokenBankPlanCommand, ReviewAccountValueRequestCommand,
    TokenBankPlanPeriod, UpsertAccountValuePackageCommand, UpsertTokenBankPlanCommand,
    ACCOUNT_VALUE_LEDGER_PORT, COUPON_REDEMPTION_PORT, PAYMENT_PAYOUT_EXECUTOR_PORT,
    PAYMENT_REFUND_EXECUTOR_PORT,
};

#[test]
fn account_value_subjects_are_unambiguous_and_map_to_assets() {
    assert_eq!(
        AccountValueOrderSubject::parse("token_bank_recharge").unwrap(),
        AccountValueOrderSubject::TokenBankRecharge
    );
    assert_eq!(
        AccountValueOrderSubject::TokenBankRecharge.fixed_target_asset(),
        Some(AccountValueAssetCode::TokenBank)
    );
    assert_eq!(
        AccountValueOrderSubject::TokenBankPlanPurchase.fixed_target_asset(),
        Some(AccountValueAssetCode::TokenBank)
    );
    assert_eq!(
        AccountValueOrderSubject::CashWithdrawal.fixed_target_asset(),
        Some(AccountValueAssetCode::Cash)
    );
    assert!(AccountValueOrderSubject::RefundRequest
        .fixed_target_asset()
        .is_none());
    assert!(AccountValueOrderSubject::CouponRecharge.payment_collection_is_optional());
    assert!(!AccountValueOrderSubject::RefundRequest.requires_payment_collection());
    assert!(!AccountValueOrderSubject::CashWithdrawal.requires_payment_collection());
}

#[test]
fn order_subject_kind_recognizes_account_value_fulfillment_subjects() {
    assert_eq!(
        OrderSubjectKind::parse(Some("token_bank_recharge")),
        OrderSubjectKind::TokenBankRecharge
    );
    assert_eq!(
        OrderSubjectKind::parse(Some("token_bank_plan_purchase")),
        OrderSubjectKind::TokenBankPlanPurchase
    );
    assert_eq!(
        OrderSubjectKind::parse(Some("token_bank_plan_renewal")),
        OrderSubjectKind::TokenBankPlanRenewal
    );
    assert_eq!(
        OrderSubjectKind::parse(Some("account_recharge_package")),
        OrderSubjectKind::AccountRechargePackage
    );
    assert_eq!(
        OrderSubjectKind::parse(Some("coupon_recharge")),
        OrderSubjectKind::CouponRecharge
    );

    assert!(OrderSubjectKind::TokenBankRecharge.is_fulfillment_implemented());
    assert!(OrderSubjectKind::TokenBankPlanPurchase.is_fulfillment_implemented());
    assert!(OrderSubjectKind::TokenBankPlanRenewal.is_fulfillment_implemented());
    assert!(OrderSubjectKind::AccountRechargePackage.is_fulfillment_implemented());
    assert!(OrderSubjectKind::CouponRecharge.is_fulfillment_implemented());
}

#[test]
fn token_bank_asset_code_rejects_ambiguous_parallel_names() {
    assert_eq!(
        AccountValueAssetCode::parse("token_bank").unwrap(),
        AccountValueAssetCode::TokenBank
    );
    assert_eq!(AccountValueAssetCode::TokenBank.as_str(), "token_bank");
    assert_eq!(AccountValueAssetCode::Points.as_str(), "points");
    assert_eq!(AccountValueAssetCode::Cash.as_str(), "cash");
    assert!(AccountValueAssetCode::parse("token").is_err());
    assert!(AccountValueAssetCode::parse("compute_credit").is_err());
    assert!(AccountValueAssetCode::parse("compute_token").is_err());
}

#[test]
fn token_bank_plan_periods_keep_continuous_plans_explicit() {
    assert_eq!(
        TokenBankPlanPeriod::parse("continuous_monthly").unwrap(),
        TokenBankPlanPeriod::ContinuousMonthly
    );
    assert!(TokenBankPlanPeriod::ContinuousMonthly.is_continuous());
    assert!(TokenBankPlanPeriod::ContinuousYearly.is_continuous());
    assert!(!TokenBankPlanPeriod::Monthly.is_continuous());
    assert_eq!(TokenBankPlanPeriod::Quarterly.as_str(), "quarterly");
    assert!(TokenBankPlanPeriod::parse("month").is_err());
}

#[test]
fn token_bank_recharge_command_requires_token_bank_target_asset() {
    let command = CreateAccountRechargeOrderCommand::new(
        "tenant-1",
        Some("org-1"),
        "user-1",
        AccountValueOrderSubject::TokenBankRecharge,
        AccountValueAssetCode::TokenBank,
        CommerceMoney::new("10000").unwrap(),
        "cny",
        "order-1",
        "item-1",
        "ORD-1",
        "OUT-1",
        "2026-07-08T00:00:00Z",
        "2026-07-08T00:30:00Z",
        "idem-1",
        Some("pkg-token-100"),
        None,
        None,
    )
    .unwrap();

    assert_eq!(command.subject, AccountValueOrderSubject::TokenBankRecharge);
    assert_eq!(command.target_asset, AccountValueAssetCode::TokenBank);
    assert_eq!(command.currency_code, "CNY");
    assert_eq!(command.package_id.as_deref(), Some("pkg-token-100"));

    assert!(CreateAccountRechargeOrderCommand::new(
        "tenant-1",
        Some("org-1"),
        "user-1",
        AccountValueOrderSubject::TokenBankRecharge,
        AccountValueAssetCode::Points,
        CommerceMoney::new("10000").unwrap(),
        "CNY",
        "order-1",
        "item-1",
        "ORD-1",
        "OUT-1",
        "2026-07-08T00:00:00Z",
        "2026-07-08T00:30:00Z",
        "idem-1",
        Some("pkg-token-100"),
        None,
        None,
    )
    .is_err());
}

#[test]
fn token_bank_plan_purchase_requires_plan_period_and_snapshot_identity() {
    let command = CreateAccountRechargeOrderCommand::new(
        "tenant-1",
        None,
        "user-1",
        AccountValueOrderSubject::TokenBankPlanPurchase,
        AccountValueAssetCode::TokenBank,
        CommerceMoney::new("19900").unwrap(),
        "usd",
        "order-plan-1",
        "item-plan-1",
        "ORD-PLAN-1",
        "OUT-PLAN-1",
        "2026-07-08T00:00:00Z",
        "2026-07-08T00:30:00Z",
        "idem-plan-1",
        None,
        Some(("pro_monthly", TokenBankPlanPeriod::ContinuousMonthly)),
        None,
    )
    .unwrap();

    assert_eq!(command.plan_code.as_deref(), Some("pro_monthly"));
    assert_eq!(
        command.plan_period,
        Some(TokenBankPlanPeriod::ContinuousMonthly)
    );
    assert_eq!(command.currency_code, "USD");

    assert!(CreateAccountRechargeOrderCommand::new(
        "tenant-1",
        None,
        "user-1",
        AccountValueOrderSubject::TokenBankPlanPurchase,
        AccountValueAssetCode::TokenBank,
        CommerceMoney::new("19900").unwrap(),
        "USD",
        "order-plan-1",
        "item-plan-1",
        "ORD-PLAN-1",
        "OUT-PLAN-1",
        "2026-07-08T00:00:00Z",
        "2026-07-08T00:30:00Z",
        "idem-plan-1",
        None,
        None,
        None,
    )
    .is_err());
}

#[test]
fn coupon_recharge_command_keeps_coupon_evidence_and_optional_payment() {
    let command = CreateCouponRechargeOrderCommand::new(
        "tenant-1",
        Some("org-1"),
        "user-1",
        AccountValueAssetCode::TokenBank,
        CommerceMoney::new("0").unwrap(),
        "CNY",
        "order-coupon-1",
        "item-coupon-1",
        "ORD-COUPON-1",
        "OUT-COUPON-1",
        "coupon-2026",
        "idem-coupon-1",
        false,
    )
    .unwrap();

    assert_eq!(command.subject, AccountValueOrderSubject::CouponRecharge);
    assert_eq!(command.coupon_code, "coupon-2026");
    assert!(!command.payment_required);
}

#[test]
fn refund_and_withdrawal_commands_keep_money_movement_direction_clear() {
    let refund = CreateOrderRefundRequestCommand::new(
        "tenant-1",
        Some("org-1"),
        "user-1",
        "refund-1",
        "order-paid-1",
        AccountValueAssetCode::TokenBank,
        CommerceMoney::new("2500").unwrap(),
        "CNY",
        "idem-refund-1",
    )
    .unwrap();
    assert_eq!(refund.subject, AccountValueOrderSubject::RefundRequest);
    assert_eq!(refund.original_order_id, "order-paid-1");
    assert_eq!(refund.target_asset, AccountValueAssetCode::TokenBank);

    let withdrawal = CreateCashWithdrawalRequestCommand::new(
        "tenant-1",
        Some("org-1"),
        "user-1",
        "withdrawal-1",
        AccountValueAssetCode::Cash,
        CommerceMoney::new("5000").unwrap(),
        "CNY",
        "idem-withdrawal-1",
    )
    .unwrap();
    assert_eq!(withdrawal.subject, AccountValueOrderSubject::CashWithdrawal);
    assert_eq!(withdrawal.asset, AccountValueAssetCode::Cash);

    assert!(CreateCashWithdrawalRequestCommand::new(
        "tenant-1",
        Some("org-1"),
        "user-1",
        "withdrawal-2",
        AccountValueAssetCode::TokenBank,
        CommerceMoney::new("5000").unwrap(),
        "CNY",
        "idem-withdrawal-2",
    )
    .is_err());
}

#[test]
fn backend_account_value_catalog_commands_normalize_business_inputs() {
    let package = UpsertAccountValuePackageCommand::new(
        "tenant-1",
        Some("org-1"),
        Some("pkg-token-100"),
        " token_bank_100 ",
        " Token Bank 100 ",
        AccountValueAssetCode::TokenBank,
        CommerceMoney::new("100000").unwrap(),
        CommerceMoney::new("5000").unwrap(),
        CommerceMoney::new("9900").unwrap(),
        "usd",
        Some("ACTIVE"),
        Some(10),
        Some("2026-07-08T00:00:00Z"),
        None,
        "req-package-1",
        "idem-package-1",
    )
    .unwrap();
    assert_eq!(package.package_id.as_deref(), Some("pkg-token-100"));
    assert_eq!(package.package_code, "token_bank_100");
    assert_eq!(package.display_name, "Token Bank 100");
    assert_eq!(package.target_asset, AccountValueAssetCode::TokenBank);
    assert_eq!(package.currency_code, "USD");
    assert_eq!(package.status, "active");
    assert_eq!(package.sort_weight, 10);

    let retire = RetireAccountValuePackageCommand::new(
        "tenant-1",
        Some("org-1"),
        "pkg-token-100",
        "req-package-retire-1",
        "idem-package-retire-1",
    )
    .unwrap();
    assert_eq!(retire.package_id, "pkg-token-100");

    assert!(UpsertAccountValuePackageCommand::new(
        "tenant-1",
        None,
        None,
        "",
        "Token Bank 100",
        AccountValueAssetCode::TokenBank,
        CommerceMoney::new("100000").unwrap(),
        CommerceMoney::new("0").unwrap(),
        CommerceMoney::new("9900").unwrap(),
        "USD",
        None,
        None,
        None,
        None,
        "req-package-2",
        "idem-package-2",
    )
    .is_err());
}

#[test]
fn backend_token_bank_plan_commands_keep_plan_identity_explicit() {
    let plan = UpsertTokenBankPlanCommand::new(
        "tenant-1",
        Some("org-1"),
        " pro_monthly ",
        " Pro Monthly ",
        TokenBankPlanPeriod::ContinuousMonthly,
        CommerceMoney::new("200000").unwrap(),
        CommerceMoney::new("10000").unwrap(),
        CommerceMoney::new("19900").unwrap(),
        "cny",
        Some("AUTO_RENEW"),
        Some("ACTIVE"),
        Some(20),
        "req-plan-1",
        "idem-plan-1",
    )
    .unwrap();

    assert_eq!(plan.plan_code, "pro_monthly");
    assert_eq!(plan.plan_period, TokenBankPlanPeriod::ContinuousMonthly);
    assert_eq!(plan.renewal_policy, "auto_renew");
    assert_eq!(plan.currency_code, "CNY");

    let retire = RetireTokenBankPlanCommand::new(
        "tenant-1",
        Some("org-1"),
        "pro_monthly",
        "req-plan-retire-1",
        "idem-plan-retire-1",
    )
    .unwrap();
    assert_eq!(retire.plan_code, "pro_monthly");
}

#[test]
fn backend_account_value_request_review_commands_express_review_action() {
    let approve = ReviewAccountValueRequestCommand::new(
        "tenant-1",
        Some("org-1"),
        AccountValueOrderSubject::RefundRequest,
        "refund-1",
        AccountValueRequestReviewAction::Approve,
        Some("risk_ok"),
        Some("approved by operator"),
        "req-review-1",
        "idem-review-1",
    )
    .unwrap();
    assert_eq!(approve.subject, AccountValueOrderSubject::RefundRequest);
    assert_eq!(approve.action, AccountValueRequestReviewAction::Approve);
    assert_eq!(approve.next_status(), "approved");

    let reject = ReviewAccountValueRequestCommand::new(
        "tenant-1",
        None,
        AccountValueOrderSubject::CashWithdrawal,
        "withdrawal-1",
        AccountValueRequestReviewAction::Reject,
        Some("risk_denied"),
        None,
        "req-review-2",
        "idem-review-2",
    )
    .unwrap();
    assert_eq!(reject.next_status(), "rejected");

    let retry = ReviewAccountValueRequestCommand::new(
        "tenant-1",
        None,
        AccountValueOrderSubject::RefundRequest,
        "refund-1",
        AccountValueRequestReviewAction::Retry,
        None,
        None,
        "req-review-3",
        "idem-review-3",
    )
    .unwrap();
    assert_eq!(retry.next_status(), "processing");

    assert!(ReviewAccountValueRequestCommand::new(
        "tenant-1",
        None,
        AccountValueOrderSubject::TokenBankRecharge,
        "order-1",
        AccountValueRequestReviewAction::Approve,
        None,
        None,
        "req-review-4",
        "idem-review-4",
    )
    .is_err());
}

#[test]
fn account_value_idempotency_scopes_are_stable_and_flow_specific() {
    assert_eq!(
        token_bank_recharge_fulfillment_idempotency_key("order-1"),
        "token-bank-recharge:fulfill:order-1"
    );
    assert_eq!(
        token_bank_plan_purchase_idempotency_key("order-1"),
        "token-bank-plan:purchase:order-1"
    );
    assert_eq!(
        token_bank_plan_renewal_idempotency_key("order-2"),
        "token-bank-plan:renewal:order-2"
    );
    assert_eq!(
        account_package_fulfillment_idempotency_key("order-3"),
        "account-package:fulfill:order-3"
    );
    assert_eq!(
        coupon_recharge_fulfillment_idempotency_key("order-4"),
        "coupon-recharge:fulfill:order-4"
    );
    assert_eq!(
        refund_account_hold_idempotency_key("refund-1"),
        "refund-request:account-hold:refund-1"
    );
    assert_eq!(
        refund_payment_execution_idempotency_key("refund-1"),
        "refund-request:payment-refund:refund-1"
    );
    assert_eq!(
        withdrawal_account_hold_idempotency_key("withdrawal-1"),
        "withdrawal:account-hold:withdrawal-1"
    );
    assert_eq!(
        withdrawal_payment_execution_idempotency_key("withdrawal-1"),
        "withdrawal:payment-payout:withdrawal-1"
    );
}

#[test]
fn account_value_ports_have_distinct_capability_names() {
    assert_eq!(ACCOUNT_VALUE_LEDGER_PORT, "account.value.ledger");
    assert_eq!(PAYMENT_REFUND_EXECUTOR_PORT, "payment.refund.executor");
    assert_eq!(PAYMENT_PAYOUT_EXECUTOR_PORT, "payment.payout.executor");
    assert_eq!(COUPON_REDEMPTION_PORT, "coupon.redemption");
}

#[tokio::test]
async fn noop_payout_executor_reports_provider_payout_boundary() {
    let err = NoopPaymentPayoutExecutorPort
        .execute_provider_payout(PaymentPayoutExecutionRequest {
            tenant_id: "tenant-1".to_owned(),
            organization_id: Some("org-1".to_owned()),
            withdrawal_request_id: "withdrawal-1".to_owned(),
            amount: CommerceMoney::new("8800").unwrap(),
            currency_code: "cny".to_owned(),
            request_no: "request-1".to_owned(),
            idempotency_key: "withdrawal:payment-payout:withdrawal-1".to_owned(),
        })
        .await
        .expect_err("default payout executor must fail closed");

    assert!(
        err.message()
            .contains("provider payout executor port is not configured"),
        "fail-closed withdrawal errors must describe the provider payout executor boundary: {err:?}",
    );
}

#[test]
fn order_service_contract_declares_account_value_operations_and_ports() {
    let contract = order_service_contract();

    for operation_id in [
        "recharges.orders.create",
        "orders.refundRequests.create",
        "withdrawals.requests.create",
        "backend.accountValuePackages.create",
        "backend.accountValuePackages.update",
        "backend.accountValuePackages.retire",
        "backend.tokenBankPlans.create",
        "backend.tokenBankPlans.update",
        "backend.tokenBankPlans.retire",
        "backend.refundRequests.approve",
        "backend.refundRequests.reject",
        "backend.refundRequests.retry",
        "backend.withdrawalRequests.approve",
        "backend.withdrawalRequests.reject",
        "backend.withdrawalRequests.retry",
    ] {
        assert!(
            contract.write_commands.contains(&operation_id),
            "account value write command must be owned by order service: {operation_id}",
        );
    }

    for operation_id in [
        "recharges.plans.list",
        "orders.refundRequests.list",
        "orders.refundRequests.retrieve",
        "withdrawals.requests.retrieve",
        "backend.accountValuePackages.list",
        "backend.tokenBankPlans.list",
        "backend.refundRequests.list",
        "backend.withdrawalRequests.list",
    ] {
        assert!(
            contract.read_queries.contains(&operation_id),
            "account value read query must be owned by order service: {operation_id}",
        );
    }

    for port in [
        ACCOUNT_VALUE_LEDGER_PORT,
        PAYMENT_REFUND_EXECUTOR_PORT,
        PAYMENT_PAYOUT_EXECUTOR_PORT,
        COUPON_REDEMPTION_PORT,
    ] {
        assert!(
            contract.ports.contains(&port),
            "account value port must be declared by order service: {port}",
        );
    }
}

#[test]
fn ledger_command_normalizes_business_facts_before_calling_account() {
    let command = AccountValueLedgerCommand::credit(
        "tenant-1",
        Some("org-1"),
        "user-1",
        AccountValueAssetCode::TokenBank,
        CommerceMoney::new("30000").unwrap(),
        "usd",
        "token_bank_recharge",
        "order-1",
        "req-1",
        "idem-ledger-1",
    )
    .unwrap();

    assert_eq!(command.asset, AccountValueAssetCode::TokenBank);
    assert_eq!(command.direction.as_str(), "credit");
    assert_eq!(command.currency_code, "USD");
    assert_eq!(command.business_type, "token_bank_recharge");
}

#[tokio::test]
async fn noop_account_value_ledger_port_fails_closed_when_unconfigured() {
    let command = AccountValueLedgerCommand::credit(
        "tenant-1",
        Some("org-1"),
        "user-1",
        AccountValueAssetCode::TokenBank,
        CommerceMoney::new("30000").unwrap(),
        "TOKEN_BANK",
        "token_bank_recharge",
        "order-1",
        "req-1",
        "idem-ledger-1",
    )
    .unwrap();

    let error = NoopAccountValueLedgerPort
        .apply_account_value_ledger_command(command)
        .await
        .expect_err("unconfigured account value ledger must fail closed");

    assert_eq!(error.code(), "unsupported-capability");
    assert!(error.message().contains("account value ledger port"));
}

#[test]
fn account_value_catalog_and_request_views_use_clear_business_terms() {
    let package = sdkwork_order_service::AccountValuePackageItem::new(
        "pkg-token-100",
        "token_bank_100",
        "Token Bank 100",
        AccountValueAssetCode::TokenBank,
        CommerceMoney::new("100").unwrap(),
        CommerceMoney::new("10").unwrap(),
        CommerceMoney::new("990").unwrap(),
        "cny",
        "active",
    )
    .unwrap();
    assert_eq!(package.target_asset, AccountValueAssetCode::TokenBank);
    assert_eq!(package.currency_code, "CNY");

    let plan = sdkwork_order_service::TokenBankPlanItem::new(
        "pro_monthly",
        "Pro Monthly",
        TokenBankPlanPeriod::ContinuousMonthly,
        CommerceMoney::new("100000").unwrap(),
        CommerceMoney::new("5000").unwrap(),
        CommerceMoney::new("19900").unwrap(),
        "usd",
        "auto_renew",
        "active",
    )
    .unwrap();
    assert_eq!(plan.plan_period, TokenBankPlanPeriod::ContinuousMonthly);
    assert_eq!(plan.currency_code, "USD");

    let refund = sdkwork_order_service::AccountValueRequestView::new(
        "refund-1",
        "RF-1",
        Some("order-1"),
        "user-1",
        AccountValueOrderSubject::RefundRequest,
        AccountValueAssetCode::TokenBank,
        CommerceMoney::new("100").unwrap(),
        "TOKEN_BANK",
        "requested",
        None,
        "2026-07-08T00:00:00Z",
        "2026-07-08T00:00:00Z",
    )
    .unwrap();
    assert_eq!(refund.subject, AccountValueOrderSubject::RefundRequest);
    assert_eq!(refund.target_asset, AccountValueAssetCode::TokenBank);
}

#[test]
fn account_value_list_queries_use_standard_offset_pagination() {
    let package_query = sdkwork_order_service::AccountValueCatalogListQuery::new(
        "tenant-1",
        Some("org-1"),
        Some(AccountValueAssetCode::TokenBank),
        Some("active"),
        Some(2),
        Some(50),
    )
    .unwrap();
    assert_eq!(package_query.limit(), 50);
    assert_eq!(package_query.offset(), 50);

    let request_query = sdkwork_order_service::AccountValueRequestListQuery::new(
        "tenant-1",
        Some("org-1"),
        Some("user-1"),
        Some(AccountValueOrderSubject::RefundRequest),
        Some("requested"),
        Some(1),
        Some(20),
    )
    .unwrap();
    assert_eq!(request_query.limit(), 20);
    assert_eq!(request_query.offset(), 0);
    assert!(sdkwork_order_service::AccountValueCatalogListQuery::new(
        "tenant-1",
        None,
        None,
        None,
        Some(1),
        Some(201),
    )
    .is_err());
}

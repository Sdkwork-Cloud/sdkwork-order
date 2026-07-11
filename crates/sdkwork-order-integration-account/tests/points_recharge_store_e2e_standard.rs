//! Store-layer E2E: payment success → fulfill → account ledger credit (in-process adapters).

use sdkwork_account_repository_sqlx::{
    account_migrated_sqlite_memory_pool, SqliteCommerceAccountStore,
};
use sdkwork_account_service::WalletAccountListQuery;
use sdkwork_contract_service::{
    CommerceAccountAssetType, CommerceLedgerBusinessType, CommerceLedgerDirection, CommerceMoney,
};
use sdkwork_order_integration_account::StoreAccountPointsCreditAdapter;
use sdkwork_order_repository_sqlx::{
    order_points_recharge_e2e_sqlite_memory_pool, SqliteCommerceRechargeStore,
};
use sdkwork_order_service::{
    default_fulfill_points_recharge_command, fulfill_points_recharge_order,
    mark_points_recharge_payment_succeeded, points_recharge_payment_success_idempotency_key,
    AccountValueAssetCode, AccountValueLedgerCommand, AccountValueLedgerPort,
    MarkPointsRechargePaymentSucceededCommand,
};

const TENANT_ID: &str = "100001";
const ORGANIZATION_ID: &str = "0";
const OWNER_USER_ID: &str = "1";
const ORDER_ID: &str = "order-e2e-1";
const ORDER_NO: &str = "ORD-E2E-1";
const POINTS: i64 = 500;
const PAID_AT: &str = "2026-06-29 12:00:00";
const REQUESTED_AT: &str = "2026-06-29 11:00:00";

#[tokio::test]
async fn points_recharge_store_e2e_payment_success_fulfill_credits_ledger() {
    let order_pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    let account_pool = account_migrated_sqlite_memory_pool().await;

    seed_pending_points_recharge_checkout(&order_pool).await;

    let recharge_store = SqliteCommerceRechargeStore::new(order_pool.clone());
    let credit_adapter = StoreAccountPointsCreditAdapter::sqlite(account_pool.clone());
    let account_store = SqliteCommerceAccountStore::new(account_pool);

    mark_points_recharge_payment_succeeded(
        &recharge_store,
        MarkPointsRechargePaymentSucceededCommand::new(
            TENANT_ID,
            Some(ORGANIZATION_ID),
            OWNER_USER_ID,
            ORDER_ID,
            PAID_AT,
            "req-pay-success-e2e",
            &points_recharge_payment_success_idempotency_key(ORDER_ID),
        )
        .expect("payment success command"),
    )
    .await
    .expect("mark payment succeeded");

    let fulfill_command = default_fulfill_points_recharge_command(
        TENANT_ID,
        Some(ORGANIZATION_ID),
        OWNER_USER_ID,
        ORDER_ID,
        "req-fulfill-e2e",
    )
    .expect("fulfill command");

    let outcome = fulfill_points_recharge_order(&recharge_store, &credit_adapter, fulfill_command)
        .await
        .expect("fulfill points recharge");

    assert!(outcome.accepted);
    assert!(!outcome.replayed);
    assert_eq!(outcome.points_credited, POINTS);
    assert_eq!(outcome.fulfillment_status, "fulfilled");

    let accounts = account_store
        .list_wallet_accounts(
            WalletAccountListQuery::new(
                TENANT_ID,
                Some(ORGANIZATION_ID),
                OWNER_USER_ID,
                Some(CommerceAccountAssetType::Points),
            )
            .expect("wallet query"),
        )
        .await
        .expect("wallet accounts");

    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].available_amount.as_str(), POINTS.to_string());

    let fulfillment_status: String = sqlx::query_scalar(
        r#"
        SELECT fulfillment_status
        FROM commerce_order
        WHERE id = ?
        "#,
    )
    .bind(ORDER_ID)
    .fetch_one(&order_pool)
    .await
    .expect("order fulfillment status");

    assert_eq!(fulfillment_status, "fulfilled");

    let replay_command = default_fulfill_points_recharge_command(
        TENANT_ID,
        Some(ORGANIZATION_ID),
        OWNER_USER_ID,
        ORDER_ID,
        "req-fulfill-e2e-replay",
    )
    .expect("replay fulfill command");

    let replay_outcome =
        fulfill_points_recharge_order(&recharge_store, &credit_adapter, replay_command)
            .await
            .expect("replay fulfill");

    assert!(replay_outcome.replayed);

    let accounts_after_replay = account_store
        .list_wallet_accounts(
            WalletAccountListQuery::new(
                TENANT_ID,
                Some(ORGANIZATION_ID),
                OWNER_USER_ID,
                Some(CommerceAccountAssetType::Points),
            )
            .expect("wallet query"),
        )
        .await
        .expect("wallet accounts after replay");

    assert_eq!(
        accounts_after_replay[0].available_amount.as_str(),
        POINTS.to_string()
    );
}

#[tokio::test]
async fn token_bank_refund_store_e2e_holds_and_settles_account_value() {
    let account_pool = account_migrated_sqlite_memory_pool().await;
    let adapter = StoreAccountPointsCreditAdapter::sqlite(account_pool.clone());
    let account_store = SqliteCommerceAccountStore::new(account_pool.clone());

    adapter
        .apply_account_value_ledger_command(
            AccountValueLedgerCommand::credit(
                TENANT_ID,
                Some(ORGANIZATION_ID),
                OWNER_USER_ID,
                AccountValueAssetCode::TokenBank,
                CommerceMoney::new("32000").expect("credit amount"),
                "TOKEN_BANK",
                CommerceLedgerBusinessType::TOKEN_BANK_PURCHASE_CREDIT,
                "order-token-bank-1",
                "req-token-bank-credit-1",
                "idem-token-bank-credit-1",
            )
            .expect("credit command"),
        )
        .await
        .expect("credit token bank");

    let hold = adapter
        .apply_account_value_ledger_command(
            AccountValueLedgerCommand::hold(
                TENANT_ID,
                Some(ORGANIZATION_ID),
                OWNER_USER_ID,
                AccountValueAssetCode::TokenBank,
                CommerceMoney::new("12000").expect("hold amount"),
                "TOKEN_BANK",
                CommerceLedgerBusinessType::TOKEN_BANK_HOLD,
                "refund-request-store-1",
                "req-token-bank-hold-1",
                "idem-token-bank-hold-1",
            )
            .expect("hold command"),
        )
        .await
        .expect("hold token bank");

    assert!(hold.accepted);
    assert_eq!(
        hold.account_effect_reference_id.as_deref().unwrap().len(),
        36
    );

    let accounts_after_hold = account_store
        .list_wallet_accounts(
            WalletAccountListQuery::new(
                TENANT_ID,
                Some(ORGANIZATION_ID),
                OWNER_USER_ID,
                Some(CommerceAccountAssetType::TokenBank),
            )
            .expect("wallet query"),
        )
        .await
        .expect("wallet accounts after hold");
    assert_eq!(accounts_after_hold[0].available_amount.as_str(), "20000");
    assert_eq!(accounts_after_hold[0].frozen_amount.as_str(), "12000");

    let settle = adapter
        .apply_account_value_ledger_command(
            AccountValueLedgerCommand::hold_settle(
                TENANT_ID,
                Some(ORGANIZATION_ID),
                OWNER_USER_ID,
                AccountValueAssetCode::TokenBank,
                CommerceMoney::new("12000").expect("settle amount"),
                "TOKEN_BANK",
                CommerceLedgerBusinessType::TOKEN_BANK_REVERSAL,
                hold.account_effect_reference_id.as_deref().unwrap(),
                "req-token-bank-settle-1",
                "idem-token-bank-settle-1",
            )
            .expect("settle command"),
        )
        .await
        .expect("settle token bank hold");

    assert!(settle.ledger_entry_id.is_some());
    assert_eq!(
        settle.account_effect_reference_id.as_deref(),
        hold.account_effect_reference_id.as_deref()
    );

    let accounts_after_settle = account_store
        .list_wallet_accounts(
            WalletAccountListQuery::new(
                TENANT_ID,
                Some(ORGANIZATION_ID),
                OWNER_USER_ID,
                Some(CommerceAccountAssetType::TokenBank),
            )
            .expect("wallet query"),
        )
        .await
        .expect("wallet accounts after settle");
    assert_eq!(accounts_after_settle[0].available_amount.as_str(), "20000");
    assert_eq!(accounts_after_settle[0].frozen_amount.as_str(), "0");

    let hold_status: i64 = sqlx::query_scalar(
        r#"
        SELECT status
        FROM acct_hold
        WHERE uuid = ?
        "#,
    )
    .bind(hold.account_effect_reference_id.as_deref().unwrap())
    .fetch_one(&account_pool)
    .await
    .expect("hold status");
    assert_eq!(hold_status, 2);

    let debit_direction: String = sqlx::query_scalar(
        r#"
        SELECT direction
        FROM acct_ledger_entry
        WHERE idempotency_key = ?
        "#,
    )
    .bind("idem-token-bank-settle-1")
    .fetch_one(&account_pool)
    .await
    .expect("settlement ledger direction");
    assert_eq!(debit_direction, CommerceLedgerDirection::Debit.as_str());
}

async fn seed_pending_points_recharge_checkout(pool: &sqlx::SqlitePool) {
    let sku_snapshot = serde_json::json!({
        "skuId": "sku-points-500",
        "productName": "500 Points",
        "points": POINTS,
    })
    .to_string();

    sqlx::query(
        r#"
        INSERT INTO commerce_order
            (id, tenant_id, organization_id, owner_user_id, order_no, status, subject,
             currency_code, payment_status, fulfillment_status, request_no, idempotency_key,
             created_at, paid_at, cancelled_at, expired_at, updated_at)
        VALUES
            (?, ?, ?, ?, ?, 'pending_payment', 'points_recharge', 'CNY', 'pending', 'unfulfilled',
             ?, ?, ?, NULL, NULL, NULL, ?)
        "#,
    )
    .bind(ORDER_ID)
    .bind(TENANT_ID)
    .bind(ORGANIZATION_ID)
    .bind(OWNER_USER_ID)
    .bind(ORDER_NO)
    .bind(ORDER_NO)
    .bind(format!("idem-{ORDER_ID}"))
    .bind(REQUESTED_AT)
    .bind(REQUESTED_AT)
    .execute(pool)
    .await
    .expect("seed order");

    sqlx::query(
        r#"
        INSERT INTO commerce_order_item
            (id, tenant_id, order_id, sku_id, sku_snapshot_json, title, quantity,
             unit_price_amount, total_amount, fulfillment_status, refund_status, created_at)
        VALUES
            (?, ?, ?, 'sku-points-500', ?, '500 Points', 1, '50.00', '50.00', 'unfulfilled', 'none', ?)
        "#,
    )
    .bind(format!("{ORDER_ID}-item"))
    .bind(TENANT_ID)
    .bind(ORDER_ID)
    .bind(&sku_snapshot)
    .bind(REQUESTED_AT)
    .execute(pool)
    .await
    .expect("seed order item");

    sqlx::query(
        r#"
        INSERT INTO commerce_payment_intent
            (id, tenant_id, organization_id, owner_user_id, order_id, status, amount,
             currency_code, created_at, updated_at)
        VALUES
            (?, ?, ?, ?, ?, 'pending', '50.00', 'CNY', ?, ?)
        "#,
    )
    .bind(format!("{ORDER_ID}-pi"))
    .bind(TENANT_ID)
    .bind(ORGANIZATION_ID)
    .bind(OWNER_USER_ID)
    .bind(ORDER_ID)
    .bind(REQUESTED_AT)
    .bind(REQUESTED_AT)
    .execute(pool)
    .await
    .expect("seed payment intent");

    sqlx::query(
        r#"
        INSERT INTO commerce_payment_attempt
            (id, tenant_id, organization_id, owner_user_id, order_id, status, amount,
             currency_code, paid_at, callback_payload, created_at, updated_at)
        VALUES
            (?, ?, ?, ?, ?, 'processing', '50.00', 'CNY', NULL, NULL, ?, ?)
        "#,
    )
    .bind(format!("{ORDER_ID}-pa"))
    .bind(TENANT_ID)
    .bind(ORGANIZATION_ID)
    .bind(OWNER_USER_ID)
    .bind(ORDER_ID)
    .bind(REQUESTED_AT)
    .bind(REQUESTED_AT)
    .execute(pool)
    .await
    .expect("seed payment attempt");
}

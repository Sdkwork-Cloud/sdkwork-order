use sdkwork_contract_service::CommerceMoney;
use sdkwork_order_repository_sqlx::{
    order_points_recharge_e2e_sqlite_memory_pool, SqliteCommerceRechargeStore,
};
use sdkwork_order_service::CreatePointsRechargeOrderCommand;
use sqlx::SqlitePool;

const TENANT_ID: &str = "100001";
const ORGANIZATION_ID: &str = "0";
const OWNER_USER_ID: &str = "recharge-expiration-owner";
const PACKAGE_ID: &str = "recharge-expiration-package";

#[tokio::test]
async fn sqlite_recharge_reuses_only_live_orders_and_replaces_expired_orders() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    seed_recharge_package(&pool).await;
    let store = SqliteCommerceRechargeStore::new(pool.clone());

    let first = store
        .create_points_recharge_order(command(
            "first",
            "2026-07-27T04:00:00.000Z",
            "2026-07-27T04:00:02.000Z",
            "recharge-expiration-first",
        ))
        .await
        .expect("create first recharge order");
    seed_pending_payment(&pool, &first.order_id).await;

    let live_replay = store
        .create_points_recharge_order(command(
            "live-replay",
            "2026-07-27T04:00:01.000Z",
            "2026-07-27T04:30:01.000Z",
            "recharge-expiration-live-replay",
        ))
        .await
        .expect("reuse live recharge order");
    assert_eq!(first.order_id, live_replay.order_id);
    assert_eq!(first.expires_at, live_replay.expires_at);

    let replacement = store
        .create_points_recharge_order(command(
            "replacement",
            "2026-07-27T04:00:03.000Z",
            "2026-07-27T04:30:03.000Z",
            "recharge-expiration-replacement",
        ))
        .await
        .expect("replace expired recharge order");
    assert_ne!(first.order_id, replacement.order_id);
    assert_eq!("2026-07-27T04:30:03.000Z", replacement.expires_at);

    let old_status: String = sqlx::query_scalar("SELECT status FROM commerce_order WHERE id = ?")
        .bind(&first.order_id)
        .fetch_one(&pool)
        .await
        .expect("expired recharge order status");
    assert_eq!("expired", old_status);

    let order_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM commerce_order WHERE tenant_id = ? AND owner_user_id = ? AND subject = 'points_recharge'",
    )
    .bind(TENANT_ID)
    .bind(OWNER_USER_ID)
    .fetch_one(&pool)
    .await
    .expect("recharge order count");
    assert_eq!(2, order_count);
}

fn command(
    suffix: &str,
    requested_at: &str,
    expire_at: &str,
    idempotency_key: &str,
) -> CreatePointsRechargeOrderCommand {
    CreatePointsRechargeOrderCommand::new(
        TENANT_ID,
        Some(ORGANIZATION_ID),
        OWNER_USER_ID,
        CommerceMoney::new("5000").expect("recharge amount"),
        "CNY",
        "wechat_pay",
        &format!("recharge-order-{suffix}"),
        &format!("recharge-order-item-{suffix}"),
        &format!("recharge-payment-intent-{suffix}"),
        &format!("recharge-payment-attempt-{suffix}"),
        &format!("RC{suffix}"),
        &format!("RECHARGE{suffix}"),
        requested_at,
        expire_at,
        idempotency_key,
        Some(PACKAGE_ID),
        None,
        Some("token-plan"),
    )
    .expect("recharge command")
}

async fn seed_recharge_package(pool: &SqlitePool) {
    for statement in [
        "ALTER TABLE commerce_payment_intent ADD COLUMN provider_code TEXT",
        "ALTER TABLE commerce_payment_attempt ADD COLUMN out_trade_no TEXT",
        "ALTER TABLE commerce_payment_attempt ADD COLUMN provider_code TEXT",
    ] {
        sqlx::query(statement)
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("recharge payment test column failed: {error}"));
    }
    sqlx::query(
        r#"
        CREATE TABLE commerce_recharge_package (
            id TEXT NOT NULL PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            organization_id TEXT,
            name TEXT NOT NULL,
            price_amount TEXT NOT NULL,
            currency_code TEXT NOT NULL,
            bonus_points INTEGER NOT NULL,
            sku_id TEXT NOT NULL,
            status TEXT NOT NULL,
            valid_from TEXT,
            valid_to TEXT,
            sort_weight INTEGER NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("recharge package table");
    sqlx::query(
        r#"
        INSERT INTO commerce_recharge_package (
            id, tenant_id, organization_id, name, price_amount, currency_code,
            bonus_points, sku_id, status, valid_from, valid_to, sort_weight
        ) VALUES (?, ?, ?, 'Expiration package', '50.00', 'CNY', 0, 'recharge-sku', 'active', NULL, NULL, 1)
        "#,
    )
    .bind(PACKAGE_ID)
    .bind(TENANT_ID)
    .bind(ORGANIZATION_ID)
    .execute(pool)
    .await
    .expect("recharge package");
}

async fn seed_pending_payment(pool: &SqlitePool, order_id: &str) {
    sqlx::query(
        r#"
        INSERT INTO commerce_payment_intent (
            id, tenant_id, organization_id, owner_user_id, order_id, status,
            amount, currency_code, payment_method, created_at, updated_at
        ) VALUES ('recharge-intent-live', ?, ?, ?, ?, 'pending', '5000', 'CNY', 'wechat_pay',
                  '2026-07-27T04:00:00.000Z', '2026-07-27T04:00:00.000Z')
        "#,
    )
    .bind(TENANT_ID)
    .bind(ORGANIZATION_ID)
    .bind(OWNER_USER_ID)
    .bind(order_id)
    .execute(pool)
    .await
    .expect("pending payment intent");
    sqlx::query(
        r#"
        INSERT INTO commerce_payment_attempt (
            id, tenant_id, organization_id, owner_user_id, order_id, status,
            amount, currency_code, payment_method, callback_payload, created_at, updated_at
        ) VALUES ('recharge-attempt-live', ?, ?, ?, ?, 'pending', '5000', 'CNY', 'wechat_pay',
                  ?, '2026-07-27T04:00:00.000Z', '2026-07-27T04:00:00.000Z')
        "#,
    )
    .bind(TENANT_ID)
    .bind(ORGANIZATION_ID)
    .bind(OWNER_USER_ID)
    .bind(order_id)
    .bind(format!(r#"{{"points":500,"packageId":"{PACKAGE_ID}"}}"#))
    .execute(pool)
    .await
    .expect("pending payment attempt");
}

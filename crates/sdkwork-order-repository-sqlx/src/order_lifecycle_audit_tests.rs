use sdkwork_order_service::CancelOwnerOrderCommand;

use crate::test_sqlite_pool::{
    order_points_recharge_e2e_postgres_pool_from_env, order_points_recharge_e2e_sqlite_memory_pool,
};
use crate::{PostgresCommerceOrderStore, SqliteCommerceOrderStore};

#[tokio::test]
async fn sqlite_cancel_owner_order_writes_event_and_cancellation_audit_rows() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    seed_pending_order_sqlite(&pool).await;

    let store = SqliteCommerceOrderStore::new(pool.clone());
    let command = sample_cancel_command();
    store
        .cancel_owner_order(command)
        .await
        .expect("cancel order");

    assert_cancel_audit_sqlite(&pool, "changed mind").await;
}

#[tokio::test]
async fn postgres_cancel_owner_order_writes_event_and_cancellation_audit_rows() {
    let Some(pool) = order_points_recharge_e2e_postgres_pool_from_env().await else {
        eprintln!("ORDER_TEST_POSTGRES_URL is unset; skipping postgres cancel audit parity test");
        return;
    };
    seed_pending_order_postgres(&pool).await;

    let store = PostgresCommerceOrderStore::new(pool.clone());
    store
        .cancel_owner_order(sample_cancel_command())
        .await
        .expect("cancel order");

    assert_cancel_audit_postgres(&pool, "changed mind").await;
}

fn sample_cancel_command() -> CancelOwnerOrderCommand {
    CancelOwnerOrderCommand {
        tenant_id: "tenant-1".to_owned(),
        organization_id: None,
        owner_user_id: "user-1".to_owned(),
        order_id: "order-1".to_owned(),
        cancel_reason: Some("changed mind".to_owned()),
        cancel_type: Some("user_cancel".to_owned()),
    }
}

async fn seed_pending_order_sqlite(pool: &sqlx::SqlitePool) {
    let now = "2026-07-05T00:00:00Z";
    sqlx::query(
        r#"
        INSERT INTO commerce_order
            (id, tenant_id, organization_id, owner_user_id, order_no, status, subject,
             currency_code, payment_status, fulfillment_status, created_at, updated_at)
        VALUES
            (?, ?, NULL, ?, ?, 'pending_payment', 'general', 'CNY', 'pending', 'unfulfilled', ?, ?)
        "#,
    )
    .bind("order-1")
    .bind("tenant-1")
    .bind("user-1")
    .bind("SN-001")
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .expect("seed order");
}

async fn seed_pending_order_postgres(pool: &sqlx::PgPool) {
    let now = "2026-07-05T00:00:00Z";
    sqlx::query(
        r#"
        INSERT INTO commerce_order
            (id, tenant_id, organization_id, owner_user_id, order_no, status, subject,
             currency_code, payment_status, fulfillment_status, created_at, updated_at)
        VALUES
            ($1, $2, NULL, $3, $4, 'pending_payment', 'general', 'CNY', 'pending', 'unfulfilled', $5, $5)
        "#,
    )
    .bind("order-1")
    .bind("tenant-1")
    .bind("user-1")
    .bind("SN-001")
    .bind(now)
    .execute(pool)
    .await
    .expect("seed order");
}

async fn assert_cancel_audit_sqlite(pool: &sqlx::SqlitePool, reason: &str) {
    let event_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM commerce_order_event WHERE tenant_id = ? AND order_id = ? AND event_type = 'cancelled'",
    )
    .bind("tenant-1")
    .bind("order-1")
    .fetch_one(pool)
    .await
    .expect("count events");
    assert_eq!(event_count, 1);

    let cancellation_reason: String = sqlx::query_scalar(
        "SELECT reason_message FROM commerce_order_cancellation WHERE tenant_id = ? AND order_id = ?",
    )
    .bind("tenant-1")
    .bind("order-1")
    .fetch_one(pool)
    .await
    .expect("cancellation reason");
    assert_eq!(cancellation_reason, reason);
}

async fn assert_cancel_audit_postgres(pool: &sqlx::PgPool, reason: &str) {
    let event_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM commerce_order_event WHERE tenant_id = $1 AND order_id = $2 AND event_type = 'cancelled'",
    )
    .bind("tenant-1")
    .bind("order-1")
    .fetch_one(pool)
    .await
    .expect("count events");
    assert_eq!(event_count, 1);

    let cancellation_reason: String = sqlx::query_scalar(
        "SELECT reason_message FROM commerce_order_cancellation WHERE tenant_id = $1 AND order_id = $2",
    )
    .bind("tenant-1")
    .bind("order-1")
    .fetch_one(pool)
    .await
    .expect("cancellation reason");
    assert_eq!(cancellation_reason, reason);
}

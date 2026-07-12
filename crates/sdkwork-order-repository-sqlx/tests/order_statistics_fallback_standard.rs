use sdkwork_order_repository_sqlx::{
    order_points_recharge_e2e_sqlite_memory_pool, SqliteCommerceOrderStore,
};
use sqlx::SqlitePool;

#[tokio::test]
async fn sqlite_missing_order_read_model_returns_zero_statistics_without_panicking() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("sqlite memory pool");
    let store = SqliteCommerceOrderStore::new(pool);

    let statistics = store
        .retrieve_owner_order_statistics("tenant-1", None, "owner-1")
        .await
        .expect("missing order read model should use the empty statistics fallback");

    assert_eq!(statistics.total_orders, 0);
    assert_eq!(statistics.pending_payment, 0);
    assert_eq!(statistics.pending_shipment, 0);
    assert_eq!(statistics.pending_receipt, 0);
    assert_eq!(statistics.completed, 0);
    assert_eq!(statistics.total_amount.as_str(), "0");
}

#[tokio::test]
async fn sqlite_empty_order_read_model_returns_zero_statistics() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    let store = SqliteCommerceOrderStore::new(pool);

    let statistics = store
        .retrieve_owner_order_statistics("tenant-1", None, "owner-1")
        .await
        .expect("an empty order read model should return zero statistics");

    assert_eq!(statistics.total_orders, 0);
    assert_eq!(statistics.total_amount.as_str(), "0");
}

#[tokio::test]
async fn sqlite_order_statistics_preserve_integer_money_units() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    sqlx::query(
        "INSERT INTO commerce_order (id, tenant_id, owner_user_id, order_no, subject, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("order-statistics-1")
    .bind("tenant-1")
    .bind("owner-1")
    .bind("order-no-1")
    .bind("subject")
    .bind("2026-07-12T00:00:00Z")
    .bind("2026-07-12T00:00:00Z")
    .execute(&pool)
    .await
    .expect("insert order");
    sqlx::query(
        "INSERT INTO commerce_order_amount_breakdown (id, tenant_id, order_id, allocation_type, payable_amount, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind("order-statistics-amount-1")
    .bind("tenant-1")
    .bind("order-statistics-1")
    .bind("order_total")
    .bind("1234")
    .bind("2026-07-12T00:00:00Z")
    .execute(&pool)
    .await
    .expect("insert order amount");

    let statistics = SqliteCommerceOrderStore::new(pool)
        .retrieve_owner_order_statistics("tenant-1", None, "owner-1")
        .await
        .expect("integer money units should remain valid");

    assert_eq!(statistics.total_orders, 1);
    assert_eq!(statistics.total_amount.as_str(), "1234");
}

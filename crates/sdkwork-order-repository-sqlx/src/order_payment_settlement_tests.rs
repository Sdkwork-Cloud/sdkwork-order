use sdkwork_order_service::{OrderOwnerListQuery, OrderPaymentSettlementAttempt};
use sqlx::Row;
use uuid::Uuid;

use crate::test_sqlite_pool::{
    order_points_recharge_e2e_postgres_pool_from_env, order_points_recharge_e2e_sqlite_memory_pool,
};
use crate::{PostgresCommerceOrderStore, SqliteCommerceOrderStore};

#[tokio::test]
async fn sqlite_payment_success_marks_order_paid_and_is_idempotent() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    let ids = seed_payment_order_sqlite(&pool).await;
    let store = SqliteCommerceOrderStore::new(pool.clone());
    let attempt = payment_attempt(&ids.0, &ids.1);

    let context = store
        .load_order_payment_settlement_context("tenant-payment", None, &ids.0)
        .await
        .expect("load settlement context")
        .expect("settlement context");
    assert_eq!(context.subject, "notary");

    store
        .mark_owner_order_payment_succeeded(&attempt, "2026-07-11T01:02:03Z")
        .await
        .expect("mark payment success");
    store
        .mark_owner_order_payment_succeeded(&attempt, "2026-07-11T04:05:06Z")
        .await
        .expect("replay payment success");

    assert_order_paid_sqlite(&pool, &ids.0, "2026-07-11T01:02:03Z").await;
    let query = owner_list_query(&ids.1);
    let page = store
        .list_owner_orders(query)
        .await
        .expect("list paid order");
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].status, "paid");
    assert_eq!(page.items[0].paid_amount.as_ref().unwrap().as_str(), "8800");
}

#[tokio::test]
async fn postgres_payment_success_marks_order_paid_and_is_idempotent() {
    let Some(pool) = order_points_recharge_e2e_postgres_pool_from_env().await else {
        eprintln!("ORDER_TEST_POSTGRES_URL is unset; skipping postgres payment settlement test");
        return;
    };
    let ids = seed_payment_order_postgres(&pool).await;
    let store = PostgresCommerceOrderStore::new(pool.clone());
    let attempt = payment_attempt(&ids.0, &ids.1);

    let context = store
        .load_order_payment_settlement_context("tenant-payment", None, &ids.0)
        .await
        .expect("load settlement context")
        .expect("settlement context");
    assert_eq!(context.subject, "notary");

    store
        .mark_owner_order_payment_succeeded(&attempt, "2026-07-11T01:02:03Z")
        .await
        .expect("mark payment success");
    store
        .mark_owner_order_payment_succeeded(&attempt, "2026-07-11T04:05:06Z")
        .await
        .expect("replay payment success");

    assert_order_paid_postgres(&pool, &ids.0, "2026-07-11T01:02:03Z").await;
    let query = owner_list_query(&ids.1);
    let page = store
        .list_owner_orders(query)
        .await
        .expect("list paid order");
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].status, "paid");
    assert_eq!(page.items[0].paid_amount.as_ref().unwrap().as_str(), "8800");
}

#[tokio::test]
async fn sqlite_late_payment_preserves_terminal_status_and_is_idempotent() {
    for terminal_status in ["cancelled", "closed", "expired"] {
        let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
        let ids = seed_payment_order_sqlite(&pool).await;
        set_terminal_order_sqlite(&pool, &ids.0, terminal_status).await;
        let store = SqliteCommerceOrderStore::new(pool.clone());
        let attempt = payment_attempt(&ids.0, &ids.1);

        store
            .mark_owner_order_payment_succeeded(&attempt, "2026-07-11T01:02:03Z")
            .await
            .expect("record late payment success");
        store
            .mark_owner_order_payment_succeeded(&attempt, "2026-07-11T04:05:06Z")
            .await
            .expect("replay late payment success");

        assert_terminal_payment_sqlite(&pool, &ids.0, terminal_status, "2026-07-11T01:02:03Z")
            .await;
        let page = store
            .list_owner_orders(owner_list_query(&ids.1))
            .await
            .expect("list terminal paid order");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].status, terminal_status);
        assert_eq!(page.items[0].paid_amount.as_ref().unwrap().as_str(), "8800");
        assert_eq!(
            page.items[0].pay_time.as_deref(),
            Some("2026-07-11T01:02:03Z")
        );
    }
}

#[tokio::test]
async fn postgres_late_payment_preserves_terminal_status_and_is_idempotent() {
    let Some(pool) = order_points_recharge_e2e_postgres_pool_from_env().await else {
        eprintln!("ORDER_TEST_POSTGRES_URL is unset; skipping postgres late-payment test");
        return;
    };

    for terminal_status in ["cancelled", "closed", "expired"] {
        let ids = seed_payment_order_postgres(&pool).await;
        set_terminal_order_postgres(&pool, &ids.0, terminal_status).await;
        let store = PostgresCommerceOrderStore::new(pool.clone());
        let attempt = payment_attempt(&ids.0, &ids.1);

        store
            .mark_owner_order_payment_succeeded(&attempt, "2026-07-11T01:02:03Z")
            .await
            .expect("record late payment success");
        store
            .mark_owner_order_payment_succeeded(&attempt, "2026-07-11T04:05:06Z")
            .await
            .expect("replay late payment success");

        assert_terminal_payment_postgres(&pool, &ids.0, terminal_status, "2026-07-11T01:02:03Z")
            .await;
        let page = store
            .list_owner_orders(owner_list_query(&ids.1))
            .await
            .expect("list terminal paid order");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].status, terminal_status);
        assert_eq!(page.items[0].paid_amount.as_ref().unwrap().as_str(), "8800");
        assert_eq!(
            page.items[0].pay_time.as_deref(),
            Some("2026-07-11T01:02:03Z")
        );
    }
}

fn owner_list_query(owner_user_id: &str) -> OrderOwnerListQuery {
    OrderOwnerListQuery::new(
        "tenant-payment",
        None,
        owner_user_id,
        None,
        Some(1),
        Some(20),
        None,
    )
    .expect("list query")
}

fn payment_attempt(order_id: &str, owner_user_id: &str) -> OrderPaymentSettlementAttempt {
    OrderPaymentSettlementAttempt {
        tenant_id: "tenant-payment".to_owned(),
        organization_id: None,
        owner_user_id: owner_user_id.to_owned(),
        order_id: order_id.to_owned(),
        payment_attempt_id: None,
        out_trade_no: None,
    }
}

async fn seed_payment_order_sqlite(pool: &sqlx::SqlitePool) -> (String, String) {
    let order_id = format!("order-payment-{}", Uuid::new_v4());
    let item_id = format!("item-payment-{}", Uuid::new_v4());
    seed_order_sqlite(pool, &order_id, &item_id).await;
    (order_id, "user-payment".to_owned())
}

async fn seed_payment_order_postgres(pool: &sqlx::PgPool) -> (String, String) {
    let order_id = format!("order-payment-{}", Uuid::new_v4());
    let item_id = format!("item-payment-{}", Uuid::new_v4());
    seed_order_postgres(pool, &order_id, &item_id).await;
    (order_id, "user-payment".to_owned())
}

async fn seed_order_sqlite(pool: &sqlx::SqlitePool, order_id: &str, item_id: &str) {
    let now = "2026-07-11T00:00:00Z";
    sqlx::query(
        r#"
        INSERT INTO commerce_order
            (id, tenant_id, organization_id, owner_user_id, order_no, status, subject,
             currency_code, payment_status, fulfillment_status, created_at, updated_at)
        VALUES (?, ?, NULL, ?, ?, 'pending_payment', ?, 'CNY', 'pending', 'unfulfilled', ?, ?)
        "#,
    )
    .bind(order_id)
    .bind("tenant-payment")
    .bind("user-payment")
    .bind(format!("SN-{order_id}"))
    .bind("Localized matter title")
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .expect("seed order");
    sqlx::query(
        r#"
        INSERT INTO commerce_order_item
            (id, tenant_id, order_id, sku_id, sku_snapshot_json, title, quantity,
             unit_price_amount, total_amount, fulfillment_status, refund_status, created_at)
        VALUES (?, ?, ?, ?, ?, ?, 1, '8800', '8800', 'unfulfilled', 'none', ?)
        "#,
    )
    .bind(item_id)
    .bind("tenant-payment")
    .bind(order_id)
    .bind("sku-notary")
    .bind(r#"{"title":"Localized matter title","fulfillment_type":"notary"}"#)
    .bind("Localized matter title")
    .bind(now)
    .execute(pool)
    .await
    .expect("seed order item");
    sqlx::query(
        r#"
        INSERT INTO commerce_order_amount_breakdown
            (id, tenant_id, organization_id, order_id, order_item_id, allocation_type,
             original_amount, discount_amount, payable_amount, currency_code, created_at)
        VALUES (?, ?, NULL, ?, ?, 'order_total', '8800', '0', '8800', 'CNY', ?)
        "#,
    )
    .bind(format!("amount-{order_id}"))
    .bind("tenant-payment")
    .bind(order_id)
    .bind(item_id)
    .bind(now)
    .execute(pool)
    .await
    .expect("seed amount breakdown");
}

async fn seed_order_postgres(pool: &sqlx::PgPool, order_id: &str, item_id: &str) {
    let now = "2026-07-11T00:00:00Z";
    sqlx::query(
        r#"
        INSERT INTO commerce_order
            (id, tenant_id, organization_id, owner_user_id, order_no, status, subject,
             currency_code, payment_status, fulfillment_status, created_at, updated_at)
        VALUES ($1, $2, NULL, $3, $4, 'pending_payment', $5, 'CNY', 'pending', 'unfulfilled', $6, $6)
        "#,
    )
    .bind(order_id)
    .bind("tenant-payment")
    .bind("user-payment")
    .bind(format!("SN-{order_id}"))
    .bind("Localized matter title")
    .bind(now)
    .execute(pool)
    .await
    .expect("seed order");
    sqlx::query(
        r#"
        INSERT INTO commerce_order_item
            (id, tenant_id, order_id, sku_id, sku_snapshot_json, title, quantity,
             unit_price_amount, total_amount, fulfillment_status, refund_status, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, 1, '8800', '8800', 'unfulfilled', 'none', $7)
        "#,
    )
    .bind(item_id)
    .bind("tenant-payment")
    .bind(order_id)
    .bind("sku-notary")
    .bind(r#"{"title":"Localized matter title","fulfillment_type":"notary"}"#)
    .bind("Localized matter title")
    .bind(now)
    .execute(pool)
    .await
    .expect("seed order item");
    sqlx::query(
        r#"
        INSERT INTO commerce_order_amount_breakdown
            (id, tenant_id, organization_id, order_id, order_item_id, allocation_type,
             original_amount, discount_amount, payable_amount, currency_code, created_at)
        VALUES ($1, $2, NULL, $3, $4, 'order_total', '8800', '0', '8800', 'CNY', $5)
        "#,
    )
    .bind(format!("amount-{order_id}"))
    .bind("tenant-payment")
    .bind(order_id)
    .bind(item_id)
    .bind(now)
    .execute(pool)
    .await
    .expect("seed amount breakdown");
}

async fn set_terminal_order_sqlite(pool: &sqlx::SqlitePool, order_id: &str, terminal_status: &str) {
    sqlx::query("UPDATE commerce_order SET status = ?, payment_status = 'closed' WHERE id = ?")
        .bind(terminal_status)
        .bind(order_id)
        .execute(pool)
        .await
        .expect("set terminal sqlite order");
}

async fn set_terminal_order_postgres(pool: &sqlx::PgPool, order_id: &str, terminal_status: &str) {
    sqlx::query("UPDATE commerce_order SET status = $1, payment_status = 'closed' WHERE id = $2")
        .bind(terminal_status)
        .bind(order_id)
        .execute(pool)
        .await
        .expect("set terminal postgres order");
}

async fn assert_order_paid_sqlite(pool: &sqlx::SqlitePool, order_id: &str, paid_at: &str) {
    let row =
        sqlx::query("SELECT status, payment_status, paid_at FROM commerce_order WHERE id = ?")
            .bind(order_id)
            .fetch_one(pool)
            .await
            .expect("load paid order");
    assert_eq!(row.try_get::<String, _>("status").unwrap(), "paid");
    assert_eq!(
        row.try_get::<String, _>("payment_status").unwrap(),
        "success"
    );
    assert_eq!(
        row.try_get::<Option<String>, _>("paid_at")
            .unwrap()
            .as_deref(),
        Some(paid_at)
    );
}

async fn assert_order_paid_postgres(pool: &sqlx::PgPool, order_id: &str, paid_at: &str) {
    let row =
        sqlx::query("SELECT status, payment_status, paid_at FROM commerce_order WHERE id = $1")
            .bind(order_id)
            .fetch_one(pool)
            .await
            .expect("load paid order");
    assert_eq!(row.try_get::<String, _>("status").unwrap(), "paid");
    assert_eq!(
        row.try_get::<String, _>("payment_status").unwrap(),
        "success"
    );
    assert_eq!(
        row.try_get::<Option<String>, _>("paid_at")
            .unwrap()
            .as_deref(),
        Some(paid_at)
    );
}

async fn assert_terminal_payment_sqlite(
    pool: &sqlx::SqlitePool,
    order_id: &str,
    terminal_status: &str,
    paid_at: &str,
) {
    let row =
        sqlx::query("SELECT status, payment_status, paid_at FROM commerce_order WHERE id = ?")
            .bind(order_id)
            .fetch_one(pool)
            .await
            .expect("load terminal paid order");
    assert_eq!(row.try_get::<String, _>("status").unwrap(), terminal_status);
    assert_eq!(
        row.try_get::<String, _>("payment_status").unwrap(),
        "success"
    );
    assert_eq!(
        row.try_get::<Option<String>, _>("paid_at")
            .unwrap()
            .as_deref(),
        Some(paid_at)
    );

    let event = sqlx::query(
        r#"
        SELECT from_status, to_status, reason_code, created_at
        FROM commerce_order_event
        WHERE tenant_id = 'tenant-payment'
          AND order_id = ?
          AND event_type = 'payment_succeeded_after_terminal'
        "#,
    )
    .bind(order_id)
    .fetch_all(pool)
    .await
    .expect("load sqlite late-payment event");
    assert_eq!(event.len(), 1);
    assert_eq!(
        event[0].try_get::<String, _>("from_status").unwrap(),
        terminal_status
    );
    assert_eq!(
        event[0].try_get::<String, _>("to_status").unwrap(),
        terminal_status
    );
    assert_eq!(
        event[0]
            .try_get::<Option<String>, _>("reason_code")
            .unwrap(),
        Some("late_payment".to_owned())
    );
    assert_eq!(
        event[0].try_get::<String, _>("created_at").unwrap(),
        paid_at
    );
}

async fn assert_terminal_payment_postgres(
    pool: &sqlx::PgPool,
    order_id: &str,
    terminal_status: &str,
    paid_at: &str,
) {
    let row =
        sqlx::query("SELECT status, payment_status, paid_at FROM commerce_order WHERE id = $1")
            .bind(order_id)
            .fetch_one(pool)
            .await
            .expect("load terminal paid order");
    assert_eq!(row.try_get::<String, _>("status").unwrap(), terminal_status);
    assert_eq!(
        row.try_get::<String, _>("payment_status").unwrap(),
        "success"
    );
    assert_eq!(
        row.try_get::<Option<String>, _>("paid_at")
            .unwrap()
            .as_deref(),
        Some(paid_at)
    );

    let event = sqlx::query(
        r#"
        SELECT from_status, to_status, reason_code, created_at
        FROM commerce_order_event
        WHERE tenant_id = 'tenant-payment'
          AND order_id = $1
          AND event_type = 'payment_succeeded_after_terminal'
        "#,
    )
    .bind(order_id)
    .fetch_all(pool)
    .await
    .expect("load postgres late-payment event");
    assert_eq!(event.len(), 1);
    assert_eq!(
        event[0].try_get::<String, _>("from_status").unwrap(),
        terminal_status
    );
    assert_eq!(
        event[0].try_get::<String, _>("to_status").unwrap(),
        terminal_status
    );
    assert_eq!(
        event[0]
            .try_get::<Option<String>, _>("reason_code")
            .unwrap(),
        Some("late_payment".to_owned())
    );
    assert_eq!(
        event[0].try_get::<String, _>("created_at").unwrap(),
        paid_at
    );
}

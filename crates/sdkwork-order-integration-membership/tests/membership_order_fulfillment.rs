use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_history::execute_sql_script;
use sdkwork_database_sqlx::{DatabasePool, PoolContext};
use sdkwork_order_integration_membership::StoreMembershipFulfillmentAdapter;
use sdkwork_order_repository_sqlx::SqliteCommerceMembershipOrderStore;
use sdkwork_order_service::{
    CreateMembershipOrderCommand, MembershipPurchaseFulfillmentPort,
    MembershipPurchaseFulfillmentRequest,
};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};

const MEMBERSHIP_BASELINE: &str = include_str!(
    "../../../../sdkwork-membership/database/ddl/baseline/sqlite/0001_membership_baseline.sql"
);
const MEMBERSHIP_CATALOG_SEED: &str =
    include_str!("../../../../sdkwork-membership/database/seeds/common/001_catalog.sql");
const ORDER_TEST_BASELINE: &str = include_str!(
    "../../sdkwork-order-repository-sqlx/test_migrations/0001_order_points_recharge_e2e.sql"
);

#[tokio::test]
async fn order_created_membership_purchase_is_atomically_activated_and_replayed() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite pool");
    let database_pool = install_fixture(&pool).await;
    let orders = SqliteCommerceMembershipOrderStore::new(pool.clone());
    let order_store = sdkwork_order_repository_sqlx::SqliteCommerceOrderStore::new(pool.clone());

    let created = orders
        .create_membership_order(
            CreateMembershipOrderCommand::new(
                "200002",
                Some("0"),
                "300003",
                "201",
                "purchase",
                "wechat_pay",
                "wechat_native",
                "order-membership-e2e",
                "item-membership-e2e",
                "MB-E2E-1",
                "MEMBERSHIP-E2E-1",
                "2026-07-26T08:00:00Z",
                "2026-07-26T08:30:00Z",
                "membership-order-e2e",
                None,
                Some("token-plan"),
            )
            .expect("membership order command"),
        )
        .await
        .expect("membership order creation");
    let context = order_store
        .load_order_payment_settlement_context("200002", Some("0"), &created.order_id)
        .await
        .expect("order settlement context")
        .expect("order context");
    let snapshot = context
        .membership_purchase
        .expect("membership purchase snapshot");
    assert_eq!(snapshot.package_id, 201);
    assert_eq!(snapshot.action, "purchase");

    let adapter = StoreMembershipFulfillmentAdapter::from_database_pool(&database_pool);
    let request = MembershipPurchaseFulfillmentRequest {
        action: snapshot.action,
        tenant_id: "200002".to_owned(),
        organization_id: Some("0".to_owned()),
        owner_user_id: "300003".to_owned(),
        order_id: created.order_id.clone(),
        order_no: snapshot.order_no,
        package_id: snapshot.package_id,
        paid_at: "2026-07-26T08:05:00Z".to_owned(),
        request_no: "webhook-membership-e2e".to_owned(),
        idempotency_key: format!("membership-purchase:fulfill:{}", created.order_id),
    };
    let activated = adapter
        .fulfill_membership_purchase(request.clone())
        .await
        .expect("membership activation");
    assert!(activated.accepted);
    assert!(!activated.replayed);

    let replay = adapter
        .fulfill_membership_purchase(request)
        .await
        .expect("membership activation replay");
    assert!(replay.replayed);
    assert_eq!(active_subscription_count(&pool, &created.order_id).await, 1);
    assert_eq!(active_period_count(&pool, &created.order_id).await, 1);
}

async fn install_fixture(pool: &SqlitePool) -> DatabasePool {
    let database_pool = DatabasePool::Sqlite(
        pool.clone(),
        PoolContext {
            config: DatabaseConfig::default(),
        },
    );
    execute_sql_script(&database_pool, MEMBERSHIP_BASELINE)
        .await
        .expect("membership baseline");
    execute_sql_script(&database_pool, MEMBERSHIP_CATALOG_SEED)
        .await
        .expect("membership catalog seed");
    execute_sql_script(&database_pool, ORDER_TEST_BASELINE)
        .await
        .expect("order baseline");
    database_pool
}

async fn active_subscription_count(pool: &SqlitePool, order_id: &str) -> i64 {
    sqlx::query(
        "SELECT COUNT(*) AS count_value FROM membership_subscription WHERE source_order_id = ? AND status = 'active'",
    )
    .bind(order_id)
    .fetch_one(pool)
    .await
    .expect("active subscription count")
    .try_get("count_value")
    .expect("subscription count value")
}

async fn active_period_count(pool: &SqlitePool, order_id: &str) -> i64 {
    sqlx::query(
        "SELECT COUNT(*) AS count_value FROM membership_period WHERE source_order_id = ? AND status = 'active'",
    )
    .bind(order_id)
    .fetch_one(pool)
    .await
    .expect("active period count")
    .try_get("count_value")
    .expect("period count value")
}

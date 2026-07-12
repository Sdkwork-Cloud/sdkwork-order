use sdkwork_order_repository_sqlx::{
    order_points_recharge_e2e_sqlite_memory_pool, SqliteCommerceOrderStore,
};
use sdkwork_order_service::{
    CheckoutLineInput, CheckoutSessionDetailQuery, CreateCheckoutQuoteCommand,
    CreateCheckoutSessionCommand, CreateOwnerOrderCommand,
};
use sqlx::SqlitePool;

const TENANT_ID: &str = "tenant-notary";
const ORGANIZATION_ID: &str = "organization-notary";
const OWNER_USER_ID: &str = "owner-notary";

#[tokio::test]
async fn sqlite_checkout_to_owner_order_preserves_minor_units_and_replays() {
    let pool = checkout_test_pool().await;
    seed_merchandise(&pool, ORGANIZATION_ID, "spu-notary", "sku-notary", "6990").await;
    let store = SqliteCommerceOrderStore::new(pool.clone());

    let session_command = CreateCheckoutSessionCommand::new(
        TENANT_ID,
        Some(ORGANIZATION_ID),
        OWNER_USER_ID,
        "CNY",
        vec![CheckoutLineInput::new("sku-notary", 2).expect("checkout line")],
        "checkout-request-1",
        "checkout-idempotency-1",
    )
    .expect("checkout session command");
    let session = store
        .create_checkout_session(session_command.clone())
        .await
        .expect("create checkout session");

    assert_checkout_amounts(
        session.original_amount.as_str(),
        session.discount_amount.as_str(),
        session.payable_amount.as_str(),
    );
    let session_replay = store
        .create_checkout_session(session_command)
        .await
        .expect("replay checkout session");
    assert_eq!(session_replay, session);

    let quote_command = CreateCheckoutQuoteCommand::new(
        TENANT_ID,
        Some(ORGANIZATION_ID),
        OWNER_USER_ID,
        &session.checkout_session_id,
        "quote-request-1",
        "quote-idempotency-1",
    )
    .expect("checkout quote command");
    let quote = store
        .create_checkout_quote(quote_command.clone())
        .await
        .expect("create checkout quote");
    assert_checkout_amounts(
        quote.original_amount.as_str(),
        quote.discount_amount.as_str(),
        quote.payable_amount.as_str(),
    );
    let quote_replay = store
        .create_checkout_quote(quote_command)
        .await
        .expect("replay checkout quote");
    assert_eq!(quote_replay, quote);

    let retrieved_session = store
        .retrieve_checkout_session(
            CheckoutSessionDetailQuery::new(
                TENANT_ID,
                Some(ORGANIZATION_ID),
                OWNER_USER_ID,
                &session.checkout_session_id,
            )
            .expect("checkout session query"),
        )
        .await
        .expect("retrieve checkout session")
        .expect("checkout session exists");
    assert_checkout_amounts(
        retrieved_session.original_amount.as_str(),
        retrieved_session.discount_amount.as_str(),
        retrieved_session.payable_amount.as_str(),
    );

    let order_command = CreateOwnerOrderCommand::new(
        TENANT_ID,
        Some(ORGANIZATION_ID),
        OWNER_USER_ID,
        &session.checkout_session_id,
        "order-request-1",
        "order-idempotency-1",
    )
    .expect("owner order command");
    let order = store
        .create_owner_order(order_command.clone())
        .await
        .expect("create owner order");
    assert_eq!(order.total_amount.as_str(), "13980");
    let order_replay = store
        .create_owner_order(order_command)
        .await
        .expect("replay owner order");
    assert_eq!(order_replay.order_id, order.order_id);
    assert_eq!(order_replay.total_amount.as_str(), "13980");

    let item = sqlx::query_as::<_, (String, i64, String, String, String)>(
        r#"
        SELECT unit_price_amount, quantity, total_amount, discount_amount, tax_amount
        FROM commerce_order_item
        WHERE tenant_id = ? AND order_id = ?
        "#,
    )
    .bind(TENANT_ID)
    .bind(&order.order_id)
    .fetch_one(&pool)
    .await
    .expect("owner order item");
    assert_eq!(
        item,
        (
            "6990".to_owned(),
            2,
            "13980".to_owned(),
            "0".to_owned(),
            "0".to_owned()
        )
    );

    let order_amounts = sqlx::query_as::<_, (String, String, String)>(
        r#"
        SELECT original_amount, discount_amount, payable_amount
        FROM commerce_order_amount_breakdown
        WHERE tenant_id = ? AND order_id = ? AND allocation_type = 'order_total'
        "#,
    )
    .bind(TENANT_ID)
    .bind(&order.order_id)
    .fetch_one(&pool)
    .await
    .expect("owner order amount breakdown");
    assert_eq!(
        order_amounts,
        ("13980".to_owned(), "0".to_owned(), "13980".to_owned())
    );

    let quote_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM commerce_checkout_quote WHERE tenant_id = ? AND checkout_session_id = ?",
    )
    .bind(TENANT_ID)
    .bind(&session.checkout_session_id)
    .fetch_one(&pool)
    .await
    .expect("checkout quote count");
    assert_eq!(
        quote_count, 2,
        "idempotent quote replay must not insert a third quote"
    );
}

#[tokio::test]
async fn sqlite_checkout_amount_overflow_fails_closed_and_rolls_back() {
    let pool = checkout_test_pool().await;
    let maximum = i64::MAX.to_string();
    seed_merchandise(
        &pool,
        ORGANIZATION_ID,
        "spu-overflow",
        "sku-overflow",
        &maximum,
    )
    .await;
    let store = SqliteCommerceOrderStore::new(pool.clone());
    let command = CreateCheckoutSessionCommand::new(
        TENANT_ID,
        Some(ORGANIZATION_ID),
        OWNER_USER_ID,
        "CNY",
        vec![CheckoutLineInput::new("sku-overflow", 2).expect("checkout line")],
        "overflow-request-1",
        "overflow-idempotency-1",
    )
    .expect("overflow checkout command");

    let error = store
        .create_checkout_session(command)
        .await
        .expect_err("overflowing checkout must fail");
    assert_eq!(error.code(), "validation");
    assert!(error.message().contains("too large"));

    assert_checkout_tables_are_empty(&pool).await;
}

#[tokio::test]
async fn sqlite_checkout_rejects_another_organizations_sku() {
    let pool = checkout_test_pool().await;
    seed_merchandise(
        &pool,
        "organization-other",
        "spu-other",
        "sku-other",
        "6990",
    )
    .await;
    let store = SqliteCommerceOrderStore::new(pool.clone());
    let command = CreateCheckoutSessionCommand::new(
        TENANT_ID,
        Some(ORGANIZATION_ID),
        OWNER_USER_ID,
        "CNY",
        vec![CheckoutLineInput::new("sku-other", 1).expect("checkout line")],
        "cross-organization-request-1",
        "cross-organization-idempotency-1",
    )
    .expect("cross-organization checkout command");

    let error = store
        .create_checkout_session(command)
        .await
        .expect_err("another organization's SKU must not be orderable");
    assert_eq!(error.code(), "not-found");
    assert_checkout_tables_are_empty(&pool).await;
}

fn assert_checkout_amounts(original: &str, discount: &str, payable: &str) {
    assert_eq!(original, "13980");
    assert_eq!(discount, "0");
    assert_eq!(payable, "13980");
}

async fn checkout_test_pool() -> SqlitePool {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    for statement in CHECKOUT_TEST_SCHEMA {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .unwrap_or_else(|error| {
                panic!("checkout test schema failed on `{statement}`: {error}")
            });
    }
    pool
}

async fn seed_merchandise(
    pool: &SqlitePool,
    organization_id: &str,
    spu_id: &str,
    sku_id: &str,
    price: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO commerce_product_spu
            (id, tenant_id, organization_id, spu_no, name, title, sales_status, status,
             created_at, updated_at)
        VALUES (?, ?, ?, ?, 'Notary matter', 'Notary matter', 'active', 'active', '1', '1')
        "#,
    )
    .bind(spu_id)
    .bind(TENANT_ID)
    .bind(organization_id)
    .bind(spu_id)
    .execute(pool)
    .await
    .expect("seed merchandise SPU");
    sqlx::query(
        r#"
        INSERT INTO commerce_product_sku
            (id, tenant_id, organization_id, spu_id, sku_no, name, title, price_amount, currency_code,
             sales_status, status, spec_json, fulfillment_type, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 'Notary matter', 'Notary matter', ?, 'CNY',
                'active', 'active', '{}', 'notary', '1', '1')
        "#,
    )
    .bind(sku_id)
    .bind(TENANT_ID)
    .bind(organization_id)
    .bind(spu_id)
    .bind(sku_id)
    .bind(price)
    .execute(pool)
    .await
    .expect("seed merchandise SKU");
}

async fn assert_checkout_tables_are_empty(pool: &SqlitePool) {
    for table in [
        "commerce_idempotency_key",
        "commerce_checkout_session",
        "commerce_checkout_line",
        "commerce_checkout_quote",
    ] {
        let count = sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(pool)
            .await
            .expect("count rolled-back checkout rows");
        assert_eq!(count, 0, "failed checkout must roll back {table}");
    }
}

const CHECKOUT_TEST_SCHEMA: &[&str] = &[
    "ALTER TABLE commerce_product_sku ADD COLUMN fulfillment_type TEXT NOT NULL DEFAULT 'digital'",
    "ALTER TABLE commerce_order_item ADD COLUMN product_id TEXT",
    "ALTER TABLE commerce_order_item ADD COLUMN shop_id TEXT",
    "ALTER TABLE commerce_order_item ADD COLUMN discount_amount TEXT",
    "ALTER TABLE commerce_order_item ADD COLUMN tax_amount TEXT",
    r#"
    CREATE TABLE commerce_checkout_session (
        id TEXT NOT NULL PRIMARY KEY,
        tenant_id TEXT NOT NULL,
        organization_id TEXT,
        checkout_session_no TEXT NOT NULL,
        owner_user_id TEXT NOT NULL,
        source_type TEXT NOT NULL,
        status TEXT NOT NULL,
        currency_code TEXT NOT NULL,
        promotion_snapshot_json TEXT NOT NULL,
        request_hash TEXT NOT NULL,
        request_no TEXT NOT NULL,
        idempotency_key TEXT NOT NULL,
        expires_at TEXT,
        submitted_at TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    r#"
    CREATE TABLE commerce_checkout_line (
        id TEXT NOT NULL PRIMARY KEY,
        tenant_id TEXT NOT NULL,
        organization_id TEXT,
        checkout_session_id TEXT NOT NULL,
        product_id TEXT,
        shop_id TEXT,
        sku_id TEXT NOT NULL,
        sku_snapshot_json TEXT NOT NULL,
        selected_options_hash TEXT,
        quantity INTEGER NOT NULL,
        purchase_type TEXT NOT NULL,
        fulfillment_type TEXT NOT NULL,
        price_amount_snapshot TEXT NOT NULL,
        currency_code TEXT NOT NULL,
        selected INTEGER NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    r#"
    CREATE TABLE commerce_checkout_quote (
        id TEXT NOT NULL PRIMARY KEY,
        tenant_id TEXT NOT NULL,
        organization_id TEXT,
        checkout_session_id TEXT NOT NULL,
        quote_no TEXT NOT NULL,
        original_amount TEXT NOT NULL,
        discount_amount TEXT NOT NULL,
        payable_amount TEXT NOT NULL,
        currency_code TEXT NOT NULL,
        quote_status TEXT NOT NULL,
        expires_at TEXT,
        created_at TEXT NOT NULL
    )
    "#,
];

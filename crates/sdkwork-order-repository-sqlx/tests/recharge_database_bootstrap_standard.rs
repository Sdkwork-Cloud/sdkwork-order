use sdkwork_order_repository_sqlx::SqliteCommerceRechargeStore;
use sdkwork_order_service::RechargePackageListQuery;
use sqlx::{Row, SqlitePool};

const SQLITE_BASELINE: &str =
    include_str!("../../../database/ddl/baseline/sqlite/0001_order_baseline.sql");
const POSTGRES_BASELINE: &str =
    include_str!("../../../database/ddl/baseline/postgres/0001_order_baseline.sql");
const RECHARGE_SEED: &str = include_str!("../../../database/seeds/common/001_bootstrap.sql");

#[tokio::test]
async fn bootstrap_seed_is_idempotent_and_lists_platform_recharge_packages() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("sqlite memory pool");

    apply_sql(&pool, SQLITE_BASELINE).await;
    apply_sql(
        &pool,
        r#"
        CREATE TABLE commerce_product_spu (
            id TEXT NOT NULL PRIMARY KEY,
            sales_status TEXT NOT NULL
        );
        CREATE TABLE commerce_product_sku (
            id TEXT NOT NULL PRIMARY KEY,
            spu_id TEXT NOT NULL,
            sales_status TEXT NOT NULL
        );
        CREATE TABLE commerce_exchange_rule (
            id TEXT NOT NULL PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            organization_id TEXT,
            rule_no TEXT NOT NULL,
            source_asset_type TEXT NOT NULL,
            target_asset_type TEXT NOT NULL,
            rate TEXT NOT NULL,
            status TEXT NOT NULL,
            remark TEXT
        );
        "#,
    )
    .await;
    sqlx::query(
        r#"
        INSERT INTO commerce_recharge_package (
            id, tenant_id, organization_id, external_id, package_no, sku_id, name,
            price_amount, currency_code, bonus_points, status, sort_weight,
            request_no, idempotency_key, created_at, updated_at
        ) VALUES (
            'bootstrap-admin-recharge-package-100001-1', '100001', '0', 1,
            'legacy-demo-1', 'legacy-demo-sku-1', 'Legacy demo', '5.00', 'CNY', 0,
            'active', 1, 'legacy-demo-1', 'legacy-demo-1', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("legacy demo package");
    apply_sql(&pool, RECHARGE_SEED).await;
    apply_sql(&pool, RECHARGE_SEED).await;

    let rows = sqlx::query(
        r#"
        SELECT id, price_amount, bonus_points
        FROM commerce_recharge_package
        WHERE status = 'active'
        ORDER BY sort_weight, id
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("seeded recharge packages");

    let actual = rows
        .iter()
        .map(|row| {
            (
                row.get::<String, _>("id"),
                row.get::<String, _>("price_amount"),
                row.get::<i64, _>("bonus_points"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            ("recharge-500".to_string(), "50.00".to_string(), 0),
            ("recharge-750".to_string(), "75.00".to_string(), 0),
            ("recharge-1500".to_string(), "150.00".to_string(), 0),
            ("recharge-2250".to_string(), "223.00".to_string(), 20),
            ("recharge-4500".to_string(), "450.00".to_string(), 0),
            ("recharge-9000".to_string(), "899.00".to_string(), 10),
        ]
    );
    let legacy_status: String = sqlx::query_scalar(
        "SELECT status FROM commerce_recharge_package WHERE id = 'bootstrap-admin-recharge-package-100001-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("legacy package status");
    assert_eq!("inactive", legacy_status);

    let store = SqliteCommerceRechargeStore::new(pool);
    let page = store
        .list_recharge_packages(
            RechargePackageListQuery::new("tenant-without-catalog", Some("0"), Some(1), Some(20))
                .expect("fallback list query"),
        )
        .await
        .expect("platform recharge package fallback");

    assert_eq!(6, page.total);
    assert_eq!(
        page.items
            .iter()
            .map(|item| (item.id.as_str(), item.price_amount.as_str(), item.points))
            .collect::<Vec<_>>(),
        vec![
            ("recharge-500", "5000", 500),
            ("recharge-750", "7500", 750),
            ("recharge-1500", "15000", 1500),
            ("recharge-2250", "22300", 2250),
            ("recharge-4500", "45000", 4500),
            ("recharge-9000", "89900", 9000),
        ]
    );
}

#[test]
fn both_engine_baselines_own_the_recharge_package_contract() {
    for (engine, baseline) in [("sqlite", SQLITE_BASELINE), ("postgres", POSTGRES_BASELINE)] {
        assert!(
            baseline.contains("CREATE TABLE IF NOT EXISTS commerce_recharge_package"),
            "{engine} baseline must create commerce_recharge_package"
        );
        assert!(
            baseline.contains("idx_recharge_package_list"),
            "{engine} baseline must index the package list query"
        );
        assert!(
            baseline.contains("uk_recharge_package_idempotency"),
            "{engine} baseline must enforce seed/runtime idempotency"
        );
    }
}

async fn apply_sql(pool: &SqlitePool, source: &str) {
    let source = strip_sql_comments(source);
    for statement in source
        .split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
    {
        sqlx::query(statement)
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("failed to execute `{statement}`: {error}"));
    }
}

fn strip_sql_comments(source: &str) -> String {
    source
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            !line.is_empty() && !line.starts_with("--")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

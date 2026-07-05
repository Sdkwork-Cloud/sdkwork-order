use sqlx::SqlitePool;

pub fn order_points_recharge_e2e_migration_sql() -> &'static str {
    include_str!("../test_migrations/0001_order_points_recharge_e2e.sql")
}

pub fn split_order_e2e_sql_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(|chunk| {
            chunk
                .lines()
                .filter(|line| {
                    let trimmed = line.trim_start();
                    !trimmed.is_empty() && !trimmed.starts_with("--")
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .map(|statement| statement.trim().to_string())
        .filter(|statement| !statement.is_empty())
        .collect()
}

pub async fn order_points_recharge_e2e_sqlite_memory_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("order points recharge e2e sqlite memory pool");
    apply_order_e2e_migration_sqlite(&pool).await;
    pool
}

pub async fn apply_order_e2e_migration_sqlite(pool: &SqlitePool) {
    for statement in split_order_e2e_sql_statements(order_points_recharge_e2e_migration_sql()) {
        sqlx::query(&statement)
            .execute(pool)
            .await
            .unwrap_or_else(|error| {
                panic!("order points recharge e2e migration failed on `{statement}`: {error}")
            });
    }
}

pub async fn order_points_recharge_e2e_postgres_pool_from_env() -> Option<sqlx::PgPool> {
    let url = std::env::var("ORDER_TEST_POSTGRES_URL")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())?;
    let pool = sqlx::PgPool::connect(&url).await.ok()?;
    for statement in split_order_e2e_sql_statements(order_points_recharge_e2e_migration_sql()) {
        if let Err(error) = sqlx::query(&statement).execute(&pool).await {
            eprintln!("postgres e2e migration skipped ({error}); statement: {statement}");
            return None;
        }
    }
    Some(pool)
}

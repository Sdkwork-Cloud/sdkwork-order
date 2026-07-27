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
        let statement = postgres_e2e_statement(&statement);
        if let Err(error) = sqlx::query(&statement).execute(&pool).await {
            eprintln!("postgres e2e migration skipped ({error}); statement: {statement}");
            return None;
        }
    }
    Some(pool)
}

fn postgres_e2e_statement(statement: &str) -> String {
    let is_insert_or_ignore = statement
        .trim_start()
        .to_ascii_uppercase()
        .starts_with("INSERT OR IGNORE INTO ");
    let mut statement = statement
        .replacen("INSERT OR IGNORE INTO", "INSERT INTO", 1)
        .replace("datetime('now')", "CURRENT_TIMESTAMP");
    if is_insert_or_ignore {
        statement.push_str(" ON CONFLICT DO NOTHING");
    }
    statement
}

#[cfg(test)]
mod tests {
    use super::postgres_e2e_statement;

    #[test]
    fn postgres_seed_translation_preserves_idempotent_bootstrap() {
        let translated = postgres_e2e_statement(
            "INSERT OR IGNORE INTO example (id, created_at) VALUES ('1', datetime('now'))",
        );
        assert_eq!(
            translated,
            "INSERT INTO example (id, created_at) VALUES ('1', CURRENT_TIMESTAMP) ON CONFLICT DO NOTHING"
        );
    }
}

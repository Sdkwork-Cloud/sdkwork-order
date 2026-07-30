use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_sqlx::{DatabasePool, PoolContext};
use sdkwork_order_integration_membership::StoreMembershipFulfillmentAdapter;

#[tokio::test]
async fn server_membership_fulfillment_rejects_client_local_sqlite() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect_lazy("sqlite::memory:")
        .expect("lazy SQLite fixture pool");
    let database_pool = DatabasePool::Sqlite(
        pool,
        PoolContext {
            config: DatabaseConfig::default(),
        },
    );

    let error = StoreMembershipFulfillmentAdapter::from_database_pool(&database_pool)
        .err()
        .expect("server membership fulfillment must reject SQLite");

    assert_eq!(
        error,
        "order membership fulfillment server requires PostgreSQL",
    );
}

use sdkwork_order_service::OrderManagementListQuery;

use crate::test_sqlite_pool::{
    order_points_recharge_e2e_postgres_pool_from_env, order_points_recharge_e2e_sqlite_memory_pool,
};
use crate::{PostgresCommerceOrderStore, SqliteCommerceOrderStore};

#[tokio::test]
async fn sqlite_list_management_orders_returns_empty_page() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    let store = SqliteCommerceOrderStore::new(pool);
    let query = OrderManagementListQuery::new("tenant-1", None, None, None, Some(1), Some(20))
        .expect("valid query");

    let page = store
        .list_management_orders(query)
        .await
        .expect("list management orders");

    assert!(page.items.is_empty());
    assert_eq!(page.total, 0);
}

#[tokio::test]
async fn postgres_list_management_orders_returns_empty_page() {
    let Some(pool) = order_points_recharge_e2e_postgres_pool_from_env().await else {
        eprintln!("ORDER_TEST_POSTGRES_URL is unset; skipping postgres management list parity test");
        return;
    };
    let store = PostgresCommerceOrderStore::new(pool);
    let query = OrderManagementListQuery::new("tenant-1", None, None, None, Some(1), Some(20))
        .expect("valid query");

    let page = store
        .list_management_orders(query)
        .await
        .expect("list management orders");

    assert!(page.items.is_empty());
    assert_eq!(page.total, 0);
}

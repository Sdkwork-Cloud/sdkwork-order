use axum::Router;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_order_service_host::OrderServiceHost;
use std::sync::Arc;

use crate::{
    backend_order_admin_router_with_postgres_pool, backend_order_admin_router_with_sqlite_pool,
};

pub fn build_order_backend_router(host: Arc<OrderServiceHost>) -> Router {
    match host.database_pool() {
        DatabasePool::Postgres(pool, _) => {
            backend_order_admin_router_with_postgres_pool(pool.clone())
        }
        DatabasePool::Sqlite(pool, _) => {
            backend_order_admin_router_with_sqlite_pool(pool.clone())
        }
    }
}

pub async fn build_order_backend_router_with_framework(host: Arc<OrderServiceHost>) -> Router {
    build_order_backend_router(host)
}

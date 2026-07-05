use axum::Router;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_order_service_host::OrderServiceHost;
use std::sync::Arc;

use crate::{
    backend_order_admin_router_with_postgres_pool, backend_order_admin_router_with_sqlite_pool,
    openapi_contract::mount_backend_openapi,
    points_recharge_fulfillment_router_with_postgres_pool,
    points_recharge_fulfillment_router_with_sqlite_pool,
};

pub fn build_order_backend_router(host: Arc<OrderServiceHost>) -> Router {
    let credit_port = host.account_credit_port();
    let router = match host.database_pool() {
        DatabasePool::Postgres(pool, _) => Router::new()
            .merge(backend_order_admin_router_with_postgres_pool(pool.clone()))
            .merge(points_recharge_fulfillment_router_with_postgres_pool(
                pool.clone(),
                credit_port,
            )),
        DatabasePool::Sqlite(pool, _) => Router::new()
            .merge(backend_order_admin_router_with_sqlite_pool(pool.clone()))
            .merge(points_recharge_fulfillment_router_with_sqlite_pool(
                pool.clone(),
                credit_port,
            )),
    };
    mount_backend_openapi(router)
}

pub async fn build_order_backend_router_with_framework(host: Arc<OrderServiceHost>) -> Router {
    crate::web_bootstrap::wrap_router_with_web_framework_from_env(build_order_backend_router(host))
        .await
}

use axum::Router;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_order_service_host::OrderServiceHost;
use std::sync::Arc;

use crate::{
    backend_commerce_admin_router_with_postgres_pool_and_ports,
    backend_commerce_admin_router_with_sqlite_pool_and_ports,
    backend_order_admin_router_with_postgres_pool, backend_order_admin_router_with_sqlite_pool,
    openapi_contract::mount_backend_openapi, payment_confirmation_router_with_postgres_pool,
    payment_confirmation_router_with_sqlite_pool,
};

pub fn build_order_backend_router(host: Arc<OrderServiceHost>) -> Router {
    let credit_port = host.account_credit_port();
    let account_value_ledger_port = host.account_value_ledger_port();
    let membership_port = host.membership_fulfillment_port();
    let payment_refund_executor_port = host.payment_refund_executor_port();
    let payment_payout_executor_port = host.payment_payout_executor_port();
    let router = match host.database_pool() {
        DatabasePool::Postgres(pool, _) => Router::new()
            .merge(backend_order_admin_router_with_postgres_pool(pool.clone()))
            .merge(backend_commerce_admin_router_with_postgres_pool_and_ports(
                pool.clone(),
                account_value_ledger_port.clone(),
                payment_refund_executor_port.clone(),
                payment_payout_executor_port.clone(),
            ))
            .merge(payment_confirmation_router_with_postgres_pool(
                pool.clone(),
                credit_port,
                account_value_ledger_port,
                membership_port,
            )),
        DatabasePool::Sqlite(pool, _) => Router::new()
            .merge(backend_order_admin_router_with_sqlite_pool(pool.clone()))
            .merge(backend_commerce_admin_router_with_sqlite_pool_and_ports(
                pool.clone(),
                account_value_ledger_port.clone(),
                payment_refund_executor_port,
                payment_payout_executor_port,
            ))
            .merge(payment_confirmation_router_with_sqlite_pool(
                pool.clone(),
                credit_port,
                account_value_ledger_port,
                membership_port,
            )),
    };
    mount_backend_openapi(router)
}

pub async fn build_order_backend_router_with_framework(host: Arc<OrderServiceHost>) -> Router {
    crate::web_bootstrap::wrap_router_with_web_framework_from_env(build_order_backend_router(host))
        .await
}

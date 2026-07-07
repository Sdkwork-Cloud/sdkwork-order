use axum::Router;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_order_service_host::OrderServiceHost;
use std::sync::Arc;

use crate::{
    app_after_sales_router_with_postgres_pool, app_after_sales_router_with_sqlite_pool,
    app_checkout_router_with_postgres_pool, app_checkout_router_with_sqlite_pool,
    app_fulfillment_router_with_postgres_pool, app_fulfillment_router_with_sqlite_pool,
    app_order_router_with_postgres_pool, app_order_router_with_sqlite_pool,
    app_membership_order_router_with_postgres_pool, app_membership_order_router_with_sqlite_pool,
    app_payment_webhook_router_with_postgres_pool, app_payment_webhook_router_with_sqlite_pool,
    app_recharge_checkout_router_with_postgres_pool, app_recharge_checkout_router_with_sqlite_pool,
    app_shipment_router_with_postgres_pool, app_shipment_router_with_sqlite_pool,
};
use sdkwork_payment_providers::{PaymentProviderRegistry, ProviderCredentialBundle};
use crate::openapi_contract::mount_app_openapi;
use crate::web_bootstrap::wrap_router_with_web_framework_from_env;

pub fn build_order_app_router(host: Arc<OrderServiceHost>) -> Router {
    let credit_port = host.account_credit_port();
    let membership_port = host.membership_fulfillment_port();
    let credentials = ProviderCredentialBundle::from_env();
    let registry = Arc::new(PaymentProviderRegistry::from_credentials(credentials.clone()));
    let router = match host.database_pool() {
        DatabasePool::Postgres(pool, _) => {
            let pool = pool.clone();
            Router::new()
                .merge(app_order_router_with_postgres_pool(
                    pool.clone(),
                    registry.clone(),
                    credentials.clone(),
                ))
                .merge(app_checkout_router_with_postgres_pool(pool.clone()))
                .merge(app_recharge_checkout_router_with_postgres_pool(
                    pool.clone(),
                    registry.clone(),
                    credentials.clone(),
                ))
                .merge(app_membership_order_router_with_postgres_pool(pool.clone()))
                .merge(app_fulfillment_router_with_postgres_pool(pool.clone()))
                .merge(app_shipment_router_with_postgres_pool(pool.clone()))
                .merge(app_after_sales_router_with_postgres_pool(pool.clone()))
                .merge(app_payment_webhook_router_with_postgres_pool(
                    pool,
                    credit_port,
                    membership_port,
                ))
        }
        DatabasePool::Sqlite(pool, _) => {
            let pool = pool.clone();
            Router::new()
                .merge(app_order_router_with_sqlite_pool(
                    pool.clone(),
                    registry.clone(),
                    credentials.clone(),
                ))
                .merge(app_checkout_router_with_sqlite_pool(pool.clone()))
                .merge(app_recharge_checkout_router_with_sqlite_pool(
                    pool.clone(),
                    registry.clone(),
                    credentials.clone(),
                ))
                .merge(app_membership_order_router_with_sqlite_pool(pool.clone()))
                .merge(app_fulfillment_router_with_sqlite_pool(pool.clone()))
                .merge(app_shipment_router_with_sqlite_pool(pool.clone()))
                .merge(app_after_sales_router_with_sqlite_pool(pool.clone()))
                .merge(app_payment_webhook_router_with_sqlite_pool(pool, credit_port, membership_port))
        }
    };
    mount_app_openapi(router)
}

pub async fn build_order_app_router_with_framework(host: Arc<OrderServiceHost>) -> Router {
    wrap_router_with_web_framework_from_env(build_order_app_router(host)).await
}

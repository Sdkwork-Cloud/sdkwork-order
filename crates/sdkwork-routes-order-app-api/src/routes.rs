use axum::Router;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_order_service_host::OrderServiceHost;
use std::sync::Arc;

use crate::openapi_contract::mount_app_openapi;
use crate::web_bootstrap::wrap_router_with_web_framework_from_env;
use crate::{
    app_after_sales_router_with_postgres_pool, app_after_sales_router_with_sqlite_pool,
    app_checkout_router_with_postgres_pool, app_checkout_router_with_sqlite_pool,
    app_fulfillment_router_with_postgres_pool, app_fulfillment_router_with_sqlite_pool,
    app_membership_order_router_with_postgres_pool_and_payments,
    app_membership_order_router_with_sqlite_pool_and_payments, app_order_router_with_postgres_pool,
    app_order_router_with_sqlite_pool, app_payment_webhook_router_with_postgres_pool_and_coupon,
    app_payment_webhook_router_with_sqlite_pool_and_coupon, app_shipment_router_with_postgres_pool,
    app_shipment_router_with_sqlite_pool, build_app_recharge_checkout_router_with_integrations,
};
use sdkwork_order_repository_sqlx::{
    PostgresCommerceOrderStore, PostgresCommerceRechargeStore, SqliteCommerceOrderStore,
    SqliteCommerceRechargeStore,
};
use sdkwork_order_service::{AccountValueLedgerPort, CouponRedemptionPort};
use sdkwork_payment_providers::{PaymentProviderRegistry, ProviderCredentialBundle};

pub fn build_order_app_router(host: Arc<OrderServiceHost>) -> Router {
    mount_app_openapi(build_order_app_business_router(host))
}

/// Builds the complete Order app-api business surface without infrastructure/OpenAPI routes.
/// Gateway assemblies use this entrypoint when Order is embedded into a shared HTTP ingress.
pub fn build_order_app_business_router(host: Arc<OrderServiceHost>) -> Router {
    let credit_port = host.account_credit_port();
    let account_value_ledger_port = host.account_value_ledger_port();
    let coupon_redemption_port = host.coupon_redemption_port();
    let membership_port = host.membership_fulfillment_port();
    let credentials = ProviderCredentialBundle::from_env();
    let registry = Arc::new(PaymentProviderRegistry::from_credentials(
        credentials.clone(),
    ));
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
                .merge(build_recharge_router_postgres(
                    pool.clone(),
                    registry.clone(),
                    credentials.clone(),
                    coupon_redemption_port.clone(),
                    account_value_ledger_port.clone(),
                ))
                .merge(app_membership_order_router_with_postgres_pool_and_payments(
                    pool.clone(),
                    registry,
                    credentials,
                ))
                .merge(app_fulfillment_router_with_postgres_pool(pool.clone()))
                .merge(app_shipment_router_with_postgres_pool(pool.clone()))
                .merge(app_after_sales_router_with_postgres_pool(pool.clone()))
                .merge(app_payment_webhook_router_with_postgres_pool_and_coupon(
                    pool,
                    credit_port,
                    account_value_ledger_port,
                    coupon_redemption_port,
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
                .merge(build_recharge_router_sqlite(
                    pool.clone(),
                    registry.clone(),
                    credentials.clone(),
                    coupon_redemption_port.clone(),
                    account_value_ledger_port.clone(),
                ))
                .merge(app_membership_order_router_with_sqlite_pool_and_payments(
                    pool.clone(),
                    registry,
                    credentials,
                ))
                .merge(app_fulfillment_router_with_sqlite_pool(pool.clone()))
                .merge(app_shipment_router_with_sqlite_pool(pool.clone()))
                .merge(app_after_sales_router_with_sqlite_pool(pool.clone()))
                .merge(app_payment_webhook_router_with_sqlite_pool_and_coupon(
                    pool,
                    credit_port,
                    account_value_ledger_port,
                    coupon_redemption_port,
                    membership_port,
                ))
        }
    };
    router
}

fn build_recharge_router_sqlite(
    pool: sqlx::SqlitePool,
    registry: Arc<PaymentProviderRegistry>,
    credentials: ProviderCredentialBundle,
    coupon: Arc<dyn CouponRedemptionPort>,
    ledger: Arc<dyn AccountValueLedgerPort>,
) -> axum::Router {
    let store = Arc::new(SqliteCommerceRechargeStore::new(pool.clone()));
    build_app_recharge_checkout_router_with_integrations(
        store.clone(),
        store,
        coupon,
        ledger,
        Arc::new(SqliteCommerceOrderStore::new(pool.clone())),
        crate::owner_order_payment_enrich::enriched_sqlite_owner_order_payments(
            pool,
            registry,
            credentials,
        ),
    )
}

fn build_recharge_router_postgres(
    pool: sqlx::PgPool,
    registry: Arc<PaymentProviderRegistry>,
    credentials: ProviderCredentialBundle,
    coupon: Arc<dyn CouponRedemptionPort>,
    ledger: Arc<dyn AccountValueLedgerPort>,
) -> axum::Router {
    let store = Arc::new(PostgresCommerceRechargeStore::new(pool.clone()));
    build_app_recharge_checkout_router_with_integrations(
        store.clone(),
        store,
        coupon,
        ledger,
        Arc::new(PostgresCommerceOrderStore::new(pool.clone())),
        crate::owner_order_payment_enrich::enriched_postgres_owner_order_payments(
            pool,
            registry,
            credentials,
        ),
    )
}

pub async fn build_order_app_router_with_framework(host: Arc<OrderServiceHost>) -> Router {
    wrap_router_with_web_framework_from_env(build_order_app_router(host)).await
}

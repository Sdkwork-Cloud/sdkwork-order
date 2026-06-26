pub mod after_sales_router;
pub mod checkout_router;
pub mod command_headers;
pub mod fulfillment_router;
pub mod order_router;
pub mod routes;
pub mod shipment_router;
pub mod subject;
pub mod web_bootstrap;

pub use routes::build_order_app_router_with_framework;

pub use after_sales_router::{
    build_app_after_sales_router, app_after_sales_router_with_postgres_pool,
    app_after_sales_router_with_sqlite_pool, CommerceAfterSalesFuture, CommerceAfterSalesStore,
};
pub use checkout_router::{
    build_app_checkout_router, app_checkout_router_with_postgres_pool,
    app_checkout_router_with_sqlite_pool, CommerceCheckoutFuture, CommerceCheckoutStore,
};
pub use fulfillment_router::{
    build_app_fulfillment_router, app_fulfillment_router_with_postgres_pool,
    app_fulfillment_router_with_sqlite_pool, CommerceFulfillmentFuture, CommerceFulfillmentStore,
};
pub use order_router::{
    build_app_order_router, app_order_router_with_postgres_pool, app_order_router_with_sqlite_pool,
    CommerceOrderFuture, CommerceOrderStore, OwnerOrderPaymentStore,
};
pub use shipment_router::{
    build_app_shipment_router, app_shipment_router_with_postgres_pool,
    app_shipment_router_with_sqlite_pool, CommerceShipmentFuture, CommerceShipmentStore,
};

use axum::Router;
use sdkwork_order_service_host::OrderServiceHost;
use std::sync::Arc;

pub async fn gateway_mount(host: Arc<OrderServiceHost>) -> Router {
    build_order_app_router_with_framework(host).await
}

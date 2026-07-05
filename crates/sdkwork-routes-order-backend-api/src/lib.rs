pub mod api_response;
pub mod backend_order_admin_router;
pub mod http_route_manifest;
pub mod openapi_contract;
pub mod points_recharge_fulfillment_router;
pub mod routes;
pub mod subject;
pub mod web_bootstrap;

pub use backend_order_admin_router::{
    backend_order_admin_router_with_postgres_pool, backend_order_admin_router_with_sqlite_pool,
};
pub use points_recharge_fulfillment_router::{
    points_recharge_fulfillment_router_with_postgres_pool,
    points_recharge_fulfillment_router_with_sqlite_pool,
};
pub use routes::build_order_backend_router_with_framework;

use axum::Router;
use sdkwork_order_service_host::OrderServiceHost;
use std::sync::Arc;

pub async fn gateway_mount(host: Arc<OrderServiceHost>) -> Router {
    build_order_backend_router_with_framework(host).await
}

use axum::Router;
use sdkwork_router_order_app_api::build_order_app_router_with_framework;
use sdkwork_router_order_backend_api::build_order_backend_router_with_framework;
use sdkwork_order_api_server::order_health_router;
use sdkwork_order_service_host::OrderServiceHost;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let host = Arc::new(OrderServiceHost::new().await);
    let app = Router::new()
        .merge(order_health_router())
        .merge(build_order_app_router_with_framework(host.clone()).await)
        .merge(build_order_backend_router_with_framework(host).await)
        .layer(CorsLayer::permissive());
    let addr = std::env::var("ORDER_API_BIND").unwrap_or_else(|_| "0.0.0.0:18093".to_owned());
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

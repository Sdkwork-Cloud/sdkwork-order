use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use sdkwork_order_repository_sqlx::order_points_recharge_e2e_sqlite_memory_pool;
use sdkwork_order_service::{
    AccountPointsCreditPort, AccountPointsCreditFuture, PointsRechargeCreditOutcome,
    PointsRechargeCreditRequest,
};
use sdkwork_routes_order_backend_api::{
    backend_order_admin_router_with_sqlite_pool, openapi_contract::mount_backend_openapi,
    points_recharge_fulfillment_router_with_sqlite_pool,
};
use serde_json::Value;
use std::sync::Arc;
use tower::util::ServiceExt;

struct NoopAccountPointsCreditPort;

impl AccountPointsCreditPort for NoopAccountPointsCreditPort {
    fn credit_points_recharge<'a>(
        &'a self,
        _request: PointsRechargeCreditRequest,
    ) -> AccountPointsCreditFuture<'a, PointsRechargeCreditOutcome> {
        Box::pin(async move {
            Ok(PointsRechargeCreditOutcome {
                accepted: true,
                replayed: false,
            })
        })
    }
}

fn build_test_backend_router(pool: sqlx::SqlitePool) -> Router {
    mount_backend_openapi(
        Router::new()
            .merge(backend_order_admin_router_with_sqlite_pool(pool.clone()))
            .merge(points_recharge_fulfillment_router_with_sqlite_pool(
                pool,
                Arc::new(NoopAccountPointsCreditPort),
            )),
    )
}

#[tokio::test]
async fn backend_openapi_document_is_served() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    let app = build_test_backend_router(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn backend_router_mounts_every_openapi_operation_path() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../apis/backend-api/order/order-backend-api.openapi.json"
    ))
    .unwrap();
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    let app = build_test_backend_router(pool);
    let paths = spec["paths"].as_object().unwrap();

    for (template_path, methods) in paths {
        for method_name in methods.as_object().unwrap().keys() {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method_from_openapi(method_name))
                        .uri(concrete_uri(template_path))
                        .header("content-type", "application/json")
                        .body(Body::from("{}"))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_ne!(
                response.status(),
                StatusCode::NOT_FOUND,
                "{method_name} {template_path} is not mounted"
            );
        }
    }
}

fn method_from_openapi(method_name: &str) -> Method {
    match method_name.to_ascii_lowercase().as_str() {
        "get" => Method::GET,
        "post" => Method::POST,
        "put" => Method::PUT,
        "patch" => Method::PATCH,
        "delete" => Method::DELETE,
        other => panic!("unsupported openapi method: {other}"),
    }
}

fn concrete_uri(template_path: &str) -> String {
    template_path.replace("{orderId}", "order-1")
}

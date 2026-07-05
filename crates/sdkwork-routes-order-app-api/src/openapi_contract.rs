use std::sync::{Arc, OnceLock};

use axum::Router;
use sdkwork_web_bootstrap::{mount_openapi_json, OpenApiMount};
use serde_json::Value;

fn app_openapi_document() -> Arc<Value> {
    static DOCUMENT: OnceLock<Arc<Value>> = OnceLock::new();
    DOCUMENT
        .get_or_init(|| {
            Arc::new(
                serde_json::from_str(include_str!(
                    "../../../apis/app-api/order/order-app-api.openapi.json"
                ))
                .expect("parse app openapi authority"),
            )
        })
        .clone()
}

pub fn mount_app_openapi(router: Router) -> Router {
    mount_openapi_json(
        router,
        &[OpenApiMount {
            path: "/app/v3/api/openapi.json",
            document: app_openapi_document(),
        }],
    )
}

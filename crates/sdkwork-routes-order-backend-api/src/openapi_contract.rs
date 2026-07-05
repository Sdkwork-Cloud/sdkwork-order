use std::sync::{Arc, OnceLock};

use axum::Router;
use sdkwork_web_bootstrap::{mount_openapi_json, OpenApiMount};
use serde_json::Value;

fn backend_openapi_document() -> Arc<Value> {
    static DOCUMENT: OnceLock<Arc<Value>> = OnceLock::new();
    DOCUMENT
        .get_or_init(|| {
            Arc::new(
                serde_json::from_str(include_str!(
                    "../../../apis/backend-api/order/order-backend-api.openapi.json"
                ))
                .expect("parse backend openapi authority"),
            )
        })
        .clone()
}

pub fn mount_backend_openapi(router: Router) -> Router {
    mount_openapi_json(
        router,
        &[OpenApiMount {
            path: "/backend/v3/api/openapi.json",
            document: backend_openapi_document(),
        }],
    )
}

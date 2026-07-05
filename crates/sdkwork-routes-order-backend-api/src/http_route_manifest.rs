//! Order backend-api HTTP route manifest (`API_SPEC.md` §4.2.1, `WEB_BACKEND_SPEC.md` §4.2).

use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

pub const BACKEND_API_PREFIX: &str = "/backend/v3/api";

pub fn order_backend_api_public_path_prefixes() -> Vec<String> {
    sdkwork_web_bootstrap::infra_public_path_prefixes()
}

const HTTP_ROUTES: &[HttpRoute] = &[
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/backend/v3/api/orders",
        "orders",
        "orders.admin.list",
    ),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/backend/v3/api/orders/cancellations",
        "orders",
        "orders.admin.cancellations.list",
    ),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/backend/v3/api/orders/{orderId}",
        "orders",
        "orders.admin.retrieve",
    ),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/backend/v3/api/orders/{orderId}/cancel",
        "orders",
        "orders.admin.cancel",
    )
    .with_idempotent(true),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/backend/v3/api/orders/{orderId}/close",
        "orders",
        "orders.admin.close",
    )
    .with_idempotent(true),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/backend/v3/api/orders/{orderId}/events",
        "orders",
        "orders.admin.events.list",
    ),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/backend/v3/api/orders/{orderId}/payment_confirmations",
        "orders",
        "orders.paymentConfirmations.create",
    )
    .with_idempotent(true),
];

pub fn backend_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_web_core::{RouteAuth, WebRequestContextProfile};

    #[test]
    fn manifest_declares_all_routes_with_dual_token_auth() {
        let manifest = backend_route_manifest();
        for route in manifest.routes() {
            assert_eq!(route.auth, RouteAuth::DualToken);
            assert!(route.path.starts_with(BACKEND_API_PREFIX));
        }
    }

    #[test]
    fn manifest_passes_framework_validations() {
        let manifest = backend_route_manifest();
        let profile = WebRequestContextProfile {
            public_path_prefixes: order_backend_api_public_path_prefixes(),
            ..WebRequestContextProfile::default()
        };
        manifest
            .validate_public_path_prefixes(&profile.public_path_prefixes)
            .expect("public prefixes must not cover protected manifest routes");
        manifest
            .validate_route_auth_for_surfaces(&profile)
            .expect("all backend-api routes must declare dual-token auth");
        manifest
            .validate_no_ambient_context_path_markers(&profile)
            .expect("manifest must not embed ambient tenant/org scoping");
    }
}

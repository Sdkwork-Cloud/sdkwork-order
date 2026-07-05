//! Order app-api HTTP route manifest (`API_SPEC.md` §4.2.1, `WEB_FRAMEWORK_SPEC.md` §2/§7).

use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

pub const APP_API_PREFIX: &str = "/app/v3/api";

pub fn order_app_api_public_path_prefixes() -> Vec<String> {
    sdkwork_web_bootstrap::infra_public_path_prefixes()
}

const HTTP_ROUTES: &[HttpRoute] = &[
    // === Checkout ===
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/app/v3/api/checkout/sessions",
        "checkout",
        "checkout.sessions.create",
    )
    .with_idempotent(true),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/checkout/sessions/{checkoutSessionId}",
        "checkout",
        "checkout.sessions.retrieve",
    ),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/app/v3/api/checkout/sessions/{checkoutSessionId}/quotes",
        "checkout",
        "checkout.sessions.quotes.create",
    )
    .with_idempotent(true),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/app/v3/api/checkout/sessions/{checkoutSessionId}/orders",
        "checkout",
        "checkout.sessions.orders.create",
    )
    .with_idempotent(true),
    // === Orders ===
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/orders", "orders", "orders.list"),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/app/v3/api/orders",
        "orders",
        "orders.create",
    )
    .with_idempotent(true),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/orders/statistics",
        "orders",
        "orders.statistics.retrieve",
    ),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/orders/{orderId}",
        "orders",
        "orders.retrieve",
    ),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/orders/{orderId}/payments",
        "payments",
        "payments.orderPayments.list",
    ),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/app/v3/api/orders/{orderId}/payments",
        "orders",
        "orders.pay",
    )
    .with_idempotent(true),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/app/v3/api/orders/{orderId}/cancel",
        "orders",
        "orders.cancel",
    )
    .with_idempotent(true),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/orders/{orderId}/status",
        "orders",
        "orders.status.retrieve",
    ),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/orders/{orderId}/payment_success",
        "orders",
        "orders.paymentSuccess.retrieve",
    ),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/orders/{orderId}/events",
        "orders",
        "orders.events.list",
    ),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/app/v3/api/orders/{orderId}/cancellations",
        "orders",
        "orders.cancellations.create",
    )
    .with_idempotent(true),
    // === After-sales ===
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/after_sales/requests",
        "afterSales",
        "afterSales.requests.list",
    ),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/app/v3/api/after_sales/requests",
        "afterSales",
        "afterSales.requests.create",
    )
    .with_idempotent(true),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/after_sales/requests/{afterSalesRequestId}",
        "afterSales",
        "afterSales.requests.retrieve",
    ),
    HttpRoute::dual_token(
        HttpMethod::Put,
        "/app/v3/api/after_sales/requests/{afterSalesRequestId}",
        "afterSales",
        "afterSales.requests.update",
    )
    .with_idempotent(true),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/after_sales/requests/{afterSalesRequestId}/events",
        "afterSales",
        "afterSales.events.list",
    ),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/after_sales/requests/{afterSalesRequestId}/return_shipments",
        "afterSales",
        "afterSales.returnShipments.list",
    ),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/app/v3/api/after_sales/requests/{afterSalesRequestId}/return_shipments",
        "afterSales",
        "afterSales.returnShipments.create",
    )
    .with_idempotent(true),
    // === Fulfillment / shipment ===
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/fulfillments",
        "fulfillments",
        "fulfillments.list",
    ),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/fulfillments/{fulfillmentId}",
        "fulfillments",
        "fulfillments.retrieve",
    ),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/shipments/{shipmentId}",
        "shipments",
        "shipments.retrieve",
    ),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/shipments/{shipmentId}/packages",
        "shipments",
        "shipments.packages.list",
    ),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/shipments/{shipmentId}/tracking_events",
        "shipments",
        "shipments.trackingEvents.list",
    ),
    // === Recharges ===
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/recharges/packages",
        "recharges",
        "recharges.packages.list",
    ),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/recharges/settings",
        "recharges",
        "recharges.settings.retrieve",
    ),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/recharges/orders",
        "recharges",
        "recharges.orders.list",
    ),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/app/v3/api/recharges/orders",
        "recharges",
        "recharges.orders.create",
    )
    .with_idempotent(true),
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/recharges/orders/{orderId}",
        "recharges",
        "recharges.orders.retrieve",
    ),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/app/v3/api/recharges/orders/{orderId}/cancel",
        "recharges",
        "recharges.orders.cancel",
    )
    .with_idempotent(true),
];

pub fn app_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_web_core::{RouteAuth, WebRequestContextProfile};

    #[test]
    fn manifest_declares_all_routes_with_dual_token_auth() {
        let manifest = app_route_manifest();
        assert!(!manifest.routes().is_empty());
        for route in manifest.routes() {
            assert_eq!(
                route.auth,
                RouteAuth::DualToken,
                "route {:?} {} must use dual-token auth",
                route.method,
                route.path,
            );
            assert!(
                route.path.starts_with(APP_API_PREFIX),
                "route {:?} {} must start with {APP_API_PREFIX}",
                route.method,
                route.path,
            );
        }
    }

    #[test]
    fn manifest_passes_framework_validations() {
        let manifest = app_route_manifest();
        let profile = WebRequestContextProfile {
            public_path_prefixes: order_app_api_public_path_prefixes(),
            ..WebRequestContextProfile::default()
        };
        manifest
            .validate_public_path_prefixes(&profile.public_path_prefixes)
            .expect("public prefixes must not cover protected manifest routes");
        manifest
            .validate_route_auth_for_surfaces(&profile)
            .expect("all app-api routes must declare dual-token auth");
        manifest
            .validate_no_ambient_context_path_markers(&profile)
            .expect("manifest must not embed ambient tenant/org scoping");
    }
}

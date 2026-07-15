use axum::Router;
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{CorsPolicy, SecurityPolicy, WebEnvironment, WebRequestContextProfile};

use crate::http_route_manifest::{app_route_manifest, order_app_api_public_path_prefixes};

pub fn wrap_router_with_web_framework(
    resolver: IamWebRequestContextResolver,
    router: Router,
) -> Router {
    let route_manifest = app_route_manifest();
    route_manifest
        .validate_public_path_prefixes(&order_app_api_public_path_prefixes())
        .expect("order app-api public prefixes must not cover protected manifest routes");

    let environment = order_web_environment_from_env();
    let security_policy = order_web_security_policy(&environment);
    let layer = WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile {
            public_path_prefixes: order_app_api_public_path_prefixes(),
            environment,
            ..WebRequestContextProfile::default()
        })
        .with_security_policy(security_policy)
        .with_route_manifest(route_manifest);
    with_web_request_context(router, layer)
}

pub async fn wrap_router_with_web_framework_from_env(router: Router) -> Router {
    let resolver = sdkwork_iam_web_adapter::iam_web_request_context_resolver_from_env().await;
    wrap_router_with_web_framework(resolver, router)
}

fn order_web_environment_from_env() -> WebEnvironment {
    for key in [
        "SDKWORK_ORDER_ENVIRONMENT",
        "ORDER_ENVIRONMENT",
        "SDKWORK_ENVIRONMENT",
        "SDKWORK_ENV",
    ] {
        let Some(value) = std::env::var(key)
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        return match value.as_str() {
            "dev" | "development" | "local" => WebEnvironment::Dev,
            "test" | "testing" => WebEnvironment::Test,
            _ => WebEnvironment::Prod,
        };
    }
    WebEnvironment::Prod
}

fn order_web_security_policy(environment: &WebEnvironment) -> SecurityPolicy {
    if matches!(environment, WebEnvironment::Dev | WebEnvironment::Test) {
        let mut policy = SecurityPolicy::default();
        policy.cors = CorsPolicy::development_private_network();
        policy.cors.allowed_headers.extend(
            [
                "sdkwork-request-hash",
                "sdkwork-request-no",
                "traceparent",
                "tracestate",
                "x-idempotency-fingerprint",
                "x-sdkwork-locale",
            ]
            .into_iter()
            .map(str::to_owned),
        );
        policy.cross_site.reject_untrusted_state_changing_origins = false;
        policy.cross_site.reject_cookie_auth_without_origin = false;
        policy
    } else {
        SecurityPolicy::production()
    }
}

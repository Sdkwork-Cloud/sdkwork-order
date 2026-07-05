//! Combined HTTP contract fallback for app-api and backend-api route manifests.

use sdkwork_routes_order_app_api::http_route_manifest::app_route_manifest;
use sdkwork_routes_order_backend_api::http_route_manifest::backend_route_manifest;
use sdkwork_web_bootstrap::ContractFallbackConfig;

/// Builds a merged contract fallback config for the standalone order gateway.
pub fn order_contract_fallback_config() -> ContractFallbackConfig {
    let mut config = ContractFallbackConfig::from_manifest(&app_route_manifest());
    let backend = ContractFallbackConfig::from_manifest(&backend_route_manifest());
    config.manifest_paths.extend(backend.manifest_paths);
    config
}

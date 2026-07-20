//! Gateway assembly for sdkwork-order.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM

mod bootstrap;
mod generated;

pub use bootstrap::{assemble_api_router, assemble_backend_business_router, ApiAssembly};

pub async fn assemble_api_router_from_env() -> Result<ApiAssembly, String> {
    let host = std::sync::Arc::new(sdkwork_order_service_host::OrderServiceHost::from_env().await?);
    Ok(assemble_api_router(host).await)
}

pub async fn assemble_backend_business_router_from_env() -> Result<ApiAssembly, String> {
    let host = std::sync::Arc::new(sdkwork_order_service_host::OrderServiceHost::from_env().await?);
    Ok(assemble_backend_business_router(host).await)
}

pub fn order_contract_fallback_config() -> sdkwork_web_bootstrap::ContractFallbackConfig {
    ApiAssembly::contract_fallback_config()
}

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}

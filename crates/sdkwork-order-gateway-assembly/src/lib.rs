//! Gateway assembly for sdkwork-order.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.

mod bootstrap;
mod contract_fallback;
mod generated;

pub use bootstrap::{assemble_application_router, ApplicationAssembly};
pub use contract_fallback::order_contract_fallback_config;

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}

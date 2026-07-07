mod http_adapter;

pub use http_adapter::HttpMembershipPurchaseFulfillmentAdapter;

use std::sync::Arc;

use sdkwork_order_service::{
    MembershipPurchaseFulfillmentPort, NoopMembershipPurchaseFulfillmentPort,
};

/// Builds the membership fulfillment port from environment.
///
/// - `SDKWORK_ORDER_MEMBERSHIP_FULFILLMENT_ADAPTER=http` (default when origin set)
/// - `SDKWORK_ORDER_MEMBERSHIP_FULFILLMENT_ADAPTER=noop` — external fulfillment only
pub fn membership_purchase_fulfillment_port_from_env(
) -> Result<Arc<dyn MembershipPurchaseFulfillmentPort>, String> {
    let mode = std::env::var("SDKWORK_ORDER_MEMBERSHIP_FULFILLMENT_ADAPTER")
        .unwrap_or_else(|_| "http".to_owned())
        .trim()
        .to_ascii_lowercase();

    match mode.as_str() {
        "noop" => Ok(Arc::new(NoopMembershipPurchaseFulfillmentPort)),
        "http" => {
            let origin = std::env::var("SDKWORK_MEMBERSHIP_BACKEND_API_ORIGIN")
                .or_else(|_| std::env::var("SDKWORK_MEMBERSHIP_API_ORIGIN"))
                .unwrap_or_else(|_| "http://127.0.0.1:18096".to_owned());
            if origin.trim().is_empty() {
                return Ok(Arc::new(NoopMembershipPurchaseFulfillmentPort));
            }
            Ok(Arc::new(HttpMembershipPurchaseFulfillmentAdapter::new(
                origin.trim().trim_end_matches('/').to_owned(),
                std::env::var("SDKWORK_ACCESS_TOKEN")
                    .ok()
                    .map(|value| value.trim().to_owned())
                    .filter(|value| !value.is_empty()),
            )))
        }
        other => Err(format!(
            "unsupported SDKWORK_ORDER_MEMBERSHIP_FULFILLMENT_ADAPTER value: {other}"
        )),
    }
}

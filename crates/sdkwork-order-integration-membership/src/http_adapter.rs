use std::sync::OnceLock;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_order_service::{
    MembershipPurchaseFulfillmentFuture, MembershipPurchaseFulfillmentOutcome,
    MembershipPurchaseFulfillmentPort, MembershipPurchaseFulfillmentRequest,
};
use sdkwork_utils_rust::SdkWorkProblemDetail;
use serde::Deserialize;

static HTTP: OnceLock<reqwest::Client> = OnceLock::new();

fn http_client() -> &'static reqwest::Client {
    HTTP.get_or_init(reqwest::Client::new)
}

#[derive(Clone, Debug)]
pub struct HttpMembershipPurchaseFulfillmentAdapter {
    origin: String,
    auth_token: Option<String>,
}

impl HttpMembershipPurchaseFulfillmentAdapter {
    pub fn new(origin: String, auth_token: Option<String>) -> Self {
        Self { origin, auth_token }
    }
}

impl MembershipPurchaseFulfillmentPort for HttpMembershipPurchaseFulfillmentAdapter {
    fn fulfill_membership_purchase<'a>(
        &'a self,
        request: MembershipPurchaseFulfillmentRequest,
    ) -> MembershipPurchaseFulfillmentFuture<'a, MembershipPurchaseFulfillmentOutcome> {
        Box::pin(async move { self.post_fulfillment(request).await })
    }
}

impl HttpMembershipPurchaseFulfillmentAdapter {
    async fn post_fulfillment(
        &self,
        request: MembershipPurchaseFulfillmentRequest,
    ) -> Result<MembershipPurchaseFulfillmentOutcome, CommerceServiceError> {
        if self.auth_token.is_none()
            && std::env::var("SDKWORK_ORDER_MEMBERSHIP_FULFILL_ALLOW_INSECURE").as_deref()
                != Ok("1")
        {
            return Err(CommerceServiceError::storage(
                "membership fulfillment requires SDKWORK_ACCESS_TOKEN",
            ));
        }

        let body = serde_json::json!({
            "tenantId": request.tenant_id,
            "organizationId": request.organization_id,
            "ownerUserId": request.owner_user_id,
            "orderId": request.order_id,
            "requestNo": request.request_no,
            "idempotencyKey": request.idempotency_key,
        });
        let url = format!(
            "{}/backend/v3/api/memberships/purchases/fulfillments",
            self.origin
        );

        let mut builder = http_client()
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .json(&body);
        if let Some(token) = &self.auth_token {
            builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
        }

        let response = builder.send().await.map_err(|error| {
            CommerceServiceError::storage(format!("membership fulfillment request failed: {error}"))
        })?;

        let status = response.status();
        if status.is_success() {
            let payload = response
                .json::<FulfillmentEnvelope>()
                .await
                .map_err(|error| {
                    CommerceServiceError::storage(format!(
                        "membership fulfillment response decode failed: {error}"
                    ))
                })?;
            let item = payload.data.item.ok_or_else(|| {
                CommerceServiceError::storage("membership fulfillment response missing data.item")
            })?;
            return Ok(MembershipPurchaseFulfillmentOutcome {
                accepted: item.accepted,
                replayed: item.replayed,
                fulfillment_status: item.fulfillment_status,
            });
        }

        if let Ok(problem) = response.json::<SdkWorkProblemDetail>().await {
            return Err(map_problem_detail(problem));
        }

        Err(CommerceServiceError::storage(format!(
            "membership fulfillment returned HTTP {status}"
        )))
    }
}

fn map_problem_detail(problem: SdkWorkProblemDetail) -> CommerceServiceError {
    let message = problem.detail.unwrap_or_else(|| problem.title.clone());
    match problem.code {
        40401 => CommerceServiceError::not_found(message),
        40901 => CommerceServiceError::conflict(message),
        40001..=40004 => CommerceServiceError::validation(message),
        40101..=40104 => CommerceServiceError::unauthorized(message),
        _ => CommerceServiceError::storage(message),
    }
}

#[derive(Debug, Deserialize)]
struct FulfillmentEnvelope {
    data: FulfillmentData,
}

#[derive(Debug, Deserialize)]
struct FulfillmentData {
    item: Option<FulfillmentItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FulfillmentItem {
    accepted: bool,
    replayed: bool,
    fulfillment_status: String,
}

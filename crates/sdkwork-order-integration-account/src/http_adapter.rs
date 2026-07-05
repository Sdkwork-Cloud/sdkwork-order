use std::sync::OnceLock;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_order_service::{
    AccountPointsCreditPort, AccountPointsCreditFuture, PointsRechargeCreditOutcome,
    PointsRechargeCreditRequest, POINTS_RECHARGE_LEDGER_BUSINESS_TYPE,
};
use sdkwork_utils_rust::SdkWorkProblemDetail;
use serde::Deserialize;

static HTTP: OnceLock<reqwest::Client> = OnceLock::new();

fn http_client() -> &'static reqwest::Client {
    HTTP.get_or_init(reqwest::Client::new)
}

#[derive(Clone, Debug)]
pub struct HttpAccountPointsCreditAdapter {
    origin: String,
    auth_token: Option<String>,
}

impl HttpAccountPointsCreditAdapter {
    pub fn from_env() -> Result<Self, String> {
        let origin = std::env::var("SDKWORK_ACCOUNT_BACKEND_API_ORIGIN")
            .unwrap_or_else(|_| "http://127.0.0.1:18095".to_owned())
            .trim()
            .trim_end_matches('/')
            .to_owned();
        let auth_token = std::env::var("SDKWORK_ORDER_ACCOUNT_SERVICE_AUTH_TOKEN")
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        Ok(Self { origin, auth_token })
    }
}

impl AccountPointsCreditPort for HttpAccountPointsCreditAdapter {
    fn credit_points_recharge<'a>(
        &'a self,
        request: PointsRechargeCreditRequest,
    ) -> AccountPointsCreditFuture<'a, PointsRechargeCreditOutcome> {
        Box::pin(async move { self.post_points_adjustment(request).await })
    }
}

impl HttpAccountPointsCreditAdapter {
    async fn post_points_adjustment(
        &self,
        request: PointsRechargeCreditRequest,
    ) -> Result<PointsRechargeCreditOutcome, CommerceServiceError> {
        let body = serde_json::json!({
            "tenantId": request.tenant_id,
            "organizationId": request.organization_id,
            "ownerUserId": request.owner_user_id,
            "direction": "credit",
            "amount": request.points.to_string(),
            "businessType": POINTS_RECHARGE_LEDGER_BUSINESS_TYPE,
            "transactionNo": request.transaction_no,
            "requestNo": request.request_no,
            "idempotencyKey": request.idempotency_key,
        });
        let url = format!(
            "{}/backend/v3/api/wallet/adjustments/points",
            self.origin
        );

        if self.auth_token.is_none()
            && std::env::var("SDKWORK_ORDER_ACCOUNT_CREDIT_ALLOW_INSECURE").as_deref() != Ok("1")
        {
            return Err(CommerceServiceError::storage(
                "SDKWORK_ORDER_ACCOUNT_SERVICE_AUTH_TOKEN is required for account points credit; set SDKWORK_ORDER_ACCOUNT_CREDIT_ALLOW_INSECURE=1 only for local development",
            ));
        }

        let mut builder = http_client()
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&body);

        if let Some(token) = &self.auth_token {
            builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
        }

        let response = builder.send().await.map_err(|error| {
            CommerceServiceError::storage(format!(
                "account backend points adjustment request failed: {error}"
            ))
        })?;

        let status = response.status();
        if status.is_success() {
            let envelope = response.json::<PointsAdjustmentEnvelope>().await.map_err(|error| {
                CommerceServiceError::storage(format!(
                    "account backend points adjustment response is invalid: {error}"
                ))
            })?;
            return Ok(PointsRechargeCreditOutcome {
                accepted: envelope.data.item.accepted,
                replayed: envelope.data.item.replayed,
            });
        }

        if let Ok(problem) = response.json::<SdkWorkProblemDetail>().await {
            return Err(map_problem_detail(problem));
        }

        Err(CommerceServiceError::storage(format!(
            "account backend points adjustment failed with HTTP {status}"
        )))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PointsAdjustmentEnvelope {
    #[allow(dead_code)]
    code: i32,
    data: PointsAdjustmentData,
    #[allow(dead_code)]
    trace_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PointsAdjustmentData {
    item: WalletAdjustmentItem,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WalletAdjustmentItem {
    accepted: bool,
    replayed: bool,
}

fn map_problem_detail(problem: SdkWorkProblemDetail) -> CommerceServiceError {
    let message = problem.detail.unwrap_or_else(|| problem.title.clone());
    match problem.code {
        40401 => CommerceServiceError::not_found(message),
        40901 => CommerceServiceError::conflict(message),
        40001 | 40002 | 40003 | 40004 => CommerceServiceError::validation(message),
        40101 | 40102 | 40103 | 40104 => CommerceServiceError::unauthorized(message),
        _ => CommerceServiceError::storage(message),
    }
}

use std::sync::OnceLock;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use sdkwork_contract_service::{
    CommerceLedgerBusinessType, CommerceLedgerDirection, CommerceServiceError,
};
use sdkwork_order_service::{
    AccountPointsCreditFuture, AccountPointsCreditPort, AccountValueAssetCode, AccountValueFuture,
    AccountValueLedgerCommand, AccountValueLedgerOperation, AccountValueLedgerOutcome,
    AccountValueLedgerPort, PointsRechargeCreditOutcome, PointsRechargeCreditRequest,
    POINTS_RECHARGE_LEDGER_BUSINESS_TYPE,
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
    pub fn new(origin: String, auth_token: Option<String>) -> Self {
        Self {
            origin: origin.trim().trim_end_matches('/').to_owned(),
            auth_token: auth_token
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
        }
    }

    pub fn from_env() -> Result<Self, String> {
        let origin = std::env::var("SDKWORK_ACCOUNT_BACKEND_API_ORIGIN")
            .unwrap_or_else(|_| "http://127.0.0.1:18095".to_owned())
            .trim()
            .trim_end_matches('/')
            .to_owned();
        let auth_token = std::env::var("SDKWORK_ACCESS_TOKEN")
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        Ok(Self::new(origin, auth_token))
    }
}

impl AccountPointsCreditPort for HttpAccountPointsCreditAdapter {
    fn credit_points_recharge<'a>(
        &'a self,
        request: PointsRechargeCreditRequest,
    ) -> AccountPointsCreditFuture<'a, PointsRechargeCreditOutcome> {
        Box::pin(async move { self.post_points_adjustment(request, "credit").await })
    }

    fn reverse_points_recharge_credit<'a>(
        &'a self,
        request: PointsRechargeCreditRequest,
    ) -> AccountPointsCreditFuture<'a, PointsRechargeCreditOutcome> {
        Box::pin(async move { self.post_points_adjustment(request, "debit").await })
    }
}

impl AccountValueLedgerPort for HttpAccountPointsCreditAdapter {
    fn apply_account_value_ledger_command<'a>(
        &'a self,
        command: AccountValueLedgerCommand,
    ) -> AccountValueFuture<'a, AccountValueLedgerOutcome> {
        Box::pin(async move {
            match command.operation {
                AccountValueLedgerOperation::Credit
                | AccountValueLedgerOperation::Debit
                | AccountValueLedgerOperation::Reversal => {
                    self.post_account_value_adjustment(command).await
                }
                AccountValueLedgerOperation::Hold => self.post_account_value_hold(command).await,
                AccountValueLedgerOperation::HoldSettle => {
                    self.post_account_value_hold_settle(command).await
                }
                AccountValueLedgerOperation::HoldRelease => {
                    self.post_account_value_hold_release(command).await
                }
            }
        })
    }
}

impl HttpAccountPointsCreditAdapter {
    async fn post_points_adjustment(
        &self,
        request: PointsRechargeCreditRequest,
        direction: &str,
    ) -> Result<PointsRechargeCreditOutcome, CommerceServiceError> {
        let body = serde_json::json!({
            "tenantId": request.tenant_id,
            "organizationId": request.organization_id,
            "ownerUserId": request.owner_user_id,
            "direction": direction,
            "amount": request.points.to_string(),
            "businessType": POINTS_RECHARGE_LEDGER_BUSINESS_TYPE,
            "transactionNo": request.transaction_no,
            "requestNo": request.request_no,
            "idempotencyKey": request.idempotency_key,
        });
        let url = format!("{}/backend/v3/api/wallet/adjustments/points", self.origin);

        if self.auth_token.is_none()
            && std::env::var("SDKWORK_ORDER_ACCOUNT_CREDIT_ALLOW_INSECURE").as_deref() != Ok("1")
        {
            return Err(CommerceServiceError::storage(
                "SDKWORK_ACCESS_TOKEN is required for account points credit; set SDKWORK_ORDER_ACCOUNT_CREDIT_ALLOW_INSECURE=1 only for local development",
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
            let envelope = response
                .json::<PointsAdjustmentEnvelope>()
                .await
                .map_err(|error| {
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

    async fn post_account_value_adjustment(
        &self,
        command: AccountValueLedgerCommand,
    ) -> Result<AccountValueLedgerOutcome, CommerceServiceError> {
        let transaction_no = account_value_transaction_no(&command);
        let path = account_value_adjustment_path(
            command.asset,
            &command.direction,
            &command.business_type,
        );
        let body = serde_json::json!({
            "tenantId": command.tenant_id,
            "organizationId": command.organization_id,
            "ownerUserId": command.owner_user_id,
            "assetType": command.asset.as_str(),
            "currencyCode": command.currency_code,
            "direction": command.direction.as_str(),
            "amount": command.amount.as_str(),
            "businessType": command.business_type,
            "transactionNo": transaction_no,
            "requestNo": command.request_no,
            "idempotencyKey": command.idempotency_key,
        });
        let url = format!("{}{}", self.origin, path);

        if self.auth_token.is_none()
            && std::env::var("SDKWORK_ORDER_ACCOUNT_CREDIT_ALLOW_INSECURE").as_deref() != Ok("1")
        {
            return Err(CommerceServiceError::storage(
                "SDKWORK_ACCESS_TOKEN is required for account value ledger commands; set SDKWORK_ORDER_ACCOUNT_CREDIT_ALLOW_INSECURE=1 only for local development",
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
                "account backend account value adjustment request failed: {error}"
            ))
        })?;

        let status = response.status();
        if status.is_success() {
            let envelope = response
                .json::<PointsAdjustmentEnvelope>()
                .await
                .map_err(|error| {
                    CommerceServiceError::storage(format!(
                        "account backend account value adjustment response is invalid: {error}"
                    ))
                })?;
            return Ok(AccountValueLedgerOutcome {
                accepted: envelope.data.item.accepted,
                replayed: envelope.data.item.replayed,
                ledger_entry_id: envelope.data.item.ledger_entry.map(|entry| entry.id),
                account_effect_reference_id: None,
            });
        }

        if let Ok(problem) = response.json::<SdkWorkProblemDetail>().await {
            return Err(map_problem_detail(problem));
        }

        Err(CommerceServiceError::storage(format!(
            "account backend account value adjustment failed with HTTP {status}"
        )))
    }

    async fn post_account_value_hold(
        &self,
        command: AccountValueLedgerCommand,
    ) -> Result<AccountValueLedgerOutcome, CommerceServiceError> {
        let body = serde_json::json!({
            "tenantId": command.tenant_id,
            "organizationId": command.organization_id,
            "ownerUserId": command.owner_user_id,
            "assetType": command.asset.as_str(),
            "amount": command.amount.as_str(),
            "businessType": command.business_type,
            "businessNo": command.resource_id,
            "sourceType": "commerce_order_request",
            "sourceId": stable_numeric_source_id(&command.resource_id),
            "requestNo": command.request_no,
            "idempotencyKey": command.idempotency_key,
        });
        let path = account_value_hold_create_path(command.asset);
        self.post_account_value_hold_command(path, body).await
    }

    async fn post_account_value_hold_settle(
        &self,
        command: AccountValueLedgerCommand,
    ) -> Result<AccountValueLedgerOutcome, CommerceServiceError> {
        let transaction_no = account_value_transaction_no(&command);
        let body = serde_json::json!({
            "tenantId": command.tenant_id,
            "businessType": command.business_type,
            "transactionNo": transaction_no,
            "requestNo": command.request_no,
            "idempotencyKey": command.idempotency_key,
        });
        let path = account_value_hold_mutation_path(
            command.asset,
            &command.resource_id,
            AccountValueLedgerOperation::HoldSettle,
        );
        self.post_account_value_hold_command(path, body).await
    }

    async fn post_account_value_hold_release(
        &self,
        command: AccountValueLedgerCommand,
    ) -> Result<AccountValueLedgerOutcome, CommerceServiceError> {
        let body = serde_json::json!({
            "tenantId": command.tenant_id,
            "requestNo": command.request_no,
            "idempotencyKey": command.idempotency_key,
        });
        let path = account_value_hold_mutation_path(
            command.asset,
            &command.resource_id,
            AccountValueLedgerOperation::HoldRelease,
        );
        self.post_account_value_hold_command(path, body).await
    }

    async fn post_account_value_hold_command(
        &self,
        path: String,
        body: serde_json::Value,
    ) -> Result<AccountValueLedgerOutcome, CommerceServiceError> {
        let url = format!("{}{}", self.origin, path);
        if self.auth_token.is_none()
            && std::env::var("SDKWORK_ORDER_ACCOUNT_CREDIT_ALLOW_INSECURE").as_deref() != Ok("1")
        {
            return Err(CommerceServiceError::storage(
                "SDKWORK_ACCESS_TOKEN is required for account value hold commands; set SDKWORK_ORDER_ACCOUNT_CREDIT_ALLOW_INSECURE=1 only for local development",
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
                "account backend account value hold request failed: {error}"
            ))
        })?;

        let status = response.status();
        if status.is_success() {
            let envelope = response
                .json::<HoldMutationEnvelope>()
                .await
                .map_err(|error| {
                    CommerceServiceError::storage(format!(
                        "account backend account value hold response is invalid: {error}"
                    ))
                })?;
            return Ok(AccountValueLedgerOutcome {
                accepted: envelope.data.item.accepted,
                replayed: envelope.data.item.replayed,
                ledger_entry_id: envelope.data.item.ledger_entry.map(|entry| entry.id),
                account_effect_reference_id: Some(envelope.data.item.hold.uuid),
            });
        }

        if let Ok(problem) = response.json::<SdkWorkProblemDetail>().await {
            return Err(map_problem_detail(problem));
        }

        Err(CommerceServiceError::storage(format!(
            "account backend account value hold failed with HTTP {status}"
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
    #[serde(default)]
    ledger_entry: Option<WalletAdjustmentLedgerEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WalletAdjustmentLedgerEntry {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HoldMutationEnvelope {
    #[allow(dead_code)]
    code: i32,
    data: HoldMutationData,
    #[allow(dead_code)]
    trace_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HoldMutationData {
    item: HoldMutationItem,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HoldMutationItem {
    accepted: bool,
    replayed: bool,
    hold: HoldItem,
    #[serde(default)]
    ledger_entry: Option<WalletAdjustmentLedgerEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HoldItem {
    uuid: String,
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

fn account_value_transaction_no(command: &AccountValueLedgerCommand) -> String {
    format!("{}:{}", command.business_type, command.resource_id)
}

fn account_value_hold_create_path(asset: AccountValueAssetCode) -> String {
    match asset {
        AccountValueAssetCode::TokenBank => "/backend/v3/api/token_bank/holds".to_owned(),
        AccountValueAssetCode::Cash | AccountValueAssetCode::Points => {
            "/backend/v3/api/wallet/holds".to_owned()
        }
    }
}

fn account_value_hold_mutation_path(
    asset: AccountValueAssetCode,
    hold_id: &str,
    operation: AccountValueLedgerOperation,
) -> String {
    let suffix = match operation {
        AccountValueLedgerOperation::HoldSettle => "settle",
        AccountValueLedgerOperation::HoldRelease => "release",
        _ => unreachable!("only hold mutations have hold-id paths"),
    };
    match asset {
        AccountValueAssetCode::TokenBank => {
            format!("/backend/v3/api/token_bank/holds/{hold_id}/{suffix}")
        }
        AccountValueAssetCode::Cash | AccountValueAssetCode::Points => {
            format!("/backend/v3/api/wallet/holds/{hold_id}/{suffix}")
        }
    }
}

fn account_value_adjustment_path(
    asset: AccountValueAssetCode,
    direction: &CommerceLedgerDirection,
    business_type: &str,
) -> &'static str {
    match asset {
        AccountValueAssetCode::TokenBank => {
            if business_type == CommerceLedgerBusinessType::TOKEN_BANK_GRANT {
                "/backend/v3/api/token_bank/grants"
            } else if business_type == CommerceLedgerBusinessType::TOKEN_BANK_REVERSAL {
                "/backend/v3/api/token_bank/reversals"
            } else if direction == &CommerceLedgerDirection::Debit {
                "/backend/v3/api/token_bank/debits"
            } else {
                "/backend/v3/api/token_bank/credits"
            }
        }
        AccountValueAssetCode::Points => "/backend/v3/api/wallet/adjustments/points",
        AccountValueAssetCode::Cash => "/backend/v3/api/wallet/adjustments/cash",
    }
}

fn stable_numeric_source_id(value: &str) -> String {
    let digest = sdkwork_utils_rust::sha256_hash(value.trim().as_bytes());
    let prefix = digest.get(..15).unwrap_or(digest.as_str());
    u64::from_str_radix(prefix, 16).unwrap_or(0).to_string()
}

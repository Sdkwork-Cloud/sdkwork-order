use sdkwork_account_repository_sqlx::{
    PostgresCommerceAccountStore, SqliteCommerceAccountStore,
};
use sdkwork_account_service::AppendLedgerEntryCommand;
use sdkwork_contract_service::{
    CommerceAccountAssetType, CommerceLedgerDirection, CommerceMoney, CommerceRequestHash,
    CommerceServiceError,
};
use sdkwork_order_service::{
    AccountPointsCreditPort, AccountPointsCreditFuture, PointsRechargeCreditOutcome,
    PointsRechargeCreditRequest, POINTS_RECHARGE_LEDGER_BUSINESS_TYPE,
};
use sdkwork_utils_rust::sha256_hash;
use sqlx::{PgPool, SqlitePool};

enum StoreKind {
    Sqlite(SqliteCommerceAccountStore),
    Postgres(PostgresCommerceAccountStore),
}

pub struct StoreAccountPointsCreditAdapter {
    store: StoreKind,
}

impl StoreAccountPointsCreditAdapter {
    pub fn sqlite(pool: SqlitePool) -> Self {
        Self {
            store: StoreKind::Sqlite(SqliteCommerceAccountStore::new(pool)),
        }
    }

    pub fn postgres(pool: PgPool) -> Self {
        Self {
            store: StoreKind::Postgres(PostgresCommerceAccountStore::new(pool)),
        }
    }
}

impl AccountPointsCreditPort for StoreAccountPointsCreditAdapter {
    fn credit_points_recharge<'a>(
        &'a self,
        request: PointsRechargeCreditRequest,
    ) -> AccountPointsCreditFuture<'a, PointsRechargeCreditOutcome> {
        Box::pin(async move { self.append_points_adjustment(request, CommerceLedgerDirection::Credit).await })
    }

    fn reverse_points_recharge_credit<'a>(
        &'a self,
        request: PointsRechargeCreditRequest,
    ) -> AccountPointsCreditFuture<'a, PointsRechargeCreditOutcome> {
        Box::pin(async move { self.append_points_adjustment(request, CommerceLedgerDirection::Debit).await })
    }
}

impl StoreAccountPointsCreditAdapter {
    async fn append_points_adjustment(
        &self,
        request: PointsRechargeCreditRequest,
        direction: CommerceLedgerDirection,
    ) -> Result<PointsRechargeCreditOutcome, CommerceServiceError> {
        let points_text = request.points.to_string();
        let amount = CommerceMoney::new(&points_text)
            .map_err(|error| CommerceServiceError::validation(error.to_string()))?;
        let direction_text = match direction {
            CommerceLedgerDirection::Credit => "credit",
            CommerceLedgerDirection::Debit => "debit",
        };
        let request_hash = ledger_request_hash(&request, direction_text)?;
        let command = AppendLedgerEntryCommand::new(
            &request.tenant_id,
            request.organization_id.as_deref(),
            "",
            &request.owner_user_id,
            CommerceAccountAssetType::Points,
            Some("POINT"),
            direction,
            amount,
            POINTS_RECHARGE_LEDGER_BUSINESS_TYPE,
            &request.transaction_no,
            &request.request_no,
            &request.idempotency_key,
        )?;
        let outcome = match &self.store {
            StoreKind::Sqlite(store) => store.append_ledger_entry(command, request_hash).await?,
            StoreKind::Postgres(store) => store.append_ledger_entry(command, request_hash).await?,
        };
        Ok(PointsRechargeCreditOutcome {
            accepted: true,
            replayed: outcome.replayed,
        })
    }
}

fn ledger_request_hash(
    request: &PointsRechargeCreditRequest,
    direction: &str,
) -> Result<CommerceRequestHash, CommerceServiceError> {
    let body = serde_json::json!({
        "tenantId": request.tenant_id,
        "organizationId": request.organization_id,
        "ownerUserId": request.owner_user_id,
        "assetType": "points",
        "currencyCode": "POINT",
        "direction": direction,
        "amount": request.points.to_string(),
        "businessType": POINTS_RECHARGE_LEDGER_BUSINESS_TYPE,
        "transactionNo": request.transaction_no,
        "requestNo": request.request_no,
        "idempotencyKey": request.idempotency_key,
    });
    let canonical = serde_json::to_string(&body).map_err(|error| {
        CommerceServiceError::validation(format!("request body is invalid: {error}"))
    })?;
    CommerceRequestHash::new(&sha256_hash(canonical.as_bytes()))
}

//! Order to Payment executor integration adapters.

use sdkwork_contract_service::{CommerceMoney, CommerceServiceError};
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_order_service::{
    AccountValueFuture, PaymentExecutorOutcome, PaymentRefundExecutionRequest,
    PaymentRefundExecutorPort,
};
use sdkwork_payment_providers::{
    create_provider_refund, normalize_provider_code, provider_registry_for_account,
    PaymentProviderRegistry, ProviderCredentialBundle,
};
use sdkwork_payment_repository_sqlx::{
    load_active_provider_account_postgres, load_active_provider_account_sqlite,
    load_payment_attempt_provider_context_by_id_postgres,
    load_payment_attempt_provider_context_by_id_sqlite, provider_account_binding,
    PostgresCommerceRefundStore, SqliteCommerceRefundStore,
};
use sdkwork_payment_service::{CreateOwnerRefundCommand, RefundView};
use sqlx::{PgPool, SqlitePool};
use std::sync::Arc;

#[derive(Clone)]
pub struct StorePaymentRefundExecutorAdapter {
    store: StorePaymentRefundExecutorStore,
}

#[derive(Clone)]
enum StorePaymentRefundExecutorStore {
    Sqlite {
        pool: SqlitePool,
        refunds: Arc<SqliteCommerceRefundStore>,
        credentials: ProviderCredentialBundle,
    },
    Postgres {
        pool: PgPool,
        refunds: Arc<PostgresCommerceRefundStore>,
        credentials: ProviderCredentialBundle,
    },
}

impl StorePaymentRefundExecutorAdapter {
    pub fn sqlite(pool: SqlitePool) -> Self {
        Self::sqlite_with_credentials(pool, ProviderCredentialBundle::from_env())
    }

    pub fn sqlite_with_credentials(
        pool: SqlitePool,
        credentials: ProviderCredentialBundle,
    ) -> Self {
        Self {
            store: StorePaymentRefundExecutorStore::Sqlite {
                refunds: Arc::new(SqliteCommerceRefundStore::new(pool.clone())),
                pool,
                credentials,
            },
        }
    }

    pub fn postgres(pool: PgPool) -> Self {
        Self::postgres_with_credentials(pool, ProviderCredentialBundle::from_env())
    }

    pub fn postgres_with_credentials(pool: PgPool, credentials: ProviderCredentialBundle) -> Self {
        Self {
            store: StorePaymentRefundExecutorStore::Postgres {
                refunds: Arc::new(PostgresCommerceRefundStore::new(pool.clone())),
                pool,
                credentials,
            },
        }
    }

    pub fn from_database_pool(pool: &DatabasePool) -> Self {
        match pool {
            DatabasePool::Sqlite(pool, _) => Self::sqlite(pool.clone()),
            DatabasePool::Postgres(pool, _) => Self::postgres(pool.clone()),
        }
    }
}

impl PaymentRefundExecutorPort for StorePaymentRefundExecutorAdapter {
    fn execute_provider_refund<'a>(
        &'a self,
        request: PaymentRefundExecutionRequest,
    ) -> AccountValueFuture<'a, PaymentExecutorOutcome> {
        Box::pin(async move {
            match &self.store {
                StorePaymentRefundExecutorStore::Sqlite {
                    pool,
                    refunds,
                    credentials,
                } => execute_sqlite_refund(pool, refunds.as_ref(), credentials, request).await,
                StorePaymentRefundExecutorStore::Postgres {
                    pool,
                    refunds,
                    credentials,
                } => execute_postgres_refund(pool, refunds.as_ref(), credentials, request).await,
            }
        })
    }
}

async fn execute_sqlite_refund(
    pool: &SqlitePool,
    refunds: &SqliteCommerceRefundStore,
    credentials: &ProviderCredentialBundle,
    request: PaymentRefundExecutionRequest,
) -> Result<PaymentExecutorOutcome, CommerceServiceError> {
    let command = payment_refund_command(&request)?;
    let refund = refunds.create_owner_refund(command).await?;
    if let Err(error) = submit_sqlite_provider_refund(
        pool,
        credentials,
        &request.tenant_id,
        request.organization_id.as_deref(),
        &refund,
        Some(request.refund_request_id.clone()),
    )
    .await
    {
        let _ = refunds
            .mark_owner_refund_provider_submission_failed(
                &request.tenant_id,
                request.organization_id.as_deref(),
                &refund.refund_id,
                "buyer",
                Some(&request.owner_user_id),
                &request.request_no,
                &request.idempotency_key,
            )
            .await;
        return Err(error);
    }
    Ok(refund_outcome(refund))
}

async fn execute_postgres_refund(
    pool: &PgPool,
    refunds: &PostgresCommerceRefundStore,
    credentials: &ProviderCredentialBundle,
    request: PaymentRefundExecutionRequest,
) -> Result<PaymentExecutorOutcome, CommerceServiceError> {
    let command = payment_refund_command(&request)?;
    let refund = refunds.create_owner_refund(command).await?;
    if let Err(error) = submit_postgres_provider_refund(
        pool,
        credentials,
        &request.tenant_id,
        request.organization_id.as_deref(),
        &refund,
        Some(request.refund_request_id.clone()),
    )
    .await
    {
        let _ = refunds
            .mark_owner_refund_provider_submission_failed(
                &request.tenant_id,
                request.organization_id.as_deref(),
                &refund.refund_id,
                "buyer",
                Some(&request.owner_user_id),
                &request.request_no,
                &request.idempotency_key,
            )
            .await;
        return Err(error);
    }
    Ok(refund_outcome(refund))
}

fn payment_refund_command(
    request: &PaymentRefundExecutionRequest,
) -> Result<CreateOwnerRefundCommand, CommerceServiceError> {
    CreateOwnerRefundCommand::new_with_currency(
        &request.tenant_id,
        request.organization_id.as_deref(),
        &request.owner_user_id,
        &request.original_order_id,
        None,
        Some(request.amount.as_str()),
        Some(&request.currency_code),
        Some("order_refund_request"),
        &request.request_no,
        &request.idempotency_key,
    )
}

async fn submit_sqlite_provider_refund(
    pool: &SqlitePool,
    credentials: &ProviderCredentialBundle,
    tenant_id: &str,
    organization_id: Option<&str>,
    refund: &RefundView,
    reason: Option<String>,
) -> Result<(), CommerceServiceError> {
    let Some(context) =
        load_payment_attempt_provider_context_by_id_sqlite(pool, &refund.payment_attempt_id)
            .await?
    else {
        return Err(CommerceServiceError::not_found(
            "payment attempt provider context was not found for refund execution",
        ));
    };
    let provider_code = normalize_provider_code(&context.provider_code);
    let registry = if provider_code == "sandbox" || provider_code.is_empty() {
        PaymentProviderRegistry::from_credentials(credentials.clone())
    } else {
        let account =
            load_active_provider_account_sqlite(pool, tenant_id, organization_id, &provider_code)
                .await?;
        provider_registry_for_account(credentials, account.as_ref().map(provider_account_binding))
    };
    submit_provider_refund(
        &registry,
        &provider_code,
        &context.out_trade_no,
        refund,
        &context.amount,
        reason,
    )
    .await
}

async fn submit_postgres_provider_refund(
    pool: &PgPool,
    credentials: &ProviderCredentialBundle,
    tenant_id: &str,
    organization_id: Option<&str>,
    refund: &RefundView,
    reason: Option<String>,
) -> Result<(), CommerceServiceError> {
    let Some(context) =
        load_payment_attempt_provider_context_by_id_postgres(pool, &refund.payment_attempt_id)
            .await?
    else {
        return Err(CommerceServiceError::not_found(
            "payment attempt provider context was not found for refund execution",
        ));
    };
    let provider_code = normalize_provider_code(&context.provider_code);
    let registry = if provider_code == "sandbox" || provider_code.is_empty() {
        PaymentProviderRegistry::from_credentials(credentials.clone())
    } else {
        let account =
            load_active_provider_account_postgres(pool, tenant_id, organization_id, &provider_code)
                .await?;
        provider_registry_for_account(credentials, account.as_ref().map(provider_account_binding))
    };
    submit_provider_refund(
        &registry,
        &provider_code,
        &context.out_trade_no,
        refund,
        &context.amount,
        reason,
    )
    .await
}

async fn submit_provider_refund(
    registry: &PaymentProviderRegistry,
    provider_code: &str,
    out_trade_no: &str,
    refund: &RefundView,
    total_amount: &str,
    reason: Option<String>,
) -> Result<(), CommerceServiceError> {
    let total_amount = CommerceMoney::new(total_amount).map_err(CommerceServiceError::storage)?;
    create_provider_refund(
        registry,
        provider_code,
        out_trade_no,
        &refund.refund_no,
        &refund.amount,
        &total_amount,
        reason,
    )
    .await
}

fn refund_outcome(refund: RefundView) -> PaymentExecutorOutcome {
    PaymentExecutorOutcome {
        accepted: true,
        replayed: false,
        provider_reference_id: Some(refund.refund_id),
        status: refund.status,
    }
}

pub fn payment_refund_executor_port_from_database_pool(
    pool: &DatabasePool,
) -> Arc<dyn PaymentRefundExecutorPort> {
    Arc::new(StorePaymentRefundExecutorAdapter::from_database_pool(pool))
}

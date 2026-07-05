//! PSP payment webhooks are owned by order-app-api (Order → Payment ingest → in-process settlement).

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Extension, Path, State};
use axum::response::Response;
use axum::routing::post;
use axum::Router;
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_order_repository_sqlx::{
    PostgresCommerceOrderStore, PostgresCommerceRechargeStore, SqliteCommerceOrderStore,
    SqliteCommerceRechargeStore,
};
use sdkwork_order_service::{
    settle_owner_order_after_payment_success, AccountPointsCreditPort,
    OrderPaymentSettlementAttempt, OwnerOrderPaymentConfirmationPort,
};
use sdkwork_payment_providers::{
    normalize_provider_code, peek_webhook_routing_fields, provider_registry_for_account,
    PaymentNormalizeWebhookRequest, PaymentProviderRegistry, ProviderAccountBinding,
    ProviderCredentialBundle, PaymentVerifyWebhookRequest,
};
use sdkwork_payment_repository_sqlx::{
    ingest_provider_webhook_sqlite, IngestProviderWebhookCommand,
    load_active_provider_account_by_merchant_id_postgres,
    load_active_provider_account_by_merchant_id_sqlite, load_active_provider_account_postgres,
    load_active_provider_account_sqlite, load_webhook_attempt_context_by_out_trade_no_postgres,
    load_webhook_attempt_context_by_out_trade_no_sqlite, PaymentProviderAccountRecord,
    PaymentWebhookAttemptContext, PostgresCommerceOwnerOrderPaymentStore,
    SqliteCommerceOwnerOrderPaymentStore,
};
use sdkwork_web_core::WebRequestContext;
use sqlx::{PgPool, SqlitePool};

use crate::api_response::{map_service_error, success_command, validation};

#[derive(Clone)]
enum PaymentWebhookState {
    Sqlite {
        registry: Arc<PaymentProviderRegistry>,
        credentials: ProviderCredentialBundle,
        pool: SqlitePool,
        payments: Arc<SqliteCommerceOwnerOrderPaymentStore>,
        recharge: Arc<SqliteCommerceRechargeStore>,
        orders: Arc<SqliteCommerceOrderStore>,
        credit_port: Arc<dyn AccountPointsCreditPort>,
    },
    Postgres {
        registry: Arc<PaymentProviderRegistry>,
        credentials: ProviderCredentialBundle,
        pool: PgPool,
        payments: Arc<PostgresCommerceOwnerOrderPaymentStore>,
        recharge: Arc<PostgresCommerceRechargeStore>,
        orders: Arc<PostgresCommerceOrderStore>,
        credit_port: Arc<dyn AccountPointsCreditPort>,
    },
}

pub fn app_payment_webhook_router_with_sqlite_pool(
    pool: SqlitePool,
    credit_port: Arc<dyn AccountPointsCreditPort>,
) -> Router {
    let credentials = ProviderCredentialBundle::from_env();
    let registry = Arc::new(PaymentProviderRegistry::from_credentials(credentials.clone()));
    Router::new()
        .route(
            "/app/v3/api/orders/payments/webhooks/{providerCode}",
            post(receive_provider_webhook),
        )
        .with_state(PaymentWebhookState::Sqlite {
            registry,
            credentials,
            pool: pool.clone(),
            payments: Arc::new(SqliteCommerceOwnerOrderPaymentStore::new(pool.clone())),
            recharge: Arc::new(SqliteCommerceRechargeStore::new(pool.clone())),
            orders: Arc::new(SqliteCommerceOrderStore::new(pool)),
            credit_port,
        })
}

pub fn app_payment_webhook_router_with_postgres_pool(
    pool: PgPool,
    credit_port: Arc<dyn AccountPointsCreditPort>,
) -> Router {
    let credentials = ProviderCredentialBundle::from_env();
    let registry = Arc::new(PaymentProviderRegistry::from_credentials(credentials.clone()));
    Router::new()
        .route(
            "/app/v3/api/orders/payments/webhooks/{providerCode}",
            post(receive_provider_webhook),
        )
        .with_state(PaymentWebhookState::Postgres {
            registry,
            credentials,
            pool: pool.clone(),
            payments: Arc::new(PostgresCommerceOwnerOrderPaymentStore::new(pool.clone())),
            recharge: Arc::new(PostgresCommerceRechargeStore::new(pool.clone())),
            orders: Arc::new(PostgresCommerceOrderStore::new(pool)),
            credit_port,
        })
}

async fn receive_provider_webhook(
    State(state): State<PaymentWebhookState>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(provider_code): Path<String>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Response {
    let ctx = request_context.as_ref().map(|Extension(value)| value);
    match state {
        PaymentWebhookState::Sqlite {
            registry,
            credentials,
            pool,
            payments,
            recharge,
            orders,
            credit_port,
        } => {
            receive_provider_webhook_inner(
                ctx,
                registry,
                credentials,
                &pool,
                payments.as_ref(),
                recharge.as_ref(),
                orders.as_ref(),
                credit_port.as_ref(),
                provider_code,
                headers,
                body,
            )
            .await
        }
        PaymentWebhookState::Postgres {
            registry,
            credentials,
            pool,
            payments,
            recharge,
            orders,
            credit_port,
        } => {
            receive_provider_webhook_inner(
                ctx,
                registry,
                credentials,
                &pool,
                payments.as_ref(),
                recharge.as_ref(),
                orders.as_ref(),
                credit_port.as_ref(),
                provider_code,
                headers,
                body,
            )
            .await
        }
    }
}

async fn receive_provider_webhook_inner<Pool, R, O, P, Payment>(
    ctx: Option<&WebRequestContext>,
    deployment_registry: Arc<PaymentProviderRegistry>,
    credentials: ProviderCredentialBundle,
    pool: &Pool,
    payment_store: &Payment,
    recharge_store: &R,
    order_store: &O,
    credit_port: &P,
    provider_code: String,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Response
where
    Pool: WebhookIngestPool + WebhookCredentialPool + Send + Sync,
    R: sdkwork_order_service::PointsRechargeFulfillmentStore,
    O: OrderSubjectLoader,
    P: AccountPointsCreditPort + ?Sized,
    Payment: OwnerOrderPaymentConfirmationPort + ?Sized,
{
    let provider_code = normalize_provider_code(&provider_code);
    let registry = match pool
        .resolve_webhook_registry(
            &deployment_registry,
            &credentials,
            &provider_code,
            &body,
        )
        .await
    {
        Ok(registry) => registry,
        Err(error) => return map_service_error(ctx, error),
    };

    let adapter = match registry.resolve(&provider_code) {
        Some(adapter) => adapter,
        None => {
            return validation(
                ctx,
                format!("payment provider {provider_code} is not configured"),
            );
        }
    };

    let header_pairs = headers
        .iter()
        .filter_map(|(name, value)| {
            Some((
                name.as_str().to_owned(),
                value.to_str().ok()?.to_owned(),
            ))
        })
        .collect::<Vec<_>>();

    let verify_request = PaymentVerifyWebhookRequest {
        headers: header_pairs.clone(),
        body: body.to_vec(),
        metadata: serde_json::json!({ "provider_code": provider_code }),
    };

    match adapter.verify_webhook(verify_request).await {
        Ok(outcome) if outcome.verified => {}
        Ok(_) => return validation(ctx, "webhook signature verification failed"),
        Err(error) => {
            return validation(ctx, format!("webhook provider error: {error:?}"));
        }
    }

    let normalize_request = PaymentNormalizeWebhookRequest {
        headers: header_pairs,
        body: body.to_vec(),
        metadata: serde_json::json!({ "provider_code": provider_code }),
    };

    let event = match adapter.normalize_webhook(normalize_request).await {
        Ok(event) => event,
        Err(error) => {
            return validation(ctx, format!("webhook provider error: {error:?}"));
        }
    };

    let provider_event_id = event
        .provider_event_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            format!(
                "{provider_code}:{}",
                event
                    .out_trade_no
                    .as_deref()
                    .unwrap_or("unknown-out-trade-no")
            )
        });

    let ingest = match pool
        .ingest_provider_webhook(IngestProviderWebhookCommand {
            provider_code: event.provider_code.clone(),
            provider_event_id,
            event_type: event.event_type.clone(),
            out_trade_no: event.out_trade_no.clone(),
            payment_status: event.payment_status.clone(),
            payload: event.payload.clone(),
        })
        .await
    {
        Ok(outcome) => outcome,
        Err(error) => return map_service_error(ctx, error),
    };

    if let Some(attempt) = ingest.payment_attempt_context.as_ref() {
        let order_subject = order_store
            .load_order_subject(
                &attempt.tenant_id,
                attempt.organization_id.as_deref(),
                &attempt.order_id,
            )
            .await;
        match order_subject {
            Ok(Some(subject)) => {
                let request_no = format!("webhook:{}", ingest.webhook_event_id);
                let settlement_attempt = order_payment_settlement_attempt_from_webhook(attempt);
                if let Err(error) = settle_owner_order_after_payment_success(
                    payment_store,
                    recharge_store,
                    credit_port,
                    &settlement_attempt,
                    Some(subject.as_str()),
                    &request_no,
                )
                .await
                {
                    return map_service_error(ctx, error);
                }
            }
            Ok(None) => {
                return map_service_error(
                    ctx,
                    CommerceServiceError::not_found("order was not found for payment webhook"),
                );
            }
            Err(error) => return map_service_error(ctx, error),
        }
    }

    success_command(
        ctx,
        ingest
            .payment_attempt_id
            .or(Some(ingest.webhook_event_id)),
        ingest.applied_status.or_else(|| {
            if ingest.replayed {
                Some("replayed".to_owned())
            } else if ingest.payment_attempt_context.is_some() {
                Some("settled".to_owned())
            } else {
                Some("accepted".to_owned())
            }
        }),
    )
}

fn order_payment_settlement_attempt_from_webhook(
    attempt: &PaymentWebhookAttemptContext,
) -> OrderPaymentSettlementAttempt {
    OrderPaymentSettlementAttempt {
        tenant_id: attempt.tenant_id.clone(),
        organization_id: attempt.organization_id.clone(),
        owner_user_id: attempt.owner_user_id.clone(),
        order_id: attempt.order_id.clone(),
    }
}

trait OrderSubjectLoader: Send + Sync {
    fn load_order_subject<'a>(
        &'a self,
        tenant_id: &'a str,
        organization_id: Option<&'a str>,
        order_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<String>, CommerceServiceError>> + Send + 'a>,
    >;
}

impl OrderSubjectLoader for SqliteCommerceOrderStore {
    fn load_order_subject<'a>(
        &'a self,
        tenant_id: &'a str,
        organization_id: Option<&'a str>,
        order_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<String>, CommerceServiceError>> + Send + 'a>,
    > {
        Box::pin(async move {
            self.load_order_subject(tenant_id, organization_id, order_id)
                .await
        })
    }
}

impl OrderSubjectLoader for PostgresCommerceOrderStore {
    fn load_order_subject<'a>(
        &'a self,
        tenant_id: &'a str,
        organization_id: Option<&'a str>,
        order_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<String>, CommerceServiceError>> + Send + 'a>,
    > {
        Box::pin(async move {
            self.load_order_subject(tenant_id, organization_id, order_id)
                .await
        })
    }
}

trait WebhookCredentialPool {
    fn resolve_webhook_registry(
        &self,
        deployment_registry: &PaymentProviderRegistry,
        credentials: &ProviderCredentialBundle,
        provider_code: &str,
        body: &[u8],
    ) -> impl std::future::Future<Output = Result<PaymentProviderRegistry, CommerceServiceError>>
           + Send;
}

trait WebhookIngestPool {
    fn ingest_provider_webhook(
        &self,
        command: IngestProviderWebhookCommand,
    ) -> impl std::future::Future<
        Output = Result<
            sdkwork_payment_repository_sqlx::IngestProviderWebhookOutcome,
            CommerceServiceError,
        >,
    > + Send;
}

async fn resolve_webhook_provider_account_sqlite(
    pool: &SqlitePool,
    credentials: &ProviderCredentialBundle,
    deployment_registry: &PaymentProviderRegistry,
    provider_code: &str,
    body: &[u8],
) -> Result<PaymentProviderRegistry, CommerceServiceError> {
    let peek = peek_webhook_routing_fields(provider_code, body);
    let account = if let Some(out_trade_no) = peek.out_trade_no.as_deref() {
        if let Some(context) =
            load_webhook_attempt_context_by_out_trade_no_sqlite(pool, out_trade_no).await?
        {
            load_active_provider_account_sqlite(
                pool,
                &context.tenant_id,
                context.organization_id.as_deref(),
                &context.provider_code,
            )
            .await?
        } else {
            None
        }
    } else if let Some(merchant_id) = peek.merchant_id.as_deref() {
        load_active_provider_account_by_merchant_id_sqlite(pool, provider_code, merchant_id).await?
    } else {
        None
    };
    Ok(registry_for_webhook_account(
        deployment_registry,
        credentials,
        account,
    ))
}

async fn resolve_webhook_provider_account_postgres(
    pool: &PgPool,
    credentials: &ProviderCredentialBundle,
    deployment_registry: &PaymentProviderRegistry,
    provider_code: &str,
    body: &[u8],
) -> Result<PaymentProviderRegistry, CommerceServiceError> {
    let peek = peek_webhook_routing_fields(provider_code, body);
    let account = if let Some(out_trade_no) = peek.out_trade_no.as_deref() {
        if let Some(context) =
            load_webhook_attempt_context_by_out_trade_no_postgres(pool, out_trade_no).await?
        {
            load_active_provider_account_postgres(
                pool,
                &context.tenant_id,
                context.organization_id.as_deref(),
                &context.provider_code,
            )
            .await?
        } else {
            None
        }
    } else if let Some(merchant_id) = peek.merchant_id.as_deref() {
        load_active_provider_account_by_merchant_id_postgres(pool, provider_code, merchant_id).await?
    } else {
        None
    };
    Ok(registry_for_webhook_account(
        deployment_registry,
        credentials,
        account,
    ))
}

fn registry_for_webhook_account(
    deployment_registry: &PaymentProviderRegistry,
    credentials: &ProviderCredentialBundle,
    account: Option<PaymentProviderAccountRecord>,
) -> PaymentProviderRegistry {
    match account {
        Some(record) => provider_registry_for_account(
            credentials,
            Some(provider_account_binding(&record)),
        ),
        None => deployment_registry.clone(),
    }
}

fn provider_account_binding(record: &PaymentProviderAccountRecord) -> ProviderAccountBinding {
    ProviderAccountBinding {
        provider_code: record.provider_code.clone(),
        merchant_id: record.merchant_id.clone(),
        environment: record.environment.clone(),
        secret_ref: record.secret_ref.clone(),
        webhook_secret_ref: record.webhook_secret_ref.clone(),
        certificate_ref: record.certificate_ref.clone(),
        metadata: record.metadata.clone(),
    }
}

impl WebhookCredentialPool for SqlitePool {
    async fn resolve_webhook_registry(
        &self,
        deployment_registry: &PaymentProviderRegistry,
        credentials: &ProviderCredentialBundle,
        provider_code: &str,
        body: &[u8],
    ) -> Result<PaymentProviderRegistry, CommerceServiceError> {
        resolve_webhook_provider_account_sqlite(
            self,
            credentials,
            deployment_registry,
            provider_code,
            body,
        )
        .await
    }
}

impl WebhookCredentialPool for PgPool {
    async fn resolve_webhook_registry(
        &self,
        deployment_registry: &PaymentProviderRegistry,
        credentials: &ProviderCredentialBundle,
        provider_code: &str,
        body: &[u8],
    ) -> Result<PaymentProviderRegistry, CommerceServiceError> {
        resolve_webhook_provider_account_postgres(
            self,
            credentials,
            deployment_registry,
            provider_code,
            body,
        )
        .await
    }
}

impl WebhookIngestPool for SqlitePool {
    async fn ingest_provider_webhook(
        &self,
        command: IngestProviderWebhookCommand,
    ) -> Result<
        sdkwork_payment_repository_sqlx::IngestProviderWebhookOutcome,
        CommerceServiceError,
    > {
        ingest_provider_webhook_sqlite(self, command).await
    }
}

impl WebhookIngestPool for PgPool {
    async fn ingest_provider_webhook(
        &self,
        command: IngestProviderWebhookCommand,
    ) -> Result<
        sdkwork_payment_repository_sqlx::IngestProviderWebhookOutcome,
        CommerceServiceError,
    > {
        sdkwork_payment_repository_sqlx::ingest_provider_webhook_postgres(self, command).await
    }
}

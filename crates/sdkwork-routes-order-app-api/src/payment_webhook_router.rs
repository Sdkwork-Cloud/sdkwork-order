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
    settle_owner_order_after_payment_success, AccountPointsCreditPort, AccountValueLedgerPort,
    CouponRedemptionPort, MembershipPurchaseFulfillmentPort, NoopCouponRedemptionPort,
    OrderPaymentSettlementAttempt, OwnerOrderSettlementPorts,
};
use sdkwork_payment_providers::{
    normalize_provider_code, peek_webhook_routing_fields, provider_registry_for_account,
    PaymentNormalizeWebhookRequest, PaymentProviderRegistry, PaymentVerifyWebhookRequest,
    ProviderAccountBinding, ProviderCredentialBundle,
};
use sdkwork_payment_repository_sqlx::{
    ingest_provider_webhook_sqlite, load_active_provider_account_by_merchant_id_postgres,
    load_active_provider_account_by_merchant_id_sqlite, load_active_provider_account_postgres,
    load_active_provider_account_sqlite, load_webhook_attempt_context_by_out_trade_no_postgres,
    load_webhook_attempt_context_by_out_trade_no_sqlite, IngestProviderWebhookCommand,
    PaymentProviderAccountRecord, PaymentWebhookAttemptContext,
    PostgresCommerceOwnerOrderPaymentStore, SqliteCommerceOwnerOrderPaymentStore,
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
        account_value_ledger_port: Arc<dyn AccountValueLedgerPort>,
        coupon_redemption_port: Arc<dyn CouponRedemptionPort>,
        membership_port: Arc<dyn MembershipPurchaseFulfillmentPort>,
    },
    Postgres {
        registry: Arc<PaymentProviderRegistry>,
        credentials: ProviderCredentialBundle,
        pool: PgPool,
        payments: Arc<PostgresCommerceOwnerOrderPaymentStore>,
        recharge: Arc<PostgresCommerceRechargeStore>,
        orders: Arc<PostgresCommerceOrderStore>,
        credit_port: Arc<dyn AccountPointsCreditPort>,
        account_value_ledger_port: Arc<dyn AccountValueLedgerPort>,
        coupon_redemption_port: Arc<dyn CouponRedemptionPort>,
        membership_port: Arc<dyn MembershipPurchaseFulfillmentPort>,
    },
}

struct PaymentWebhookRuntime<'a, Pool> {
    deployment_registry: &'a PaymentProviderRegistry,
    credentials: &'a ProviderCredentialBundle,
    pool: &'a Pool,
    order_subject_loader: &'a dyn OrderSubjectLoader,
    settlement_ports: OwnerOrderSettlementPorts<'a>,
}

struct ProviderWebhookRequest<'a> {
    context: Option<&'a WebRequestContext>,
    provider_code: String,
    headers: axum::http::HeaderMap,
    body: Bytes,
}

struct WebhookProviderScope {
    tenant_id: String,
    organization_id: Option<String>,
}

struct WebhookProviderResolution {
    registry: PaymentProviderRegistry,
    scope: Option<WebhookProviderScope>,
}

pub fn app_payment_webhook_router_with_sqlite_pool(
    pool: SqlitePool,
    credit_port: Arc<dyn AccountPointsCreditPort>,
    account_value_ledger_port: Arc<dyn AccountValueLedgerPort>,
    membership_port: Arc<dyn MembershipPurchaseFulfillmentPort>,
) -> Router {
    app_payment_webhook_router_with_sqlite_pool_and_coupon(
        pool,
        credit_port,
        account_value_ledger_port,
        Arc::new(NoopCouponRedemptionPort),
        membership_port,
    )
}

pub fn app_payment_webhook_router_with_sqlite_pool_and_coupon(
    pool: SqlitePool,
    credit_port: Arc<dyn AccountPointsCreditPort>,
    account_value_ledger_port: Arc<dyn AccountValueLedgerPort>,
    coupon_redemption_port: Arc<dyn CouponRedemptionPort>,
    membership_port: Arc<dyn MembershipPurchaseFulfillmentPort>,
) -> Router {
    let credentials = ProviderCredentialBundle::from_env();
    let registry = Arc::new(PaymentProviderRegistry::from_credentials(
        credentials.clone(),
    ));
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
            account_value_ledger_port,
            coupon_redemption_port,
            membership_port,
        })
}

pub fn app_payment_webhook_router_with_postgres_pool(
    pool: PgPool,
    credit_port: Arc<dyn AccountPointsCreditPort>,
    account_value_ledger_port: Arc<dyn AccountValueLedgerPort>,
    membership_port: Arc<dyn MembershipPurchaseFulfillmentPort>,
) -> Router {
    app_payment_webhook_router_with_postgres_pool_and_coupon(
        pool,
        credit_port,
        account_value_ledger_port,
        Arc::new(NoopCouponRedemptionPort),
        membership_port,
    )
}

pub fn app_payment_webhook_router_with_postgres_pool_and_coupon(
    pool: PgPool,
    credit_port: Arc<dyn AccountPointsCreditPort>,
    account_value_ledger_port: Arc<dyn AccountValueLedgerPort>,
    coupon_redemption_port: Arc<dyn CouponRedemptionPort>,
    membership_port: Arc<dyn MembershipPurchaseFulfillmentPort>,
) -> Router {
    let credentials = ProviderCredentialBundle::from_env();
    let registry = Arc::new(PaymentProviderRegistry::from_credentials(
        credentials.clone(),
    ));
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
            account_value_ledger_port,
            coupon_redemption_port,
            membership_port,
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
            account_value_ledger_port,
            coupon_redemption_port,
            membership_port,
        } => {
            receive_provider_webhook_inner(
                PaymentWebhookRuntime {
                    deployment_registry: registry.as_ref(),
                    credentials: &credentials,
                    pool: &pool,
                    order_subject_loader: orders.as_ref(),
                    settlement_ports: OwnerOrderSettlementPorts {
                        payment_store: payments.as_ref(),
                        order_state_store: orders.as_ref(),
                        recharge_store: recharge.as_ref(),
                        account_value_store: recharge.as_ref(),
                        credit_port: credit_port.as_ref(),
                        account_value_ledger_port: account_value_ledger_port.as_ref(),
                        coupon_redemption_port: coupon_redemption_port.as_ref(),
                        membership_port: membership_port.as_ref(),
                    },
                },
                ProviderWebhookRequest {
                    context: ctx,
                    provider_code,
                    headers,
                    body,
                },
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
            account_value_ledger_port,
            coupon_redemption_port,
            membership_port,
        } => {
            receive_provider_webhook_inner(
                PaymentWebhookRuntime {
                    deployment_registry: registry.as_ref(),
                    credentials: &credentials,
                    pool: &pool,
                    order_subject_loader: orders.as_ref(),
                    settlement_ports: OwnerOrderSettlementPorts {
                        payment_store: payments.as_ref(),
                        order_state_store: orders.as_ref(),
                        recharge_store: recharge.as_ref(),
                        account_value_store: recharge.as_ref(),
                        credit_port: credit_port.as_ref(),
                        account_value_ledger_port: account_value_ledger_port.as_ref(),
                        coupon_redemption_port: coupon_redemption_port.as_ref(),
                        membership_port: membership_port.as_ref(),
                    },
                },
                ProviderWebhookRequest {
                    context: ctx,
                    provider_code,
                    headers,
                    body,
                },
            )
            .await
        }
    }
}

async fn receive_provider_webhook_inner<Pool>(
    runtime: PaymentWebhookRuntime<'_, Pool>,
    request: ProviderWebhookRequest<'_>,
) -> Response
where
    Pool: WebhookIngestPool + WebhookCredentialPool + Send + Sync,
{
    let PaymentWebhookRuntime {
        deployment_registry,
        credentials,
        pool,
        order_subject_loader,
        settlement_ports,
    } = runtime;
    let ProviderWebhookRequest {
        context: ctx,
        provider_code,
        headers,
        body,
    } = request;
    let provider_code = normalize_provider_code(&provider_code);
    let resolution = match pool
        .resolve_webhook_registry(deployment_registry, credentials, &provider_code, &body)
        .await
    {
        Ok(resolution) => resolution,
        Err(error) => return map_service_error(ctx, error),
    };
    let WebhookProviderResolution { registry, scope } = resolution;

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
            Some((name.as_str().to_owned(), value.to_str().ok()?.to_owned()))
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
            tenant_id: scope.as_ref().map(|scope| scope.tenant_id.clone()),
            organization_id: scope.and_then(|scope| scope.organization_id),
        })
        .await
    {
        Ok(outcome) => outcome,
        Err(error) => return map_service_error(ctx, error),
    };

    if let Some(attempt) = ingest.payment_attempt_context.as_ref() {
        let order_subject = order_subject_loader
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
                    settlement_ports,
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
        ingest.payment_attempt_id.or(Some(ingest.webhook_event_id)),
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
        payment_attempt_id: Some(attempt.payment_attempt_id.clone()),
        out_trade_no: Some(attempt.out_trade_no.clone()),
    }
}

trait OrderSubjectLoader: Send + Sync {
    fn load_order_subject<'a>(
        &'a self,
        tenant_id: &'a str,
        organization_id: Option<&'a str>,
        order_id: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Option<String>, CommerceServiceError>>
                + Send
                + 'a,
        >,
    >;
}

impl OrderSubjectLoader for SqliteCommerceOrderStore {
    fn load_order_subject<'a>(
        &'a self,
        tenant_id: &'a str,
        organization_id: Option<&'a str>,
        order_id: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Option<String>, CommerceServiceError>>
                + Send
                + 'a,
        >,
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
        Box<
            dyn std::future::Future<Output = Result<Option<String>, CommerceServiceError>>
                + Send
                + 'a,
        >,
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
    ) -> impl std::future::Future<Output = Result<WebhookProviderResolution, CommerceServiceError>> + Send;
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
) -> Result<WebhookProviderResolution, CommerceServiceError> {
    let peek = peek_webhook_routing_fields(provider_code, body);
    let attempt_context = if let Some(out_trade_no) = peek.out_trade_no.as_deref() {
        load_webhook_attempt_context_by_out_trade_no_sqlite(pool, provider_code, out_trade_no)
            .await?
    } else {
        None
    };
    let fallback_scope = attempt_context
        .as_ref()
        .map(|context| WebhookProviderScope {
            tenant_id: context.tenant_id.clone(),
            organization_id: context.organization_id.clone(),
        });
    let account = if let Some(context) = attempt_context.as_ref() {
        load_active_provider_account_sqlite(
            pool,
            &context.tenant_id,
            context.organization_id.as_deref(),
            &context.provider_code,
        )
        .await?
    } else if let Some(merchant_id) = peek.merchant_id.as_deref() {
        load_active_provider_account_by_merchant_id_sqlite(pool, provider_code, merchant_id).await?
    } else {
        None
    };
    Ok(webhook_provider_resolution(
        deployment_registry,
        credentials,
        account,
        fallback_scope,
    ))
}

async fn resolve_webhook_provider_account_postgres(
    pool: &PgPool,
    credentials: &ProviderCredentialBundle,
    deployment_registry: &PaymentProviderRegistry,
    provider_code: &str,
    body: &[u8],
) -> Result<WebhookProviderResolution, CommerceServiceError> {
    let peek = peek_webhook_routing_fields(provider_code, body);
    let attempt_context = if let Some(out_trade_no) = peek.out_trade_no.as_deref() {
        load_webhook_attempt_context_by_out_trade_no_postgres(pool, provider_code, out_trade_no)
            .await?
    } else {
        None
    };
    let fallback_scope = attempt_context
        .as_ref()
        .map(|context| WebhookProviderScope {
            tenant_id: context.tenant_id.clone(),
            organization_id: context.organization_id.clone(),
        });
    let account = if let Some(context) = attempt_context.as_ref() {
        load_active_provider_account_postgres(
            pool,
            &context.tenant_id,
            context.organization_id.as_deref(),
            &context.provider_code,
        )
        .await?
    } else if let Some(merchant_id) = peek.merchant_id.as_deref() {
        load_active_provider_account_by_merchant_id_postgres(pool, provider_code, merchant_id)
            .await?
    } else {
        None
    };
    Ok(webhook_provider_resolution(
        deployment_registry,
        credentials,
        account,
        fallback_scope,
    ))
}

fn webhook_provider_resolution(
    deployment_registry: &PaymentProviderRegistry,
    credentials: &ProviderCredentialBundle,
    account: Option<PaymentProviderAccountRecord>,
    fallback_scope: Option<WebhookProviderScope>,
) -> WebhookProviderResolution {
    let scope = account
        .as_ref()
        .map(|record| WebhookProviderScope {
            tenant_id: record.tenant_id.clone(),
            organization_id: record.organization_id.clone(),
        })
        .or(fallback_scope);
    let registry = account.as_ref().map_or_else(
        || deployment_registry.clone(),
        |record| provider_registry_for_account(credentials, Some(provider_account_binding(record))),
    );
    WebhookProviderResolution { registry, scope }
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
    ) -> Result<WebhookProviderResolution, CommerceServiceError> {
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
    ) -> Result<WebhookProviderResolution, CommerceServiceError> {
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
    ) -> Result<sdkwork_payment_repository_sqlx::IngestProviderWebhookOutcome, CommerceServiceError>
    {
        ingest_provider_webhook_sqlite(self, command).await
    }
}

impl WebhookIngestPool for PgPool {
    async fn ingest_provider_webhook(
        &self,
        command: IngestProviderWebhookCommand,
    ) -> Result<sdkwork_payment_repository_sqlx::IngestProviderWebhookOutcome, CommerceServiceError>
    {
        sdkwork_payment_repository_sqlx::ingest_provider_webhook_postgres(self, command).await
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_webhook_provider_account_sqlite;
    use sdkwork_payment_providers::{PaymentProviderRegistry, ProviderCredentialBundle};

    #[tokio::test]
    async fn webhook_scope_falls_back_to_merchant_account_when_trade_is_unmatched() {
        let pool = webhook_scope_test_pool().await;
        sqlx::query(
            r#"
            INSERT INTO commerce_payment_provider_account
                (id, tenant_id, organization_id, account_no, provider_code, merchant_id,
                 environment, secret_ref, status)
            VALUES ('provider-account-1', 'tenant-merchant', 'org-merchant', 'PA-1', 'alipay',
                    'merchant-1', 'production', 'secret://alipay', 'active')
            "#,
        )
        .execute(&pool)
        .await
        .expect("seed provider account");
        let credentials = ProviderCredentialBundle::from_env();
        let deployment_registry = PaymentProviderRegistry::from_credentials(credentials.clone());

        let resolution = resolve_webhook_provider_account_sqlite(
            &pool,
            &credentials,
            &deployment_registry,
            "alipay",
            b"out_trade_no=missing-trade&app_id=merchant-1&trade_status=TRADE_SUCCESS",
        )
        .await
        .expect("resolve webhook account");
        let scope = resolution.scope.expect("provider account scope");
        assert_eq!(scope.tenant_id, "tenant-merchant");
        assert_eq!(scope.organization_id.as_deref(), Some("org-merchant"));
    }

    #[tokio::test]
    async fn webhook_scope_uses_exact_attempt_when_provider_account_is_absent() {
        let pool = webhook_scope_test_pool().await;
        sqlx::query(
            r#"
            INSERT INTO commerce_payment_attempt
                (id, tenant_id, organization_id, provider_code, out_trade_no)
            VALUES ('attempt-1', 'tenant-attempt', 'org-attempt', 'stripe', 'trade-1')
            "#,
        )
        .execute(&pool)
        .await
        .expect("seed payment attempt");
        let credentials = ProviderCredentialBundle::from_env();
        let deployment_registry = PaymentProviderRegistry::from_credentials(credentials.clone());

        let resolution = resolve_webhook_provider_account_sqlite(
            &pool,
            &credentials,
            &deployment_registry,
            "stripe",
            br#"{"data":{"object":{"metadata":{"merchant_order_no":"trade-1"}}}}"#,
        )
        .await
        .expect("resolve webhook attempt scope");
        let scope = resolution.scope.expect("payment attempt scope");
        assert_eq!(scope.tenant_id, "tenant-attempt");
        assert_eq!(scope.organization_id.as_deref(), Some("org-attempt"));
    }

    async fn webhook_scope_test_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("sqlite memory pool");
        for statement in [
            r#"
            CREATE TABLE commerce_payment_attempt (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                organization_id TEXT,
                provider_code TEXT NOT NULL,
                out_trade_no TEXT NOT NULL,
                deleted_at TEXT
            )
            "#,
            r#"
            CREATE TABLE commerce_payment_provider_account (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                organization_id TEXT,
                account_no TEXT NOT NULL,
                provider_code TEXT NOT NULL,
                merchant_id TEXT,
                environment TEXT NOT NULL,
                secret_ref TEXT NOT NULL,
                webhook_secret_ref TEXT,
                certificate_ref TEXT,
                status TEXT NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}',
                updated_at TEXT NOT NULL DEFAULT '2026-07-12T00:00:00Z',
                deleted_at TEXT
            )
            "#,
        ] {
            sqlx::query(statement)
                .execute(&pool)
                .await
                .expect("create webhook scope test table");
        }
        pool
    }
}

//! PSP-enriched owner-order payment store for order app-api routers.

use std::sync::Arc;

use sdkwork_contract_service::CommerceServiceError;
use sdkwork_payment_providers::{PaymentProviderRegistry, ProviderCredentialBundle};
use sdkwork_payment_repository_sqlx::{
    enrich_owner_order_payment_postgres, enrich_owner_order_payment_sqlite,
    OwnerOrderPaymentEnrichmentContext, PostgresCommerceOwnerOrderPaymentStore,
    SqliteCommerceOwnerOrderPaymentStore,
};
use sdkwork_payment_service::{
    CancelOrderPaymentsCommand, PayOwnerOrderCommand, PayOwnerOrderOutcome,
};
use sqlx::{PgPool, SqlitePool};

use crate::order_router::{CommerceOrderFuture, OwnerOrderPaymentStore};

pub struct ProviderEnrichedSqliteOwnerOrderPayments {
    inner: Arc<SqliteCommerceOwnerOrderPaymentStore>,
    pool: SqlitePool,
    registry: Arc<PaymentProviderRegistry>,
    credentials: ProviderCredentialBundle,
}

pub struct ProviderEnrichedPostgresOwnerOrderPayments {
    inner: Arc<PostgresCommerceOwnerOrderPaymentStore>,
    pool: PgPool,
    registry: Arc<PaymentProviderRegistry>,
    credentials: ProviderCredentialBundle,
}

pub fn enriched_sqlite_owner_order_payments(
    pool: SqlitePool,
    registry: Arc<PaymentProviderRegistry>,
    credentials: ProviderCredentialBundle,
) -> Arc<dyn OwnerOrderPaymentStore> {
    Arc::new(ProviderEnrichedSqliteOwnerOrderPayments {
        inner: Arc::new(SqliteCommerceOwnerOrderPaymentStore::new(pool.clone())),
        pool,
        registry,
        credentials,
    })
}

pub fn enriched_postgres_owner_order_payments(
    pool: PgPool,
    registry: Arc<PaymentProviderRegistry>,
    credentials: ProviderCredentialBundle,
) -> Arc<dyn OwnerOrderPaymentStore> {
    Arc::new(ProviderEnrichedPostgresOwnerOrderPayments {
        inner: Arc::new(PostgresCommerceOwnerOrderPaymentStore::new(pool.clone())),
        pool,
        registry,
        credentials,
    })
}

impl OwnerOrderPaymentStore for ProviderEnrichedSqliteOwnerOrderPayments {
    fn pay_owner_order<'a>(
        &'a self,
        command: PayOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, PayOwnerOrderOutcome> {
        let registry = self.registry.clone();
        let credentials = self.credentials.clone();
        let pool = self.pool.clone();
        let inner = self.inner.clone();
        Box::pin(async move {
            let tenant_id = command.tenant_id.clone();
            let organization_id = command.organization_id.clone();
            let order_id = command.order_id.clone();
            let idempotency_key = command.idempotency_key.clone();
            let payment_scene = command.payment_scene.clone();
            let outcome = inner.pay_owner_order(command).await?;
            let fallback = outcome.clone();
            let enriched = enrich_owner_order_payment_sqlite(
                &pool,
                OwnerOrderPaymentEnrichmentContext {
                    deployment_registry: &registry,
                    credentials: &credentials,
                    tenant_id: &tenant_id,
                    organization_id: organization_id.as_deref(),
                    order_id: &order_id,
                    idempotency_key: &idempotency_key,
                    payment_scene: payment_scene.as_deref(),
                },
                outcome,
            )
            .await;
            checkout_enrichment_or_development_fallback(enriched, fallback)
        })
    }

    fn cancel_owner_order_payments<'a>(
        &'a self,
        command: sdkwork_order_service::CancelOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, ()> {
        let inner = self.inner.clone();
        Box::pin(async move {
            let payment_command = CancelOrderPaymentsCommand::new(
                &command.tenant_id,
                command.organization_id.as_deref(),
                &command.owner_user_id,
                &command.order_id,
            )?;
            inner.cancel_order_payments(payment_command).await
        })
    }
}

impl OwnerOrderPaymentStore for ProviderEnrichedPostgresOwnerOrderPayments {
    fn pay_owner_order<'a>(
        &'a self,
        command: PayOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, PayOwnerOrderOutcome> {
        let registry = self.registry.clone();
        let credentials = self.credentials.clone();
        let pool = self.pool.clone();
        let inner = self.inner.clone();
        Box::pin(async move {
            let tenant_id = command.tenant_id.clone();
            let organization_id = command.organization_id.clone();
            let order_id = command.order_id.clone();
            let idempotency_key = command.idempotency_key.clone();
            let payment_scene = command.payment_scene.clone();
            let outcome = inner.pay_owner_order(command).await?;
            let fallback = outcome.clone();
            let enriched = enrich_owner_order_payment_postgres(
                &pool,
                OwnerOrderPaymentEnrichmentContext {
                    deployment_registry: &registry,
                    credentials: &credentials,
                    tenant_id: &tenant_id,
                    organization_id: organization_id.as_deref(),
                    order_id: &order_id,
                    idempotency_key: &idempotency_key,
                    payment_scene: payment_scene.as_deref(),
                },
                outcome,
            )
            .await;
            checkout_enrichment_or_development_fallback(enriched, fallback)
        })
    }

    fn cancel_owner_order_payments<'a>(
        &'a self,
        command: sdkwork_order_service::CancelOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, ()> {
        let inner = self.inner.clone();
        Box::pin(async move {
            let payment_command = CancelOrderPaymentsCommand::new(
                &command.tenant_id,
                command.organization_id.as_deref(),
                &command.owner_user_id,
                &command.order_id,
            )?;
            inner.cancel_order_payments(payment_command).await
        })
    }
}

fn checkout_enrichment_or_development_fallback(
    result: Result<PayOwnerOrderOutcome, CommerceServiceError>,
    fallback: PayOwnerOrderOutcome,
) -> Result<PayOwnerOrderOutcome, CommerceServiceError> {
    match result {
        Ok(outcome) => Ok(outcome),
        Err(error) if should_use_development_cashier_fallback(&error, runtime_environment()) => {
            tracing::warn!(
                provider_code = fallback
                    .payment_params
                    .get("providerCode")
                    .map(String::as_str),
                order_id = fallback.order_id,
                "payment provider is not configured; returning the pending development cashier"
            );
            Ok(fallback)
        }
        Err(error) => Err(error),
    }
}

fn runtime_environment() -> Option<String> {
    ["SDKWORK_ORDER_ENVIRONMENT", "SDKWORK_ENV"]
        .into_iter()
        .find_map(|key| {
            std::env::var(key)
                .ok()
                .map(|value| value.trim().to_ascii_lowercase())
                .filter(|value| !value.is_empty())
        })
}

fn should_use_development_cashier_fallback(
    error: &CommerceServiceError,
    environment: Option<String>,
) -> bool {
    error.code() == "provider-unavailable"
        && error.message().contains("is not configured")
        && matches!(
            environment.as_deref(),
            Some("development" | "dev" | "local" | "test")
        )
}

#[cfg(test)]
mod tests {
    use sdkwork_contract_service::CommerceServiceError;

    use super::should_use_development_cashier_fallback;

    #[test]
    fn development_cashier_fallback_only_accepts_unconfigured_provider_errors() {
        let unconfigured = CommerceServiceError::provider_unavailable(
            "payment provider wechat_pay is not configured",
        );
        let transport = CommerceServiceError::provider_unavailable("wechat transport failed");

        assert!(should_use_development_cashier_fallback(
            &unconfigured,
            Some("development".to_owned())
        ));
        assert!(should_use_development_cashier_fallback(
            &unconfigured,
            Some("dev".to_owned())
        ));
        assert!(!should_use_development_cashier_fallback(
            &unconfigured,
            Some("production".to_owned())
        ));
        assert!(!should_use_development_cashier_fallback(
            &transport,
            Some("development".to_owned())
        ));
        assert!(!should_use_development_cashier_fallback(
            &unconfigured,
            None
        ));
    }
}

//! PSP-enriched owner-order payment store for order app-api routers.

use std::sync::Arc;

use sdkwork_payment_providers::{PaymentProviderRegistry, ProviderCredentialBundle};
use sdkwork_payment_repository_sqlx::{
    enrich_owner_order_payment_postgres, enrich_owner_order_payment_sqlite,
    PostgresCommerceOwnerOrderPaymentStore, SqliteCommerceOwnerOrderPaymentStore,
};
use sdkwork_payment_service::{CancelOrderPaymentsCommand, PayOwnerOrderCommand, PayOwnerOrderOutcome};
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
            enrich_owner_order_payment_sqlite(
                &pool,
                &registry,
                &credentials,
                &tenant_id,
                organization_id.as_deref(),
                &order_id,
                &idempotency_key,
                payment_scene.as_deref(),
                outcome,
            )
            .await
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
            enrich_owner_order_payment_postgres(
                &pool,
                &registry,
                &credentials,
                &tenant_id,
                organization_id.as_deref(),
                &order_id,
                &idempotency_key,
                payment_scene.as_deref(),
                outcome,
            )
            .await
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

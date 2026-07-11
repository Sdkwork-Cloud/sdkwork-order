//! Management order lifecycle orchestration (payments before order state).

use std::sync::Arc;

use sdkwork_contract_service::CommerceServiceError;
use sdkwork_order_repository_sqlx::{PostgresCommerceOrderStore, SqliteCommerceOrderStore};
use sdkwork_order_service::{CancelManagementOrderCommand, CloseManagementOrderCommand};
use sdkwork_payment_repository_sqlx::{
    PostgresCommerceOwnerOrderPaymentStore, SqliteCommerceOwnerOrderPaymentStore,
};
use sdkwork_payment_service::CancelOrderPaymentsCommand;

#[derive(Clone)]
pub enum BackendManagementPaymentStore {
    Postgres(Arc<PostgresCommerceOwnerOrderPaymentStore>),
    Sqlite(Arc<SqliteCommerceOwnerOrderPaymentStore>),
}

#[derive(Clone)]
pub enum BackendManagementOrderStore {
    Postgres(Arc<PostgresCommerceOrderStore>),
    Sqlite(Arc<SqliteCommerceOrderStore>),
}

pub async fn cancel_management_order_with_payments(
    orders: &BackendManagementOrderStore,
    payments: &BackendManagementPaymentStore,
    command: CancelManagementOrderCommand,
) -> Result<(), CommerceServiceError> {
    close_management_order_payments(
        orders,
        payments,
        &command.tenant_id,
        command.organization_id.as_deref(),
        &command.order_id,
    )
    .await?;
    match orders {
        BackendManagementOrderStore::Postgres(store) => {
            store.cancel_management_order(command).await
        }
        BackendManagementOrderStore::Sqlite(store) => store.cancel_management_order(command).await,
    }
}

pub async fn close_management_order_with_payments(
    orders: &BackendManagementOrderStore,
    payments: &BackendManagementPaymentStore,
    command: CloseManagementOrderCommand,
) -> Result<(), CommerceServiceError> {
    close_management_order_payments(
        orders,
        payments,
        &command.tenant_id,
        command.organization_id.as_deref(),
        &command.order_id,
    )
    .await?;
    match orders {
        BackendManagementOrderStore::Postgres(store) => store.close_management_order(command).await,
        BackendManagementOrderStore::Sqlite(store) => store.close_management_order(command).await,
    }
}

async fn close_management_order_payments(
    orders: &BackendManagementOrderStore,
    payments: &BackendManagementPaymentStore,
    tenant_id: &str,
    organization_id: Option<&str>,
    order_id: &str,
) -> Result<(), CommerceServiceError> {
    let owner_user_id =
        resolve_management_order_owner_user_id(orders, tenant_id, organization_id, order_id)
            .await?;
    let payment_command =
        CancelOrderPaymentsCommand::new(tenant_id, organization_id, &owner_user_id, order_id)?;
    match payments {
        BackendManagementPaymentStore::Postgres(store) => {
            store.cancel_order_payments(payment_command).await
        }
        BackendManagementPaymentStore::Sqlite(store) => {
            store.cancel_order_payments(payment_command).await
        }
    }
}

async fn resolve_management_order_owner_user_id(
    orders: &BackendManagementOrderStore,
    tenant_id: &str,
    organization_id: Option<&str>,
    order_id: &str,
) -> Result<String, CommerceServiceError> {
    let owner_user_id = match orders {
        BackendManagementOrderStore::Postgres(store) => {
            store
                .resolve_management_order_owner_user_id(tenant_id, organization_id, order_id)
                .await?
        }
        BackendManagementOrderStore::Sqlite(store) => {
            store
                .resolve_management_order_owner_user_id(tenant_id, organization_id, order_id)
                .await?
        }
    };
    owner_user_id.ok_or_else(|| CommerceServiceError::not_found("order was not found"))
}

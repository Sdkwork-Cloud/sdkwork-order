use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::post;
use axum::{Json, Router};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_order_repository_sqlx::{
    OrderPaymentSettlementContext, PostgresCommerceOrderStore, PostgresCommerceRechargeStore,
    SqliteCommerceOrderStore, SqliteCommerceRechargeStore,
};
use sdkwork_order_service::{
    settle_owner_order_after_payment_success, AccountPointsCreditPort, AccountValueLedgerPort,
    MembershipPurchaseFulfillmentPort, OrderPaymentSettlementAttempt,
    OwnerOrderPaymentConfirmationPort, OwnerOrderPaymentStatePort, OwnerOrderSettlementPorts,
};
use sdkwork_payment_repository_sqlx::{
    PostgresCommerceOwnerOrderPaymentStore, SqliteCommerceOwnerOrderPaymentStore,
};
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::api_response::{
    forbidden, map_service_error, not_found, success_created_item, unauthorized, validation,
};
use crate::backend_command_headers::{
    validate_backend_write_payload, write_payload_with_route_param,
};

use crate::subject::{backend_operator_scope_from_iam, BackendOperatorScope};

mod permissions {
    pub const CONFIRM: &str = "commerce.orders.fulfill";
}

#[derive(Clone)]
enum PaymentConfirmationStoreKind {
    Sqlite {
        payments: Arc<SqliteCommerceOwnerOrderPaymentStore>,
        recharge: Arc<SqliteCommerceRechargeStore>,
        orders: Arc<SqliteCommerceOrderStore>,
    },
    Postgres {
        payments: Arc<PostgresCommerceOwnerOrderPaymentStore>,
        recharge: Arc<PostgresCommerceRechargeStore>,
        orders: Arc<PostgresCommerceOrderStore>,
    },
}

#[derive(Clone)]
struct PaymentConfirmationState {
    store: PaymentConfirmationStoreKind,
    credit_port: Arc<dyn AccountPointsCreditPort>,
    account_value_ledger_port: Arc<dyn AccountValueLedgerPort>,
    membership_port: Arc<dyn MembershipPurchaseFulfillmentPort>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfirmOrderPaymentRequest {
    request_no: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfirmOrderPaymentResponse {
    payment_confirmed: bool,
    payment_replayed: bool,
    fulfillment_accepted: bool,
    fulfillment_replayed: bool,
    order_id: String,
    points_credited: i64,
    fulfillment_status: String,
}

pub fn payment_confirmation_router_with_sqlite_pool(
    pool: SqlitePool,
    credit_port: Arc<dyn AccountPointsCreditPort>,
    account_value_ledger_port: Arc<dyn AccountValueLedgerPort>,
    membership_port: Arc<dyn MembershipPurchaseFulfillmentPort>,
) -> Router {
    build_payment_confirmation_router(PaymentConfirmationState {
        store: PaymentConfirmationStoreKind::Sqlite {
            payments: Arc::new(SqliteCommerceOwnerOrderPaymentStore::new(pool.clone())),
            recharge: Arc::new(SqliteCommerceRechargeStore::new(pool.clone())),
            orders: Arc::new(SqliteCommerceOrderStore::new(pool)),
        },
        credit_port,
        account_value_ledger_port,
        membership_port,
    })
}

pub fn payment_confirmation_router_with_postgres_pool(
    pool: PgPool,
    credit_port: Arc<dyn AccountPointsCreditPort>,
    account_value_ledger_port: Arc<dyn AccountValueLedgerPort>,
    membership_port: Arc<dyn MembershipPurchaseFulfillmentPort>,
) -> Router {
    build_payment_confirmation_router(PaymentConfirmationState {
        store: PaymentConfirmationStoreKind::Postgres {
            payments: Arc::new(PostgresCommerceOwnerOrderPaymentStore::new(pool.clone())),
            recharge: Arc::new(PostgresCommerceRechargeStore::new(pool.clone())),
            orders: Arc::new(PostgresCommerceOrderStore::new(pool)),
        },
        credit_port,
        account_value_ledger_port,
        membership_port,
    })
}

fn build_payment_confirmation_router(state: PaymentConfirmationState) -> Router {
    Router::new()
        .route(
            "/backend/v3/api/orders/{orderId}/payment_confirmations",
            post(confirm_order_payment),
        )
        .with_state(state)
}

async fn confirm_order_payment(
    State(state): State<PaymentConfirmationState>,
    Extension(runtime_context): Extension<IamAppContext>,
    request_context: Extension<WebRequestContext>,
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(body): Json<ConfirmOrderPaymentRequest>,
) -> Response {
    let ctx = Some(&request_context.0);
    let subject = match require_confirmation_subject(runtime_context, ctx) {
        Ok(subject) => subject,
        Err(response) => return response,
    };

    if body.request_no.trim().is_empty() {
        return validation(ctx, "request_no is required");
    }

    let payload = write_payload_with_route_param("orderId", &order_id, &body);
    let _write_headers = match validate_backend_write_payload(
        ctx,
        &headers,
        "orders.paymentConfirmations.create",
        &payload,
        |idempotency_key| format!("pay-confirm-{order_id}-{idempotency_key}"),
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };

    let credit_port = state.credit_port.clone();
    let account_value_ledger_port = state.account_value_ledger_port.clone();
    let membership_port = state.membership_port.clone();
    match state.store {
        PaymentConfirmationStoreKind::Sqlite {
            ref payments,
            ref recharge,
            ref orders,
        } => {
            confirm_order_payment_inner(
                ctx,
                &subject,
                &order_id,
                &body.request_no,
                payments.as_ref(),
                recharge.as_ref(),
                orders.as_ref(),
                credit_port.as_ref(),
                account_value_ledger_port.as_ref(),
                membership_port.as_ref(),
            )
            .await
        }
        PaymentConfirmationStoreKind::Postgres {
            ref payments,
            ref recharge,
            ref orders,
        } => {
            confirm_order_payment_inner(
                ctx,
                &subject,
                &order_id,
                &body.request_no,
                payments.as_ref(),
                recharge.as_ref(),
                orders.as_ref(),
                credit_port.as_ref(),
                account_value_ledger_port.as_ref(),
                membership_port.as_ref(),
            )
            .await
        }
    }
}

async fn confirm_order_payment_inner(
    ctx: Option<&WebRequestContext>,
    subject: &BackendOperatorScope,
    order_id: &str,
    request_no: &str,
    payment_store: &impl OwnerOrderPaymentConfirmationPort,
    recharge_store: &(impl sdkwork_order_service::PointsRechargeFulfillmentStore
          + sdkwork_order_service::AccountValueFulfillmentStore),
    order_store: &(impl OrderSettlementContextLoader + OwnerOrderPaymentStatePort),
    credit_port: &dyn AccountPointsCreditPort,
    account_value_ledger_port: &dyn AccountValueLedgerPort,
    membership_port: &dyn MembershipPurchaseFulfillmentPort,
) -> Response {
    let order_context = match order_store
        .load_order_payment_settlement_context(
            &subject.tenant_id,
            subject.organization_id.as_deref(),
            order_id,
        )
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => return not_found(ctx, "order was not found"),
        Err(error) => return map_service_error(ctx, error),
    };

    let attempt = OrderPaymentSettlementAttempt {
        tenant_id: subject.tenant_id.clone(),
        organization_id: subject.organization_id.clone(),
        owner_user_id: order_context.owner_user_id,
        order_id: order_id.to_owned(),
        payment_attempt_id: None,
        out_trade_no: None,
    };

    let settlement_outcome = match settle_owner_order_after_payment_success(
        OwnerOrderSettlementPorts {
            payment_store,
            order_state_store: order_store,
            recharge_store,
            account_value_store: recharge_store,
            credit_port,
            account_value_ledger_port,
            membership_port,
        },
        &attempt,
        Some(order_context.subject.as_str()),
        request_no,
    )
    .await
    {
        Ok(outcome) => outcome,
        Err(error) => return map_service_error(ctx, error),
    };

    success_created_item(
        ctx,
        ConfirmOrderPaymentResponse {
            payment_confirmed: settlement_outcome.payment_confirmed,
            payment_replayed: settlement_outcome.payment_replayed,
            fulfillment_accepted: settlement_outcome.fulfillment_accepted,
            fulfillment_replayed: settlement_outcome.fulfillment_replayed,
            order_id: settlement_outcome.order_id,
            points_credited: settlement_outcome.points_credited,
            fulfillment_status: settlement_outcome.fulfillment_status,
        },
    )
}

fn require_confirmation_subject(
    context: IamAppContext,
    web_context: Option<&WebRequestContext>,
) -> Result<BackendOperatorScope, Response> {
    if !context.can_access_backend_api() {
        return Err(forbidden(
            web_context,
            "backend api access requires an organization-scoped session",
        ));
    }
    if !context.has_permission(permissions::CONFIRM) {
        return Err(forbidden(
            web_context,
            format!("missing required permission: {}", permissions::CONFIRM),
        ));
    }
    match backend_operator_scope_from_iam(&context) {
        Ok(subject) => Ok(subject),
        Err(message) => Err(unauthorized(web_context, message)),
    }
}

pub(crate) trait OrderSettlementContextLoader: Send + Sync {
    fn load_order_payment_settlement_context<'a>(
        &'a self,
        tenant_id: &'a str,
        organization_id: Option<&'a str>,
        order_id: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Option<OrderPaymentSettlementContext>, CommerceServiceError>,
                > + Send
                + 'a,
        >,
    >;
}

impl OrderSettlementContextLoader for SqliteCommerceOrderStore {
    fn load_order_payment_settlement_context<'a>(
        &'a self,
        tenant_id: &'a str,
        organization_id: Option<&'a str>,
        order_id: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Option<OrderPaymentSettlementContext>, CommerceServiceError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            self.load_order_payment_settlement_context(tenant_id, organization_id, order_id)
                .await
        })
    }
}

impl OrderSettlementContextLoader for PostgresCommerceOrderStore {
    fn load_order_payment_settlement_context<'a>(
        &'a self,
        tenant_id: &'a str,
        organization_id: Option<&'a str>,
        order_id: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Option<OrderPaymentSettlementContext>, CommerceServiceError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            self.load_order_payment_settlement_context(tenant_id, organization_id, order_id)
                .await
        })
    }
}

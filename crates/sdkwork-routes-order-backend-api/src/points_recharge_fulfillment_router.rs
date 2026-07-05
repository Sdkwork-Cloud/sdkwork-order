use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use axum::routing::post;
use axum::{Json, Router};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_order_repository_sqlx::{
    PostgresCommerceRechargeStore, SqliteCommerceRechargeStore,
};
use sdkwork_order_service::{
    default_fulfill_points_recharge_command, fulfill_points_recharge_order,
    mark_points_recharge_payment_succeeded, points_recharge_payment_success_idempotency_key,
    AccountPointsCreditPort, MarkPointsRechargePaymentSucceededCommand,
};
use sdkwork_web_core::WebRequestContext;
use serde::Deserialize;
use sqlx::{PgPool, SqlitePool};

use crate::api_response::{
    forbidden, map_service_error, success_command, unauthorized, validation,
};
use crate::subject::{backend_operator_scope_from_iam, BackendOperatorScope};

mod permissions {
    /// Saga write: mark payment success and fulfill points recharge orders.
    pub const FULFILL: &str = "commerce.orders.fulfill";
}

#[derive(Clone)]
enum PointsRechargeFulfillmentStoreKind {
    Sqlite(Arc<SqliteCommerceRechargeStore>),
    Postgres(Arc<PostgresCommerceRechargeStore>),
}

#[derive(Clone)]
struct PointsRechargeFulfillmentState {
    store: PointsRechargeFulfillmentStoreKind,
    credit_port: Arc<dyn AccountPointsCreditPort>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatePointsRechargeFulfillmentRequest {
    request_no: String,
    #[serde(default)]
    idempotency_key: Option<String>,
    #[serde(default)]
    paid_at: Option<String>,
}

pub fn points_recharge_fulfillment_router_with_sqlite_pool(
    pool: SqlitePool,
    credit_port: Arc<dyn AccountPointsCreditPort>,
) -> Router {
    build_points_recharge_fulfillment_router(PointsRechargeFulfillmentState {
        store: PointsRechargeFulfillmentStoreKind::Sqlite(Arc::new(
            SqliteCommerceRechargeStore::new(pool),
        )),
        credit_port,
    })
}

pub fn points_recharge_fulfillment_router_with_postgres_pool(
    pool: PgPool,
    credit_port: Arc<dyn AccountPointsCreditPort>,
) -> Router {
    build_points_recharge_fulfillment_router(PointsRechargeFulfillmentState {
        store: PointsRechargeFulfillmentStoreKind::Postgres(Arc::new(
            PostgresCommerceRechargeStore::new(pool),
        )),
        credit_port,
    })
}

fn build_points_recharge_fulfillment_router(state: PointsRechargeFulfillmentState) -> Router {
    Router::new()
        .route(
            "/backend/v3/api/orders/{orderId}/points_recharge/fulfillments",
            post(create_points_recharge_fulfillment),
        )
        .with_state(state)
}

async fn create_points_recharge_fulfillment(
    State(state): State<PointsRechargeFulfillmentState>,
    Extension(runtime_context): Extension<IamAppContext>,
    request_context: Extension<WebRequestContext>,
    Path(order_id): Path<String>,
    Json(body): Json<CreatePointsRechargeFulfillmentRequest>,
) -> Response {
    let ctx = request_context.0;
    let subject = match require_fulfillment_subject(runtime_context, Some(&ctx)) {
        Ok(subject) => subject,
        Err(response) => return response,
    };

    let owner_user_id = match resolve_points_recharge_order_owner(&state.store, &subject, &order_id).await {
        Ok(Some(owner_user_id)) => owner_user_id,
        Ok(None) => {
            return crate::api_response::not_found(
                Some(&ctx),
                "points recharge order was not found",
            )
        }
        Err(error) => return map_service_error(Some(&ctx), error),
    };

    if body.request_no.trim().is_empty() {
        return validation(Some(&ctx), "request_no is required");
    }

    let idempotency_key = body
        .idempotency_key
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| points_recharge_payment_success_idempotency_key(&order_id));

    if let Some(paid_at) = body.paid_at.as_deref().filter(|value| !value.trim().is_empty()) {
        let payment_command = match MarkPointsRechargePaymentSucceededCommand::new(
            &subject.tenant_id,
            subject.organization_id.as_deref(),
            owner_user_id.as_str(),
            &order_id,
            paid_at,
            &body.request_no,
            &idempotency_key,
        ) {
            Ok(command) => command,
            Err(error) => return map_service_error(Some(&ctx), error),
        };
        if let Err(error) = mark_payment_succeeded(&state.store, payment_command).await {
            return map_service_error(Some(&ctx), error);
        }
    }

    let fulfill_command = match default_fulfill_points_recharge_command(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        owner_user_id.as_str(),
        &order_id,
        &body.request_no,
    ) {
        Ok(command) => command,
        Err(error) => return map_service_error(Some(&ctx), error),
    };

    match fulfill_order(&state.store, state.credit_port.as_ref(), fulfill_command).await {
        Ok(outcome) => success_command(
            Some(&ctx),
            Some(outcome.order_id),
            Some(outcome.fulfillment_status),
        ),
        Err(error) => map_service_error(Some(&ctx), error),
    }
}

fn require_fulfillment_subject(
    context: IamAppContext,
    web_context: Option<&WebRequestContext>,
) -> Result<BackendOperatorScope, Response> {
    if !context.can_access_backend_api() {
        return Err(forbidden(
            web_context,
            "backend api access requires an organization-scoped session",
        ));
    }
    if !context.has_permission(permissions::FULFILL) {
        tracing::warn!(
            target = "order.acl",
            user_id = %context.user_id,
            tenant_id = %context.tenant_id,
            required_permission = permissions::FULFILL,
            "points recharge fulfillment permission denied"
        );
        return Err(forbidden(
            web_context,
            format!("missing required permission: {}", permissions::FULFILL),
        ));
    }
    match backend_operator_scope_from_iam(&context) {
        Ok(subject) => Ok(subject),
        Err(message) => Err(unauthorized(web_context, message)),
    }
}

async fn mark_payment_succeeded(
    store: &PointsRechargeFulfillmentStoreKind,
    command: MarkPointsRechargePaymentSucceededCommand,
) -> Result<(), CommerceServiceError> {
    match store {
        PointsRechargeFulfillmentStoreKind::Sqlite(store) => {
            mark_points_recharge_payment_succeeded(store.as_ref(), command).await
        }
        PointsRechargeFulfillmentStoreKind::Postgres(store) => {
            mark_points_recharge_payment_succeeded(store.as_ref(), command).await
        }
    }
}

async fn fulfill_order(
    store: &PointsRechargeFulfillmentStoreKind,
    credit_port: &dyn AccountPointsCreditPort,
    command: sdkwork_order_service::FulfillPointsRechargeOrderCommand,
) -> Result<sdkwork_order_service::FulfillPointsRechargeOrderOutcome, CommerceServiceError> {
    match store {
        PointsRechargeFulfillmentStoreKind::Sqlite(store) => {
            fulfill_points_recharge_order(store.as_ref(), credit_port, command).await
        }
        PointsRechargeFulfillmentStoreKind::Postgres(store) => {
            fulfill_points_recharge_order(store.as_ref(), credit_port, command).await
        }
    }
}

async fn resolve_points_recharge_order_owner(
    store: &PointsRechargeFulfillmentStoreKind,
    subject: &BackendOperatorScope,
    order_id: &str,
) -> Result<Option<String>, CommerceServiceError> {
    match store {
        PointsRechargeFulfillmentStoreKind::Sqlite(store) => {
            store
                .resolve_points_recharge_order_owner(
                    &subject.tenant_id,
                    subject.organization_id.as_deref(),
                    order_id,
                )
                .await
        }
        PointsRechargeFulfillmentStoreKind::Postgres(store) => {
            store
                .resolve_points_recharge_order_owner(
                    &subject.tenant_id,
                    subject.organization_id.as_deref(),
                    order_id,
                )
                .await
        }
    }
}

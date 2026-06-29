use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_order_repository_sqlx::{
    PostgresCommerceOrderStore, SqliteCommerceOrderStore,
};
use sdkwork_order_service::{
    CancelManagementOrderCommand, CloseManagementOrderCommand, OrderCancellationListQuery,
    OrderCancellationView, OrderManagementDetailQuery, OrderManagementEventListQuery,
    OrderManagementEventView, OrderManagementListQuery, OrderOwnerDetail, OrderOwnerSummary,
};
use sdkwork_iam_context_service::IamAppContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::subject::{app_runtime_subject_from_extension, app_runtime_subject_from_iam};

/// Permission codes enforced on the backend order admin API surface.
///
/// The codes are domain-scoped (`commerce.orders.*`) so they can be granted
/// through standard SDKWork IAM permission catalogs without colliding with
/// other commerce capabilities (payment, catalog, etc.).
mod permissions {
    /// Read access to orders, events, and cancellations.
    pub const READ: &str = "commerce.orders.read";
    /// Write access: cancel, close, or otherwise mutate order state.
    pub const MANAGE: &str = "commerce.orders.manage";
}

#[derive(Clone)]
enum BackendOrderAdminStore {
    Postgres(Arc<PostgresCommerceOrderStore>),
    Sqlite(Arc<SqliteCommerceOrderStore>),
}

#[derive(Clone)]
struct BackendOrderAdminState {
    store: BackendOrderAdminStore,
}

#[derive(Debug, Deserialize)]
struct OrderListParams {
    status: Option<String>,
    q: Option<String>,
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct OrderEventListParams {
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct CancellationListParams {
    status: Option<String>,
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct CancelOrderBody {
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CloseOrderBody {
    reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BackendOrderApiResult<T: Serialize> {
    code: String,
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderSummaryResponse {
    order_id: String,
    order_sn: String,
    status: String,
    status_name: String,
    subject: String,
    total_amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    paid_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    discount_amount: Option<String>,
    quantity: i64,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pay_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expire_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payment_method: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderDetailResponse {
    #[serde(flatten)]
    summary: OrderSummaryResponse,
    items: Vec<OrderItemResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    out_trade_no: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transaction_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderItemResponse {
    id: String,
    product_name: String,
    quantity: i64,
    unit_price: String,
    total_amount: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderEventResponse {
    id: String,
    event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_status: Option<String>,
    to_status: String,
    actor_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    actor_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderCancellationResponse {
    id: String,
    order_id: String,
    status: String,
    reason_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason_message: Option<String>,
    created_at: String,
}

pub fn backend_order_admin_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_backend_order_admin_router(BackendOrderAdminStore::Sqlite(Arc::new(
        SqliteCommerceOrderStore::new(pool),
    )))
}

pub fn backend_order_admin_router_with_postgres_pool(pool: PgPool) -> Router {
    build_backend_order_admin_router(BackendOrderAdminStore::Postgres(Arc::new(
        PostgresCommerceOrderStore::new(pool),
    )))
}

fn build_backend_order_admin_router(store: BackendOrderAdminStore) -> Router {
    let state = BackendOrderAdminState { store };
    Router::new()
        .route("/backend/v3/api/orders", get(list_orders))
        .route("/backend/v3/api/orders/cancellations", get(list_cancellations))
        .route("/backend/v3/api/orders/{orderId}", get(retrieve_order))
        .route("/backend/v3/api/orders/{orderId}/cancel", post(cancel_order))
        .route("/backend/v3/api/orders/{orderId}/close", post(close_order))
        .route("/backend/v3/api/orders/{orderId}/events", get(list_order_events))
        .with_state(state)
}

impl BackendOrderAdminStore {
    async fn list_management_orders(
        &self,
        query: OrderManagementListQuery,
    ) -> Result<Vec<OrderOwnerSummary>, CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.list_management_orders(query).await,
            Self::Sqlite(store) => store.list_management_orders(query).await,
        }
    }

    async fn retrieve_management_order(
        &self,
        query: OrderManagementDetailQuery,
    ) -> Result<Option<OrderOwnerDetail>, CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.retrieve_management_order(query).await,
            Self::Sqlite(store) => store.retrieve_management_order(query).await,
        }
    }

    async fn cancel_management_order(
        &self,
        command: CancelManagementOrderCommand,
    ) -> Result<(), CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.cancel_management_order(command).await,
            Self::Sqlite(store) => store.cancel_management_order(command).await,
        }
    }

    async fn close_management_order(
        &self,
        command: CloseManagementOrderCommand,
    ) -> Result<(), CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.close_management_order(command).await,
            Self::Sqlite(store) => store.close_management_order(command).await,
        }
    }

    async fn list_management_order_events(
        &self,
        query: OrderManagementEventListQuery,
    ) -> Result<Vec<OrderManagementEventView>, CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.list_management_order_events(query).await,
            Self::Sqlite(store) => store.list_management_order_events(query).await,
        }
    }

    async fn list_order_cancellations(
        &self,
        query: OrderCancellationListQuery,
    ) -> Result<Vec<OrderCancellationView>, CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.list_order_cancellations(query).await,
            Self::Sqlite(store) => store.list_order_cancellations(query).await,
        }
    }
}

async fn list_orders(
    State(state): State<BackendOrderAdminState>,
    Extension(runtime_context): Extension<IamAppContext>,
    Query(params): Query<OrderListParams>,
) -> Response {
    let subject = match require_backend_subject(runtime_context, permissions::READ) {
        Ok(subject) => subject,
        Err(response) => return response,
    };
    let query = match OrderManagementListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        params.status.as_deref(),
        params.q.as_deref(),
        params.page,
        params.page_size,
    ) {
        Ok(query) => query,
        Err(error) => return validation_error(error.message()),
    };

    match state.store.list_management_orders(query).await {
        Ok(items) => ok_json(
            items
                .into_iter()
                .map(map_order_summary)
                .collect::<Vec<_>>(),
        ),
        Err(error) => storage_error(error),
    }
}

async fn retrieve_order(
    State(state): State<BackendOrderAdminState>,
    Extension(runtime_context): Extension<IamAppContext>,
    Path(order_id): Path<String>,
) -> Response {
    let subject = match require_backend_subject(runtime_context, permissions::READ) {
        Ok(subject) => subject,
        Err(response) => return response,
    };
    let query = match OrderManagementDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &order_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation_error(error.message()),
    };

    match state.store.retrieve_management_order(query).await {
        Ok(Some(detail)) => ok_json(map_order_detail(detail)),
        Ok(None) => not_found("order not found"),
        Err(error) => storage_error(error),
    }
}

async fn cancel_order(
    State(state): State<BackendOrderAdminState>,
    Extension(runtime_context): Extension<IamAppContext>,
    Path(order_id): Path<String>,
    Json(body): Json<CancelOrderBody>,
) -> Response {
    let subject = match require_backend_subject(runtime_context, permissions::MANAGE) {
        Ok(subject) => subject,
        Err(response) => return response,
    };
    let command = match CancelManagementOrderCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &order_id,
        body.reason.as_deref(),
    ) {
        Ok(command) => command,
        Err(error) => return validation_error(error.message()),
    };

    match state.store.cancel_management_order(command).await {
        Ok(()) => ok_json(serde_json::json!({ "orderId": order_id, "status": "cancelled" })),
        Err(error) if error.code() == "conflict" => conflict(error.message().to_owned()),
        Err(error) => storage_error(error),
    }
}

async fn close_order(
    State(state): State<BackendOrderAdminState>,
    Extension(runtime_context): Extension<IamAppContext>,
    Path(order_id): Path<String>,
    Json(body): Json<CloseOrderBody>,
) -> Response {
    let subject = match require_backend_subject(runtime_context, permissions::MANAGE) {
        Ok(subject) => subject,
        Err(response) => return response,
    };
    let command = match CloseManagementOrderCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &order_id,
        body.reason.as_deref(),
    ) {
        Ok(command) => command,
        Err(error) => return validation_error(error.message()),
    };

    match state.store.close_management_order(command).await {
        Ok(()) => ok_json(serde_json::json!({ "orderId": order_id, "status": "closed" })),
        Err(error) if error.code() == "conflict" => conflict(error.message().to_owned()),
        Err(error) => storage_error(error),
    }
}

async fn list_order_events(
    State(state): State<BackendOrderAdminState>,
    Extension(runtime_context): Extension<IamAppContext>,
    Path(order_id): Path<String>,
    Query(params): Query<OrderEventListParams>,
) -> Response {
    let subject = match require_backend_subject(runtime_context, permissions::READ) {
        Ok(subject) => subject,
        Err(response) => return response,
    };
    let query = match OrderManagementEventListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &order_id,
        params.page,
        params.page_size,
    ) {
        Ok(query) => query,
        Err(error) => return validation_error(error.message()),
    };

    match state.store.list_management_order_events(query).await {
        Ok(items) => ok_json(items.into_iter().map(map_order_event).collect::<Vec<_>>()),
        Err(error) => storage_error(error),
    }
}

async fn list_cancellations(
    State(state): State<BackendOrderAdminState>,
    Extension(runtime_context): Extension<IamAppContext>,
    Query(params): Query<CancellationListParams>,
) -> Response {
    let subject = match require_backend_subject(runtime_context, permissions::READ) {
        Ok(subject) => subject,
        Err(response) => return response,
    };
    let query = match OrderCancellationListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        params.status.as_deref(),
        params.page,
        params.page_size,
    ) {
        Ok(query) => query,
        Err(error) => return validation_error(error.message()),
    };

    match state.store.list_order_cancellations(query).await {
        Ok(items) => ok_json(
            items
                .into_iter()
                .map(map_order_cancellation)
                .collect::<Vec<_>>(),
        ),
        Err(error) => storage_error(error),
    }
}

/// Authorizes the caller against the backend order admin API surface.
///
/// The caller must:
/// 1. Be a member of an organization (`can_access_backend_api()`), since the
///    backend API is gated to organization- scoped sessions, not personal app
///    sessions.
/// 2. Hold the required permission code in their `permission_scope`.
///
/// Returns the resolved [`AppRuntimeSubject`] on success, or an HTTP error
/// response ready to be returned from the handler.
fn require_backend_subject(
    context: IamAppContext,
    required_permission: &str,
) -> Result<crate::subject::AppRuntimeSubject, Response> {
    if !context.can_access_backend_api() {
        return Err(forbidden(
            "backend api access requires an organization-scoped session",
        ));
    }
    if !context.has_permission(required_permission) {
        tracing::warn!(
            target = "order.acl",
            user_id = %context.user_id,
            tenant_id = %context.tenant_id,
            required_permission,
            "backend order admin permission denied"
        );
        return Err(forbidden(&format!(
            "missing required permission: {required_permission}"
        )));
    }
    match app_runtime_subject_from_iam(&context) {
        Ok(subject) => Ok(subject),
        Err(message) => Err(unauthorized(message)),
    }
}

fn ok_json<T: Serialize>(data: T) -> Response {
    (
        StatusCode::OK,
        Json(BackendOrderApiResult {
            code: "0".to_owned(),
            msg: "ok".to_owned(),
            data: Some(data),
        }),
    )
        .into_response()
}

fn unauthorized(message: String) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(BackendOrderApiResult::<()> {
            code: "4010".to_owned(),
            msg: message.into(),
            data: None,
        }),
    )
        .into_response()
}

fn forbidden(message: impl Into<String>) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(BackendOrderApiResult::<()> {
            code: "4030".to_owned(),
            msg: message.into(),
            data: None,
        }),
    )
        .into_response()
}

fn not_found(message: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(BackendOrderApiResult::<()> {
            code: "4040".to_owned(),
            msg: message.to_owned(),
            data: None,
        }),
    )
        .into_response()
}

fn validation_error(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(BackendOrderApiResult::<()> {
            code: "4000".to_owned(),
            msg: message.into(),
            data: None,
        }),
    )
        .into_response()
}

fn conflict(message: String) -> Response {
    (
        StatusCode::CONFLICT,
        Json(BackendOrderApiResult::<()> {
            code: "4090".to_owned(),
            msg: message.into(),
            data: None,
        }),
    )
        .into_response()
}

fn storage_error(error: CommerceServiceError) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(BackendOrderApiResult::<()> {
            code: "5000".to_owned(),
            msg: error.message().to_owned(),
            data: None,
        }),
    )
        .into_response()
}

fn map_order_summary(value: OrderOwnerSummary) -> OrderSummaryResponse {
    OrderSummaryResponse {
        order_id: value.order_id,
        order_sn: value.order_sn,
        status: value.status.clone(),
        status_name: format_order_status_name(&value.status),
        subject: value.subject,
        total_amount: value.total_amount.as_str().to_owned(),
        paid_amount: value.paid_amount.map(|amount| amount.as_str().to_owned()),
        discount_amount: value
            .discount_amount
            .map(|amount| amount.as_str().to_owned()),
        quantity: value.quantity,
        created_at: value.created_at,
        pay_time: value.pay_time,
        expire_time: value.expire_time,
        payment_method: value.payment_method,
    }
}

fn map_order_detail(value: OrderOwnerDetail) -> OrderDetailResponse {
    OrderDetailResponse {
        summary: map_order_summary(value.summary),
        items: value
            .items
            .into_iter()
            .map(|item| OrderItemResponse {
                id: item.id,
                product_name: item.product_name,
                quantity: item.quantity,
                unit_price: item.unit_price.as_str().to_owned(),
                total_amount: item.total_amount.as_str().to_owned(),
            })
            .collect(),
        out_trade_no: value.out_trade_no,
        transaction_id: value.transaction_id,
    }
}

fn map_order_event(value: OrderManagementEventView) -> OrderEventResponse {
    OrderEventResponse {
        id: value.id,
        event_type: value.event_type,
        from_status: value.from_status,
        to_status: value.to_status,
        actor_type: value.actor_type,
        actor_id: value.actor_id,
        message: value.message,
        created_at: value.created_at,
    }
}

fn map_order_cancellation(value: OrderCancellationView) -> OrderCancellationResponse {
    OrderCancellationResponse {
        id: value.id,
        order_id: value.order_id,
        status: value.status,
        reason_code: value.reason_code,
        reason_message: value.reason_message,
        created_at: value.created_at,
    }
}

fn format_order_status_name(status: &str) -> String {
    match status.trim().to_ascii_lowercase().as_str() {
        "pending_payment" | "unpaid" | "wait_pay" => "Pending payment".to_owned(),
        "paid" => "Paid".to_owned(),
        "completed" | "finished" => "Completed".to_owned(),
        "cancelled" | "canceled" | "closed" => "Cancelled".to_owned(),
        "expired" | "timeout" => "Expired".to_owned(),
        other => other.to_owned(),
    }
}

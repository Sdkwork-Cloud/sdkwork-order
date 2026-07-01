use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_order_service::{
    checkout_owner_order_request_hash, CancelOwnerOrderCommand, CreateOwnerOrderCommand,
    CreateOwnerOrderOutcome, OrderOwnerDetail, OrderOwnerDetailQuery, OrderOwnerListPage,
    OrderOwnerListQuery, OrderOwnerStatistics, OrderOwnerSummary, PayOwnerOrderCommand,
    PayOwnerOrderOutcome,
};
use sdkwork_order_repository_sqlx::{
    PostgresCommerceOrderStore, SqliteCommerceOrderStore,
};
use sdkwork_payment_repository_sqlx::{
    PostgresCommerceOwnerOrderPaymentStore, SqliteCommerceOwnerOrderPaymentStore,
};
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::api_response::{
    conflict, map_service_error, not_found, not_implemented, success_command, success_item,
    success_items, unauthorized, validation,
};
use crate::command_headers::{ensure_request_hash_matches, required_app_write_command_headers};
use crate::subject::{app_runtime_subject_from_extension, AppRuntimeSubject};

pub type CommerceOrderFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceOrderStore: Send + Sync {
    fn list_owner_orders<'a>(
        &'a self,
        query: OrderOwnerListQuery,
    ) -> CommerceOrderFuture<'a, OrderOwnerListPage>;

    fn retrieve_owner_order<'a>(
        &'a self,
        query: OrderOwnerDetailQuery,
    ) -> CommerceOrderFuture<'a, Option<OrderOwnerDetail>>;

    fn retrieve_owner_order_statistics<'a>(
        &'a self,
        tenant_id: String,
        organization_id: Option<String>,
        owner_user_id: String,
    ) -> CommerceOrderFuture<'a, OrderOwnerStatistics>;

    fn cancel_owner_order<'a>(
        &'a self,
        command: CancelOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, ()>;

    fn create_owner_order<'a>(
        &'a self,
        command: CreateOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, CreateOwnerOrderOutcome>;
}

#[derive(Clone)]
struct AppOrderState {
    store: Arc<dyn CommerceOrderStore>,
    payments: Arc<dyn OwnerOrderPaymentStore>,
}

pub trait OwnerOrderPaymentStore: Send + Sync {
    fn pay_owner_order<'a>(
        &'a self,
        command: PayOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, PayOwnerOrderOutcome>;

    fn cancel_owner_order_payments<'a>(
        &'a self,
        command: CancelOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, ()>;
}

#[derive(Debug, Deserialize)]
struct OrderListQueryParams {
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
    status: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderPageResponse {
    content: Vec<OrderSummaryResponse>,
    page: i64,
    #[serde(rename = "pageSize")]
    page_size: i64,
    total: i64,
    #[serde(rename = "hasMore")]
    has_more: bool,
    #[serde(rename = "totalPages", skip_serializing_if = "Option::is_none")]
    total_pages: Option<i64>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payment_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    product_image: Option<()>,
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
struct OrderStatusResponse {
    status: String,
    status_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderPaymentSuccessResponse {
    paid: bool,
    status: String,
    status_name: String,
}

#[derive(Debug, Deserialize)]
struct CancelOrderRequest {
    #[serde(rename = "cancelReason", alias = "cancel_reason")]
    cancel_reason: Option<String>,
    #[serde(rename = "cancelType", alias = "cancel_type")]
    cancel_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PayOrderRequest {
    #[serde(rename = "paymentMethod", alias = "payment_method")]
    payment_method: Option<String>,
    #[serde(rename = "paymentPassword", alias = "payment_password")]
    payment_password: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateOrderRequest {
    checkout_session_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateOrderResponse {
    order_id: String,
    order_sn: String,
    status: String,
    total_amount: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderPaymentParamsResponse {
    amount: String,
    order_id: String,
    out_trade_no: String,
    payment_id: String,
    payment_method: String,
    payment_params: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderStatisticsResponse {
    total_orders: i64,
    pending_payment: i64,
    pending_shipment: i64,
    pending_receipt: i64,
    completed: i64,
    total_amount: String,
}

pub fn app_order_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_app_order_router(
        Arc::new(SqliteCommerceOrderStore::new(pool.clone())),
        Arc::new(SqliteCommerceOwnerOrderPaymentStore::new(pool)),
    )
}

pub fn app_order_router_with_postgres_pool(pool: PgPool) -> Router {
    build_app_order_router(
        Arc::new(PostgresCommerceOrderStore::new(pool.clone())),
        Arc::new(PostgresCommerceOwnerOrderPaymentStore::new(pool)),
    )
}

pub fn build_app_order_router(
    store: Arc<dyn CommerceOrderStore>,
    payments: Arc<dyn OwnerOrderPaymentStore>,
) -> Router {
    Router::new()
            .route("/app/v3/api/orders", get(list_orders).post(create_order))
            .route("/app/v3/api/orders/statistics", get(fetch_order_statistics))
            .route(
                "/app/v3/api/orders/{orderId}",
                get(fetch_order).patch(unavailable_command),
            )
            .route("/app/v3/api/orders/{orderId}/payments", post(pay_order))
            .route("/app/v3/api/orders/{orderId}/cancel", post(cancel_order))
            .route(
                "/app/v3/api/orders/{orderId}/status",
                get(fetch_order_status),
            )
            .route(
                "/app/v3/api/orders/{orderId}/payment_success",
                get(fetch_order_payment_success),
            )
            .route(
                "/app/v3/api/orders/{orderId}/events",
                get(fetch_order_events),
            )
            .route(
                "/app/v3/api/orders/{orderId}/cancellations",
                post(cancel_order),
            )
            .with_state(AppOrderState { store, payments })
}

async fn list_orders(
    State(state): State<AppOrderState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Query(params): Query<OrderListQueryParams>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let query = match OrderOwnerListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        params.status.as_deref(),
        params.page,
        params.page_size,
        None,
    ) {
        Ok(query) => query,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.store.list_owner_orders(query).await {
        Ok(page) => {
            let has_more = page.has_more();
            let total_pages = page.total_pages();
            success_item(ctx, OrderPageResponse {
                content: page.items.into_iter().map(map_order_summary).collect(),
                page: page.page,
                page_size: page.page_size,
                total: page.total,
                has_more,
                total_pages: Some(total_pages),
            })
        }
        Err(error) => map_service_error(ctx, error),
    }
}

async fn fetch_order_statistics(
    State(state): State<AppOrderState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };

    match state
        .store
        .retrieve_owner_order_statistics(
            subject.tenant_id,
            subject.organization_id,
            subject.user_id,
        )
        .await
    {
        Ok(statistics) => success_item(ctx, map_statistics(statistics)),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn fetch_order(
    State(state): State<AppOrderState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(order_id): Path<String>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let query = match OrderOwnerDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &order_id,
    ) {
        Ok(query) => query,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.store.retrieve_owner_order(query).await {
        Ok(Some(detail)) => success_item(ctx, map_order_detail(detail)),
        Ok(None) => not_found(ctx, "order was not found"),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn fetch_order_status(
    State(state): State<AppOrderState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(order_id): Path<String>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let query = match OrderOwnerDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &order_id,
    ) {
        Ok(query) => query,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.store.retrieve_owner_order(query).await {
        Ok(Some(detail)) => {
            let summary = map_order_summary(detail.summary);
            success_item(ctx, OrderStatusResponse {
                status: summary.status,
                status_name: summary.status_name,
            })
        }
        Ok(None) => not_found(ctx, "order was not found"),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn fetch_order_payment_success(
    State(state): State<AppOrderState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(order_id): Path<String>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let query = match OrderOwnerDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &order_id,
    ) {
        Ok(query) => query,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.store.retrieve_owner_order(query).await {
        Ok(Some(detail)) => {
            let summary = map_order_summary(detail.summary);
            let paid = summary.paid_amount.is_some()
                || matches!(
                    summary.status.to_ascii_lowercase().as_str(),
                    "paid" | "completed" | "fulfilled"
                );
            success_item(ctx, OrderPaymentSuccessResponse {
                paid,
                status: summary.status,
                status_name: summary.status_name,
            })
        }
        Ok(None) => not_found(ctx, "order was not found"),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn fetch_order_events(
    State(state): State<AppOrderState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(order_id): Path<String>,
) -> Response {
    let _ = (state, order_id);
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let _ = subject;
    success_items(ctx, Vec::<serde_json::Value>::new(), 1, 1)
}

async fn cancel_order(
    State(state): State<AppOrderState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(order_id): Path<String>,
    body: Option<Json<CancelOrderRequest>>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let cancel_reason = body.as_ref().and_then(|body| body.cancel_reason.clone());
    let _ = body.as_ref().and_then(|body| body.cancel_type.clone());
    let command = match CancelOwnerOrderCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &order_id,
        cancel_reason.as_deref(),
    ) {
        Ok(command) => command,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.store.cancel_owner_order(command.clone()).await {
        Ok(()) => match state.payments.cancel_owner_order_payments(command).await {
            Ok(()) => success_command(ctx, ()),
            Err(error) => map_service_error(ctx, error),
        },
        Err(error) => map_service_error(ctx, error),
    }
}

async fn pay_order(
    State(state): State<AppOrderState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(order_id): Path<String>,
    body: Option<Json<PayOrderRequest>>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let payment_method = body
        .as_ref()
        .and_then(|body| body.payment_method.clone())
        .unwrap_or_else(|| "wechat_pay".to_owned());
    let _ = body.as_ref().and_then(|body| body.payment_password.clone());
    let command = match PayOwnerOrderCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &order_id,
        &payment_method,
    ) {
        Ok(command) => command,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.payments.pay_owner_order(command).await {
        Ok(outcome) => success_item(ctx, map_pay_outcome(outcome)),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn create_order(
    State(state): State<AppOrderState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    headers: HeaderMap,
    body: Json<CreateOrderRequest>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let write_headers = match required_app_write_command_headers(&headers, |idempotency_key| {
        fallback_order_request_no(&subject, &body.checkout_session_id, idempotency_key)
    }) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let command = match CreateOwnerOrderCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &body.checkout_session_id,
        &write_headers.request_no,
        &write_headers.idempotency_key,
    ) {
        Ok(command) => command,
        Err(error) => return map_service_error(ctx, error),
    };
    if let Err(response) = ensure_request_hash_matches(
        &checkout_owner_order_request_hash(&command),
        &write_headers.request_hash,
    ) {
        return response;
    }

    match state.store.create_owner_order(command).await {
        Ok(outcome) => success_item(ctx, map_create_order(outcome)),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn unavailable_command(
    request_context: Option<Extension<WebRequestContext>>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    not_implemented(ctx, "commerce order command store is not configured")
}

fn map_create_order(value: CreateOwnerOrderOutcome) -> CreateOrderResponse {
    CreateOrderResponse {
        order_id: value.order_id,
        order_sn: value.order_sn,
        status: value.status,
        total_amount: value.total_amount.as_str().to_owned(),
    }
}

fn map_pay_outcome(value: PayOwnerOrderOutcome) -> OrderPaymentParamsResponse {
    OrderPaymentParamsResponse {
        amount: value.amount.as_str().to_owned(),
        order_id: value.order_id,
        out_trade_no: value.out_trade_no,
        payment_id: value.payment_id,
        payment_method: value.payment_method,
        payment_params: value.payment_params,
    }
}

fn map_order_summary(value: OrderOwnerSummary) -> OrderSummaryResponse {
    let status_name = format_order_status_name(&value.status);
    OrderSummaryResponse {
        order_id: value.order_id,
        order_sn: value.order_sn,
        status: value.status,
        status_name,
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
        remark: None,
        payment_provider: None,
        product_image: None,
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

fn map_statistics(value: OrderOwnerStatistics) -> OrderStatisticsResponse {
    OrderStatisticsResponse {
        total_orders: value.total_orders,
        pending_payment: value.pending_payment,
        pending_shipment: value.pending_shipment,
        pending_receipt: value.pending_receipt,
        completed: value.completed,
        total_amount: value.total_amount.as_str().to_owned(),
    }
}

fn format_order_status_name(status: &str) -> String {
    match status.trim().to_ascii_lowercase().as_str() {
        "pending_payment" | "unpaid" | "wait_pay" => "Pending payment".to_owned(),
        "paid" => "Paid".to_owned(),
        "completed" | "finished" => "Completed".to_owned(),
        "cancelled" | "canceled" | "closed" => "Cancelled".to_owned(),
        "expired" | "timeout" => "Expired".to_owned(),
        "refunding" => "Refunding".to_owned(),
        "refunded" => "Refunded".to_owned(),
        "fulfilled" => "Fulfilled".to_owned(),
        other => other.to_owned(),
    }
}

fn fallback_order_request_no(
    subject: &AppRuntimeSubject,
    checkout_session_id: &str,
    idempotency_key: &str,
) -> String {
    format!(
        "ORD-{}-{}-{}",
        subject.tenant_id, checkout_session_id, idempotency_key
    )
}

impl CommerceOrderStore for SqliteCommerceOrderStore {
    fn list_owner_orders<'a>(
        &'a self,
        query: OrderOwnerListQuery,
    ) -> CommerceOrderFuture<'a, OrderOwnerListPage> {
        Box::pin(async move { self.list_owner_orders(query).await })
    }

    fn retrieve_owner_order<'a>(
        &'a self,
        query: OrderOwnerDetailQuery,
    ) -> CommerceOrderFuture<'a, Option<OrderOwnerDetail>> {
        Box::pin(async move { self.retrieve_owner_order(query).await })
    }

    fn retrieve_owner_order_statistics<'a>(
        &'a self,
        tenant_id: String,
        organization_id: Option<String>,
        owner_user_id: String,
    ) -> CommerceOrderFuture<'a, OrderOwnerStatistics> {
        Box::pin(async move {
            self.retrieve_owner_order_statistics(
                tenant_id.as_str(),
                organization_id.as_deref(),
                owner_user_id.as_str(),
            )
            .await
        })
    }

    fn cancel_owner_order<'a>(
        &'a self,
        command: CancelOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, ()> {
        Box::pin(async move { self.cancel_owner_order(command).await })
    }

    fn create_owner_order<'a>(
        &'a self,
        command: CreateOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, CreateOwnerOrderOutcome> {
        Box::pin(async move { self.create_owner_order(command).await })
    }
}

impl CommerceOrderStore for PostgresCommerceOrderStore {
    fn list_owner_orders<'a>(
        &'a self,
        query: OrderOwnerListQuery,
    ) -> CommerceOrderFuture<'a, OrderOwnerListPage> {
        Box::pin(async move { self.list_owner_orders(query).await })
    }

    fn retrieve_owner_order<'a>(
        &'a self,
        query: OrderOwnerDetailQuery,
    ) -> CommerceOrderFuture<'a, Option<OrderOwnerDetail>> {
        Box::pin(async move { self.retrieve_owner_order(query).await })
    }

    fn retrieve_owner_order_statistics<'a>(
        &'a self,
        tenant_id: String,
        organization_id: Option<String>,
        owner_user_id: String,
    ) -> CommerceOrderFuture<'a, OrderOwnerStatistics> {
        Box::pin(async move {
            self.retrieve_owner_order_statistics(
                tenant_id.as_str(),
                organization_id.as_deref(),
                owner_user_id.as_str(),
            )
            .await
        })
    }

    fn cancel_owner_order<'a>(
        &'a self,
        command: CancelOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, ()> {
        Box::pin(async move { self.cancel_owner_order(command).await })
    }

    fn create_owner_order<'a>(
        &'a self,
        command: CreateOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, CreateOwnerOrderOutcome> {
        Box::pin(async move { self.create_owner_order(command).await })
    }
}

impl OwnerOrderPaymentStore for SqliteCommerceOwnerOrderPaymentStore {
    fn pay_owner_order<'a>(
        &'a self,
        command: PayOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, PayOwnerOrderOutcome> {
        Box::pin(async move { self.pay_owner_order(command).await })
    }

    fn cancel_owner_order_payments<'a>(
        &'a self,
        command: CancelOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, ()> {
        Box::pin(async move { self.cancel_owner_order_payments(command).await })
    }
}

impl OwnerOrderPaymentStore for PostgresCommerceOwnerOrderPaymentStore {
    fn pay_owner_order<'a>(
        &'a self,
        command: PayOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, PayOwnerOrderOutcome> {
        Box::pin(async move { self.pay_owner_order(command).await })
    }

    fn cancel_owner_order_payments<'a>(
        &'a self,
        command: CancelOwnerOrderCommand,
    ) -> CommerceOrderFuture<'a, ()> {
        Box::pin(async move { self.cancel_owner_order_payments(command).await })
    }
}

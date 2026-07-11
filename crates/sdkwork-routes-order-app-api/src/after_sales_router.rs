use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_order_repository_sqlx::{PostgresCommerceOrderStore, SqliteCommerceOrderStore};
use sdkwork_order_service::{
    AfterSalesEventListQuery, AfterSalesEventPage, AfterSalesEventView,
    AfterSalesRequestDetailQuery, AfterSalesRequestListQuery, AfterSalesRequestPage,
    AfterSalesRequestView, AfterSalesReturnShipmentListQuery, AfterSalesReturnShipmentPage,
    AfterSalesReturnShipmentView, CreateAfterSalesRequestCommand, CreateAfterSalesRequestItemInput,
    CreateAfterSalesReturnShipmentCommand, UpdateAfterSalesRequestCommand,
};
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::api_response::{
    map_service_error, not_found, offset_list_page_params_from_query, success_created_item,
    success_item, success_items, unauthorized, validation,
};
use crate::command_headers::{validate_app_write_payload, write_payload_with_route_param};
use crate::subject::app_runtime_subject_from_contexts;

pub type CommerceAfterSalesFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceAfterSalesStore: Send + Sync {
    fn create_after_sales_request<'a>(
        &'a self,
        command: CreateAfterSalesRequestCommand,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesRequestView>;

    fn retrieve_after_sales_request<'a>(
        &'a self,
        query: AfterSalesRequestDetailQuery,
    ) -> CommerceAfterSalesFuture<'a, Option<AfterSalesRequestView>>;

    fn list_after_sales_requests<'a>(
        &'a self,
        query: AfterSalesRequestListQuery,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesRequestPage>;

    fn list_after_sales_events<'a>(
        &'a self,
        query: AfterSalesEventListQuery,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesEventPage>;

    fn list_after_sales_return_shipments<'a>(
        &'a self,
        query: AfterSalesReturnShipmentListQuery,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesReturnShipmentPage>;

    fn create_after_sales_return_shipment<'a>(
        &'a self,
        command: CreateAfterSalesReturnShipmentCommand,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesReturnShipmentView>;

    fn update_after_sales_request<'a>(
        &'a self,
        command: UpdateAfterSalesRequestCommand,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesRequestView>;
}

#[derive(Clone)]
struct AppAfterSalesState {
    store: Arc<dyn CommerceAfterSalesStore>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateAfterSalesRequestBody {
    order_id: String,
    reason_code: String,
    #[serde(rename = "afterSalesType", alias = "after_sales_type")]
    after_sales_type: Option<String>,
    description: Option<String>,
    requested_amount: Option<String>,
    currency_code: Option<String>,
    items: Option<Vec<CreateAfterSalesRequestItemBody>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateAfterSalesRequestItemBody {
    order_item_id: String,
    quantity: i64,
    requested_amount: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateReturnShipmentBody {
    tracking_no: Option<String>,
    carrier_code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateAfterSalesRequestBody {
    status: Option<String>,
    reason_code: Option<String>,
    description: Option<String>,
    requested_amount: Option<String>,
    approved_amount: Option<String>,
    currency_code: Option<String>,
    reviewer_note: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AfterSalesRequestResponse {
    after_sales_request_id: String,
    after_sales_no: String,
    order_id: String,
    after_sales_type: String,
    reason_code: String,
    requested_amount: String,
    currency_code: String,
    status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AfterSalesReturnShipmentResponse {
    return_shipment_id: String,
    after_sales_request_id: String,
    return_shipment_no: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tracking_no: Option<String>,
    status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AfterSalesEventResponse {
    event_id: String,
    after_sales_request_id: String,
    event_no: String,
    event_type: String,
    to_status: String,
}

impl CommerceAfterSalesStore for SqliteCommerceOrderStore {
    fn create_after_sales_request<'a>(
        &'a self,
        command: CreateAfterSalesRequestCommand,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesRequestView> {
        Box::pin(async move { self.create_after_sales_request(command).await })
    }

    fn retrieve_after_sales_request<'a>(
        &'a self,
        query: AfterSalesRequestDetailQuery,
    ) -> CommerceAfterSalesFuture<'a, Option<AfterSalesRequestView>> {
        Box::pin(async move { self.retrieve_after_sales_request(query).await })
    }

    fn list_after_sales_requests<'a>(
        &'a self,
        query: AfterSalesRequestListQuery,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesRequestPage> {
        Box::pin(async move { self.list_after_sales_requests(query).await })
    }

    fn list_after_sales_events<'a>(
        &'a self,
        query: AfterSalesEventListQuery,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesEventPage> {
        Box::pin(async move { self.list_after_sales_events(query).await })
    }

    fn list_after_sales_return_shipments<'a>(
        &'a self,
        query: AfterSalesReturnShipmentListQuery,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesReturnShipmentPage> {
        Box::pin(async move { self.list_after_sales_return_shipments(query).await })
    }

    fn create_after_sales_return_shipment<'a>(
        &'a self,
        command: CreateAfterSalesReturnShipmentCommand,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesReturnShipmentView> {
        Box::pin(async move { self.create_after_sales_return_shipment(command).await })
    }

    fn update_after_sales_request<'a>(
        &'a self,
        command: UpdateAfterSalesRequestCommand,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesRequestView> {
        Box::pin(async move { self.update_after_sales_request(command).await })
    }
}

impl CommerceAfterSalesStore for PostgresCommerceOrderStore {
    fn create_after_sales_request<'a>(
        &'a self,
        command: CreateAfterSalesRequestCommand,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesRequestView> {
        Box::pin(async move { self.create_after_sales_request(command).await })
    }

    fn retrieve_after_sales_request<'a>(
        &'a self,
        query: AfterSalesRequestDetailQuery,
    ) -> CommerceAfterSalesFuture<'a, Option<AfterSalesRequestView>> {
        Box::pin(async move { self.retrieve_after_sales_request(query).await })
    }

    fn list_after_sales_requests<'a>(
        &'a self,
        query: AfterSalesRequestListQuery,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesRequestPage> {
        Box::pin(async move { self.list_after_sales_requests(query).await })
    }

    fn list_after_sales_events<'a>(
        &'a self,
        query: AfterSalesEventListQuery,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesEventPage> {
        Box::pin(async move { self.list_after_sales_events(query).await })
    }

    fn list_after_sales_return_shipments<'a>(
        &'a self,
        query: AfterSalesReturnShipmentListQuery,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesReturnShipmentPage> {
        Box::pin(async move { self.list_after_sales_return_shipments(query).await })
    }

    fn create_after_sales_return_shipment<'a>(
        &'a self,
        command: CreateAfterSalesReturnShipmentCommand,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesReturnShipmentView> {
        Box::pin(async move { self.create_after_sales_return_shipment(command).await })
    }

    fn update_after_sales_request<'a>(
        &'a self,
        command: UpdateAfterSalesRequestCommand,
    ) -> CommerceAfterSalesFuture<'a, AfterSalesRequestView> {
        Box::pin(async move { self.update_after_sales_request(command).await })
    }
}

pub fn app_after_sales_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_app_after_sales_router(Arc::new(SqliteCommerceOrderStore::new(pool)))
}

pub fn app_after_sales_router_with_postgres_pool(pool: PgPool) -> Router {
    build_app_after_sales_router(Arc::new(PostgresCommerceOrderStore::new(pool)))
}

pub fn build_app_after_sales_router(store: Arc<dyn CommerceAfterSalesStore>) -> Router {
    Router::new()
        .route(
            "/app/v3/api/after_sales/requests",
            get(list_after_sales_requests).post(create_after_sales_request),
        )
        .route(
            "/app/v3/api/after_sales/requests/{afterSalesRequestId}",
            get(retrieve_after_sales_request).patch(update_after_sales_request),
        )
        .route(
            "/app/v3/api/after_sales/requests/{afterSalesRequestId}/events",
            get(list_after_sales_events),
        )
        .route(
            "/app/v3/api/after_sales/requests/{afterSalesRequestId}/return_shipments",
            get(list_after_sales_return_shipments).post(create_after_sales_return_shipment),
        )
        .with_state(AppAfterSalesState { store })
}

async fn create_after_sales_request(
    State(state): State<AppAfterSalesState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    headers: HeaderMap,
    Json(body): Json<CreateAfterSalesRequestBody>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_contexts(runtime_context, ctx) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let write_headers = match validate_app_write_payload(
        ctx,
        &headers,
        "afterSales.requests.create",
        &body,
        |idempotency_key| format!("after-sales-{}-{}", subject.user_id, idempotency_key),
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let after_sales_type = body.after_sales_type.unwrap_or_else(|| "refund".to_owned());
    let items = match map_after_sales_request_items(body.items.as_deref()) {
        Ok(items) => items,
        Err(error) => return validation(ctx, error.message()),
    };
    let command = match CreateAfterSalesRequestCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &body.order_id,
        &body.reason_code,
        &after_sales_type,
        body.description.as_deref(),
        body.requested_amount.as_deref(),
        body.currency_code.as_deref(),
        items,
        &write_headers.request_no,
        &write_headers.idempotency_key,
    ) {
        Ok(command) => command,
        Err(error) => return validation(ctx, error.message()),
    };

    match state.store.create_after_sales_request(command).await {
        Ok(request) => success_created_item(ctx, map_after_sales_request(request)),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn retrieve_after_sales_request(
    State(state): State<AppAfterSalesState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(after_sales_request_id): Path<String>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_contexts(runtime_context, ctx) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let query = match AfterSalesRequestDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &after_sales_request_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation(ctx, error.message()),
    };

    match state.store.retrieve_after_sales_request(query).await {
        Ok(Some(request)) => success_item(ctx, map_after_sales_request(request)),
        Ok(None) => not_found(ctx, "after sales request was not found"),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn update_after_sales_request(
    State(state): State<AppAfterSalesState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    headers: HeaderMap,
    Path(after_sales_request_id): Path<String>,
    Json(body): Json<UpdateAfterSalesRequestBody>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_contexts(runtime_context, ctx) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    if body.approved_amount.is_some() {
        return validation(ctx, "approved_amount is not writable by owner");
    }
    if body.reviewer_note.is_some() {
        return validation(ctx, "reviewer_note is not writable by owner");
    }
    let payload =
        write_payload_with_route_param("afterSalesRequestId", &after_sales_request_id, &body);
    let write_headers = match validate_app_write_payload(
        ctx,
        &headers,
        "afterSales.requests.update",
        &payload,
        |idempotency_key| format!("after-sales-update-{}-{}", subject.user_id, idempotency_key),
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let command = match UpdateAfterSalesRequestCommand::new_for_owner(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &after_sales_request_id,
        body.status.as_deref(),
        body.reason_code.as_deref(),
        body.description.as_deref(),
        body.requested_amount.as_deref(),
        body.currency_code.as_deref(),
        &write_headers.request_no,
        &write_headers.idempotency_key,
    ) {
        Ok(command) => command,
        Err(error) => return validation(ctx, error.message()),
    };

    match state.store.update_after_sales_request(command).await {
        Ok(request) => success_item(ctx, map_after_sales_request(request)),
        Err(error) => map_service_error(ctx, error),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AfterSalesRequestListQueryParams {
    page: Option<i64>,
    page_size: Option<i64>,
    after_sales_request_id: Option<String>,
    order_id: Option<String>,
    after_sales_type: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AfterSalesEventListQueryParams {
    page: Option<i64>,
    page_size: Option<i64>,
}

async fn list_after_sales_requests(
    State(state): State<AppAfterSalesState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Query(params): Query<AfterSalesRequestListQueryParams>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_contexts(runtime_context, ctx) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let query = match AfterSalesRequestListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        params.order_id.as_deref(),
        params.after_sales_type.as_deref(),
        params.status.as_deref(),
        params.after_sales_request_id.as_deref(),
        params.page,
        params.page_size,
    ) {
        Ok(query) => query,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.store.list_after_sales_requests(query.clone()).await {
        Ok(page) => {
            let mapped = page
                .items
                .into_iter()
                .map(map_after_sales_request)
                .collect::<Vec<_>>();
            let page_params = offset_list_page_params_from_query(query.page, query.page_size);
            success_items(ctx, mapped, page.total, page_params)
        }
        Err(error) => map_service_error(ctx, error),
    }
}

async fn list_after_sales_events(
    State(state): State<AppAfterSalesState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(after_sales_request_id): Path<String>,
    Query(params): Query<AfterSalesEventListQueryParams>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_contexts(runtime_context, ctx) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let query = match AfterSalesEventListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &after_sales_request_id,
        params.page,
        params.page_size,
    ) {
        Ok(query) => query,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.store.list_after_sales_events(query.clone()).await {
        Ok(page) => {
            let mapped = page
                .items
                .into_iter()
                .map(map_after_sales_event)
                .collect::<Vec<_>>();
            let page_params = offset_list_page_params_from_query(query.page, query.page_size);
            success_items(ctx, mapped, page.total, page_params)
        }
        Err(error) => map_service_error(ctx, error),
    }
}

#[derive(Debug, Deserialize)]
struct AfterSalesReturnShipmentListQueryParams {
    page: Option<i64>,
    page_size: Option<i64>,
    status: Option<String>,
}

async fn list_after_sales_return_shipments(
    State(state): State<AppAfterSalesState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(after_sales_request_id): Path<String>,
    Query(params): Query<AfterSalesReturnShipmentListQueryParams>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_contexts(runtime_context, ctx) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let query = match AfterSalesReturnShipmentListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &after_sales_request_id,
        params.status.as_deref(),
        params.page,
        params.page_size,
    ) {
        Ok(query) => query,
        Err(error) => return map_service_error(ctx, error),
    };

    match state
        .store
        .list_after_sales_return_shipments(query.clone())
        .await
    {
        Ok(page) => {
            let mapped = page
                .items
                .into_iter()
                .map(map_return_shipment)
                .collect::<Vec<_>>();
            let page_params = offset_list_page_params_from_query(query.page, query.page_size);
            success_items(ctx, mapped, page.total, page_params)
        }
        Err(error) => map_service_error(ctx, error),
    }
}

async fn create_after_sales_return_shipment(
    State(state): State<AppAfterSalesState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    headers: HeaderMap,
    Path(after_sales_request_id): Path<String>,
    body: Json<CreateReturnShipmentBody>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_contexts(runtime_context, ctx) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let payload =
        write_payload_with_route_param("afterSalesRequestId", &after_sales_request_id, &*body);
    let write_headers = match validate_app_write_payload(
        ctx,
        &headers,
        "afterSales.returnShipments.create",
        &payload,
        |idempotency_key| format!("after-sales-return-{}-{}", subject.user_id, idempotency_key),
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let command = match CreateAfterSalesReturnShipmentCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &after_sales_request_id,
        body.tracking_no.as_deref(),
        body.carrier_code.as_deref(),
        &write_headers.request_no,
        &write_headers.idempotency_key,
    ) {
        Ok(command) => command,
        Err(error) => return validation(ctx, error.message()),
    };

    match state
        .store
        .create_after_sales_return_shipment(command)
        .await
    {
        Ok(shipment) => success_created_item(ctx, map_return_shipment(shipment)),
        Err(error) => map_service_error(ctx, error),
    }
}

fn map_after_sales_request_items(
    items: Option<&[CreateAfterSalesRequestItemBody]>,
) -> Result<Vec<CreateAfterSalesRequestItemInput>, CommerceServiceError> {
    let Some(items) = items else {
        return Ok(Vec::new());
    };
    items
        .iter()
        .map(|item| {
            CreateAfterSalesRequestItemInput::new(
                &item.order_item_id,
                item.quantity,
                item.requested_amount.as_deref(),
            )
        })
        .collect()
}

fn map_after_sales_request(value: AfterSalesRequestView) -> AfterSalesRequestResponse {
    AfterSalesRequestResponse {
        after_sales_request_id: value.after_sales_request_id,
        after_sales_no: value.after_sales_no,
        order_id: value.order_id,
        after_sales_type: value.after_sales_type,
        reason_code: value.reason_code,
        requested_amount: value.requested_amount.as_str().to_owned(),
        currency_code: value.currency_code,
        status: value.status,
    }
}

fn map_return_shipment(value: AfterSalesReturnShipmentView) -> AfterSalesReturnShipmentResponse {
    AfterSalesReturnShipmentResponse {
        return_shipment_id: value.return_shipment_id,
        after_sales_request_id: value.after_sales_request_id,
        return_shipment_no: value.return_shipment_no,
        tracking_no: value.tracking_no,
        status: value.status,
    }
}

fn map_after_sales_event(value: AfterSalesEventView) -> AfterSalesEventResponse {
    AfterSalesEventResponse {
        event_id: value.event_id,
        after_sales_request_id: value.after_sales_request_id,
        event_no: value.event_no,
        event_type: value.event_type,
        to_status: value.to_status,
    }
}

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use sdkwork_commerce_contract_service::CommerceServiceError;
use sdkwork_commerce_order_service::{
    AfterSalesEventListQuery, AfterSalesEventView, AfterSalesRequestDetailQuery,
    AfterSalesRequestView, AfterSalesReturnShipmentView, CreateAfterSalesRequestCommand,
    CreateAfterSalesReturnShipmentCommand, UpdateAfterSalesRequestCommand,
};
use sdkwork_commerce_order_repository_sqlx::{
    PostgresCommerceOrderStore, SqliteCommerceOrderStore,
};
use sdkwork_iam_context_service::IamAppContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::command_headers::{validate_app_write_payload, write_payload_with_route_param};
use crate::subject::app_runtime_subject_from_extension;

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

    fn list_after_sales_events<'a>(
        &'a self,
        query: AfterSalesEventListQuery,
    ) -> CommerceAfterSalesFuture<'a, Vec<AfterSalesEventView>>;

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
struct AppAfterSalesApiResult<T: Serialize> {
    code: String,
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
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

    fn list_after_sales_events<'a>(
        &'a self,
        query: AfterSalesEventListQuery,
    ) -> CommerceAfterSalesFuture<'a, Vec<AfterSalesEventView>> {
        Box::pin(async move { self.list_after_sales_events(query).await })
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

    fn list_after_sales_events<'a>(
        &'a self,
        query: AfterSalesEventListQuery,
    ) -> CommerceAfterSalesFuture<'a, Vec<AfterSalesEventView>> {
        Box::pin(async move { self.list_after_sales_events(query).await })
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

impl<T: Serialize> AppAfterSalesApiResult<T> {
    fn success(data: T) -> Self {
        Self {
            code: "0".to_owned(),
            msg: "success".to_owned(),
            data: Some(data),
        }
    }

    fn error(code: &str, msg: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            msg: msg.into(),
            data: None,
        }
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
                post(create_after_sales_request),
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
                post(create_after_sales_return_shipment),
            )
            .with_state(AppAfterSalesState { store })
}

async fn create_after_sales_request(
    State(state): State<AppAfterSalesState>,
    runtime_context: Option<Extension<IamAppContext>>,
    headers: HeaderMap,
    Json(body): Json<CreateAfterSalesRequestBody>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let write_headers = match validate_app_write_payload(
        &headers,
        "afterSales.requests.create",
        &body,
        |idempotency_key| format!("after-sales-{}-{}", subject.user_id, idempotency_key),
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let after_sales_type = body.after_sales_type.unwrap_or_else(|| "refund".to_owned());
    let command = match CreateAfterSalesRequestCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &body.order_id,
        &body.reason_code,
        &after_sales_type,
        body.description.as_deref(),
        &write_headers.request_no,
        &write_headers.idempotency_key,
    ) {
        Ok(command) => command,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.create_after_sales_request(command).await {
        Ok(request) => Json(AppAfterSalesApiResult::success(map_after_sales_request(
            request,
        )))
        .into_response(),
        Err(error) => after_sales_system_response("after sales request create failed", error),
    }
}

async fn retrieve_after_sales_request(
    State(state): State<AppAfterSalesState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Path(after_sales_request_id): Path<String>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match AfterSalesRequestDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &after_sales_request_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.retrieve_after_sales_request(query).await {
        Ok(Some(request)) => Json(AppAfterSalesApiResult::success(map_after_sales_request(
            request,
        )))
        .into_response(),
        Ok(None) => not_found_response("after sales request was not found"),
        Err(error) => {
            after_sales_system_response("after sales request read model is unavailable", error)
        }
    }
}

async fn update_after_sales_request(
    State(state): State<AppAfterSalesState>,
    runtime_context: Option<Extension<IamAppContext>>,
    headers: HeaderMap,
    Path(after_sales_request_id): Path<String>,
    Json(body): Json<UpdateAfterSalesRequestBody>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let payload =
        write_payload_with_route_param("afterSalesRequestId", &after_sales_request_id, &body);
    let write_headers = match validate_app_write_payload(
        &headers,
        "afterSales.requests.update",
        &payload,
        |idempotency_key| format!("after-sales-update-{}-{}", subject.user_id, idempotency_key),
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let command = match UpdateAfterSalesRequestCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &after_sales_request_id,
        body.status.as_deref(),
        body.reason_code.as_deref(),
        body.description.as_deref(),
        body.requested_amount.as_deref(),
        body.approved_amount.as_deref(),
        body.currency_code.as_deref(),
        body.reviewer_note.as_deref(),
        &write_headers.request_no,
        &write_headers.idempotency_key,
    ) {
        Ok(command) => command,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.update_after_sales_request(command).await {
        Ok(request) => Json(AppAfterSalesApiResult::success(map_after_sales_request(
            request,
        )))
        .into_response(),
        Err(error) => after_sales_system_response("after sales request update failed", error),
    }
}

async fn list_after_sales_events(
    State(state): State<AppAfterSalesState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Path(after_sales_request_id): Path<String>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match AfterSalesEventListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &after_sales_request_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.list_after_sales_events(query).await {
        Ok(events) => Json(AppAfterSalesApiResult::success(
            events
                .into_iter()
                .map(map_after_sales_event)
                .collect::<Vec<_>>(),
        ))
        .into_response(),
        Err(error) => {
            after_sales_system_response("after sales events read model is unavailable", error)
        }
    }
}

async fn create_after_sales_return_shipment(
    State(state): State<AppAfterSalesState>,
    runtime_context: Option<Extension<IamAppContext>>,
    headers: HeaderMap,
    Path(after_sales_request_id): Path<String>,
    body: Json<CreateReturnShipmentBody>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let payload =
        write_payload_with_route_param("afterSalesRequestId", &after_sales_request_id, &*body);
    let write_headers = match validate_app_write_payload(
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
        Err(error) => return validation_response(error.message()),
    };

    match state
        .store
        .create_after_sales_return_shipment(command)
        .await
    {
        Ok(shipment) => Json(AppAfterSalesApiResult::success(map_return_shipment(
            shipment,
        )))
        .into_response(),
        Err(error) => {
            after_sales_system_response("after sales return shipment create failed", error)
        }
    }
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

fn unauthorized_response(message: impl Into<String>) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(AppAfterSalesApiResult::<()>::error("4010", message)),
    )
        .into_response()
}

fn validation_response(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(AppAfterSalesApiResult::<()>::error("4001", message)),
    )
        .into_response()
}

fn not_found_response(message: impl Into<String>) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(AppAfterSalesApiResult::<()>::error("4040", message)),
    )
        .into_response()
}

fn after_sales_system_response(context: &str, error: CommerceServiceError) -> Response {
    match error.code() {
        "validation" => validation_response(error.message()),
        "not_found" => not_found_response(error.message()),
        "conflict" => (
            StatusCode::CONFLICT,
            Json(AppAfterSalesApiResult::<()>::error("4090", error.message())),
        )
            .into_response(),
        "unauthenticated" => unauthorized_response(error.message()),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppAfterSalesApiResult::<()>::error(
                "5000",
                format!("{context}: {}", error.message()),
            )),
        )
            .into_response(),
    }
}

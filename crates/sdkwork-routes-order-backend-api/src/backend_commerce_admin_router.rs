//! Backend admin routes for after-sales management and shipment operations.

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_order_repository_sqlx::{PostgresCommerceOrderStore, SqliteCommerceOrderStore};
use sdkwork_order_service::{
    AfterSalesManagementDetailQuery, AfterSalesManagementListQuery, AfterSalesRequestView,
    CreateShipmentPackageCommand, ReviewAfterSalesRequestCommand, ShipmentManagementDetailQuery,
    ShipmentManagementListQuery, ShipmentPackageManagementListQuery, ShipmentPackageView,
    ShipmentView, UpdateShipmentPackageCommand,
};
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::api_response::{
    map_service_error, parse_offset_list_params_validated, success_item, success_items,
};
use crate::backend_command_headers::{
    validate_backend_write_payload, write_payload_with_route_param,
};
use crate::backend_acl::require_backend_operator;

mod permissions {
    pub const AFTER_SALES_READ: &str = "commerce.afterSales.read";
    pub const AFTER_SALES_REVIEW: &str = "commerce.afterSales.review";
    pub const ORDERS_READ: &str = "commerce.orders.read";
    pub const ORDERS_MANAGE: &str = "commerce.orders.manage";
}

#[derive(Clone)]
enum BackendCommerceAdminStore {
    Postgres(Arc<PostgresCommerceOrderStore>),
    Sqlite(Arc<SqliteCommerceOrderStore>),
}

#[derive(Clone)]
struct BackendCommerceAdminState {
    store: BackendCommerceAdminStore,
}

#[derive(Debug, Deserialize)]
struct AfterSalesListParams {
    status: Option<String>,
    #[serde(rename = "afterSalesType", alias = "after_sales_type")]
    after_sales_type: Option<String>,
    #[serde(rename = "orderId", alias = "order_id")]
    order_id: Option<String>,
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ShipmentListParams {
    status: Option<String>,
    #[serde(rename = "orderId", alias = "order_id")]
    order_id: Option<String>,
    #[serde(rename = "fulfillmentId", alias = "fulfillment_id")]
    fulfillment_id: Option<String>,
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct PackageListParams {
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReviewAfterSalesRequestBody {
    review_action: String,
    status: Option<String>,
    refund_status: Option<String>,
    return_status: Option<String>,
    exchange_status: Option<String>,
    approved_amount: Option<String>,
    reason_code: Option<String>,
    #[serde(alias = "reasonDetail")]
    reason_detail: Option<String>,
    review_comment: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateShipmentPackageBody {
    package_type: String,
    package_no: Option<String>,
    tracking_no: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateShipmentPackageBody {
    package_type: Option<String>,
    tracking_no: Option<String>,
    status: Option<String>,
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
struct ShipmentResponse {
    shipment_id: String,
    shipment_no: String,
    fulfillment_id: String,
    carrier_code: String,
    tracking_no: Option<String>,
    status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShipmentPackageResponse {
    package_id: String,
    shipment_id: String,
    package_no: String,
    package_type: String,
    tracking_no: Option<String>,
    status: String,
}

impl BackendCommerceAdminStore {
    async fn list_management_after_sales(
        &self,
        query: AfterSalesManagementListQuery,
    ) -> Result<sdkwork_order_service::AfterSalesRequestPage, CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.list_management_after_sales_requests(query).await,
            Self::Sqlite(store) => store.list_management_after_sales_requests(query).await,
        }
    }

    async fn retrieve_management_after_sales(
        &self,
        query: AfterSalesManagementDetailQuery,
    ) -> Result<Option<AfterSalesRequestView>, CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.retrieve_management_after_sales_request(query).await,
            Self::Sqlite(store) => store.retrieve_management_after_sales_request(query).await,
        }
    }

    async fn review_after_sales(
        &self,
        command: ReviewAfterSalesRequestCommand,
    ) -> Result<AfterSalesRequestView, CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.review_after_sales_request(command).await,
            Self::Sqlite(store) => store.review_after_sales_request(command).await,
        }
    }

    async fn list_management_shipments(
        &self,
        query: ShipmentManagementListQuery,
    ) -> Result<sdkwork_order_service::ShipmentManagementListPage, CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.list_management_shipments(query).await,
            Self::Sqlite(store) => store.list_management_shipments(query).await,
        }
    }

    async fn retrieve_management_shipment(
        &self,
        query: ShipmentManagementDetailQuery,
    ) -> Result<Option<ShipmentView>, CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.retrieve_management_shipment(query).await,
            Self::Sqlite(store) => store.retrieve_management_shipment(query).await,
        }
    }

    async fn list_management_packages(
        &self,
        query: ShipmentPackageManagementListQuery,
    ) -> Result<sdkwork_order_service::ShipmentPackagePage, CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.list_management_shipment_packages(query).await,
            Self::Sqlite(store) => store.list_management_shipment_packages(query).await,
        }
    }

    async fn create_management_package(
        &self,
        command: CreateShipmentPackageCommand,
    ) -> Result<ShipmentPackageView, CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.create_management_shipment_package(command).await,
            Self::Sqlite(store) => store.create_management_shipment_package(command).await,
        }
    }

    async fn update_management_package(
        &self,
        command: UpdateShipmentPackageCommand,
    ) -> Result<ShipmentPackageView, CommerceServiceError> {
        match self {
            Self::Postgres(store) => store.update_management_shipment_package(command).await,
            Self::Sqlite(store) => store.update_management_shipment_package(command).await,
        }
    }
}

pub fn backend_commerce_admin_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_backend_commerce_admin_router(BackendCommerceAdminStore::Sqlite(Arc::new(
        SqliteCommerceOrderStore::new(pool),
    )))
}

pub fn backend_commerce_admin_router_with_postgres_pool(pool: PgPool) -> Router {
    build_backend_commerce_admin_router(BackendCommerceAdminStore::Postgres(Arc::new(
        PostgresCommerceOrderStore::new(pool),
    )))
}

fn build_backend_commerce_admin_router(store: BackendCommerceAdminStore) -> Router {
    Router::new()
        .route(
            "/backend/v3/api/after_sales/requests",
            get(list_management_after_sales),
        )
        .route(
            "/backend/v3/api/after_sales/requests/{afterSalesRequestId}",
            get(retrieve_management_after_sales),
        )
        .route(
            "/backend/v3/api/after_sales/requests/{afterSalesRequestId}/reviews",
            post(review_after_sales_request),
        )
        .route("/backend/v3/api/shipments", get(list_management_shipments))
        .route(
            "/backend/v3/api/shipments/{shipmentId}",
            get(retrieve_management_shipment),
        )
        .route(
            "/backend/v3/api/shipments/{shipmentId}/packages",
            get(list_management_shipment_packages).post(create_management_shipment_package),
        )
        .route(
            "/backend/v3/api/shipments/{shipmentId}/packages/{packageId}",
            patch(update_management_shipment_package),
        )
        .with_state(BackendCommerceAdminState { store })
}

async fn list_management_after_sales(
    State(state): State<BackendCommerceAdminState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Query(params): Query<AfterSalesListParams>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match runtime_context {
        Some(Extension(context)) => {
            require_backend_operator(ctx, context, permissions::AFTER_SALES_READ)
        }
        None => return crate::api_response::unauthorized(ctx, "authentication is required"),
    };
    let Ok(subject) = subject else {
        return subject.err().expect("permission error response");
    };
    let page_params = match parse_offset_list_params_validated(ctx, params.page, params.page_size)
    {
        Ok(value) => value,
        Err(response) => return response,
    };
    let query = match AfterSalesManagementListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        params.order_id.as_deref(),
        params.after_sales_type.as_deref(),
        params.status.as_deref(),
        None,
        Some(page_params.page),
        Some(page_params.page_size),
    ) {
        Ok(query) => query,
        Err(error) => return crate::api_response::validation(ctx, error.message()),
    };

    match state.store.list_management_after_sales(query).await {
        Ok(page) => success_items(
            ctx,
            page
                .items
                .into_iter()
                .map(map_after_sales_request)
                .collect(),
            page.total,
            page_params,
        ),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn retrieve_management_after_sales(
    State(state): State<BackendCommerceAdminState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(after_sales_request_id): Path<String>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match runtime_context {
        Some(Extension(context)) => {
            require_backend_operator(ctx, context, permissions::AFTER_SALES_READ)
        }
        None => return crate::api_response::unauthorized(ctx, "authentication is required"),
    };
    let Ok(subject) = subject else {
        return subject.err().expect("permission error response");
    };
    let query = match AfterSalesManagementDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &after_sales_request_id,
    ) {
        Ok(query) => query,
        Err(error) => return crate::api_response::validation(ctx, error.message()),
    };

    match state.store.retrieve_management_after_sales(query).await {
        Ok(Some(request)) => success_item(ctx, map_after_sales_request(request)),
        Ok(None) => crate::api_response::not_found(ctx, "after sales request was not found"),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn review_after_sales_request(
    State(state): State<BackendCommerceAdminState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    headers: HeaderMap,
    Path(after_sales_request_id): Path<String>,
    Json(body): Json<ReviewAfterSalesRequestBody>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match runtime_context {
        Some(Extension(context)) => {
            require_backend_operator(ctx, context, permissions::AFTER_SALES_REVIEW)
        }
        None => return crate::api_response::unauthorized(ctx, "authentication is required"),
    };
    let Ok(subject) = subject else {
        return subject.err().expect("permission error response");
    };
    let payload =
        write_payload_with_route_param("afterSalesRequestId", &after_sales_request_id, &body);
    let write_headers = match validate_backend_write_payload(
        ctx,
        &headers,
        "afterSales.reviews.create",
        &payload,
        |idempotency_key| format!("review-{after_sales_request_id}-{idempotency_key}"),
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let command = match ReviewAfterSalesRequestCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &after_sales_request_id,
        &body.review_action,
        &write_headers.request_no,
        &write_headers.idempotency_key,
    ) {
        Ok(command) => command
            .with_status(body.status)
            .with_refund_status(body.refund_status)
            .with_return_status(body.return_status)
            .with_exchange_status(body.exchange_status)
            .with_approved_amount(body.approved_amount)
            .with_reason_code(body.reason_code)
            .with_review_comment(
                body.review_comment.or(body.reason_detail),
            ),
        Err(error) => return crate::api_response::validation(ctx, error.message()),
    };

    match state.store.review_after_sales(command).await {
        Ok(request) => success_item(ctx, map_after_sales_request(request)),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn list_management_shipments(
    State(state): State<BackendCommerceAdminState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Query(params): Query<ShipmentListParams>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match runtime_context {
        Some(Extension(context)) => {
            require_backend_operator(ctx, context, permissions::ORDERS_READ)
        }
        None => return crate::api_response::unauthorized(ctx, "authentication is required"),
    };
    let Ok(subject) = subject else {
        return subject.err().expect("permission error response");
    };
    let page_params = match parse_offset_list_params_validated(ctx, params.page, params.page_size)
    {
        Ok(value) => value,
        Err(response) => return response,
    };
    let query = match ShipmentManagementListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        params.order_id.as_deref(),
        params.fulfillment_id.as_deref(),
        params.status.as_deref(),
        Some(page_params.page),
        Some(page_params.page_size),
    ) {
        Ok(query) => query,
        Err(error) => return crate::api_response::validation(ctx, error.message()),
    };

    match state.store.list_management_shipments(query).await {
        Ok(page) => success_items(
            ctx,
            page.items.into_iter().map(map_shipment).collect(),
            page.total,
            page_params,
        ),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn retrieve_management_shipment(
    State(state): State<BackendCommerceAdminState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(shipment_id): Path<String>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match runtime_context {
        Some(Extension(context)) => {
            require_backend_operator(ctx, context, permissions::ORDERS_READ)
        }
        None => return crate::api_response::unauthorized(ctx, "authentication is required"),
    };
    let Ok(subject) = subject else {
        return subject.err().expect("permission error response");
    };
    let query = match ShipmentManagementDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &shipment_id,
    ) {
        Ok(query) => query,
        Err(error) => return crate::api_response::validation(ctx, error.message()),
    };

    match state.store.retrieve_management_shipment(query).await {
        Ok(Some(shipment)) => success_item(ctx, map_shipment(shipment)),
        Ok(None) => crate::api_response::not_found(ctx, "shipment was not found"),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn list_management_shipment_packages(
    State(state): State<BackendCommerceAdminState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(shipment_id): Path<String>,
    Query(params): Query<PackageListParams>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match runtime_context {
        Some(Extension(context)) => {
            require_backend_operator(ctx, context, permissions::ORDERS_READ)
        }
        None => return crate::api_response::unauthorized(ctx, "authentication is required"),
    };
    let Ok(subject) = subject else {
        return subject.err().expect("permission error response");
    };
    let page_params = match parse_offset_list_params_validated(ctx, params.page, params.page_size)
    {
        Ok(value) => value,
        Err(response) => return response,
    };
    let query = match ShipmentPackageManagementListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &shipment_id,
        Some(page_params.page),
        Some(page_params.page_size),
    ) {
        Ok(query) => query,
        Err(error) => return crate::api_response::validation(ctx, error.message()),
    };

    match state.store.list_management_packages(query).await {
        Ok(page) => success_items(
            ctx,
            page
                .items
                .into_iter()
                .map(map_shipment_package)
                .collect(),
            page.total,
            page_params,
        ),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn create_management_shipment_package(
    State(state): State<BackendCommerceAdminState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    headers: HeaderMap,
    Path(shipment_id): Path<String>,
    Json(body): Json<CreateShipmentPackageBody>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match runtime_context {
        Some(Extension(context)) => {
            require_backend_operator(ctx, context, permissions::ORDERS_MANAGE)
        }
        None => return crate::api_response::unauthorized(ctx, "authentication is required"),
    };
    let Ok(subject) = subject else {
        return subject.err().expect("permission error response");
    };
    let payload = write_payload_with_route_param("shipmentId", &shipment_id, &body);
    let write_headers = match validate_backend_write_payload(
        ctx,
        &headers,
        "shipments.packages.create",
        &payload,
        |idempotency_key| format!("pkg-{shipment_id}-{idempotency_key}"),
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let mut command = match CreateShipmentPackageCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &shipment_id,
        &body.package_type,
        &write_headers.request_no,
        &write_headers.idempotency_key,
    ) {
        Ok(command) => command,
        Err(error) => return crate::api_response::validation(ctx, error.message()),
    };
    command.package_no = body.package_no;
    command.tracking_no = body.tracking_no;
    command.status = body.status;

    match state.store.create_management_package(command).await {
        Ok(package) => success_item(ctx, map_shipment_package(package)),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn update_management_shipment_package(
    State(state): State<BackendCommerceAdminState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    headers: HeaderMap,
    Path((shipment_id, package_id)): Path<(String, String)>,
    Json(body): Json<UpdateShipmentPackageBody>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match runtime_context {
        Some(Extension(context)) => {
            require_backend_operator(ctx, context, permissions::ORDERS_MANAGE)
        }
        None => return crate::api_response::unauthorized(ctx, "authentication is required"),
    };
    let Ok(subject) = subject else {
        return subject.err().expect("permission error response");
    };
    let payload =
        write_payload_with_route_param("packageId", &package_id, &body);
    let mut payload = payload;
    if let serde_json::Value::Object(ref mut fields) = payload {
        fields.insert(
            "shipmentId".to_string(),
            serde_json::Value::String(shipment_id.clone()),
        );
    }
    let write_headers = match validate_backend_write_payload(
        ctx,
        &headers,
        "shipments.packages.update",
        &payload,
        |idempotency_key| format!("pkg-update-{package_id}-{idempotency_key}"),
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let mut command = match UpdateShipmentPackageCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &shipment_id,
        &package_id,
        &write_headers.request_no,
        &write_headers.idempotency_key,
    ) {
        Ok(command) => command,
        Err(error) => return crate::api_response::validation(ctx, error.message()),
    };
    command.package_type = body.package_type;
    command.tracking_no = body.tracking_no;
    command.status = body.status;

    match state.store.update_management_package(command).await {
        Ok(package) => success_item(ctx, map_shipment_package(package)),
        Err(error) => map_service_error(ctx, error),
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

fn map_shipment(value: ShipmentView) -> ShipmentResponse {
    ShipmentResponse {
        shipment_id: value.shipment_id,
        shipment_no: value.shipment_no,
        fulfillment_id: value.fulfillment_id,
        carrier_code: value.carrier_code,
        tracking_no: value.tracking_no,
        status: value.status,
    }
}

fn map_shipment_package(value: ShipmentPackageView) -> ShipmentPackageResponse {
    ShipmentPackageResponse {
        package_id: value.package_id,
        shipment_id: value.shipment_id,
        package_no: value.package_no,
        package_type: value.package_type,
        tracking_no: value.tracking_no,
        status: value.status,
    }
}

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_order_service::{
    ShipmentDetailQuery, ShipmentPackageListQuery, ShipmentPackagePage, ShipmentPackageView,
    ShipmentTrackingEventListQuery, ShipmentTrackingEventPage, ShipmentTrackingEventView,
    ShipmentView,
};
use sdkwork_order_repository_sqlx::{
    PostgresCommerceOrderStore, SqliteCommerceOrderStore,
};
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_web_core::WebRequestContext;
use serde::Serialize;
use sqlx::{PgPool, SqlitePool};

use crate::api_response::{
    map_service_error, not_found, offset_list_page_params_from_query, success_item, success_items,
    unauthorized, validation,
};
use crate::subject::app_runtime_subject_from_extension;

pub type CommerceShipmentFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceShipmentStore: Send + Sync {
    fn retrieve_owner_shipment<'a>(
        &'a self,
        query: ShipmentDetailQuery,
    ) -> CommerceShipmentFuture<'a, Option<ShipmentView>>;

    fn list_owner_shipment_packages<'a>(
        &'a self,
        query: ShipmentPackageListQuery,
    ) -> CommerceShipmentFuture<'a, ShipmentPackagePage>;

    fn list_owner_shipment_tracking_events<'a>(
        &'a self,
        query: ShipmentTrackingEventListQuery,
    ) -> CommerceShipmentFuture<'a, ShipmentTrackingEventPage>;
}

#[derive(Clone)]
struct AppShipmentState {
    store: Arc<dyn CommerceShipmentStore>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ShipmentListQueryParams {
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShipmentResponse {
    shipment_id: String,
    shipment_no: String,
    fulfillment_id: String,
    carrier_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    tracking_no: Option<String>,
    status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShipmentTrackingEventResponse {
    event_id: String,
    shipment_id: String,
    tracking_event_no: String,
    event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_status: Option<String>,
    event_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    location_text: Option<String>,
}

impl CommerceShipmentStore for SqliteCommerceOrderStore {
    fn retrieve_owner_shipment<'a>(
        &'a self,
        query: ShipmentDetailQuery,
    ) -> CommerceShipmentFuture<'a, Option<ShipmentView>> {
        Box::pin(async move { self.retrieve_owner_shipment(query).await })
    }

    fn list_owner_shipment_packages<'a>(
        &'a self,
        query: ShipmentPackageListQuery,
    ) -> CommerceShipmentFuture<'a, ShipmentPackagePage> {
        Box::pin(async move { self.list_owner_shipment_packages(query).await })
    }

    fn list_owner_shipment_tracking_events<'a>(
        &'a self,
        query: ShipmentTrackingEventListQuery,
    ) -> CommerceShipmentFuture<'a, ShipmentTrackingEventPage> {
        Box::pin(async move { self.list_owner_shipment_tracking_events(query).await })
    }
}

impl CommerceShipmentStore for PostgresCommerceOrderStore {
    fn retrieve_owner_shipment<'a>(
        &'a self,
        query: ShipmentDetailQuery,
    ) -> CommerceShipmentFuture<'a, Option<ShipmentView>> {
        Box::pin(async move { self.retrieve_owner_shipment(query).await })
    }

    fn list_owner_shipment_packages<'a>(
        &'a self,
        query: ShipmentPackageListQuery,
    ) -> CommerceShipmentFuture<'a, ShipmentPackagePage> {
        Box::pin(async move { self.list_owner_shipment_packages(query).await })
    }

    fn list_owner_shipment_tracking_events<'a>(
        &'a self,
        query: ShipmentTrackingEventListQuery,
    ) -> CommerceShipmentFuture<'a, ShipmentTrackingEventPage> {
        Box::pin(async move { self.list_owner_shipment_tracking_events(query).await })
    }
}

pub fn app_shipment_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_app_shipment_router(Arc::new(SqliteCommerceOrderStore::new(pool)))
}

pub fn app_shipment_router_with_postgres_pool(pool: PgPool) -> Router {
    build_app_shipment_router(Arc::new(PostgresCommerceOrderStore::new(pool)))
}

pub fn build_app_shipment_router(store: Arc<dyn CommerceShipmentStore>) -> Router {
    Router::new()
        .route("/app/v3/api/shipments/{shipmentId}", get(retrieve_shipment))
        .route(
            "/app/v3/api/shipments/{shipmentId}/packages",
            get(list_shipment_packages),
        )
        .route(
            "/app/v3/api/shipments/{shipmentId}/tracking_events",
            get(list_shipment_tracking_events),
        )
        .with_state(AppShipmentState { store })
}

async fn retrieve_shipment(
    State(state): State<AppShipmentState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(shipment_id): Path<String>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let query = match ShipmentDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &shipment_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation(ctx, error.message()),
    };

    match state.store.retrieve_owner_shipment(query).await {
        Ok(Some(shipment)) => success_item(ctx, map_shipment(shipment)),
        Ok(None) => not_found(ctx, "shipment was not found"),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn list_shipment_packages(
    State(state): State<AppShipmentState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(shipment_id): Path<String>,
    Query(list_params): Query<ShipmentListQueryParams>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let query = match ShipmentPackageListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &shipment_id,
        list_params.page,
        list_params.page_size,
    ) {
        Ok(query) => query,
        Err(error) => return validation(ctx, error.message()),
    };

    match state.store.list_owner_shipment_packages(query).await {
        Ok(page) => {
            let page_params = offset_list_page_params_from_query(page.page, page.page_size);
            let mapped = page.items.into_iter().map(map_shipment_package).collect::<Vec<_>>();
            success_items(ctx, mapped, page.total, page_params)
        }
        Err(error) => map_service_error(ctx, error),
    }
}

async fn list_shipment_tracking_events(
    State(state): State<AppShipmentState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(shipment_id): Path<String>,
    Query(list_params): Query<ShipmentListQueryParams>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let query = match ShipmentTrackingEventListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &shipment_id,
        list_params.page,
        list_params.page_size,
    ) {
        Ok(query) => query,
        Err(error) => return validation(ctx, error.message()),
    };

    match state.store.list_owner_shipment_tracking_events(query).await {
        Ok(page) => {
            let page_params = offset_list_page_params_from_query(page.page, page.page_size);
            let mapped = page.items.into_iter().map(map_tracking_event).collect::<Vec<_>>();
            success_items(ctx, mapped, page.total, page_params)
        }
        Err(error) => map_service_error(ctx, error),
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

fn map_tracking_event(value: ShipmentTrackingEventView) -> ShipmentTrackingEventResponse {
    ShipmentTrackingEventResponse {
        event_id: value.event_id,
        shipment_id: value.shipment_id,
        tracking_event_no: value.tracking_event_no,
        event_type: value.event_type,
        event_status: value.event_status,
        event_time: value.event_time,
        location_text: value.location_text,
    }
}

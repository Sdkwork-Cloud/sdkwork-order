use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use sdkwork_commerce_contract_service::CommerceServiceError;
use sdkwork_commerce_order_service::{
    ShipmentDetailQuery, ShipmentPackageListQuery, ShipmentPackageView,
    ShipmentTrackingEventListQuery, ShipmentTrackingEventView, ShipmentView,
};
use sdkwork_commerce_order_repository_sqlx::{
    PostgresCommerceOrderStore, SqliteCommerceOrderStore,
};
use sdkwork_iam_context_service::IamAppContext;
use serde::Serialize;
use sqlx::{PgPool, SqlitePool};

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
    ) -> CommerceShipmentFuture<'a, Vec<ShipmentPackageView>>;

    fn list_owner_shipment_tracking_events<'a>(
        &'a self,
        query: ShipmentTrackingEventListQuery,
    ) -> CommerceShipmentFuture<'a, Vec<ShipmentTrackingEventView>>;
}

#[derive(Clone)]
struct AppShipmentState {
    store: Arc<dyn CommerceShipmentStore>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppShipmentApiResult<T: Serialize> {
    code: String,
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
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
    ) -> CommerceShipmentFuture<'a, Vec<ShipmentPackageView>> {
        Box::pin(async move { self.list_owner_shipment_packages(query).await })
    }

    fn list_owner_shipment_tracking_events<'a>(
        &'a self,
        query: ShipmentTrackingEventListQuery,
    ) -> CommerceShipmentFuture<'a, Vec<ShipmentTrackingEventView>> {
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
    ) -> CommerceShipmentFuture<'a, Vec<ShipmentPackageView>> {
        Box::pin(async move { self.list_owner_shipment_packages(query).await })
    }

    fn list_owner_shipment_tracking_events<'a>(
        &'a self,
        query: ShipmentTrackingEventListQuery,
    ) -> CommerceShipmentFuture<'a, Vec<ShipmentTrackingEventView>> {
        Box::pin(async move { self.list_owner_shipment_tracking_events(query).await })
    }
}

impl<T: Serialize> AppShipmentApiResult<T> {
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
    Path(shipment_id): Path<String>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match ShipmentDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &shipment_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.retrieve_owner_shipment(query).await {
        Ok(Some(shipment)) => {
            Json(AppShipmentApiResult::success(map_shipment(shipment))).into_response()
        }
        Ok(None) => not_found_response("shipment was not found"),
        Err(error) => shipment_system_response("shipment read model is unavailable", error),
    }
}

async fn list_shipment_packages(
    State(state): State<AppShipmentState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Path(shipment_id): Path<String>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match ShipmentPackageListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &shipment_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.list_owner_shipment_packages(query).await {
        Ok(packages) => Json(AppShipmentApiResult::success(
            packages
                .into_iter()
                .map(map_shipment_package)
                .collect::<Vec<_>>(),
        ))
        .into_response(),
        Err(error) => {
            shipment_system_response("shipment packages read model is unavailable", error)
        }
    }
}

async fn list_shipment_tracking_events(
    State(state): State<AppShipmentState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Path(shipment_id): Path<String>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match ShipmentTrackingEventListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &shipment_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.list_owner_shipment_tracking_events(query).await {
        Ok(events) => Json(AppShipmentApiResult::success(
            events
                .into_iter()
                .map(map_tracking_event)
                .collect::<Vec<_>>(),
        ))
        .into_response(),
        Err(error) => {
            shipment_system_response("shipment tracking events read model is unavailable", error)
        }
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

fn unauthorized_response(message: impl Into<String>) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(AppShipmentApiResult::<()>::error("4010", message)),
    )
        .into_response()
}

fn validation_response(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(AppShipmentApiResult::<()>::error("4001", message)),
    )
        .into_response()
}

fn not_found_response(message: impl Into<String>) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(AppShipmentApiResult::<()>::error("4040", message)),
    )
        .into_response()
}

fn shipment_system_response(context: &str, error: CommerceServiceError) -> Response {
    match error.code() {
        "validation" => validation_response(error.message()),
        "not_found" => not_found_response(error.message()),
        "conflict" => (
            StatusCode::CONFLICT,
            Json(AppShipmentApiResult::<()>::error("4090", error.message())),
        )
            .into_response(),
        "unauthenticated" => unauthorized_response(error.message()),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppShipmentApiResult::<()>::error(
                "5000",
                format!("{context}: {}", error.message()),
            )),
        )
            .into_response(),
    }
}

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use sdkwork_commerce_contract_service::CommerceServiceError;
use sdkwork_commerce_order_service::{
    FulfillmentDetailQuery, FulfillmentListQuery, FulfillmentView,
};
use sdkwork_commerce_order_repository_sqlx::{
    PostgresCommerceOrderStore, SqliteCommerceOrderStore,
};
use sdkwork_iam_context_service::IamAppContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::subject::app_runtime_subject_from_extension;

pub type CommerceFulfillmentFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceFulfillmentStore: Send + Sync {
    fn list_owner_fulfillments<'a>(
        &'a self,
        query: FulfillmentListQuery,
    ) -> CommerceFulfillmentFuture<'a, Vec<FulfillmentView>>;

    fn retrieve_owner_fulfillment<'a>(
        &'a self,
        query: FulfillmentDetailQuery,
    ) -> CommerceFulfillmentFuture<'a, Option<FulfillmentView>>;
}

#[derive(Clone)]
struct AppFulfillmentState {
    store: Arc<dyn CommerceFulfillmentStore>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FulfillmentListParams {
    order_id: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppFulfillmentApiResult<T: Serialize> {
    code: String,
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FulfillmentResponse {
    fulfillment_id: String,
    fulfillment_no: String,
    order_id: String,
    fulfillment_type: String,
    status: String,
}

impl CommerceFulfillmentStore for SqliteCommerceOrderStore {
    fn list_owner_fulfillments<'a>(
        &'a self,
        query: FulfillmentListQuery,
    ) -> CommerceFulfillmentFuture<'a, Vec<FulfillmentView>> {
        Box::pin(async move { self.list_owner_fulfillments(query).await })
    }

    fn retrieve_owner_fulfillment<'a>(
        &'a self,
        query: FulfillmentDetailQuery,
    ) -> CommerceFulfillmentFuture<'a, Option<FulfillmentView>> {
        Box::pin(async move { self.retrieve_owner_fulfillment(query).await })
    }
}

impl CommerceFulfillmentStore for PostgresCommerceOrderStore {
    fn list_owner_fulfillments<'a>(
        &'a self,
        query: FulfillmentListQuery,
    ) -> CommerceFulfillmentFuture<'a, Vec<FulfillmentView>> {
        Box::pin(async move { self.list_owner_fulfillments(query).await })
    }

    fn retrieve_owner_fulfillment<'a>(
        &'a self,
        query: FulfillmentDetailQuery,
    ) -> CommerceFulfillmentFuture<'a, Option<FulfillmentView>> {
        Box::pin(async move { self.retrieve_owner_fulfillment(query).await })
    }
}

impl<T: Serialize> AppFulfillmentApiResult<T> {
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

pub fn app_fulfillment_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_app_fulfillment_router(Arc::new(SqliteCommerceOrderStore::new(pool)))
}

pub fn app_fulfillment_router_with_postgres_pool(pool: PgPool) -> Router {
    build_app_fulfillment_router(Arc::new(PostgresCommerceOrderStore::new(pool)))
}

pub fn build_app_fulfillment_router(store: Arc<dyn CommerceFulfillmentStore>) -> Router {
    Router::new()
            .route("/app/v3/api/fulfillments", get(list_fulfillments))
            .route(
                "/app/v3/api/fulfillments/{fulfillmentId}",
                get(retrieve_fulfillment),
            )
            .with_state(AppFulfillmentState { store })
}

async fn list_fulfillments(
    State(state): State<AppFulfillmentState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(params): Query<FulfillmentListParams>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match FulfillmentListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        params.order_id.as_deref(),
        params.status.as_deref(),
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.list_owner_fulfillments(query).await {
        Ok(fulfillments) => Json(AppFulfillmentApiResult::success(
            fulfillments
                .into_iter()
                .map(map_fulfillment)
                .collect::<Vec<_>>(),
        ))
        .into_response(),
        Err(error) => fulfillment_system_response("fulfillment read model is unavailable", error),
    }
}

async fn retrieve_fulfillment(
    State(state): State<AppFulfillmentState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Path(fulfillment_id): Path<String>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match FulfillmentDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &fulfillment_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.retrieve_owner_fulfillment(query).await {
        Ok(Some(fulfillment)) => Json(AppFulfillmentApiResult::success(map_fulfillment(
            fulfillment,
        )))
        .into_response(),
        Ok(None) => not_found_response("fulfillment was not found"),
        Err(error) => fulfillment_system_response("fulfillment read model is unavailable", error),
    }
}

fn map_fulfillment(value: FulfillmentView) -> FulfillmentResponse {
    FulfillmentResponse {
        fulfillment_id: value.fulfillment_id,
        fulfillment_no: value.fulfillment_no,
        order_id: value.order_id,
        fulfillment_type: value.fulfillment_type,
        status: value.status,
    }
}

fn unauthorized_response(message: impl Into<String>) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(AppFulfillmentApiResult::<()>::error("4010", message)),
    )
        .into_response()
}

fn validation_response(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(AppFulfillmentApiResult::<()>::error("4001", message)),
    )
        .into_response()
}

fn not_found_response(message: impl Into<String>) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(AppFulfillmentApiResult::<()>::error("4040", message)),
    )
        .into_response()
}

fn fulfillment_system_response(context: &str, error: CommerceServiceError) -> Response {
    match error.code() {
        "validation" => validation_response(error.message()),
        "not_found" => not_found_response(error.message()),
        "conflict" => (
            StatusCode::CONFLICT,
            Json(AppFulfillmentApiResult::<()>::error(
                "4090",
                error.message(),
            )),
        )
            .into_response(),
        "unauthenticated" => unauthorized_response(error.message()),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppFulfillmentApiResult::<()>::error(
                "5000",
                format!("{context}: {}", error.message()),
            )),
        )
            .into_response(),
    }
}

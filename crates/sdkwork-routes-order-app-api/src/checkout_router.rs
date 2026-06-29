use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_order_service::{
    checkout_owner_order_request_hash, checkout_quote_request_hash, checkout_session_request_hash,
    CheckoutLineInput, CheckoutQuoteView, CheckoutSessionDetailQuery, CheckoutSessionView,
    CreateCheckoutQuoteCommand, CreateCheckoutSessionCommand, CreateOwnerOrderCommand,
    CreateOwnerOrderOutcome,
};
use sdkwork_order_repository_sqlx::{
    PostgresCommerceOrderStore, SqliteCommerceOrderStore,
};
use sdkwork_iam_context_service::IamAppContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::command_headers::{ensure_request_hash_matches, required_app_write_command_headers};
use crate::subject::{app_runtime_subject_from_extension, AppRuntimeSubject};

pub type CommerceCheckoutFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceCheckoutStore: Send + Sync {
    fn create_checkout_session<'a>(
        &'a self,
        command: CreateCheckoutSessionCommand,
    ) -> CommerceCheckoutFuture<'a, CheckoutSessionView>;

    fn retrieve_checkout_session<'a>(
        &'a self,
        query: CheckoutSessionDetailQuery,
    ) -> CommerceCheckoutFuture<'a, Option<CheckoutSessionView>>;

    fn create_checkout_quote<'a>(
        &'a self,
        command: CreateCheckoutQuoteCommand,
    ) -> CommerceCheckoutFuture<'a, CheckoutQuoteView>;

    fn create_owner_order<'a>(
        &'a self,
        command: CreateOwnerOrderCommand,
    ) -> CommerceCheckoutFuture<'a, CreateOwnerOrderOutcome>;
}

#[derive(Clone)]
struct AppCheckoutState {
    store: Arc<dyn CommerceCheckoutStore>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckoutLineRequest {
    sku_id: String,
    quantity: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateCheckoutSessionRequest {
    items: Option<Vec<CheckoutLineRequest>>,
    lines: Option<Vec<CheckoutLineRequest>>,
    currency_code: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppCheckoutApiResult<T: Serialize> {
    code: String,
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckoutSessionResponse {
    checkout_session_id: String,
    status: String,
    currency_code: String,
    original_amount: String,
    discount_amount: String,
    payable_amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    quote_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckoutQuoteResponse {
    checkout_session_id: String,
    quote_id: String,
    currency_code: String,
    original_amount: String,
    discount_amount: String,
    payable_amount: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckoutOrderResponse {
    order_id: String,
    order_no: String,
    order_sn: String,
    status: String,
    total_amount: String,
}

impl CommerceCheckoutStore for SqliteCommerceOrderStore {
    fn create_checkout_session<'a>(
        &'a self,
        command: CreateCheckoutSessionCommand,
    ) -> CommerceCheckoutFuture<'a, CheckoutSessionView> {
        Box::pin(async move { self.create_checkout_session(command).await })
    }

    fn retrieve_checkout_session<'a>(
        &'a self,
        query: CheckoutSessionDetailQuery,
    ) -> CommerceCheckoutFuture<'a, Option<CheckoutSessionView>> {
        Box::pin(async move { self.retrieve_checkout_session(query).await })
    }

    fn create_checkout_quote<'a>(
        &'a self,
        command: CreateCheckoutQuoteCommand,
    ) -> CommerceCheckoutFuture<'a, CheckoutQuoteView> {
        Box::pin(async move { self.create_checkout_quote(command).await })
    }

    fn create_owner_order<'a>(
        &'a self,
        command: CreateOwnerOrderCommand,
    ) -> CommerceCheckoutFuture<'a, CreateOwnerOrderOutcome> {
        Box::pin(async move { self.create_owner_order(command).await })
    }
}

impl CommerceCheckoutStore for PostgresCommerceOrderStore {
    fn create_checkout_session<'a>(
        &'a self,
        command: CreateCheckoutSessionCommand,
    ) -> CommerceCheckoutFuture<'a, CheckoutSessionView> {
        Box::pin(async move { self.create_checkout_session(command).await })
    }

    fn retrieve_checkout_session<'a>(
        &'a self,
        query: CheckoutSessionDetailQuery,
    ) -> CommerceCheckoutFuture<'a, Option<CheckoutSessionView>> {
        Box::pin(async move { self.retrieve_checkout_session(query).await })
    }

    fn create_checkout_quote<'a>(
        &'a self,
        command: CreateCheckoutQuoteCommand,
    ) -> CommerceCheckoutFuture<'a, CheckoutQuoteView> {
        Box::pin(async move { self.create_checkout_quote(command).await })
    }

    fn create_owner_order<'a>(
        &'a self,
        command: CreateOwnerOrderCommand,
    ) -> CommerceCheckoutFuture<'a, CreateOwnerOrderOutcome> {
        Box::pin(async move { self.create_owner_order(command).await })
    }
}

impl<T: Serialize> AppCheckoutApiResult<T> {
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

pub fn app_checkout_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_app_checkout_router(Arc::new(SqliteCommerceOrderStore::new(pool)))
}

pub fn app_checkout_router_with_postgres_pool(pool: PgPool) -> Router {
    build_app_checkout_router(Arc::new(PostgresCommerceOrderStore::new(pool)))
}

pub fn build_app_checkout_router(store: Arc<dyn CommerceCheckoutStore>) -> Router {
    Router::new()
            .route(
                "/app/v3/api/checkout/sessions",
                post(create_checkout_session),
            )
            .route(
                "/app/v3/api/checkout/sessions/{checkoutSessionId}",
                get(retrieve_checkout_session),
            )
            .route(
                "/app/v3/api/checkout/sessions/{checkoutSessionId}/quotes",
                post(create_checkout_quote),
            )
            .route(
                "/app/v3/api/checkout/sessions/{checkoutSessionId}/orders",
                post(create_checkout_order),
            )
            .with_state(AppCheckoutState { store })
}

async fn create_checkout_session(
    State(state): State<AppCheckoutState>,
    runtime_context: Option<Extension<IamAppContext>>,
    headers: HeaderMap,
    body: Json<CreateCheckoutSessionRequest>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let write_headers = match required_app_write_command_headers(&headers, |idempotency_key| {
        fallback_request_no(&subject, "checkout-session", idempotency_key)
    }) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let request = body.0;
    let lines = match parse_checkout_lines(&request) {
        Ok(lines) => lines,
        Err(message) => return validation_response(message),
    };
    let currency_code = request
        .currency_code
        .as_deref()
        .unwrap_or("CNY")
        .trim()
        .to_ascii_uppercase();
    let command = match CreateCheckoutSessionCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &currency_code,
        lines,
        &write_headers.request_no,
        &write_headers.idempotency_key,
    ) {
        Ok(command) => command,
        Err(error) => return validation_response(error.message()),
    };
    if let Err(response) = ensure_request_hash_matches(
        &checkout_session_request_hash(&command),
        &write_headers.request_hash,
    ) {
        return response;
    }

    match state.store.create_checkout_session(command).await {
        Ok(session) => {
            Json(AppCheckoutApiResult::success(map_checkout_session(session))).into_response()
        }
        Err(error) => checkout_system_response("checkout session create failed", error),
    }
}

async fn retrieve_checkout_session(
    State(state): State<AppCheckoutState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Path(checkout_session_id): Path<String>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match CheckoutSessionDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &checkout_session_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.retrieve_checkout_session(query).await {
        Ok(Some(session)) => {
            Json(AppCheckoutApiResult::success(map_checkout_session(session))).into_response()
        }
        Ok(None) => not_found_response("checkout session was not found"),
        Err(error) => checkout_system_response("checkout session read model is unavailable", error),
    }
}

async fn create_checkout_quote(
    State(state): State<AppCheckoutState>,
    runtime_context: Option<Extension<IamAppContext>>,
    headers: HeaderMap,
    Path(checkout_session_id): Path<String>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let write_headers = match required_app_write_command_headers(&headers, |idempotency_key| {
        fallback_request_no(&subject, &checkout_session_id, idempotency_key)
    }) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let command = match CreateCheckoutQuoteCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &checkout_session_id,
        &write_headers.request_no,
        &write_headers.idempotency_key,
    ) {
        Ok(command) => command,
        Err(error) => return validation_response(error.message()),
    };
    if let Err(response) = ensure_request_hash_matches(
        &checkout_quote_request_hash(&command),
        &write_headers.request_hash,
    ) {
        return response;
    }

    match state.store.create_checkout_quote(command).await {
        Ok(quote) => Json(AppCheckoutApiResult::success(map_checkout_quote(quote))).into_response(),
        Err(error) => checkout_system_response("checkout quote create failed", error),
    }
}

async fn create_checkout_order(
    State(state): State<AppCheckoutState>,
    runtime_context: Option<Extension<IamAppContext>>,
    headers: HeaderMap,
    Path(checkout_session_id): Path<String>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let write_headers = match required_app_write_command_headers(&headers, |idempotency_key| {
        fallback_request_no(&subject, &checkout_session_id, idempotency_key)
    }) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let command = match CreateOwnerOrderCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &checkout_session_id,
        &write_headers.request_no,
        &write_headers.idempotency_key,
    ) {
        Ok(command) => command,
        Err(error) => return validation_response(error.message()),
    };
    if let Err(response) = ensure_request_hash_matches(
        &checkout_owner_order_request_hash(&command),
        &write_headers.request_hash,
    ) {
        return response;
    }

    match state.store.create_owner_order(command).await {
        Ok(outcome) => {
            Json(AppCheckoutApiResult::success(map_checkout_order(outcome))).into_response()
        }
        Err(error) => checkout_system_response("checkout order create failed", error),
    }
}

fn parse_checkout_lines(
    request: &CreateCheckoutSessionRequest,
) -> Result<Vec<CheckoutLineInput>, String> {
    let source = request.items.as_ref().or(request.lines.as_ref());
    let Some(source) = source else {
        return Err("checkout session requires at least one line".to_owned());
    };
    if source.is_empty() {
        return Err("checkout session requires at least one line".to_owned());
    }
    source
        .iter()
        .map(|line| {
            CheckoutLineInput::new(&line.sku_id, line.quantity.unwrap_or(1).max(1))
                .map_err(|error| error.message().to_owned())
        })
        .collect()
}

fn map_checkout_session(value: CheckoutSessionView) -> CheckoutSessionResponse {
    CheckoutSessionResponse {
        checkout_session_id: value.checkout_session_id,
        status: value.status,
        currency_code: value.currency_code,
        original_amount: value.original_amount.as_str().to_owned(),
        discount_amount: value.discount_amount.as_str().to_owned(),
        payable_amount: value.payable_amount.as_str().to_owned(),
        quote_id: value.quote_id,
    }
}

fn map_checkout_quote(value: CheckoutQuoteView) -> CheckoutQuoteResponse {
    CheckoutQuoteResponse {
        checkout_session_id: value.checkout_session_id,
        quote_id: value.quote_id,
        currency_code: value.currency_code,
        original_amount: value.original_amount.as_str().to_owned(),
        discount_amount: value.discount_amount.as_str().to_owned(),
        payable_amount: value.payable_amount.as_str().to_owned(),
    }
}

fn map_checkout_order(value: CreateOwnerOrderOutcome) -> CheckoutOrderResponse {
    CheckoutOrderResponse {
        order_id: value.order_id.clone(),
        order_no: value.order_sn.clone(),
        order_sn: value.order_sn,
        status: value.status,
        total_amount: value.total_amount.as_str().to_owned(),
    }
}

fn fallback_request_no(subject: &AppRuntimeSubject, suffix: &str, idempotency_key: &str) -> String {
    format!(
        "checkout-{}-{}-{}",
        subject.user_id, suffix, idempotency_key
    )
}

fn unauthorized_response(message: impl Into<String>) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(AppCheckoutApiResult::<()>::error("4010", message)),
    )
        .into_response()
}

fn validation_response(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(AppCheckoutApiResult::<()>::error("4001", message)),
    )
        .into_response()
}

fn not_found_response(message: impl Into<String>) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(AppCheckoutApiResult::<()>::error("4040", message)),
    )
        .into_response()
}

fn checkout_system_response(context: &str, error: CommerceServiceError) -> Response {
    match error.code() {
        "validation" => validation_response(error.message()),
        "not_found" => not_found_response(error.message()),
        "conflict" => (
            StatusCode::CONFLICT,
            Json(AppCheckoutApiResult::<()>::error("4090", error.message())),
        )
            .into_response(),
        "unauthenticated" => unauthorized_response(error.message()),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppCheckoutApiResult::<()>::error(
                "5000",
                format!("{context}: {}", error.message()),
            )),
        )
            .into_response(),
    }
}

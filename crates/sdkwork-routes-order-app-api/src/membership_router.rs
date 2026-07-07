use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::post;
use axum::{Json, Router};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_order_repository_sqlx::{
    PostgresCommerceMembershipOrderStore, SqliteCommerceMembershipOrderStore,
};
use sdkwork_order_service::{CreateMembershipOrderCommand, CreateMembershipOrderOutcome};
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};
use uuid::Uuid;

use crate::api_response::{map_service_error, success_item, unauthorized, validation};
use crate::command_headers::validate_app_write_payload;
use crate::subject::{app_runtime_subject_from_extension, AppRuntimeSubject};

const PAYMENT_EXPIRE_SECONDS: i64 = 1_800;
const ALLOWED_PAYMENT_METHODS: &[&str] = &["wechat_pay", "alipay", "balance"];

pub type CommerceMembershipOrderFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceMembershipOrderStore: Send + Sync {
    fn create_membership_order<'a>(
        &'a self,
        command: CreateMembershipOrderCommand,
    ) -> CommerceMembershipOrderFuture<'a, CreateMembershipOrderOutcome>;
}

#[derive(Clone)]
struct AppMembershipOrderState {
    store: Arc<dyn CommerceMembershipOrderStore>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateMembershipOrderRequest {
    package_id: Option<String>,
    payment_method: Option<String>,
    client_request_no: Option<String>,
    source: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateMembershipOrderResponse {
    order_id: String,
    order_no: String,
    out_trade_no: String,
    amount: String,
    currency_code: String,
    package_id: String,
    package_name: String,
    duration_days: i64,
    payment_method: String,
    status: String,
    cashier_url: String,
}

struct CreateMembershipCommandInput<'a> {
    subject: &'a AppRuntimeSubject,
    package_id: &'a str,
    method: &'a str,
    request_no: &'a str,
    idempotency_key: &'a str,
    client_request_no: Option<&'a str>,
    source: Option<&'a str>,
}

impl CreateMembershipOrderRequest {
    fn package_id(&self) -> Option<&str> {
        self.package_id.as_deref()
    }

    fn payment_method(&self) -> Option<&str> {
        self.payment_method.as_deref()
    }

    fn client_request_no(&self) -> Option<&str> {
        self.client_request_no.as_deref()
    }

    fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }
}

impl CommerceMembershipOrderStore for SqliteCommerceMembershipOrderStore {
    fn create_membership_order<'a>(
        &'a self,
        command: CreateMembershipOrderCommand,
    ) -> CommerceMembershipOrderFuture<'a, CreateMembershipOrderOutcome> {
        Box::pin(async move { self.create_membership_order(command).await })
    }
}

impl CommerceMembershipOrderStore for PostgresCommerceMembershipOrderStore {
    fn create_membership_order<'a>(
        &'a self,
        command: CreateMembershipOrderCommand,
    ) -> CommerceMembershipOrderFuture<'a, CreateMembershipOrderOutcome> {
        Box::pin(async move { self.create_membership_order(command).await })
    }
}

pub fn app_membership_order_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_app_membership_order_router(Arc::new(SqliteCommerceMembershipOrderStore::new(pool)))
}

pub fn app_membership_order_router_with_postgres_pool(pool: PgPool) -> Router {
    build_app_membership_order_router(Arc::new(PostgresCommerceMembershipOrderStore::new(pool)))
}

pub fn build_app_membership_order_router(store: Arc<dyn CommerceMembershipOrderStore>) -> Router {
    Router::new()
        .route(
            "/app/v3/api/memberships/orders",
            post(create_membership_order),
        )
        .with_state(AppMembershipOrderState { store })
}

async fn create_membership_order(
    State(state): State<AppMembershipOrderState>,
    runtime_context: Option<axum::extract::Extension<IamAppContext>>,
    request_context: Option<axum::extract::Extension<WebRequestContext>>,
    headers: HeaderMap,
    Json(request): Json<CreateMembershipOrderRequest>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let package_id = match validate_package_id(request.package_id()) {
        Ok(value) => value,
        Err(message) => return validation(ctx, message),
    };
    let method = match validate_payment_method(request.payment_method()) {
        Ok(value) => value,
        Err(message) => return validation(ctx, message),
    };
    let write_headers = match validate_app_write_payload(
        ctx,
        &headers,
        "memberships.orders.create",
        &request,
        |idempotency_key| {
            fallback_request_no(&subject, &package_id, &method, idempotency_key)
        },
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let command = match build_create_membership_command(CreateMembershipCommandInput {
        subject: &subject,
        package_id: &package_id,
        method: &method,
        request_no: &write_headers.request_no,
        idempotency_key: &write_headers.idempotency_key,
        client_request_no: request.client_request_no(),
        source: request.source(),
    }) {
        Ok(command) => command,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.store.create_membership_order(command).await {
        Ok(outcome) => success_item(ctx, map_membership_order_outcome(outcome)),
        Err(error) => map_service_error(ctx, error),
    }
}

fn validate_package_id(value: Option<&str>) -> Result<String, String> {
    let package_id = value.unwrap_or_default().trim();
    if package_id.is_empty() {
        return Err("package id must be provided".to_string());
    }
    Ok(package_id.to_string())
}

fn validate_payment_method(value: Option<&str>) -> Result<String, String> {
    let method = value.unwrap_or_default().trim().to_ascii_lowercase();
    if method.is_empty() {
        return Err("payment method must be provided".to_string());
    }
    if !ALLOWED_PAYMENT_METHODS.iter().any(|allowed| *allowed == method) {
        return Err(format!(
            "payment method must be one of: {}",
            ALLOWED_PAYMENT_METHODS.join(", ")
        ));
    }
    Ok(method)
}

fn build_create_membership_command(
    input: CreateMembershipCommandInput<'_>,
) -> Result<CreateMembershipOrderCommand, CommerceServiceError> {
    let now = current_unix_timestamp();
    let requested_at = format_unix_timestamp(now);
    let expire_at = format_unix_timestamp(now + PAYMENT_EXPIRE_SECONDS);
    let order_id = Uuid::new_v4().to_string();
    let order_item_id = Uuid::new_v4().to_string();
    let token = stable_hex_token(&format!(
        "{}|{}|{}|{}|{}|{}",
        input.subject.tenant_id,
        input.subject.organization_id.as_deref().unwrap_or(""),
        input.subject.user_id,
        input.package_id,
        input.request_no,
        input.idempotency_key,
    ));
    let order_no = format!("MB{token}");
    let out_trade_no = format!("MEMBERSHIP{token}");

    CreateMembershipOrderCommand::new(
        &input.subject.tenant_id,
        input.subject.organization_id.as_deref(),
        &input.subject.user_id,
        input.package_id,
        input.method,
        &order_id,
        &order_item_id,
        &order_no,
        &out_trade_no,
        &requested_at,
        &expire_at,
        input.idempotency_key,
        input.client_request_no,
        input.source,
    )
}

fn map_membership_order_outcome(value: CreateMembershipOrderOutcome) -> CreateMembershipOrderResponse {
    CreateMembershipOrderResponse {
        order_id: value.order_id,
        order_no: value.order_no,
        out_trade_no: value.out_trade_no,
        amount: value.amount.as_str().to_string(),
        currency_code: value.currency_code,
        package_id: value.package_id,
        package_name: value.package_name,
        duration_days: value.duration_days,
        payment_method: value.payment_method,
        status: value.status,
        cashier_url: value.cashier_url,
    }
}

fn fallback_request_no(
    subject: &AppRuntimeSubject,
    package_id: &str,
    method: &str,
    idempotency_key: &str,
) -> String {
    stable_header_token(&format!(
        "membership-order-{}-{}-{}-{}",
        subject.user_id, package_id, method, idempotency_key
    ))
}

fn stable_header_token(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn stable_hex_token(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn format_unix_timestamp(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

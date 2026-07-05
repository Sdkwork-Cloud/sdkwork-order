use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Extension, Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use sdkwork_contract_service::{CommerceMoney, CommerceServiceError};
use std::collections::BTreeMap;

use sdkwork_order_service::{
    CancelOwnerOrderCommand, CheckoutStatusQuery, CheckoutStatusSnapshot,
    CreatePointsRechargeOrderCommand, CreatePointsRechargeOrderOutcome, OrderOwnerListQuery,
    PayOwnerOrderCommand, PayOwnerOrderOutcome, RechargeGrantPreview, RechargePackageItem,
    RechargePackageListPage, RechargePackageListQuery, RechargeSettingsQuery, RechargeSettingsSnapshot,
};
use sdkwork_order_repository_sqlx::{
    PostgresCommerceOrderStore, PostgresCommerceRechargeStore, SqliteCommerceOrderStore,
    SqliteCommerceRechargeStore,
};
use sdkwork_payment_repository_sqlx::{
    PostgresCommerceOwnerOrderPaymentStore, SqliteCommerceOwnerOrderPaymentStore,
};
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_web_core::WebRequestContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::api_response::{
    map_service_error, not_found, offset_list_page_params_from_query, success_command, success_item,
    success_items, unauthorized, validation,
};
use crate::order_router::{CommerceOrderStore, OwnerOrderPaymentStore};
use crate::command_headers::validate_app_write_payload;
use crate::subject::{app_runtime_subject_from_extension, AppRuntimeSubject};

const MAX_CHECKOUT_ORDER_NO_LEN: usize = 128;
const MAX_RECHARGE_CENTS: i64 = 1_000_000;
const PAYMENT_EXPIRE_SECONDS: i64 = 1_800;

/// 允许的支付方式白名单。新增支付方式时只需扩展此处。
const ALLOWED_PAYMENT_METHODS: &[&str] = &["wechat_pay", "alipay", "balance"];

pub type CommerceRechargeCheckoutFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceRechargeCheckoutStore: Send + Sync {
    fn list_recharge_packages<'a>(
        &'a self,
        query: RechargePackageListQuery,
    ) -> CommerceRechargeCheckoutFuture<'a, RechargePackageListPage>;

    fn load_recharge_settings<'a>(
        &'a self,
        query: RechargeSettingsQuery,
    ) -> CommerceRechargeCheckoutFuture<'a, RechargeSettingsSnapshot>;

    fn create_points_recharge_order<'a>(
        &'a self,
        command: CreatePointsRechargeOrderCommand,
    ) -> CommerceRechargeCheckoutFuture<'a, CreatePointsRechargeOrderOutcome>;

    fn retrieve_checkout_status<'a>(
        &'a self,
        query: CheckoutStatusQuery,
    ) -> CommerceRechargeCheckoutFuture<'a, Option<CheckoutStatusSnapshot>>;
}

#[derive(Clone)]
struct AppRechargeCheckoutState {
    store: Arc<dyn CommerceRechargeCheckoutStore>,
    orders: Arc<dyn CommerceOrderStore>,
    payments: Arc<dyn OwnerOrderPaymentStore>,
}

#[derive(Debug, Deserialize)]
struct RechargeOrderListQueryParams {
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RechargePackageListQueryParams {
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RechargeOrderSummaryResponse {
    order_id: String,
    order_no: String,
    status: String,
    subject: String,
    amount: String,
    points: i64,
    created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SubmitRechargeRequest {
    amount: Option<serde_json::Value>,
    client_request_no: Option<String>,
    currency_code: Option<String>,
    package_id: Option<String>,
    payment_method: Option<String>,
    payment_password: Option<String>,
    source: Option<String>,
}

struct CreateRechargeCommandInput<'a> {
    subject: &'a AppRuntimeSubject,
    amount: CommerceMoney,
    currency_code: &'a str,
    method: &'a str,
    request_no: &'a str,
    idempotency_key: &'a str,
    package_id: Option<&'a str>,
    client_request_no: Option<&'a str>,
    source: Option<&'a str>,
}

impl SubmitRechargeRequest {
    fn amount_value(&self) -> Option<&serde_json::Value> {
        self.amount.as_ref()
    }

    fn currency_code(&self) -> Option<&str> {
        self.currency_code.as_deref()
    }

    fn package_id(&self) -> Option<&str> {
        self.package_id.as_deref()
    }

    fn client_request_no(&self) -> Option<&str> {
        self.client_request_no.as_deref()
    }

    fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    fn payment_method(&self) -> Option<&str> {
        self.payment_method.as_deref()
    }

    fn payment_password(&self) -> Option<&str> {
        self.payment_password.as_deref()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RechargePackageResponse {
    id: String,
    price_amount: String,
    currency_code: String,
    bonus_points: i64,
    grant_amount: i64,
    points: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RechargeGrantPreviewResponse {
    grant_amount: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RechargeSettingsResponse {
    base_currency_code: String,
    base_points_per_cny: String,
    currency_to_cny_rates: BTreeMap<String, String>,
    preview_examples: BTreeMap<String, BTreeMap<String, RechargeGrantPreviewResponse>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SubmitRechargeResponse {
    success: bool,
    order_no: String,
    out_trade_no: String,
    amount: String,
    currency_code: String,
    points: i64,
    provider_code: String,
    payment_method: String,
    payment_product: String,
    status: String,
    next_action: String,
    cashier_url: String,
    qr_code_payload: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_payment_payload: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckoutStatusResponse {
    order_no: String,
    out_trade_no: String,
    amount: String,
    currency_code: String,
    points: i64,
    provider_code: String,
    payment_method: String,
    payment_product: String,
    order_status: String,
    payment_status: String,
    recharge_status: String,
    status: String,
    created_at: String,
    expires_at: String,
    paid_at: String,
    next_action: String,
    cashier_url: String,
    qr_code_payload: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_payment_payload: Option<String>,
}

impl CommerceRechargeCheckoutStore for SqliteCommerceRechargeStore {
    fn list_recharge_packages<'a>(
        &'a self,
        query: RechargePackageListQuery,
    ) -> CommerceRechargeCheckoutFuture<'a, RechargePackageListPage> {
        Box::pin(async move { self.list_recharge_packages(query).await })
    }

    fn create_points_recharge_order<'a>(
        &'a self,
        command: CreatePointsRechargeOrderCommand,
    ) -> CommerceRechargeCheckoutFuture<'a, CreatePointsRechargeOrderOutcome> {
        Box::pin(async move { self.create_points_recharge_order(command).await })
    }

    fn load_recharge_settings<'a>(
        &'a self,
        query: RechargeSettingsQuery,
    ) -> CommerceRechargeCheckoutFuture<'a, RechargeSettingsSnapshot> {
        Box::pin(async move { self.load_recharge_settings(query).await })
    }

    fn retrieve_checkout_status<'a>(
        &'a self,
        query: CheckoutStatusQuery,
    ) -> CommerceRechargeCheckoutFuture<'a, Option<CheckoutStatusSnapshot>> {
        Box::pin(async move { self.load_checkout_status(query).await })
    }
}

impl CommerceRechargeCheckoutStore for PostgresCommerceRechargeStore {
    fn list_recharge_packages<'a>(
        &'a self,
        query: RechargePackageListQuery,
    ) -> CommerceRechargeCheckoutFuture<'a, RechargePackageListPage> {
        Box::pin(async move { self.list_recharge_packages(query).await })
    }

    fn create_points_recharge_order<'a>(
        &'a self,
        command: CreatePointsRechargeOrderCommand,
    ) -> CommerceRechargeCheckoutFuture<'a, CreatePointsRechargeOrderOutcome> {
        Box::pin(async move { self.create_points_recharge_order(command).await })
    }

    fn load_recharge_settings<'a>(
        &'a self,
        query: RechargeSettingsQuery,
    ) -> CommerceRechargeCheckoutFuture<'a, RechargeSettingsSnapshot> {
        Box::pin(async move { self.load_recharge_settings(query).await })
    }

    fn retrieve_checkout_status<'a>(
        &'a self,
        query: CheckoutStatusQuery,
    ) -> CommerceRechargeCheckoutFuture<'a, Option<CheckoutStatusSnapshot>> {
        Box::pin(async move { self.load_checkout_status(query).await })
    }
}

pub fn app_recharge_checkout_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    let pool_for_orders = pool.clone();
    build_app_recharge_checkout_router(
        Arc::new(SqliteCommerceRechargeStore::new(pool)),
        Arc::new(SqliteCommerceOrderStore::new(pool_for_orders.clone())),
        Arc::new(SqliteCommerceOwnerOrderPaymentStore::new(pool_for_orders)),
    )
}

pub fn app_recharge_checkout_router_with_postgres_pool(pool: PgPool) -> Router {
    let pool_for_orders = pool.clone();
    build_app_recharge_checkout_router(
        Arc::new(PostgresCommerceRechargeStore::new(pool)),
        Arc::new(PostgresCommerceOrderStore::new(pool_for_orders.clone())),
        Arc::new(PostgresCommerceOwnerOrderPaymentStore::new(pool_for_orders)),
    )
}

pub fn build_app_recharge_checkout_router(
    store: Arc<dyn CommerceRechargeCheckoutStore>,
    orders: Arc<dyn CommerceOrderStore>,
    payments: Arc<dyn OwnerOrderPaymentStore>,
) -> Router {
    Router::new()
            .route(
                "/app/v3/api/recharges/packages",
                get(fetch_recharge_packages),
            )
            .route(
                "/app/v3/api/recharges/settings",
                get(fetch_recharge_settings),
            )
            .route(
                "/app/v3/api/recharges/orders",
                get(list_recharge_orders).post(submit_recharge),
            )
            .route(
                "/app/v3/api/recharges/orders/{orderId}",
                get(fetch_checkout_status),
            )
            .route(
                "/app/v3/api/recharges/orders/{orderId}/cancel",
                post(cancel_recharge_order),
            )
            .with_state(AppRechargeCheckoutState {
                store,
                orders,
                payments,
            })
}

async fn fetch_recharge_packages(
    State(state): State<AppRechargeCheckoutState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Query(params): Query<RechargePackageListQueryParams>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let query = match RechargePackageListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        params.page,
        params.page_size,
    ) {
        Ok(query) => query,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.store.list_recharge_packages(query.clone()).await {
        Ok(page) => {
            let mapped = page
                .items
                .into_iter()
                .map(map_recharge_package)
                .collect::<Vec<_>>();
            let page_params = offset_list_page_params_from_query(query.page, query.page_size);
            success_items(ctx, mapped, page.total, page_params)
        }
        Err(error) => map_service_error(ctx, error),
    }
}

async fn fetch_recharge_settings(
    State(state): State<AppRechargeCheckoutState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let query = match RechargeSettingsQuery::new(&subject.tenant_id, subject.organization_id.as_deref())
    {
        Ok(query) => query,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.store.load_recharge_settings(query).await {
        Ok(settings) => success_item(ctx, map_recharge_settings(settings)),
        Err(error) => map_service_error(ctx, error),
    }
}

async fn list_recharge_orders(
    State(state): State<AppRechargeCheckoutState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Query(params): Query<RechargeOrderListQueryParams>,
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
        Some("points_recharge"),
    ) {
        Ok(query) => query,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.orders.list_owner_orders(query).await {
        Ok(page) => {
            let page_params = offset_list_page_params_from_query(page.page, page.page_size);
            success_items(
                ctx,
                page
                    .items
                    .into_iter()
                    .map(|item| RechargeOrderSummaryResponse {
                        order_id: item.order_id,
                        order_no: item.order_sn,
                        status: item.status,
                        subject: item.subject,
                        amount: item.total_amount.as_str().to_string(),
                        points: 0,
                        created_at: item.created_at,
                    })
                    .collect(),
                page.total,
                page_params,
            )
        }
        Err(error) => map_service_error(ctx, error),
    }
}

async fn cancel_recharge_order(
    State(state): State<AppRechargeCheckoutState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(order_id): Path<String>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let command = match CancelOwnerOrderCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &order_id,
        Some("recharge cancelled by owner"),
    ) {
        Ok(command) => command,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.orders.cancel_owner_order(command.clone()).await {
        Ok(()) => match state.payments.cancel_owner_order_payments(command).await {
            Ok(()) => success_command(
                ctx,
                Some(order_id.clone()),
                Some("cancelled".to_string()),
            ),
            Err(error) => map_service_error(ctx, error),
        },
        Err(error) => map_service_error(ctx, error),
    }
}

async fn submit_recharge(
    State(state): State<AppRechargeCheckoutState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    headers: HeaderMap,
    Json(request): Json<SubmitRechargeRequest>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let amount = match validate_recharge_amount(request.amount_value()) {
        Ok(amount) => amount,
        Err(message) => return validation(ctx, message),
    };
    let currency_code = match validate_currency_code(request.currency_code()) {
        Ok(value) => value,
        Err(message) => return validation(ctx, message),
    };
    let method = match validate_payment_method(request.payment_method()) {
        Ok(value) => value,
        Err(message) => return validation(ctx, message),
    };
    let write_headers =
        match validate_app_write_payload(ctx, &headers, "recharge.submit", &request, |idempotency_key| {
            fallback_request_no(&subject, amount.as_str(), &method, idempotency_key)
        }) {
            Ok(value) => value,
            Err(response) => return response,
        };
    let command = match build_create_recharge_command(CreateRechargeCommandInput {
        subject: &subject,
        amount,
        currency_code: &currency_code,
        method: &method,
        request_no: &write_headers.request_no,
        idempotency_key: &write_headers.idempotency_key,
        package_id: request.package_id(),
        client_request_no: request.client_request_no(),
        source: request.source(),
    }) {
        Ok(command) => command,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.store.create_points_recharge_order(command.clone()).await {
        Ok(mut outcome) => {
            let callback_payload = serde_json::json!({
                "points": outcome.points,
                "packageId": command.package_id,
                "clientRequestNo": command.client_request_no,
                "source": command.source,
                "paymentPassword": request.payment_password(),
            })
            .to_string();
            let pay_command = match PayOwnerOrderCommand::with_payment_attempt_callback_payload(
                &subject.tenant_id,
                subject.organization_id.as_deref(),
                &subject.user_id,
                &command.order_id,
                &method,
                Some(callback_payload),
                &format!("{}:pay", write_headers.request_no),
                &format!("{}:pay", write_headers.idempotency_key),
            ) {
                Ok(command) => command,
                Err(error) => return map_service_error(ctx, error),
            };
            match state.payments.pay_owner_order(pay_command).await {
                Ok(pay_outcome) => {
                    outcome = merge_recharge_pay_outcome(outcome, pay_outcome);
                    success_item(ctx, map_recharge_outcome(outcome))
                }
                Err(error) => map_service_error(ctx, error),
            }
        }
        Err(error) => map_service_error(ctx, error),
    }
}

async fn fetch_checkout_status(
    State(state): State<AppRechargeCheckoutState>,
    runtime_context: Option<Extension<IamAppContext>>,
    request_context: Option<Extension<WebRequestContext>>,
    Path(order_no): Path<String>,
) -> Response {
    let ctx = request_context.as_ref().map(|value| &value.0);
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized(ctx, message),
    };
    let order_no = match validate_checkout_order_no(order_no) {
        Ok(order_no) => order_no,
        Err(message) => return validation(ctx, message),
    };
    let query = match CheckoutStatusQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &order_no,
    ) {
        Ok(query) => query,
        Err(error) => return map_service_error(ctx, error),
    };

    match state.store.retrieve_checkout_status(query).await {
        Ok(Some(snapshot)) => success_item(ctx, map_checkout_status(snapshot)),
        Ok(None) => not_found(ctx, "checkout order was not found"),
        Err(error) => map_service_error(ctx, error),
    }
}

fn validate_recharge_amount(value: Option<&serde_json::Value>) -> Result<CommerceMoney, String> {
    let Some(value) = value else {
        return Err("recharge amount must be greater than zero".to_string());
    };
    let raw = match value {
        serde_json::Value::String(value) => value.trim().to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        _ => return Err("recharge amount must be a decimal amount".to_string()),
    };
    let cents = money_cents(&raw).map_err(|_| "recharge amount must be a decimal amount")?;
    if cents <= 0 {
        return Err("recharge amount must be greater than zero".to_string());
    }
    if cents > MAX_RECHARGE_CENTS {
        return Err("recharge amount must not exceed 10000.00".to_string());
    }
    CommerceMoney::new(&format_money_minor(cents)).map_err(str::to_string)
}

fn validate_currency_code(value: Option<&str>) -> Result<String, String> {
    let currency_code = value.unwrap_or_default().trim().to_ascii_uppercase();
    if currency_code.len() != 3
        || !currency_code
            .chars()
            .all(|character| character.is_ascii_uppercase())
    {
        return Err("currency code must be a 3-letter uppercase code".to_string());
    }
    Ok(currency_code)
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

fn validate_checkout_order_no(order_no: String) -> Result<String, String> {
    let order_no = order_no.trim().to_string();
    if order_no.is_empty() {
        return Err("checkout order number must not be empty".to_string());
    }
    if order_no.chars().count() > MAX_CHECKOUT_ORDER_NO_LEN {
        return Err(format!(
            "checkout order number length must not exceed {MAX_CHECKOUT_ORDER_NO_LEN} characters"
        ));
    }
    if !order_no.bytes().all(|byte| (0x21..=0x7e).contains(&byte)) {
        return Err("checkout order number must contain only visible ASCII characters".to_string());
    }
    Ok(order_no)
}

fn build_create_recharge_command(
    input: CreateRechargeCommandInput<'_>,
) -> Result<CreatePointsRechargeOrderCommand, CommerceServiceError> {
    let now = current_unix_timestamp();
    let requested_at = format_unix_timestamp(now);
    let expire_at = format_unix_timestamp(now + PAYMENT_EXPIRE_SECONDS);
    let seed = format!(
        "{}|{}|{}|{}|{}|{}|{}",
        input.subject.tenant_id,
        input.subject.organization_id.as_deref().unwrap_or(""),
        input.subject.user_id,
        input.amount.as_str(),
        input.method,
        input.request_no,
        input.idempotency_key,
    );
    let token = stable_hex_token(&seed);
    let order_no = format!("RC{}", token);
    let out_trade_no = format!("RECHARGE{}", token);

    CreatePointsRechargeOrderCommand::new(
        &input.subject.tenant_id,
        input.subject.organization_id.as_deref(),
        &input.subject.user_id,
        input.amount,
        input.currency_code,
        input.method,
        &format!("order-{token}"),
        &format!("order-item-{token}"),
        &format!("payment-intent-{token}"),
        &format!("payment-attempt-{token}"),
        &order_no,
        &out_trade_no,
        &requested_at,
        &expire_at,
        input.idempotency_key,
        input.package_id,
        input.client_request_no,
        input.source,
    )
}

fn map_recharge_package(value: RechargePackageItem) -> RechargePackageResponse {
    RechargePackageResponse {
        id: value.id,
        price_amount: value.price_amount.as_str().to_string(),
        currency_code: value.currency_code,
        bonus_points: value.bonus_points,
        grant_amount: value.grant_amount,
        points: value.points,
    }
}

fn map_recharge_settings(value: RechargeSettingsSnapshot) -> RechargeSettingsResponse {
    RechargeSettingsResponse {
        base_currency_code: value.base_currency_code,
        base_points_per_cny: value.base_points_per_cny,
        currency_to_cny_rates: value.currency_to_cny_rates,
        preview_examples: value
            .preview_examples
            .into_iter()
            .map(|(currency_code, amount_map)| {
                (
                    currency_code,
                    amount_map
                        .into_iter()
                        .map(|(amount, preview)| (amount, map_recharge_preview(preview)))
                        .collect::<BTreeMap<_, _>>(),
                )
            })
            .collect(),
    }
}

fn map_recharge_preview(value: RechargeGrantPreview) -> RechargeGrantPreviewResponse {
    RechargeGrantPreviewResponse {
        grant_amount: value.grant_amount,
    }
}

fn merge_recharge_pay_outcome(
    mut order: CreatePointsRechargeOrderOutcome,
    pay: PayOwnerOrderOutcome,
) -> CreatePointsRechargeOrderOutcome {
    order.out_trade_no = pay.out_trade_no;
    order.payment_method = pay.payment_method;
    if let Some(cashier_url) = pay.payment_params.get("cashierUrl") {
        order.cashier_url = cashier_url.clone();
        order.qr_code_payload = cashier_url.clone();
    }
    order.next_action = pay
        .payment_params
        .get("nextAction")
        .cloned()
        .unwrap_or_else(|| "scan_qr".to_string());
    order
}

fn map_recharge_outcome(value: CreatePointsRechargeOrderOutcome) -> SubmitRechargeResponse {
    SubmitRechargeResponse {
        success: value.success,
        order_no: value.order_no,
        out_trade_no: value.out_trade_no,
        amount: value.amount.as_str().to_string(),
        currency_code: value.currency_code,
        points: value.points,
        provider_code: value.provider_code,
        payment_method: value.payment_method,
        payment_product: value.payment_product,
        status: value.status,
        next_action: value.next_action,
        cashier_url: value.cashier_url,
        qr_code_payload: value.qr_code_payload,
        request_payment_payload: value.request_payment_payload,
    }
}

fn map_checkout_status(value: CheckoutStatusSnapshot) -> CheckoutStatusResponse {
    CheckoutStatusResponse {
        order_no: value.order_no,
        out_trade_no: value.out_trade_no,
        amount: value.amount.as_str().to_string(),
        currency_code: value.currency_code,
        points: value.points,
        provider_code: value.provider_code,
        payment_method: value.payment_method,
        payment_product: value.payment_product,
        order_status: value.order_status,
        payment_status: value.payment_status,
        recharge_status: value.recharge_status,
        status: value.status,
        created_at: value.created_at,
        expires_at: value.expires_at,
        paid_at: value.paid_at,
        next_action: value.next_action,
        cashier_url: value.cashier_url,
        qr_code_payload: value.qr_code_payload,
        request_payment_payload: value.request_payment_payload,
    }
}

fn fallback_request_no(
    subject: &AppRuntimeSubject,
    amount: &str,
    method: &str,
    idempotency_key: &str,
) -> String {
    stable_header_token(&format!(
        "points-recharge-{}-{}-{}-{}",
        subject.user_id, amount, method, idempotency_key
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

fn money_cents(amount: &str) -> Result<i64, ()> {
    let value = amount.trim();
    let mut parts = value.split('.');
    let whole = parts
        .next()
        .unwrap_or_default()
        .parse::<i64>()
        .map_err(|_| ())?;
    let fraction = parts.next().unwrap_or_default();
    if parts.next().is_some() || fraction.len() > 2 {
        return Err(());
    }
    let mut padded = fraction.to_string();
    while padded.len() < 2 {
        padded.push('0');
    }
    let cents = if padded.is_empty() {
        0
    } else {
        padded.parse::<i64>().map_err(|_| ())?
    };
    whole
        .checked_mul(100)
        .and_then(|amount| amount.checked_add(cents))
        .ok_or(())
}

fn format_money_minor(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.abs();
    format!("{sign}{}.{:02}", abs / 100, abs % 100)
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

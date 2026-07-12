use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use sdkwork_order_service::stable_json_request_hash;
use sdkwork_utils_rust::{SdkWorkProblemDetail, SdkWorkResultCode};
use sdkwork_web_core::WebRequestContext;
use serde::Serialize;

pub(crate) const IDEMPOTENCY_KEY_HEADER: &str = "Idempotency-Key";
pub(crate) const REQUEST_HASH_HEADER: &str = "Sdkwork-Request-Hash";
pub(crate) const REQUEST_NO_HEADER: &str = "Sdkwork-Request-No";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppWriteCommandHeaders {
    pub idempotency_key: String,
    pub request_hash: String,
    pub request_no: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WriteCommandHeaderError {
    MissingHeader(&'static str),
    InvalidHeader(String),
}

pub(crate) fn validate_write_payload(
    headers: &HeaderMap,
    scope: &str,
    body: &impl Serialize,
    fallback_request_no: impl FnOnce(&str) -> String,
) -> Result<AppWriteCommandHeaders, WriteCommandHeaderError> {
    let write_headers = parse_required_write_command_headers(headers, fallback_request_no)?;
    let expected_hash = stable_json_request_hash(scope, body).map_err(|_| {
        WriteCommandHeaderError::InvalidHeader(
            "request body could not be canonicalized for request hash validation".to_owned(),
        )
    })?;
    if expected_hash.trim() != write_headers.request_hash.trim() {
        return Err(WriteCommandHeaderError::InvalidHeader(
            "Sdkwork-Request-Hash does not match the command payload".to_owned(),
        ));
    }
    Ok(write_headers)
}

pub(crate) fn validate_app_write_payload(
    context: Option<&WebRequestContext>,
    headers: &HeaderMap,
    scope: &str,
    body: &impl Serialize,
    fallback_request_no: impl FnOnce(&str) -> String,
) -> Result<AppWriteCommandHeaders, Box<Response>> {
    validate_write_payload(headers, scope, body, fallback_request_no)
        .map_err(|error| Box::new(write_command_header_error_to_app_response(context, error)))
}

pub(crate) fn write_payload_with_route_param(
    route_param_key: &str,
    route_param_value: &str,
    body: &impl Serialize,
) -> serde_json::Value {
    let mut payload = serde_json::to_value(body).expect("write payload must serialize");
    if let serde_json::Value::Object(ref mut fields) = payload {
        fields.insert(
            route_param_key.to_string(),
            serde_json::Value::String(route_param_value.to_string()),
        );
    }
    payload
}

pub(crate) fn parse_required_write_command_headers(
    headers: &HeaderMap,
    fallback_request_no: impl FnOnce(&str) -> String,
) -> Result<AppWriteCommandHeaders, WriteCommandHeaderError> {
    let idempotency_key = required_text_header(headers, IDEMPOTENCY_KEY_HEADER)
        .map_err(|_| WriteCommandHeaderError::MissingHeader(IDEMPOTENCY_KEY_HEADER))?;
    let request_hash = required_text_header(headers, REQUEST_HASH_HEADER)
        .map_err(|_| WriteCommandHeaderError::MissingHeader(REQUEST_HASH_HEADER))?;
    let request_no = optional_text_header(headers, REQUEST_NO_HEADER)
        .unwrap_or_else(|| fallback_request_no(&idempotency_key));
    Ok(AppWriteCommandHeaders {
        idempotency_key,
        request_hash,
        request_no,
    })
}

pub(crate) fn required_app_write_command_headers(
    context: Option<&WebRequestContext>,
    headers: &HeaderMap,
    fallback_request_no: impl FnOnce(&str) -> String,
) -> Result<AppWriteCommandHeaders, Box<Response>> {
    parse_required_write_command_headers(headers, fallback_request_no)
        .map_err(|error| Box::new(write_command_header_error_to_app_response(context, error)))
}

fn write_command_header_error_to_app_response(
    context: Option<&WebRequestContext>,
    error: WriteCommandHeaderError,
) -> Response {
    let trace_id = context
        .and_then(|ctx| ctx.trace_id.clone())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(sdkwork_utils_rust::uuid);
    match error {
        WriteCommandHeaderError::MissingHeader(name) => problem_response(
            StatusCode::BAD_REQUEST,
            SdkWorkResultCode::MissingRequiredField,
            format!("{name} header is required"),
            &trace_id,
        ),
        WriteCommandHeaderError::InvalidHeader(message) => problem_response(
            StatusCode::BAD_REQUEST,
            SdkWorkResultCode::ValidationError,
            message,
            &trace_id,
        ),
    }
}

pub(crate) fn ensure_request_hash_matches(
    context: Option<&WebRequestContext>,
    expected_hash: &str,
    provided_hash: &str,
) -> Result<(), Box<Response>> {
    if expected_hash.trim() == provided_hash.trim() {
        return Ok(());
    }

    let trace_id = context
        .and_then(|ctx| ctx.trace_id.clone())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(sdkwork_utils_rust::uuid);
    Err(Box::new(problem_response(
        StatusCode::BAD_REQUEST,
        SdkWorkResultCode::ValidationError,
        "Sdkwork-Request-Hash does not match the command payload",
        &trace_id,
    )))
}

fn required_text_header(
    headers: &HeaderMap,
    name: &'static str,
) -> Result<String, WriteCommandHeaderError> {
    let value = headers
        .get(name)
        .ok_or(WriteCommandHeaderError::MissingHeader(name))?
        .to_str()
        .map(str::trim)
        .map_err(|_| {
            WriteCommandHeaderError::InvalidHeader(format!("{name} header value is invalid"))
        })?;
    if value.is_empty() {
        return Err(WriteCommandHeaderError::MissingHeader(name));
    }
    Ok(value.to_owned())
}

fn optional_text_header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn problem_response(
    status: StatusCode,
    result_code: SdkWorkResultCode,
    detail: impl Into<String>,
    trace_id: &str,
) -> Response {
    let problem = SdkWorkProblemDetail::platform(result_code, detail, trace_id.to_owned());
    (status, Json(problem)).into_response()
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, StatusCode};
    use sdkwork_web_core::WebRequestContext;
    use serde::{Deserialize, Serialize};

    use sdkwork_order_service::{
        stable_canonical_json_request_hash, stable_command_request_hash, stable_json_request_hash,
    };

    use super::*;

    fn ctx() -> Option<WebRequestContext> {
        None
    }

    #[test]
    fn required_app_write_command_headers_requires_idempotency_and_request_hash() {
        let mut headers = HeaderMap::new();
        headers.insert(IDEMPOTENCY_KEY_HEADER, HeaderValue::from_static("idem-1"));
        headers.insert(REQUEST_HASH_HEADER, HeaderValue::from_static("hash-1"));

        let parsed = required_app_write_command_headers(ctx().as_ref(), &headers, |_| {
            "request-1".to_owned()
        })
        .expect("headers");
        assert_eq!(parsed.idempotency_key, "idem-1");
        assert_eq!(parsed.request_hash, "hash-1");
        assert_eq!(parsed.request_no, "request-1");
    }

    #[test]
    fn stable_command_request_hash_is_deterministic() {
        let first = stable_command_request_hash("scope", &["100001", "request-1"]);
        let second = stable_command_request_hash("scope", &["100001", "request-1"]);
        assert_eq!(first, second);
        assert!(!first.is_empty());
    }

    #[test]
    fn stable_json_request_hash_matches_struct_and_value_payloads() {
        let body_json = r#"{"methodKey":"wechat_pay","displayName":"WeChat Pay","providerCode":"wechat_pay","status":"active"}"#;
        let value: serde_json::Value = serde_json::from_str(body_json).expect("json");
        let from_value = stable_canonical_json_request_hash("payment-method-upsert", &value);

        #[derive(Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct UpsertPaymentMethodBody {
            method_key: Option<String>,
            display_name: Option<String>,
            provider_code: Option<String>,
            status: Option<String>,
            sort_order: Option<i64>,
        }

        let body: UpsertPaymentMethodBody = serde_json::from_str(body_json).expect("body");
        let from_struct = stable_json_request_hash("payment-method-upsert", &body).expect("hash");

        assert_eq!(from_value, from_struct);
    }

    #[test]
    fn ensure_request_hash_matches_rejects_mismatch_with_problem_detail() {
        let error = ensure_request_hash_matches(ctx().as_ref(), "expected", "provided")
            .expect_err("mismatch");
        assert_eq!(error.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn missing_header_returns_numeric_validation_problem() {
        let headers = HeaderMap::new();
        let response = required_app_write_command_headers(ctx().as_ref(), &headers, |_| {
            "request-1".to_owned()
        })
        .expect_err("missing header");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

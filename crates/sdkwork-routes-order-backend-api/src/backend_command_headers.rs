//! Backend write-command header validation (`API_SPEC.md` idempotent commands).

use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use sdkwork_order_service::stable_json_request_hash;
use sdkwork_utils_rust::{SdkWorkProblemDetail, SdkWorkResultCode};
use sdkwork_web_core::WebRequestContext;
use serde::Serialize;

use crate::api_response::resolve_trace_id;

pub const IDEMPOTENCY_KEY_HEADER: &str = "Idempotency-Key";
pub const REQUEST_HASH_HEADER: &str = "Sdkwork-Request-Hash";
pub const REQUEST_NO_HEADER: &str = "Sdkwork-Request-No";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendWriteCommandHeaders {
    pub idempotency_key: String,
    pub request_hash: String,
    pub request_no: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WriteCommandHeaderError {
    MissingHeader(&'static str),
    InvalidHeader(String),
}

pub fn write_payload_with_route_param(
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

pub fn validate_backend_write_payload(
    context: Option<&WebRequestContext>,
    headers: &HeaderMap,
    scope: &str,
    payload: &impl Serialize,
    fallback_request_no: impl FnOnce(&str) -> String,
) -> Result<BackendWriteCommandHeaders, Response> {
    let parsed = parse_required_write_command_headers(headers, fallback_request_no)
        .map_err(|error| write_command_header_error_to_response(context, error))?;
    let expected_hash = stable_json_request_hash(scope, payload).map_err(|_| {
        write_command_header_error_to_response(
            context,
            WriteCommandHeaderError::InvalidHeader("command payload must serialize".to_string()),
        )
    })?;
    if expected_hash.trim() != parsed.request_hash.trim() {
        let trace_id = resolve_trace_id(context);
        return Err(problem_response(
            StatusCode::BAD_REQUEST,
            SdkWorkResultCode::ValidationError,
            "Sdkwork-Request-Hash does not match the command payload",
            &trace_id,
        ));
    }
    Ok(parsed)
}

fn parse_required_write_command_headers(
    headers: &HeaderMap,
    fallback_request_no: impl FnOnce(&str) -> String,
) -> Result<BackendWriteCommandHeaders, WriteCommandHeaderError> {
    let idempotency_key = required_text_header(headers, IDEMPOTENCY_KEY_HEADER)?;
    let request_hash = required_text_header(headers, REQUEST_HASH_HEADER)?;
    let request_no = optional_text_header(headers, REQUEST_NO_HEADER)
        .unwrap_or_else(|| fallback_request_no(&idempotency_key));
    Ok(BackendWriteCommandHeaders {
        idempotency_key,
        request_hash,
        request_no,
    })
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

fn write_command_header_error_to_response(
    context: Option<&WebRequestContext>,
    error: WriteCommandHeaderError,
) -> Response {
    let trace_id = resolve_trace_id(context);
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

fn problem_response(
    status: StatusCode,
    result_code: SdkWorkResultCode,
    detail: impl Into<String>,
    trace_id: &str,
) -> Response {
    let problem = SdkWorkProblemDetail::platform(result_code, detail, trace_id.to_owned());
    (status, Json(problem)).into_response()
}

//! Backend write-command header validation (`API_SPEC.md` idempotent commands).

use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use sdkwork_utils_rust::{SdkWorkProblemDetail, SdkWorkResultCode};
use sdkwork_web_core::WebRequestContext;

use crate::api_response::resolve_trace_id;

pub const IDEMPOTENCY_KEY_HEADER: &str = "Idempotency-Key";
pub const REQUEST_NO_HEADER: &str = "Sdkwork-Request-No";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendWriteCommandHeaders {
    pub idempotency_key: String,
    pub request_no: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WriteCommandHeaderError {
    InvalidHeader(String),
}

pub fn resolve_backend_write_command_headers(
    context: Option<&WebRequestContext>,
    headers: &HeaderMap,
    fallback_request_no: impl FnOnce(&str) -> String,
) -> Result<BackendWriteCommandHeaders, Box<Response>> {
    let idempotency_key = match optional_text_header(headers, IDEMPOTENCY_KEY_HEADER) {
        Some(value) => validate_idempotency_key(value)
            .map_err(|error| Box::new(write_command_header_error_to_response(context, error)))?,
        None => sdkwork_utils_rust::uuid(),
    };
    let request_no = optional_text_header(headers, REQUEST_NO_HEADER)
        .unwrap_or_else(|| fallback_request_no(&idempotency_key));
    Ok(BackendWriteCommandHeaders {
        idempotency_key,
        request_no,
    })
}

fn validate_idempotency_key(value: String) -> Result<String, WriteCommandHeaderError> {
    let valid_length = (8..=128).contains(&value.len());
    let valid_characters = value.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | ':' | '-')
    });
    if valid_length && valid_characters {
        Ok(value)
    } else {
        Err(WriteCommandHeaderError::InvalidHeader(
            "Idempotency-Key must contain 8 to 128 letters, digits, dots, underscores, colons, or hyphens"
                .to_string(),
        ))
    }
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

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    #[test]
    fn omitted_client_idempotency_headers_are_generated_server_side() {
        let parsed = resolve_backend_write_command_headers(None, &HeaderMap::new(), |key| {
            format!("cancel-{key}")
        })
        .expect("optional headers");

        assert!(!parsed.idempotency_key.is_empty());
        assert!(parsed.request_no.starts_with("cancel-"));
    }

    #[test]
    fn malformed_client_idempotency_key_is_rejected() {
        let mut headers = HeaderMap::new();
        headers.insert(IDEMPOTENCY_KEY_HEADER, HeaderValue::from_static("short"));

        assert!(
            resolve_backend_write_command_headers(None, &headers, |key| format!("cancel-{key}"),)
                .is_err()
        );
    }
}

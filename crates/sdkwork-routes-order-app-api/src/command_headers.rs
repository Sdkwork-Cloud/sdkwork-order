use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use sdkwork_utils_rust::{SdkWorkProblemDetail, SdkWorkResultCode};
use sdkwork_web_core::WebRequestContext;

pub(crate) const IDEMPOTENCY_KEY_HEADER: &str = "Idempotency-Key";
pub(crate) const REQUEST_NO_HEADER: &str = "Sdkwork-Request-No";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppWriteCommandHeaders {
    pub idempotency_key: String,
    pub request_no: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WriteCommandHeaderError {
    MissingHeader(&'static str),
    InvalidHeader(String),
}

pub(crate) fn parse_required_write_command_headers(
    headers: &HeaderMap,
    fallback_request_no: impl FnOnce(&str) -> String,
) -> Result<AppWriteCommandHeaders, WriteCommandHeaderError> {
    let idempotency_key = required_text_header(headers, IDEMPOTENCY_KEY_HEADER)
        .map_err(|_| WriteCommandHeaderError::MissingHeader(IDEMPOTENCY_KEY_HEADER))?;
    let request_no = optional_text_header(headers, REQUEST_NO_HEADER)
        .unwrap_or_else(|| fallback_request_no(&idempotency_key));
    Ok(AppWriteCommandHeaders {
        idempotency_key,
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

    use super::*;

    fn ctx() -> Option<WebRequestContext> {
        None
    }

    #[test]
    fn required_app_write_command_headers_requires_only_idempotency_key() {
        let mut headers = HeaderMap::new();
        headers.insert(IDEMPOTENCY_KEY_HEADER, HeaderValue::from_static("idem-1"));

        let parsed = required_app_write_command_headers(ctx().as_ref(), &headers, |_| {
            "request-1".to_owned()
        })
        .expect("headers");
        assert_eq!(parsed.idempotency_key, "idem-1");
        assert_eq!(parsed.request_no, "request-1");
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

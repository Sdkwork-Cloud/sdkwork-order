use sdkwork_contract_service::CommerceServiceError;
use sdkwork_utils_rust::{validated_offset_list_params, MAX_LIST_PAGE_SIZE};

pub mod request_hash;
pub mod write_command_hash;

pub fn require_non_empty(field: &str, value: &str) -> Result<(), CommerceServiceError> {
    if value.trim().is_empty() {
        return Err(CommerceServiceError::validation(format!(
            "{field} is required"
        )));
    }

    Ok(())
}

pub fn offset_list_params(
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<(i64, i64), CommerceServiceError> {
    let params = validated_offset_list_params(page, page_size).map_err(|_| {
        CommerceServiceError::validation(format!(
            "page must be >= 1 and page_size must be between 1 and {MAX_LIST_PAGE_SIZE}"
        ))
    })?;
    Ok((params.page, params.page_size))
}

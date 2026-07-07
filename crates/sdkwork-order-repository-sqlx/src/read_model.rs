//! Read-model error policy: production fails fast; tests may opt into lenient empty pages.

/// When `ORDER_READ_MODEL_LENIENT=1`, missing `commerce_*` tables yield empty list/detail reads.
/// Production deployments must leave this unset so schema drift surfaces as storage errors.
pub(crate) fn tolerate_missing_read_model_tables() -> bool {
    matches!(
        std::env::var("ORDER_READ_MODEL_LENIENT").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

pub(crate) fn read_model_table_is_missing(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(database_error) => {
            let message = database_error.message().to_ascii_lowercase();
            message.contains("does not exist")
                || message.contains("no such table")
                || message.contains("no such column")
        }
        _ => false,
    }
}

pub(crate) fn empty_rows_when_read_model_is_missing<T>(
    error: sqlx::Error,
) -> Result<Vec<T>, sqlx::Error> {
    if tolerate_missing_read_model_tables() && read_model_table_is_missing(&error) {
        Ok(Vec::new())
    } else {
        Err(error)
    }
}

pub(crate) fn none_when_read_model_is_missing<T>(error: sqlx::Error) -> Result<Option<T>, sqlx::Error> {
    if tolerate_missing_read_model_tables() && read_model_table_is_missing(&error) {
        Ok(None)
    } else {
        Err(error)
    }
}

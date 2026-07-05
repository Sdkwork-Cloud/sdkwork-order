use sdkwork_iam_context_service::IamAppContext;

#[derive(Debug, Clone)]
pub(crate) struct BackendOperatorScope {
    pub tenant_id: String,
    pub organization_id: Option<String>,
}

pub(crate) fn backend_operator_scope_from_iam(
    context: &IamAppContext,
) -> Result<BackendOperatorScope, String> {
    let tenant_id = required_context_text(&context.tenant_id, "tenant_id")?;
    let _user_id = required_context_text(&context.user_id, "user_id")?;
    let organization_id = context
        .organization_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);

    Ok(BackendOperatorScope {
        tenant_id,
        organization_id,
    })
}

fn required_context_text(value: &str, field_name: &'static str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!(
            "authenticated runtime context {field_name} is required"
        ));
    }
    Ok(value.to_owned())
}

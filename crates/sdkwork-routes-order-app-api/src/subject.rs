use axum::Extension;
use sdkwork_iam_context_service::IamAppContext;

#[derive(Debug, Clone)]
pub(crate) struct AppRuntimeSubject {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub user_id: String,
}

pub(crate) fn app_runtime_subject_from_extension(
    context: Option<Extension<IamAppContext>>,
) -> Result<AppRuntimeSubject, String> {
    let Some(Extension(context)) = context else {
        return Err("authenticated runtime context is required".to_owned());
    };
    app_runtime_subject_from_iam(&context)
}

pub(crate) fn app_runtime_subject_from_iam(
    context: &IamAppContext,
) -> Result<AppRuntimeSubject, String> {
    let tenant_id = required_context_text(&context.tenant_id, "tenant_id")?;
    let user_id = required_context_text(&context.user_id, "user_id")?;
    let organization_id = context
        .organization_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);

    Ok(AppRuntimeSubject {
        tenant_id,
        organization_id,
        user_id,
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

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_web_core::{
        ServerRequestId, WebApiSurface, WebAuthMode, WebRequestContext, WebRequestPrincipal,
        WebTransportFacts,
    };

    #[test]
    fn builds_app_runtime_subject_from_web_request_context() {
        let context = WebRequestContext {
            request_id: ServerRequestId::new("test-request"),
            api_surface: WebApiSurface::AppApi,
            auth_mode: WebAuthMode::DualToken,
            transport: WebTransportFacts::default(),
            principal: Some(
                WebRequestPrincipal::builder()
                    .tenant_id("100001")
                    .organization_id(Some("0".to_owned()))
                    .user_id("user-1")
                    .app_id("sdkwork-order-pc")
                    .build(),
            ),
            locale: None,
            client_kind: None,
            operation: None,
            trace_id: None,
        };

        let subject = app_runtime_subject_from_web_context(Some(&context)).expect("subject");

        assert_eq!("100001", subject.tenant_id);
        assert_eq!(Some("0"), subject.organization_id.as_deref());
        assert_eq!("user-1", subject.user_id);
    }
}

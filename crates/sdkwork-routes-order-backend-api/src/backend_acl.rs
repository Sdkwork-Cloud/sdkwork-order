use axum::response::Response;
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_web_core::WebRequestContext;

use crate::api_response::{forbidden as api_forbidden, unauthorized as api_unauthorized};
use crate::subject::backend_operator_scope_from_iam;
pub(crate) use crate::subject::BackendOperatorScope;

pub(crate) fn require_backend_operator(
    ctx: Option<&WebRequestContext>,
    context: IamAppContext,
    required_permission: &str,
) -> Result<BackendOperatorScope, Box<Response>> {
    if !context.can_access_backend_api() {
        return Err(Box::new(api_forbidden(
            ctx,
            "backend api access requires an organization-scoped session",
        )));
    }
    if !context.has_permission(required_permission) {
        tracing::warn!(
            target = "order.acl",
            user_id = %context.user_id,
            tenant_id = %context.tenant_id,
            required_permission,
            "backend permission denied"
        );
        return Err(Box::new(api_forbidden(
            ctx,
            format!("missing required permission: {required_permission}"),
        )));
    }
    match backend_operator_scope_from_iam(&context) {
        Ok(subject) => Ok(subject),
        Err(message) => Err(Box::new(api_unauthorized(ctx, message))),
    }
}

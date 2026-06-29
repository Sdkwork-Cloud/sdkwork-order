//! Order API server entrypoint.
//!
//! Production-grade bootstrap:
//! - Returns `Result` from host bootstrap so DB errors don't panic the process.
//! - CORS is restricted to an explicit allow-list read from `ORDER_CORS_ALLOW_ORIGINS`.
//! - Readiness probe reflects the real database health via `SELECT 1`.
//! - Graceful shutdown drains in-flight requests on SIGINT / SIGTERM.

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use sdkwork_order_gateway_assembly::assemble_application_router;
use sdkwork_order_service_host::OrderServiceHost;
use sdkwork_web_bootstrap::{
    service_router, ReadinessCheck, ReadinessFuture, ServiceRouterConfig,
};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let host = match OrderServiceHost::from_env().await {
        Ok(host) => Arc::new(host),
        Err(error) => {
            tracing::error!(target = "order.bootstrap", error = %error, "order service host bootstrap failed");
            return Err(error.into());
        }
    };

    let business = assemble_application_router(host.clone()).await.router
        .layer(TraceLayer::new_for_http())
        .layer(build_cors_layer());

    let readiness = Arc::new(OrderReadiness {
        host: host.clone(),
    });
    let app = service_router(
        business,
        ServiceRouterConfig::default().with_readiness_check(readiness.clone()),
    );

    let addr = std::env::var("ORDER_API_BIND")
        .unwrap_or_else(|_| "0.0.0.0:18093".to_owned());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(target = "order.bootstrap", %addr, "order api server listening");

    // `with_graceful_shutdown` makes axum::serve stop accepting new
    // connections once the signal future resolves, then drain in-flight
    // requests. We don't duplicate the signal with tokio::select! here.
    let serve = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal());

    if let Err(error) = serve.await {
        tracing::error!(target = "order.runtime", error = %error, "axum serve failed");
        return Err(error.into());
    }

    // Drain DB connections after handlers finish.
    tokio::time::timeout(Duration::from_secs(30), host.database_pool().close())
        .await
        .map_err(|_| std::io::Error::other("database pool close timed out after 30s"))?;
    tracing::info!(target = "order.runtime", "order api server stopped");
    Ok(())
}

/// Readiness probe that checks the database can answer `SELECT 1`.
#[derive(Clone)]
struct OrderReadiness {
    host: Arc<OrderServiceHost>,
}

impl ReadinessCheck for OrderReadiness {
    fn check(&self) -> ReadinessFuture<'_> {
        Box::pin(async move {
            use sdkwork_database_sqlx::DatabasePool;
            let result = match self.host.database_pool() {
                DatabasePool::Postgres(pool, _) => {
                    sqlx::query_scalar::<_, i64>("SELECT 1")
                        .fetch_one(pool)
                        .await
                }
                DatabasePool::Sqlite(pool, _) => {
                    sqlx::query_scalar::<_, i64>("SELECT 1")
                        .fetch_one(pool)
                        .await
                }
            };
            match result {
                Ok(_) => Ok(()),
                Err(error) => {
                    tracing::error!(target = "order.readiness", error = %error, "database readiness probe failed");
                    Err("database is not ready".to_owned())
                }
            }
        })
    }
}

/// Builds the CORS layer from the `ORDER_CORS_ALLOW_ORIGINS` env var.
///
/// - When unset or empty: deny all cross-origin requests (fail-closed).
/// - When set to `*`: emit a warning and fall back to a permissive policy
///   ONLY when `ORDER_CORS_PERMISSIVE_DEV=1` is also set, otherwise deny.
/// - Otherwise: comma-separated list of allowed origins.
fn build_cors_layer() -> CorsLayer {
    let raw = std::env::var("ORDER_CORS_ALLOW_ORIGINS")
        .unwrap_or_default()
        .trim()
        .to_owned();

    let allow_origin = if raw.is_empty() {
        tracing::warn!(
            target = "order.security",
            "ORDER_CORS_ALLOW_ORIGINS is not set; cross-origin requests are denied"
        );
        AllowOrigin::list([])
    } else if raw == "*" {
        if std::env::var("ORDER_CORS_PERMISSIVE_DEV").as_deref() == Ok("1") {
            tracing::warn!(
                target = "order.security",
                "CORS is permissive (dev mode) — never use in production"
            );
            AllowOrigin::mirror_request()
        } else {
            tracing::error!(
                target = "order.security",
                "ORDER_CORS_ALLOW_ORIGINS='*' ignored without ORDER_CORS_PERMISSIVE_DEV=1; cross-origin requests are denied"
            );
            AllowOrigin::list([])
        }
    } else {
        let origins: Vec<_> = raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter_map(|s| match s.parse::<axum::http::HeaderValue>() {
                Ok(value) => Some(value),
                Err(error) => {
                    tracing::warn!(target = "order.security", origin = %s, error = %error, "invalid CORS origin ignored");
                    None
                }
            })
            .collect();
        AllowOrigin::list(origins)
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(tower_http::cors::Any)
        .allow_credentials(true)
        .max_age(Duration::from_secs(600))
}

/// Waits for SIGINT (Ctrl+C) or SIGTERM to trigger graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::warn!(target = "order.runtime", error = %error, "ctrl_c signal handler failed");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(error) => {
                tracing::warn!(target = "order.runtime", error = %error, "SIGTERM signal handler failed");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

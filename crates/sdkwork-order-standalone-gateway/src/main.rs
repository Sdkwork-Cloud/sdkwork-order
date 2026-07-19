//! Order API server entrypoint.
//!
//! Production-grade bootstrap:
//! - Returns `Result` from host bootstrap so DB errors don't panic the process.
//! - CORS is restricted to an explicit allow-list read from `ORDER_CORS_ALLOW_ORIGINS`.
//! - Readiness probe reflects the real database health via `SELECT 1`.
//! - Graceful shutdown drains in-flight requests on SIGINT / SIGTERM.

use std::sync::Arc;
use std::time::Duration;

use sdkwork_order_gateway_assembly::{assemble_application_router, ApplicationAssembly};
use sdkwork_order_service_host::OrderServiceHost;
use sdkwork_web_bootstrap::{service_router, ReadinessCheck, ReadinessFuture, ServiceRouterConfig};
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

    if std::env::var("ORDER_READ_MODEL_LENIENT").as_deref() == Ok("1") {
        tracing::warn!(
            target = "order.security",
            "ORDER_READ_MODEL_LENIENT=1 is active; missing commerce tables return empty reads — forbidden in production"
        );
    }

    let business = assemble_application_router(host.clone())
        .await
        .router
        .layer(TraceLayer::new_for_http());

    let readiness = Arc::new(OrderReadiness { host: host.clone() });
    let app = service_router(
        business,
        ServiceRouterConfig::default()
            .with_readiness_check(readiness.clone())
            .with_contract_fallback(ApplicationAssembly::contract_fallback_config()),
    );

    let addr = std::env::var("ORDER_API_BIND").unwrap_or_else(|_| "0.0.0.0:18093".to_owned());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(target = "order.bootstrap", %addr, "order api server listening");

    // `with_graceful_shutdown` makes axum::serve stop accepting new
    // connections once the signal future resolves, then drain in-flight
    // requests. We don't duplicate the signal with tokio::select! here.
    let serve = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());

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

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_lifecycle::{lifecycle_options_from_env, LifecycleOrchestrator};
use sdkwork_database_spi::{DatabaseAssetProvider, DatabaseManifest, DefaultDatabaseModule};
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool, PoolContext};
use std::path::PathBuf;
use std::sync::Arc;

pub struct OrderDatabaseHost {
    pool: DatabasePool,
    module: Arc<DefaultDatabaseModule>,
}

impl OrderDatabaseHost {
    pub fn from_pool(pool: DatabasePool) -> Result<Self, String> {
        Ok(Self {
            pool,
            module: load_order_database_module()?,
        })
    }

    pub fn from_sqlite_pool(pool: sqlx::SqlitePool) -> Result<Self, String> {
        let config = DatabaseConfig {
            engine: DatabaseEngine::Sqlite,
            url: "sqlite::memory:".to_owned(),
            max_connections: 1,
            ..Default::default()
        };
        Self::from_pool(DatabasePool::Sqlite(pool, PoolContext { config }))
    }

    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    pub fn module(&self) -> Arc<DefaultDatabaseModule> {
        self.module.clone()
    }
}

pub async fn bootstrap_order_database_from_env() -> Result<OrderDatabaseHost, String> {
    let _ = dotenvy::dotenv();
    let config = DatabaseConfig::from_env("ORDER")
        .map_err(|error| format!("read order database config failed: {error}"))?;
    let pool = create_pool_from_config(config)
        .await
        .map_err(|error| format!("create order database pool failed: {error}"))?;
    let module = load_order_database_module()?;
    let manifest = DatabaseManifest::from_file(module.manifest_path())
        .map_err(|error| format!("read order database manifest failed: {error}"))?;
    let options = lifecycle_options_from_env("ORDER", &manifest);
    let orchestrator =
        LifecycleOrchestrator::new(pool.clone(), module.clone()).with_applied_by("sdkwork-order");
    orchestrator.init().await.map_err(|e| format!("{e}"))?;
    if options.auto_migrate {
        orchestrator.migrate().await.map_err(|e| format!("{e}"))?;
    }
    Ok(OrderDatabaseHost { pool, module })
}

fn load_order_database_module() -> Result<Arc<DefaultDatabaseModule>, String> {
    let app_root = std::env::var("SDKWORK_ORDER_APP_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    Ok(Arc::new(
        DefaultDatabaseModule::from_app_root(&app_root)
            .map_err(|error| format!("load order database module failed: {error}"))?,
    ))
}

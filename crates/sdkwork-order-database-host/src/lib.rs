use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_lifecycle::{lifecycle_options_from_env, LifecycleOrchestrator};
use sdkwork_database_spi::{
    DatabaseAssetProvider, DatabaseManifest, DefaultDatabaseModule, SpiError,
};
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
            module: Arc::new(
                database_module()
                    .map_err(|error| format!("load order database module failed: {error}"))?,
            ),
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

/// Loads the Order database assets for registration by standalone or federated hosts.
pub fn database_module() -> Result<DefaultDatabaseModule, SpiError> {
    let app_root = std::env::var("SDKWORK_ORDER_APP_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let raw = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            std::fs::canonicalize(&raw).unwrap_or(raw)
        });
    DefaultDatabaseModule::from_app_root(&app_root)
}

pub async fn bootstrap_order_database_from_env() -> Result<OrderDatabaseHost, String> {
    let _ = dotenvy::dotenv();
    let config = DatabaseConfig::from_env("ORDER")
        .map_err(|error| format!("read order database config failed: {error}"))?;
    let pool = create_pool_from_config(config)
        .await
        .map_err(|error| format!("create order database pool failed: {error}"))?;
    let module = Arc::new(
        database_module().map_err(|error| format!("load order database module failed: {error}"))?,
    );
    let manifest = DatabaseManifest::from_file(module.manifest_path())
        .map_err(|error| format!("read order database manifest failed: {error}"))?;
    let options = lifecycle_options_from_env("ORDER", &manifest);
    let orchestrator =
        LifecycleOrchestrator::new(pool.clone(), module.clone()).with_applied_by("sdkwork-order");
    orchestrator.init().await.map_err(|e| format!("{e}"))?;
    if options.auto_migrate {
        orchestrator.migrate().await.map_err(|e| format!("{e}"))?;
    }
    if options.seed_on_boot {
        orchestrator
            .seed(&options.seed_locale, &options.seed_profile)
            .await
            .map_err(|e| format!("{e}"))?;
    }
    Ok(OrderDatabaseHost { pool, module })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_module_exposes_seeded_order_assets() {
        let module = database_module().expect("load order database module");
        let manifest = DatabaseManifest::from_file(module.manifest_path())
            .expect("read order database manifest");

        assert_eq!(manifest.module_id, "order");
        assert_eq!(manifest.service_code, "ORDER");
        assert!(manifest.lifecycle.seed_on_boot);
        assert!(module.seeds_dir().join("seed.manifest.json").is_file());
    }
}

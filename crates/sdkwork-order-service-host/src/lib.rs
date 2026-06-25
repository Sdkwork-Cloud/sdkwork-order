use sdkwork_database_sqlx::DatabasePool;
use sdkwork_order_database_host::{bootstrap_order_database_from_env, OrderDatabaseHost};

pub struct OrderServiceHost {
    database: OrderDatabaseHost,
}

impl OrderServiceHost {
    pub async fn new() -> Self {
        Self::from_env().await.expect("order service host bootstrap failed")
    }

    pub async fn from_env() -> Result<Self, String> {
        let database = bootstrap_order_database_from_env().await?;
        Ok(Self { database })
    }

    pub fn database_pool(&self) -> &DatabasePool {
        self.database.pool()
    }

    pub fn database_module(&self) -> std::sync::Arc<sdkwork_database_spi::DefaultDatabaseModule> {
        self.database.module()
    }
}

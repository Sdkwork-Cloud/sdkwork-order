use sdkwork_database_sqlx::DatabasePool;
use sdkwork_order_database_host::{bootstrap_order_database_from_env, OrderDatabaseHost};
use sdkwork_order_integration_account::account_points_credit_port_from_env;
use sdkwork_order_service::AccountPointsCreditPort;
use std::sync::Arc;

pub struct OrderServiceHost {
    database: OrderDatabaseHost,
    account_credit_port: Arc<dyn AccountPointsCreditPort>,
}

impl OrderServiceHost {
    pub async fn new() -> Self {
        Self::from_env().await.expect("order service host bootstrap failed")
    }

    pub async fn from_env() -> Result<Self, String> {
        let database = bootstrap_order_database_from_env().await?;
        let account_credit_port = account_points_credit_port_from_env().await?;
        Ok(Self {
            database,
            account_credit_port,
        })
    }

    pub fn database_pool(&self) -> &DatabasePool {
        self.database.pool()
    }

    pub fn database_module(&self) -> std::sync::Arc<sdkwork_database_spi::DefaultDatabaseModule> {
        self.database.module()
    }

    pub fn account_credit_port(&self) -> Arc<dyn AccountPointsCreditPort> {
        self.account_credit_port.clone()
    }
}

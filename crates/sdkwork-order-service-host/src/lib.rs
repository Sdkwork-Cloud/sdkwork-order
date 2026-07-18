use sdkwork_database_sqlx::DatabasePool;
use sdkwork_order_database_host::{bootstrap_order_database_from_env, OrderDatabaseHost};
use sdkwork_order_integration_account::{
    account_points_credit_port_from_env, account_value_ledger_port_from_env,
};
use sdkwork_order_integration_membership::membership_purchase_fulfillment_port_from_env;
use sdkwork_order_integration_payment::payment_refund_executor_port_from_database_pool;
use sdkwork_order_integration_promotion::promotion_coupon_redemption_port_from_database_pool;
pub use sdkwork_order_service::order_service_contract;
use sdkwork_order_service::{
    AccountPointsCreditPort, AccountValueLedgerPort, CouponRedemptionPort,
    MembershipPurchaseFulfillmentPort, NoopCouponRedemptionPort, NoopPaymentPayoutExecutorPort,
    PaymentPayoutExecutorPort, PaymentRefundExecutorPort,
};
use std::sync::Arc;

pub struct OrderServiceHost {
    database: OrderDatabaseHost,
    account_credit_port: Arc<dyn AccountPointsCreditPort>,
    account_value_ledger_port: Arc<dyn AccountValueLedgerPort>,
    coupon_redemption_port: Arc<dyn CouponRedemptionPort>,
    membership_fulfillment_port: Arc<dyn MembershipPurchaseFulfillmentPort>,
    payment_refund_executor_port: Arc<dyn PaymentRefundExecutorPort>,
    payment_payout_executor_port: Arc<dyn PaymentPayoutExecutorPort>,
}

impl OrderServiceHost {
    pub async fn new() -> Self {
        Self::from_env()
            .await
            .expect("order service host bootstrap failed")
    }

    pub async fn from_env() -> Result<Self, String> {
        let database = bootstrap_order_database_from_env().await?;
        Self::from_database_with_env_integrations(database).await
    }

    /// Builds the Order service container on a pool owned by an embedding gateway assembly.
    pub async fn from_database_pool(pool: DatabasePool) -> Result<Self, String> {
        let database = OrderDatabaseHost::from_pool(pool)?;
        Self::from_database_with_env_integrations(database).await
    }

    async fn from_database_with_env_integrations(
        database: OrderDatabaseHost,
    ) -> Result<Self, String> {
        let account_credit_port = account_points_credit_port_from_env().await?;
        let account_value_ledger_port = account_value_ledger_port_from_env().await?;
        let coupon_redemption_port =
            promotion_coupon_redemption_port_from_database_pool(database.pool());
        let membership_fulfillment_port = membership_purchase_fulfillment_port_from_env()?;
        let payment_refund_executor_port =
            payment_refund_executor_port_from_database_pool(database.pool());
        let payment_payout_executor_port = Arc::new(NoopPaymentPayoutExecutorPort);
        Ok(Self {
            database,
            account_credit_port,
            account_value_ledger_port,
            coupon_redemption_port,
            membership_fulfillment_port,
            payment_refund_executor_port,
            payment_payout_executor_port,
        })
    }

    pub fn from_parts(
        database: OrderDatabaseHost,
        account_credit_port: Arc<dyn AccountPointsCreditPort>,
        account_value_ledger_port: Arc<dyn AccountValueLedgerPort>,
        membership_fulfillment_port: Arc<dyn MembershipPurchaseFulfillmentPort>,
        payment_refund_executor_port: Arc<dyn PaymentRefundExecutorPort>,
        payment_payout_executor_port: Arc<dyn PaymentPayoutExecutorPort>,
    ) -> Self {
        Self::from_parts_with_coupon(
            database,
            account_credit_port,
            account_value_ledger_port,
            Arc::new(NoopCouponRedemptionPort),
            membership_fulfillment_port,
            payment_refund_executor_port,
            payment_payout_executor_port,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts_with_coupon(
        database: OrderDatabaseHost,
        account_credit_port: Arc<dyn AccountPointsCreditPort>,
        account_value_ledger_port: Arc<dyn AccountValueLedgerPort>,
        coupon_redemption_port: Arc<dyn CouponRedemptionPort>,
        membership_fulfillment_port: Arc<dyn MembershipPurchaseFulfillmentPort>,
        payment_refund_executor_port: Arc<dyn PaymentRefundExecutorPort>,
        payment_payout_executor_port: Arc<dyn PaymentPayoutExecutorPort>,
    ) -> Self {
        Self {
            database,
            account_credit_port,
            account_value_ledger_port,
            coupon_redemption_port,
            membership_fulfillment_port,
            payment_refund_executor_port,
            payment_payout_executor_port,
        }
    }

    pub fn from_sqlite_pool(
        pool: sqlx::SqlitePool,
        account_credit_port: Arc<dyn AccountPointsCreditPort>,
        account_value_ledger_port: Arc<dyn AccountValueLedgerPort>,
        membership_fulfillment_port: Arc<dyn MembershipPurchaseFulfillmentPort>,
        payment_refund_executor_port: Arc<dyn PaymentRefundExecutorPort>,
        payment_payout_executor_port: Arc<dyn PaymentPayoutExecutorPort>,
    ) -> Result<Self, String> {
        let database = OrderDatabaseHost::from_sqlite_pool(pool)?;
        Ok(Self::from_parts(
            database,
            account_credit_port,
            account_value_ledger_port,
            membership_fulfillment_port,
            payment_refund_executor_port,
            payment_payout_executor_port,
        ))
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

    pub fn account_value_ledger_port(&self) -> Arc<dyn AccountValueLedgerPort> {
        self.account_value_ledger_port.clone()
    }

    pub fn coupon_redemption_port(&self) -> Arc<dyn CouponRedemptionPort> {
        self.coupon_redemption_port.clone()
    }

    pub fn membership_fulfillment_port(&self) -> Arc<dyn MembershipPurchaseFulfillmentPort> {
        self.membership_fulfillment_port.clone()
    }

    pub fn payment_refund_executor_port(&self) -> Arc<dyn PaymentRefundExecutorPort> {
        self.payment_refund_executor_port.clone()
    }

    pub fn payment_payout_executor_port(&self) -> Arc<dyn PaymentPayoutExecutorPort> {
        self.payment_payout_executor_port.clone()
    }
}

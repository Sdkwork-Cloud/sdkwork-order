pub mod order_lifecycle;
pub mod order_limits;
pub mod order_payment_settlement;
pub mod order_settlement_context;
pub mod postgres_account_value;
pub mod postgres_after_sales;
pub mod postgres_checkout;
pub mod postgres_fulfillment;
pub mod postgres_management;
pub mod postgres_membership_order;
pub mod postgres_order;
pub mod postgres_recharge;
pub mod postgres_shipment;
pub mod read_model;
pub mod recharge_platform_catalog;
pub mod sql_store_error;
pub mod sqlite_account_value;
pub mod sqlite_after_sales;
pub mod sqlite_checkout;
pub mod sqlite_fulfillment;
pub mod sqlite_management;
pub mod sqlite_membership_order;
pub mod sqlite_order;
pub mod sqlite_recharge;
pub mod sqlite_shipment;

#[cfg(any(test, feature = "test-support"))]
mod test_sqlite_pool;

#[cfg(test)]
mod order_lifecycle_audit_tests;

#[cfg(test)]
mod order_management_parity_tests;

#[cfg(test)]
mod order_payment_settlement_tests;

#[cfg(any(test, feature = "test-support"))]
pub use test_sqlite_pool::{
    order_points_recharge_e2e_postgres_pool_from_env, order_points_recharge_e2e_sqlite_memory_pool,
};

pub use order_settlement_context::OrderPaymentSettlementContext;
pub use postgres_membership_order::PostgresCommerceMembershipOrderStore;
pub use postgres_order::PostgresCommerceOrderStore;
pub use postgres_recharge::PostgresCommerceRechargeStore;
pub use sqlite_membership_order::SqliteCommerceMembershipOrderStore;
pub use sqlite_order::SqliteCommerceOrderStore;
pub use sqlite_recharge::SqliteCommerceRechargeStore;

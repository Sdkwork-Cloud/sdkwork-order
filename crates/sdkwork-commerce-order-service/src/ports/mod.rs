use crate::{CreateOrderCommand, OrderDetailQuery, OrderListQuery, PaidOrderReference};
use sdkwork_commerce_contract_service::CommerceServiceError;

pub trait OrderRepositoryPort {
    fn create_order(
        &self,
        command: &CreateOrderCommand,
    ) -> Result<PaidOrderReference, CommerceServiceError>;

    fn retrieve_order(
        &self,
        query: &OrderDetailQuery,
    ) -> Result<Option<PaidOrderReference>, CommerceServiceError>;

    fn list_orders(&self, query: &OrderListQuery) -> Result<Vec<String>, CommerceServiceError>;
}

pub const ORDER_REPOSITORY_PORT: &str = "order.repository";
pub const IDEMPOTENCY_REPOSITORY_PORT: &str = "idempotency.repository";

use crate::OrderItemDraft;
use sdkwork_commerce_contract_service::CommerceMoney;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateOrderCommand {
    pub discount_amount: CommerceMoney,
    pub idempotency_key: String,
    pub items: Vec<OrderItemDraft>,
    pub owner_user_id: String,
    pub request_no: String,
    pub subject: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancelOrderCommand {
    pub idempotency_key: String,
    pub order_id: String,
    pub request_no: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderPaymentSettlementContext {
    pub owner_user_id: String,
    pub subject: String,
}

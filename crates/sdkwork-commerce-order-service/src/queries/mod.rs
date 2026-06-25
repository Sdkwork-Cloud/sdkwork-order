mod after_sales;
mod checkout;
mod fulfillment;
mod management;
mod owner;
mod shipment;

pub use after_sales::{
    AfterSalesEventListQuery, AfterSalesEventView, AfterSalesRequestDetailQuery,
    AfterSalesRequestView, AfterSalesReturnShipmentView, CreateAfterSalesRequestCommand,
    CreateAfterSalesReturnShipmentCommand, UpdateAfterSalesRequestCommand,
};
pub use checkout::{
    CheckoutLineInput, CheckoutQuoteView, CheckoutSessionDetailQuery, CheckoutSessionView,
    CreateCheckoutQuoteCommand, CreateCheckoutSessionCommand,
};
pub use fulfillment::{FulfillmentDetailQuery, FulfillmentListQuery, FulfillmentView};
pub use management::{
    CancelManagementOrderCommand, CloseManagementOrderCommand, OrderCancellationListQuery,
    OrderCancellationView, OrderManagementDetailQuery, OrderManagementEventListQuery,
    OrderManagementEventView, OrderManagementListQuery,
};
pub use owner::{
    CancelOwnerOrderCommand, CreateOwnerOrderCommand, CreateOwnerOrderOutcome, OrderOwnerDetail,
    OrderOwnerDetailQuery, OrderOwnerItem, OrderOwnerListQuery, OrderOwnerStatistics,
    OrderOwnerSummary, PayOwnerOrderCommand, PayOwnerOrderOutcome,
};
pub use shipment::{
    ShipmentDetailQuery, ShipmentPackageListQuery, ShipmentPackageView,
    ShipmentTrackingEventListQuery, ShipmentTrackingEventView, ShipmentView,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderListQuery {
    pub owner_user_id: String,
    pub status: Option<String>,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderDetailQuery {
    pub order_id: String,
    pub tenant_id: String,
}

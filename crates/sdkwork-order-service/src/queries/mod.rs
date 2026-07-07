mod after_sales;
mod checkout;
mod fulfillment;
mod management;
mod owner;
mod recharge;
mod shipment;

pub use recharge::{
    CheckoutStatusQuery, RechargePackageListPage, RechargePackageListQuery, RechargeSettingsQuery,
};

pub use after_sales::{
    AfterSalesEventListQuery, AfterSalesEventPage, AfterSalesEventView, AfterSalesManagementDetailQuery,
    AfterSalesManagementListQuery, AfterSalesRequestDetailQuery, AfterSalesRequestListQuery,
    AfterSalesRequestPage, AfterSalesRequestView, AfterSalesReturnShipmentListQuery,
    AfterSalesReturnShipmentPage, AfterSalesReturnShipmentView, CreateAfterSalesRequestCommand,
    CreateAfterSalesRequestItemInput, CreateAfterSalesReturnShipmentCommand,
    ReviewAfterSalesRequestCommand, UpdateAfterSalesRequestCommand,
};
pub use checkout::{
    CheckoutLineInput, CheckoutQuoteView, CheckoutSessionDetailQuery, CheckoutSessionView,
    CreateCheckoutQuoteCommand, CreateCheckoutSessionCommand,
};
pub use fulfillment::{
    FulfillmentDetailQuery, FulfillmentListPage, FulfillmentListQuery, FulfillmentView,
};
pub use management::{
    CancelManagementOrderCommand, CloseManagementOrderCommand, OrderCancellationListQuery,
    OrderCancellationPage, OrderCancellationView, OrderManagementDetailQuery,
    OrderManagementEventListQuery, OrderManagementEventPage, OrderManagementEventView,
    OrderManagementListPage, OrderManagementListQuery,
};
pub use owner::{
    CancelOwnerOrderCommand, CreateOwnerOrderCommand, CreateOwnerOrderOutcome, OrderOwnerDetail,
    OrderOwnerDetailQuery, OrderOwnerEventListQuery, OrderOwnerEventPage, OrderOwnerEventView,
    OrderOwnerItem, OrderOwnerListPage, OrderOwnerListQuery, OrderOwnerStatistics,
    OrderOwnerSummary, PayOwnerOrderCommand, PayOwnerOrderOutcome,
};
pub use shipment::{
    CreateShipmentPackageCommand, ShipmentDetailQuery, ShipmentManagementDetailQuery,
    ShipmentManagementListPage, ShipmentManagementListQuery, ShipmentPackageListQuery,
    ShipmentPackageManagementListQuery, ShipmentPackagePage, ShipmentPackageView,
    ShipmentTrackingEventListQuery, ShipmentTrackingEventPage, ShipmentTrackingEventView,
    ShipmentView, UpdateShipmentPackageCommand,
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

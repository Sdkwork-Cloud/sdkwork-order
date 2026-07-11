use sdkwork_contract_service::CommerceMoney;
use sdkwork_order_service::{
    order_service_contract, OrderAmountBreakdown, OrderItemDraft, OrderStatus, OrderTransition,
    PaidOrderReference,
};

#[test]
fn computes_order_payable_amount_from_items_and_discount() {
    let item =
        OrderItemDraft::new("sku-1", "Pro plan", 2, CommerceMoney::new("1000").unwrap()).unwrap();
    let breakdown =
        OrderAmountBreakdown::from_items(vec![item], CommerceMoney::new("300").unwrap()).unwrap();

    assert_eq!(breakdown.original_amount.as_str(), "2000");
    assert_eq!(breakdown.discount_amount.as_str(), "300");
    assert_eq!(breakdown.payable_amount.as_str(), "1700");
}

#[test]
fn rejects_order_amount_overflow_instead_of_panicking_or_zeroing() {
    let huge_item = OrderItemDraft::new(
        "sku-huge",
        "Huge plan",
        1,
        CommerceMoney::new("9223372036854775807").unwrap(),
    )
    .unwrap();
    let extra_item = OrderItemDraft::new(
        "sku-extra",
        "Extra plan",
        1,
        CommerceMoney::new("1").unwrap(),
    )
    .unwrap();

    assert!(OrderAmountBreakdown::from_items(
        vec![huge_item, extra_item],
        CommerceMoney::new("0").unwrap()
    )
    .is_err());
}

#[test]
fn rejects_order_line_total_overflow() {
    let item = OrderItemDraft::new(
        "sku-many",
        "Many seats",
        u32::MAX,
        CommerceMoney::new("2147483649").unwrap(),
    )
    .unwrap();

    assert!(
        OrderAmountBreakdown::from_items(vec![item], CommerceMoney::new("0").unwrap()).is_err()
    );
}

#[test]
fn validates_order_status_lifecycle() {
    assert_eq!(
        OrderTransition::new(OrderStatus::PendingPayment, OrderStatus::Paid).validate(),
        Ok(())
    );
    assert!(
        OrderTransition::new(OrderStatus::Completed, OrderStatus::Paid)
            .validate()
            .is_err()
    );
}

#[test]
fn only_pending_payment_orders_can_be_cancelled_or_expired() {
    assert!(OrderStatus::PendingPayment.can_cancel());
    assert!(OrderStatus::PendingPayment.can_expire());
    assert!(!OrderStatus::Paid.can_cancel());
}

#[test]
fn paid_order_reference_requires_payment_id_before_invoice_linking() {
    let reference = PaidOrderReference::new("order-1", "payment-1").unwrap();

    assert_eq!(reference.order_id, "order-1");
    assert_eq!(reference.payment_id, "payment-1");
    assert!(PaidOrderReference::new("order-1", "").is_err());
}

#[test]
fn order_service_contract_owns_shipment_package_queries_and_commands() {
    let contract = order_service_contract();

    for operation_id in ["shipments.packages.create", "shipments.packages.update"] {
        assert!(
            contract.write_commands.contains(&operation_id),
            "shipment package command must be owned by order service: {operation_id}",
        );
    }

    for operation_id in [
        "shipments.packages.list",
        "shipments.packages.management.list",
        "shipments.list",
    ] {
        assert!(
            contract.read_queries.contains(&operation_id),
            "shipment package reads must be owned by order service: {operation_id}",
        );
    }
}

#[test]
fn order_service_contract_owns_after_sales_lifecycle_queries_and_commands() {
    let contract = order_service_contract();

    for operation_id in [
        "afterSales.requests.create",
        "afterSales.requests.update",
        "afterSales.returnShipments.create",
        "afterSales.reviews.create",
    ] {
        assert!(
            contract.write_commands.contains(&operation_id),
            "after-sales command must be owned by order service: {operation_id}",
        );
    }

    for operation_id in [
        "afterSales.requests.list",
        "afterSales.requests.retrieve",
        "afterSales.management.list",
        "afterSales.management.retrieve",
        "afterSales.returnShipments.list",
        "afterSales.events.list",
    ] {
        assert!(
            contract.read_queries.contains(&operation_id),
            "after-sales read must be owned by order service: {operation_id}",
        );
    }
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sdkwork_contract_service::CommerceMoney;
use sdkwork_order_service::{
    default_fulfill_points_recharge_command, fulfill_points_recharge_order,
    mark_points_recharge_payment_succeeded, points_recharge_fulfillment_idempotency_key,
    points_recharge_fulfillment_transaction_no, points_recharge_payment_success_idempotency_key,
    AccountPointsCreditPort, FulfillPointsRechargeOrderCommand,
    FulfillPointsRechargeOrderOutcome, MarkPointsRechargePaymentSucceededCommand,
    OrderSubjectKind, POINTS_RECHARGE_LEDGER_BUSINESS_TYPE, PointsRechargeCreditOutcome,
    PointsRechargeCreditRequest, PointsRechargeFulfillmentContext, PointsRechargeFulfillmentStore,
};

#[test]
fn points_recharge_fulfillment_uses_stable_idempotency_and_transaction_keys() {
    assert_eq!(
        points_recharge_fulfillment_idempotency_key("order-42"),
        "points-recharge:fulfill:order-42"
    );
    assert_eq!(
        points_recharge_fulfillment_transaction_no("order-42"),
        "points-recharge:order-42"
    );
    assert_eq!(
        points_recharge_payment_success_idempotency_key("order-42"),
        "points-recharge:payment-success:order-42"
    );
    assert_eq!(POINTS_RECHARGE_LEDGER_BUSINESS_TYPE, "points_recharge");
}

#[test]
fn fulfillment_context_rejects_unpaid_orders() {
    let context = PointsRechargeFulfillmentContext {
        order_id: "order-1".to_owned(),
        order_no: "ORD-1".to_owned(),
        order_status: "pending_payment".to_owned(),
        fulfillment_status: "unfulfilled".to_owned(),
        payment_status: "pending".to_owned(),
        payment_attempt_status: "pending".to_owned(),
        points: 100,
        amount: CommerceMoney::new("10.00").expect("money"),
        currency_code: "CNY".to_owned(),
        billing_history_status: Some("pending".to_owned()),
    };

    assert!(context.validate_for_fulfillment().is_err());
}

#[test]
fn fulfillment_context_accepts_succeeded_payment() {
    let context = PointsRechargeFulfillmentContext {
        order_id: "order-1".to_owned(),
        order_no: "ORD-1".to_owned(),
        order_status: "pending_payment".to_owned(),
        fulfillment_status: "unfulfilled".to_owned(),
        payment_status: "pending".to_owned(),
        payment_attempt_status: "succeeded".to_owned(),
        points: 100,
        amount: CommerceMoney::new("10.00").expect("money"),
        currency_code: "CNY".to_owned(),
        billing_history_status: Some("pending".to_owned()),
    };

    assert!(context.validate_for_fulfillment().is_ok());
}

#[tokio::test]
async fn fulfill_points_recharge_order_credits_account_then_commits_order() {
    let store = Arc::new(MockFulfillmentStore::default());
    let credit_port = Arc::new(MockAccountPointsCreditPort::default());

    store.seed_context(PointsRechargeFulfillmentContext {
        order_id: "order-99".to_owned(),
        order_no: "ORD-99".to_owned(),
        order_status: "pending_payment".to_owned(),
        fulfillment_status: "unfulfilled".to_owned(),
        payment_status: "success".to_owned(),
        payment_attempt_status: "succeeded".to_owned(),
        points: 250,
        amount: CommerceMoney::new("25.00").expect("money"),
        currency_code: "CNY".to_owned(),
        billing_history_status: Some("pending".to_owned()),
    });

    let command = default_fulfill_points_recharge_command(
        "100001",
        Some("0"),
        "1",
        "order-99",
        "req-fulfill-1",
    )
    .expect("command");

    let outcome = fulfill_points_recharge_order(store.as_ref(), credit_port.as_ref(), command)
        .await
        .expect("fulfillment");

    assert!(outcome.accepted);
    assert!(!outcome.replayed);
    assert_eq!(outcome.points_credited, 250);
    assert_eq!(store.commit_calls(), 1);
    assert_eq!(credit_port.credit_calls(), 1);
}

#[tokio::test]
async fn fulfill_points_recharge_order_replays_when_already_fulfilled() {
    let store = Arc::new(MockFulfillmentStore::default());
    let credit_port = Arc::new(MockAccountPointsCreditPort::default());

    store.seed_context(PointsRechargeFulfillmentContext {
        order_id: "order-88".to_owned(),
        order_no: "ORD-88".to_owned(),
        order_status: "paid".to_owned(),
        fulfillment_status: "fulfilled".to_owned(),
        payment_status: "success".to_owned(),
        payment_attempt_status: "succeeded".to_owned(),
        points: 120,
        amount: CommerceMoney::new("12.00").expect("money"),
        currency_code: "CNY".to_owned(),
        billing_history_status: Some("completed".to_owned()),
    });

    let command = default_fulfill_points_recharge_command(
        "100001",
        Some("0"),
        "1",
        "order-88",
        "req-fulfill-2",
    )
    .expect("command");

    let outcome = fulfill_points_recharge_order(store.as_ref(), credit_port.as_ref(), command)
        .await
        .expect("fulfillment");

    assert!(outcome.replayed);
    assert_eq!(credit_port.credit_calls(), 0);
    assert_eq!(store.commit_calls(), 0);
}

#[tokio::test]
async fn mark_points_recharge_payment_succeeded_delegates_to_store() {
    let store = Arc::new(MockFulfillmentStore::default());
    let command = MarkPointsRechargePaymentSucceededCommand::new(
        "100001",
        Some("0"),
        "1",
        "order-77",
        "2026-06-29 12:00:00",
        "req-pay-success-1",
        &points_recharge_payment_success_idempotency_key("order-77"),
    )
    .expect("command");

    mark_points_recharge_payment_succeeded(store.as_ref(), command)
        .await
        .expect("payment success");

    assert_eq!(store.payment_success_calls(), 1);
}

#[derive(Default)]
struct MockFulfillmentStore {
    contexts: Mutex<HashMap<String, PointsRechargeFulfillmentContext>>,
    commit_calls: Mutex<u32>,
    payment_success_calls: Mutex<u32>,
}

impl MockFulfillmentStore {
    fn seed_context(&self, context: PointsRechargeFulfillmentContext) {
        self.contexts
            .lock()
            .expect("context lock")
            .insert(context.order_id.clone(), context);
    }

    fn commit_calls(&self) -> u32 {
        *self.commit_calls.lock().expect("commit lock")
    }

    fn payment_success_calls(&self) -> u32 {
        *self.payment_success_calls.lock().expect("payment lock")
    }
}

impl PointsRechargeFulfillmentStore for MockFulfillmentStore {
    fn load_points_recharge_fulfillment_context<'a>(
        &'a self,
        command: &'a FulfillPointsRechargeOrderCommand,
    ) -> sdkwork_order_service::PointsRechargeFulfillmentFuture<'a, Option<PointsRechargeFulfillmentContext>> {
        let context = self
            .contexts
            .lock()
            .expect("context lock")
            .get(&command.order_id)
            .cloned();
        Box::pin(async move { Ok(context) })
    }

    fn commit_points_recharge_fulfillment<'a>(
        &'a self,
        command: FulfillPointsRechargeOrderCommand,
        context: &'a PointsRechargeFulfillmentContext,
    ) -> sdkwork_order_service::PointsRechargeFulfillmentFuture<'a, FulfillPointsRechargeOrderOutcome> {
        *self.commit_calls.lock().expect("commit lock") += 1;
        let order_no = context.order_no.clone();
        let points = context.points;
        Box::pin(async move {
            Ok(FulfillPointsRechargeOrderOutcome::fulfilled(
                &command.order_id,
                &order_no,
                points,
            ))
        })
    }

    fn rollback_points_recharge_fulfillment<'a>(
        &'a self,
        _command: &'a FulfillPointsRechargeOrderCommand,
        _context: &'a PointsRechargeFulfillmentContext,
    ) -> sdkwork_order_service::PointsRechargeFulfillmentFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }

    fn mark_points_recharge_payment_succeeded<'a>(
        &'a self,
        _command: MarkPointsRechargePaymentSucceededCommand,
    ) -> sdkwork_order_service::PointsRechargeFulfillmentFuture<'a, ()> {
        *self.payment_success_calls.lock().expect("payment lock") += 1;
        Box::pin(async { Ok(()) })
    }
}

#[derive(Default)]
struct MockAccountPointsCreditPort {
    credit_calls: Mutex<u32>,
}

impl MockAccountPointsCreditPort {
    fn credit_calls(&self) -> u32 {
        *self.credit_calls.lock().expect("credit lock")
    }
}

impl AccountPointsCreditPort for MockAccountPointsCreditPort {
    fn credit_points_recharge<'a>(
        &'a self,
        request: PointsRechargeCreditRequest,
    ) -> sdkwork_order_service::AccountPointsCreditFuture<'a, PointsRechargeCreditOutcome> {
        *self.credit_calls.lock().expect("credit lock") += 1;
        assert_eq!(request.points, 250);
        assert_eq!(
            request.idempotency_key,
            points_recharge_fulfillment_idempotency_key("order-99")
        );
        Box::pin(async {
            Ok(PointsRechargeCreditOutcome {
                accepted: true,
                replayed: false,
            })
        })
    }
}

#[test]
fn order_subject_kind_parses_checkout_subjects_case_insensitively() {
    assert_eq!(
        OrderSubjectKind::parse(Some("points_recharge")),
        OrderSubjectKind::PointsRecharge
    );
    assert_eq!(
        OrderSubjectKind::parse(Some("PRODUCT")),
        OrderSubjectKind::Product
    );
    assert!(OrderSubjectKind::PointsRecharge.is_fulfillment_implemented());
    assert!(!OrderSubjectKind::Product.is_fulfillment_implemented());
}

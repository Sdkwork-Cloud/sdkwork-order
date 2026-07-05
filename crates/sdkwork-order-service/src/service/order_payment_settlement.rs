use sdkwork_contract_service::CommerceServiceError;

use crate::{
    default_fulfill_points_recharge_command, fulfill_points_recharge_order,
    mark_points_recharge_payment_succeeded, points_recharge_payment_success_idempotency_key,
    AccountPointsCreditPort, MarkPointsRechargePaymentSucceededCommand,
    OrderPaymentSettlementAttempt, OwnerOrderPaymentConfirmationPort, PointsRechargeFulfillmentStore,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OrderSubjectKind {
    PointsRecharge,
    Product,
    VirtualGoods,
    Membership,
    CouponPackage,
    Unknown,
}

impl OrderSubjectKind {
    pub fn parse(subject: Option<&str>) -> Self {
        match subject.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) if value.eq_ignore_ascii_case("points_recharge") => Self::PointsRecharge,
            Some(value) if value.eq_ignore_ascii_case("product") => Self::Product,
            Some(value) if value.eq_ignore_ascii_case("virtual_goods") => Self::VirtualGoods,
            Some(value) if value.eq_ignore_ascii_case("membership") => Self::Membership,
            Some(value) if value.eq_ignore_ascii_case("coupon_package") => Self::CouponPackage,
            Some(_) => Self::Unknown,
            None => Self::Unknown,
        }
    }

    pub fn is_fulfillment_implemented(self) -> bool {
        matches!(self, Self::PointsRecharge)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OwnerOrderSettlementOutcome {
    pub payment_confirmed: bool,
    pub payment_replayed: bool,
    pub fulfillment_accepted: bool,
    pub fulfillment_replayed: bool,
    pub order_id: String,
    pub points_credited: i64,
    pub fulfillment_status: String,
}

pub async fn settle_owner_order_after_payment_success<S, P, Payment>(
    payment_store: &Payment,
    recharge_store: &S,
    credit_port: &P,
    attempt: &OrderPaymentSettlementAttempt,
    order_subject: Option<&str>,
    request_no: &str,
) -> Result<OwnerOrderSettlementOutcome, CommerceServiceError>
where
    S: PointsRechargeFulfillmentStore,
    P: AccountPointsCreditPort + ?Sized,
    Payment: OwnerOrderPaymentConfirmationPort + ?Sized,
{
    let payment_outcome = payment_store
        .confirm_owner_order_payment(
            &attempt.tenant_id,
            attempt.organization_id.as_deref(),
            &attempt.owner_user_id,
            &attempt.order_id,
        )
        .await?;

    let subject_kind = OrderSubjectKind::parse(order_subject);
    let fulfillment = dispatch_subject_fulfillment(
        subject_kind,
        recharge_store,
        credit_port,
        attempt,
        &payment_outcome.paid_at,
        request_no,
    )
    .await?;

    Ok(OwnerOrderSettlementOutcome {
        payment_confirmed: true,
        payment_replayed: payment_outcome.replayed,
        fulfillment_accepted: fulfillment.accepted,
        fulfillment_replayed: fulfillment.replayed,
        order_id: attempt.order_id.clone(),
        points_credited: fulfillment.points_credited,
        fulfillment_status: fulfillment.status,
    })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SubjectFulfillmentOutcome {
    accepted: bool,
    replayed: bool,
    points_credited: i64,
    status: String,
}

async fn dispatch_subject_fulfillment<S, P>(
    subject: OrderSubjectKind,
    recharge_store: &S,
    credit_port: &P,
    attempt: &OrderPaymentSettlementAttempt,
    paid_at: &str,
    request_no: &str,
) -> Result<SubjectFulfillmentOutcome, CommerceServiceError>
where
    S: PointsRechargeFulfillmentStore,
    P: AccountPointsCreditPort + ?Sized,
{
    match subject {
        OrderSubjectKind::PointsRecharge => {
            settle_points_recharge_subject(recharge_store, credit_port, attempt, paid_at, request_no)
                .await
        }
        OrderSubjectKind::Product
        | OrderSubjectKind::VirtualGoods
        | OrderSubjectKind::Membership
        | OrderSubjectKind::CouponPackage => {
            tracing::warn!(
                target = "order.settlement",
                order_id = %attempt.order_id,
                ?subject,
                "payment confirmed; subject fulfillment handler is not implemented yet"
            );
            Ok(SubjectFulfillmentOutcome {
                accepted: false,
                replayed: false,
                points_credited: 0,
                status: "fulfillment_pending".to_owned(),
            })
        }
        OrderSubjectKind::Unknown => {
            tracing::warn!(
                target = "order.settlement",
                order_id = %attempt.order_id,
                "payment confirmed; order subject is missing or unsupported for fulfillment"
            );
            Ok(SubjectFulfillmentOutcome {
                accepted: false,
                replayed: false,
                points_credited: 0,
                status: "fulfillment_skipped".to_owned(),
            })
        }
    }
}

async fn settle_points_recharge_subject<S, P>(
    recharge_store: &S,
    credit_port: &P,
    attempt: &OrderPaymentSettlementAttempt,
    paid_at: &str,
    request_no: &str,
) -> Result<SubjectFulfillmentOutcome, CommerceServiceError>
where
    S: PointsRechargeFulfillmentStore,
    P: AccountPointsCreditPort + ?Sized,
{
    let idempotency_key = points_recharge_payment_success_idempotency_key(&attempt.order_id);
    let payment_command = MarkPointsRechargePaymentSucceededCommand::new(
        &attempt.tenant_id,
        attempt.organization_id.as_deref(),
        &attempt.owner_user_id,
        &attempt.order_id,
        paid_at,
        request_no,
        &idempotency_key,
    )?;
    mark_points_recharge_payment_succeeded(recharge_store, payment_command).await?;

    let fulfill_command = default_fulfill_points_recharge_command(
        &attempt.tenant_id,
        attempt.organization_id.as_deref(),
        &attempt.owner_user_id,
        &attempt.order_id,
        request_no,
    )?;
    let fulfill_outcome =
        fulfill_points_recharge_order(recharge_store, credit_port, fulfill_command).await?;

    Ok(SubjectFulfillmentOutcome {
        accepted: fulfill_outcome.accepted,
        replayed: fulfill_outcome.replayed,
        points_credited: fulfill_outcome.points_credited,
        status: fulfill_outcome.fulfillment_status,
    })
}

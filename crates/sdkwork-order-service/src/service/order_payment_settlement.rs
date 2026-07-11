use sdkwork_contract_service::CommerceServiceError;

use crate::{
    default_fulfill_account_value_order_command, default_fulfill_points_recharge_command,
    fulfill_account_value_order, fulfill_points_recharge_order,
    mark_points_recharge_payment_succeeded, membership_purchase_fulfillment_idempotency_key,
    points_recharge_payment_success_idempotency_key, AccountPointsCreditPort,
    AccountValueFulfillmentStore, AccountValueLedgerPort, AccountValueOrderSubject,
    MarkPointsRechargePaymentSucceededCommand, MembershipPurchaseFulfillmentPort,
    MembershipPurchaseFulfillmentRequest, OrderPaymentSettlementAttempt,
    OwnerOrderPaymentConfirmationPort, PointsRechargeFulfillmentStore,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OrderSubjectKind {
    PointsRecharge,
    TokenBankRecharge,
    TokenBankPlanPurchase,
    TokenBankPlanRenewal,
    AccountRechargePackage,
    CouponRecharge,
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
            Some(value) if value.eq_ignore_ascii_case("token_bank_recharge") => {
                Self::TokenBankRecharge
            }
            Some(value) if value.eq_ignore_ascii_case("token_bank_plan_purchase") => {
                Self::TokenBankPlanPurchase
            }
            Some(value) if value.eq_ignore_ascii_case("token_bank_plan_renewal") => {
                Self::TokenBankPlanRenewal
            }
            Some(value) if value.eq_ignore_ascii_case("account_recharge_package") => {
                Self::AccountRechargePackage
            }
            Some(value) if value.eq_ignore_ascii_case("coupon_recharge") => Self::CouponRecharge,
            Some(value) if value.eq_ignore_ascii_case("product") => Self::Product,
            Some(value) if value.eq_ignore_ascii_case("virtual_goods") => Self::VirtualGoods,
            Some(value) if value.eq_ignore_ascii_case("membership") => Self::Membership,
            Some(value) if value.eq_ignore_ascii_case("coupon_package") => Self::CouponPackage,
            Some(_) => Self::Unknown,
            None => Self::Unknown,
        }
    }

    pub fn is_fulfillment_implemented(self) -> bool {
        matches!(
            self,
            Self::PointsRecharge
                | Self::TokenBankRecharge
                | Self::TokenBankPlanPurchase
                | Self::TokenBankPlanRenewal
                | Self::AccountRechargePackage
                | Self::CouponRecharge
                | Self::Membership
        )
    }

    fn account_value_subject(self) -> Option<AccountValueOrderSubject> {
        match self {
            Self::TokenBankRecharge => Some(AccountValueOrderSubject::TokenBankRecharge),
            Self::TokenBankPlanPurchase => Some(AccountValueOrderSubject::TokenBankPlanPurchase),
            Self::TokenBankPlanRenewal => Some(AccountValueOrderSubject::TokenBankPlanRenewal),
            Self::AccountRechargePackage => Some(AccountValueOrderSubject::AccountRechargePackage),
            Self::CouponRecharge => Some(AccountValueOrderSubject::CouponRecharge),
            _ => None,
        }
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

pub async fn settle_owner_order_after_payment_success<S, A, P, L, M, Payment>(
    payment_store: &Payment,
    recharge_store: &S,
    account_value_store: &A,
    credit_port: &P,
    account_value_ledger_port: &L,
    membership_port: &M,
    attempt: &OrderPaymentSettlementAttempt,
    order_subject: Option<&str>,
    request_no: &str,
) -> Result<OwnerOrderSettlementOutcome, CommerceServiceError>
where
    S: PointsRechargeFulfillmentStore,
    A: AccountValueFulfillmentStore,
    P: AccountPointsCreditPort + ?Sized,
    L: AccountValueLedgerPort + ?Sized,
    M: MembershipPurchaseFulfillmentPort + ?Sized,
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
        account_value_store,
        credit_port,
        account_value_ledger_port,
        membership_port,
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

async fn dispatch_subject_fulfillment<S, A, P, L, M>(
    subject: OrderSubjectKind,
    recharge_store: &S,
    account_value_store: &A,
    credit_port: &P,
    account_value_ledger_port: &L,
    membership_port: &M,
    attempt: &OrderPaymentSettlementAttempt,
    paid_at: &str,
    request_no: &str,
) -> Result<SubjectFulfillmentOutcome, CommerceServiceError>
where
    S: PointsRechargeFulfillmentStore,
    A: AccountValueFulfillmentStore,
    P: AccountPointsCreditPort + ?Sized,
    L: AccountValueLedgerPort + ?Sized,
    M: MembershipPurchaseFulfillmentPort + ?Sized,
{
    match subject {
        OrderSubjectKind::PointsRecharge => {
            settle_points_recharge_subject(
                recharge_store,
                credit_port,
                attempt,
                paid_at,
                request_no,
            )
            .await
        }
        OrderSubjectKind::TokenBankRecharge
        | OrderSubjectKind::TokenBankPlanPurchase
        | OrderSubjectKind::TokenBankPlanRenewal
        | OrderSubjectKind::AccountRechargePackage
        | OrderSubjectKind::CouponRecharge => {
            settle_account_value_subject(
                subject,
                account_value_store,
                account_value_ledger_port,
                attempt,
                request_no,
            )
            .await
        }
        OrderSubjectKind::Membership => {
            settle_membership_subject(membership_port, attempt, request_no).await
        }
        OrderSubjectKind::Product
        | OrderSubjectKind::VirtualGoods
        | OrderSubjectKind::CouponPackage => {
            tracing::info!(
                target = "order.settlement",
                order_id = %attempt.order_id,
                ?subject,
                "payment confirmed; fulfillment is owned by external commerce capabilities"
            );
            Ok(SubjectFulfillmentOutcome {
                accepted: false,
                replayed: false,
                points_credited: 0,
                status: "awaiting_external_fulfillment".to_owned(),
            })
        }
        OrderSubjectKind::Unknown => {
            tracing::warn!(
                target = "order.settlement",
                order_id = %attempt.order_id,
                "payment confirmed; order subject is missing or unsupported for automated fulfillment"
            );
            Ok(SubjectFulfillmentOutcome {
                accepted: false,
                replayed: false,
                points_credited: 0,
                status: "awaiting_subject_resolution".to_owned(),
            })
        }
    }
}

async fn settle_account_value_subject<A, L>(
    subject: OrderSubjectKind,
    account_value_store: &A,
    account_value_ledger_port: &L,
    attempt: &OrderPaymentSettlementAttempt,
    request_no: &str,
) -> Result<SubjectFulfillmentOutcome, CommerceServiceError>
where
    A: AccountValueFulfillmentStore,
    L: AccountValueLedgerPort + ?Sized,
{
    let account_value_subject = subject.account_value_subject().ok_or_else(|| {
        CommerceServiceError::validation("order subject does not support account value fulfillment")
    })?;
    let command = default_fulfill_account_value_order_command(
        account_value_subject,
        &attempt.tenant_id,
        attempt.organization_id.as_deref(),
        &attempt.owner_user_id,
        &attempt.order_id,
        request_no,
    )?;
    let outcome =
        fulfill_account_value_order(account_value_store, account_value_ledger_port, command)
            .await?;

    Ok(SubjectFulfillmentOutcome {
        accepted: outcome.accepted,
        replayed: outcome.replayed,
        points_credited: 0,
        status: outcome.fulfillment_status,
    })
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

async fn settle_membership_subject<M>(
    membership_port: &M,
    attempt: &OrderPaymentSettlementAttempt,
    request_no: &str,
) -> Result<SubjectFulfillmentOutcome, CommerceServiceError>
where
    M: MembershipPurchaseFulfillmentPort + ?Sized,
{
    let idempotency_key = membership_purchase_fulfillment_idempotency_key(&attempt.order_id);
    let outcome = membership_port
        .fulfill_membership_purchase(MembershipPurchaseFulfillmentRequest {
            tenant_id: attempt.tenant_id.clone(),
            organization_id: attempt.organization_id.clone(),
            owner_user_id: attempt.owner_user_id.clone(),
            order_id: attempt.order_id.clone(),
            request_no: request_no.to_owned(),
            idempotency_key,
        })
        .await?;

    Ok(SubjectFulfillmentOutcome {
        accepted: outcome.accepted,
        replayed: outcome.replayed,
        points_credited: 0,
        status: outcome.fulfillment_status,
    })
}

use sdkwork_contract_service::CommerceServiceError;

use crate::{
    default_fulfill_account_value_order_command, default_fulfill_points_recharge_command,
    fulfill_account_value_order, fulfill_points_recharge_order,
    mark_points_recharge_payment_succeeded, membership_purchase_fulfillment_idempotency_key,
    points_recharge_payment_success_idempotency_key, AccountPointsCreditPort,
    AccountValueFulfillmentStore, AccountValueLedgerPort, AccountValueOrderSubject,
    MarkPointsRechargePaymentSucceededCommand, MembershipPurchaseFulfillmentPort,
    MembershipPurchaseFulfillmentRequest, OrderPaymentSettlementAttempt,
    OwnerOrderPaymentConfirmationPort, OwnerOrderPaymentStatePort, PointsRechargeFulfillmentStore,
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
    External,
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
            Some(value) if value.eq_ignore_ascii_case("physical") => Self::Product,
            Some(value) if value.eq_ignore_ascii_case("physical_shipment") => Self::Product,
            Some(value) if value.eq_ignore_ascii_case("virtual_goods") => Self::VirtualGoods,
            Some(value) if value.eq_ignore_ascii_case("virtual") => Self::VirtualGoods,
            Some(value) if value.eq_ignore_ascii_case("virtual_delivery") => Self::VirtualGoods,
            Some(value) if value.eq_ignore_ascii_case("membership") => Self::Membership,
            Some(value) if value.eq_ignore_ascii_case("membership_activation") => Self::Membership,
            Some(value) if value.eq_ignore_ascii_case("coupon_package") => Self::CouponPackage,
            Some(value) if value.eq_ignore_ascii_case("points_credit") => Self::PointsRecharge,
            Some(value) if is_machine_subject(value) => Self::External,
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

/// Resolve a checkout/order subject from machine-readable merchandise facts.
/// Display titles are intentionally ignored because they are localized and mutable.
pub fn stable_checkout_order_subject(
    fulfillment_type: Option<&str>,
    sku_snapshot_json: Option<&str>,
) -> String {
    normalized_machine_subject(fulfillment_type)
        .or_else(|| stable_subject_from_snapshot(sku_snapshot_json))
        .unwrap_or_else(|| "product".to_owned())
}

/// Resolve the subject used by payment settlement for existing and new orders.
/// Snapshot metadata wins for checkout orders; canonical header subjects remain
/// the fallback for recharge and membership orders that do not use SKU snapshots.
pub fn stable_order_settlement_subject(
    stored_subject: Option<&str>,
    sku_snapshot_json: Option<&str>,
) -> String {
    stable_subject_from_snapshot(sku_snapshot_json)
        .or_else(|| canonical_stored_order_subject(stored_subject))
        .unwrap_or_else(|| "product".to_owned())
}

fn canonical_stored_order_subject(subject: Option<&str>) -> Option<String> {
    let subject = normalized_machine_subject(subject)?;
    match OrderSubjectKind::parse(Some(&subject)) {
        OrderSubjectKind::External | OrderSubjectKind::Unknown => None,
        _ => Some(subject),
    }
}

fn stable_subject_from_snapshot(sku_snapshot_json: Option<&str>) -> Option<String> {
    let snapshot = sku_snapshot_json
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    let value = serde_json::from_str::<serde_json::Value>(snapshot).ok()?;
    [
        "fulfillment_type",
        "fulfillmentType",
        "product_type",
        "productType",
    ]
    .into_iter()
    .find_map(|key| value.get(key).and_then(serde_json::Value::as_str))
    .and_then(|subject| normalized_machine_subject(Some(subject)))
}

fn normalized_machine_subject(subject: Option<&str>) -> Option<String> {
    let subject = subject?.trim();
    if !is_machine_subject(subject) {
        return None;
    }
    Some(subject.to_ascii_lowercase())
}

fn is_machine_subject(subject: &str) -> bool {
    !subject.is_empty()
        && subject
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':'))
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

pub struct OwnerOrderSettlementPorts<'a> {
    pub payment_store: &'a dyn OwnerOrderPaymentConfirmationPort,
    pub order_state_store: &'a dyn OwnerOrderPaymentStatePort,
    pub recharge_store: &'a dyn PointsRechargeFulfillmentStore,
    pub account_value_store: &'a dyn AccountValueFulfillmentStore,
    pub credit_port: &'a dyn AccountPointsCreditPort,
    pub account_value_ledger_port: &'a dyn AccountValueLedgerPort,
    pub membership_port: &'a dyn MembershipPurchaseFulfillmentPort,
}

pub async fn settle_owner_order_after_payment_success(
    ports: OwnerOrderSettlementPorts<'_>,
    attempt: &OrderPaymentSettlementAttempt,
    order_subject: Option<&str>,
    request_no: &str,
) -> Result<OwnerOrderSettlementOutcome, CommerceServiceError> {
    let payment_outcome = ports
        .payment_store
        .confirm_owner_order_payment(attempt)
        .await?;

    ports
        .order_state_store
        .mark_owner_order_payment_succeeded(attempt, &payment_outcome.paid_at)
        .await?;

    let subject_kind = OrderSubjectKind::parse(order_subject);
    let fulfillment = dispatch_subject_fulfillment(
        &ports,
        subject_kind,
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

async fn dispatch_subject_fulfillment(
    ports: &OwnerOrderSettlementPorts<'_>,
    subject: OrderSubjectKind,
    attempt: &OrderPaymentSettlementAttempt,
    paid_at: &str,
    request_no: &str,
) -> Result<SubjectFulfillmentOutcome, CommerceServiceError> {
    match subject {
        OrderSubjectKind::PointsRecharge => {
            settle_points_recharge_subject(
                ports.recharge_store,
                ports.credit_port,
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
                ports.account_value_store,
                ports.account_value_ledger_port,
                attempt,
                request_no,
            )
            .await
        }
        OrderSubjectKind::Membership => {
            settle_membership_subject(ports.membership_port, attempt, request_no).await
        }
        OrderSubjectKind::Product
        | OrderSubjectKind::VirtualGoods
        | OrderSubjectKind::CouponPackage
        | OrderSubjectKind::External => {
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
    A: AccountValueFulfillmentStore + ?Sized,
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
    S: PointsRechargeFulfillmentStore + ?Sized,
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

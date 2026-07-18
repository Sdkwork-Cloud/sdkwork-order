use sdkwork_contract_service::{CommerceLedgerBusinessType, CommerceMoney, CommerceServiceError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountValueAssetCode {
    Cash,
    Points,
    TokenBank,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountValueOrderSubject {
    PointsRecharge,
    TokenBankRecharge,
    TokenBankPlanPurchase,
    TokenBankPlanRenewal,
    AccountRechargePackage,
    CouponRecharge,
    RefundRequest,
    CashWithdrawal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenBankPlanPeriod {
    Monthly,
    Quarterly,
    Yearly,
    ContinuousMonthly,
    ContinuousYearly,
}

impl AccountValueAssetCode {
    pub fn parse(value: &str) -> Result<Self, CommerceServiceError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "cash" => Ok(Self::Cash),
            "points" => Ok(Self::Points),
            "token_bank" => Ok(Self::TokenBank),
            "token" | "compute_credit" | "compute_token" => Err(CommerceServiceError::validation(
                "ambiguous account asset name; use token_bank for Token Bank",
            )),
            _ => Err(CommerceServiceError::validation(
                "unsupported account value asset code",
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cash => "cash",
            Self::Points => "points",
            Self::TokenBank => "token_bank",
        }
    }

    pub fn default_unit_code(self) -> &'static str {
        match self {
            Self::Cash => "",
            Self::Points => "POINT",
            Self::TokenBank => "TOKEN_BANK",
        }
    }
}

impl AccountValueOrderSubject {
    pub fn parse(value: &str) -> Result<Self, CommerceServiceError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "points_recharge" => Ok(Self::PointsRecharge),
            "token_bank_recharge" => Ok(Self::TokenBankRecharge),
            "token_bank_plan_purchase" => Ok(Self::TokenBankPlanPurchase),
            "token_bank_plan_renewal" => Ok(Self::TokenBankPlanRenewal),
            "account_recharge_package" => Ok(Self::AccountRechargePackage),
            "coupon_recharge" => Ok(Self::CouponRecharge),
            "refund_request" => Ok(Self::RefundRequest),
            "cash_withdrawal" => Ok(Self::CashWithdrawal),
            _ => Err(CommerceServiceError::validation(
                "unsupported account value order subject",
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PointsRecharge => "points_recharge",
            Self::TokenBankRecharge => "token_bank_recharge",
            Self::TokenBankPlanPurchase => "token_bank_plan_purchase",
            Self::TokenBankPlanRenewal => "token_bank_plan_renewal",
            Self::AccountRechargePackage => "account_recharge_package",
            Self::CouponRecharge => "coupon_recharge",
            Self::RefundRequest => "refund_request",
            Self::CashWithdrawal => "cash_withdrawal",
        }
    }

    pub fn fixed_target_asset(self) -> Option<AccountValueAssetCode> {
        match self {
            Self::PointsRecharge => Some(AccountValueAssetCode::Points),
            Self::TokenBankRecharge | Self::TokenBankPlanPurchase | Self::TokenBankPlanRenewal => {
                Some(AccountValueAssetCode::TokenBank)
            }
            Self::CashWithdrawal => Some(AccountValueAssetCode::Cash),
            Self::AccountRechargePackage | Self::CouponRecharge | Self::RefundRequest => None,
        }
    }

    pub fn requires_payment_collection(self) -> bool {
        matches!(
            self,
            Self::PointsRecharge
                | Self::TokenBankRecharge
                | Self::TokenBankPlanPurchase
                | Self::TokenBankPlanRenewal
                | Self::AccountRechargePackage
        )
    }

    pub fn payment_collection_is_optional(self) -> bool {
        matches!(self, Self::CouponRecharge)
    }

    pub fn validate_target_asset(
        self,
        asset: AccountValueAssetCode,
    ) -> Result<(), CommerceServiceError> {
        if let Some(expected) = self.fixed_target_asset() {
            if expected != asset {
                return Err(CommerceServiceError::validation(format!(
                    "{} must target asset {}",
                    self.as_str(),
                    expected.as_str()
                )));
            }
        }
        Ok(())
    }

    pub fn is_recharge_order_subject(self) -> bool {
        matches!(
            self,
            Self::PointsRecharge
                | Self::TokenBankRecharge
                | Self::TokenBankPlanPurchase
                | Self::TokenBankPlanRenewal
                | Self::AccountRechargePackage
        )
    }

    pub fn is_account_value_fulfillment_subject(self) -> bool {
        matches!(
            self,
            Self::TokenBankRecharge
                | Self::TokenBankPlanPurchase
                | Self::TokenBankPlanRenewal
                | Self::AccountRechargePackage
                | Self::CouponRecharge
        )
    }

    pub fn fulfillment_business_type(self, target_asset: AccountValueAssetCode) -> &'static str {
        match (self, target_asset) {
            (
                Self::TokenBankRecharge | Self::AccountRechargePackage,
                AccountValueAssetCode::TokenBank,
            ) => CommerceLedgerBusinessType::TOKEN_BANK_PURCHASE_CREDIT,
            (
                Self::TokenBankPlanPurchase | Self::TokenBankPlanRenewal | Self::CouponRecharge,
                AccountValueAssetCode::TokenBank,
            ) => CommerceLedgerBusinessType::TOKEN_BANK_GRANT,
            (_, AccountValueAssetCode::TokenBank) => {
                CommerceLedgerBusinessType::TOKEN_BANK_PURCHASE_CREDIT
            }
            (_, AccountValueAssetCode::Points) => CommerceLedgerBusinessType::POINTS_RECHARGE,
            (_, AccountValueAssetCode::Cash) => CommerceLedgerBusinessType::CASH_ADJUSTMENT,
        }
    }

    pub fn compensation_business_type(self, target_asset: AccountValueAssetCode) -> &'static str {
        match target_asset {
            AccountValueAssetCode::TokenBank => CommerceLedgerBusinessType::TOKEN_BANK_REVERSAL,
            AccountValueAssetCode::Points => CommerceLedgerBusinessType::POINTS_CLAWBACK,
            AccountValueAssetCode::Cash => CommerceLedgerBusinessType::CASH_ADJUSTMENT,
        }
    }

    pub fn fulfillment_idempotency_key(
        self,
        order_id: &str,
    ) -> Result<String, CommerceServiceError> {
        match self {
            Self::TokenBankRecharge => Ok(crate::token_bank_recharge_fulfillment_idempotency_key(
                order_id,
            )),
            Self::TokenBankPlanPurchase => {
                Ok(crate::token_bank_plan_purchase_idempotency_key(order_id))
            }
            Self::TokenBankPlanRenewal => {
                Ok(crate::token_bank_plan_renewal_idempotency_key(order_id))
            }
            Self::AccountRechargePackage => {
                Ok(crate::account_package_fulfillment_idempotency_key(order_id))
            }
            Self::CouponRecharge => {
                Ok(crate::coupon_recharge_fulfillment_idempotency_key(order_id))
            }
            _ => Err(CommerceServiceError::validation(
                "order subject does not support account value fulfillment",
            )),
        }
    }
}

impl TokenBankPlanPeriod {
    pub fn parse(value: &str) -> Result<Self, CommerceServiceError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "monthly" => Ok(Self::Monthly),
            "quarterly" => Ok(Self::Quarterly),
            "yearly" => Ok(Self::Yearly),
            "continuous_monthly" => Ok(Self::ContinuousMonthly),
            "continuous_yearly" => Ok(Self::ContinuousYearly),
            _ => Err(CommerceServiceError::validation(
                "unsupported Token Bank plan period",
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Monthly => "monthly",
            Self::Quarterly => "quarterly",
            Self::Yearly => "yearly",
            Self::ContinuousMonthly => "continuous_monthly",
            Self::ContinuousYearly => "continuous_yearly",
        }
    }

    pub fn is_continuous(self) -> bool {
        matches!(self, Self::ContinuousMonthly | Self::ContinuousYearly)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountValueFulfillmentContext {
    pub order_id: String,
    pub order_no: String,
    pub subject: AccountValueOrderSubject,
    pub target_asset: AccountValueAssetCode,
    pub order_status: String,
    pub fulfillment_status: String,
    pub payment_status: String,
    pub payment_attempt_status: String,
    pub grant_amount: CommerceMoney,
    pub asset_unit_code: String,
    pub coupon_code: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FulfillAccountValueOrderOutcome {
    pub accepted: bool,
    pub replayed: bool,
    pub order_id: String,
    pub order_no: String,
    pub target_asset: AccountValueAssetCode,
    pub amount: CommerceMoney,
    pub fulfillment_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountValuePackageItem {
    pub package_id: String,
    pub package_code: String,
    pub display_name: String,
    pub target_asset: AccountValueAssetCode,
    pub grant_amount: CommerceMoney,
    pub bonus_amount: CommerceMoney,
    pub price_amount: CommerceMoney,
    pub currency_code: String,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenBankPlanItem {
    pub plan_code: String,
    pub display_name: String,
    pub plan_period: TokenBankPlanPeriod,
    pub grant_amount: CommerceMoney,
    pub bonus_amount: CommerceMoney,
    pub price_amount: CommerceMoney,
    pub currency_code: String,
    pub renewal_policy: String,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountValueRequestView {
    pub request_id: String,
    pub request_no: String,
    pub original_order_id: Option<String>,
    pub owner_user_id: String,
    pub subject: AccountValueOrderSubject,
    pub target_asset: AccountValueAssetCode,
    /// Account-side asset amount affected by refund reversal or withdrawal hold.
    pub amount: CommerceMoney,
    /// Account-side unit code, such as TOKEN_BANK, POINT, or CNY for cash.
    pub currency_code: String,
    /// Provider-side money amount for refund or payout execution.
    pub provider_amount: Option<CommerceMoney>,
    /// Provider-side currency code for refund or payout execution.
    pub provider_currency_code: Option<String>,
    pub status: String,
    pub provider_reference_id: Option<String>,
    pub account_effect_reference_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateAccountRechargeOrderOutcome {
    pub success: bool,
    pub order_id: String,
    pub order_no: String,
    pub out_trade_no: String,
    pub subject: AccountValueOrderSubject,
    pub target_asset: AccountValueAssetCode,
    pub amount: CommerceMoney,
    pub grant_amount: CommerceMoney,
    pub currency_code: String,
    pub provider_code: String,
    pub payment_method: String,
    pub payment_product: String,
    pub status: String,
    pub next_action: String,
    pub cashier_url: String,
    pub qr_code_payload: String,
    pub request_payment_payload: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountValuePackageListPage {
    pub items: Vec<AccountValuePackageItem>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenBankPlanListPage {
    pub items: Vec<TokenBankPlanItem>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountValueRequestListPage {
    pub items: Vec<AccountValueRequestView>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

impl AccountValuePackageItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        package_id: &str,
        package_code: &str,
        display_name: &str,
        target_asset: AccountValueAssetCode,
        grant_amount: CommerceMoney,
        bonus_amount: CommerceMoney,
        price_amount: CommerceMoney,
        currency_code: &str,
        status: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("package_id", package_id)?;
        crate::validation::require_non_empty("package_code", package_code)?;
        crate::validation::require_non_empty("display_name", display_name)?;
        crate::validation::require_non_empty("currency_code", currency_code)?;
        crate::validation::require_non_empty("status", status)?;

        Ok(Self {
            package_id: package_id.trim().to_string(),
            package_code: package_code.trim().to_string(),
            display_name: display_name.trim().to_string(),
            target_asset,
            grant_amount,
            bonus_amount,
            price_amount,
            currency_code: currency_code.trim().to_ascii_uppercase(),
            status: status.trim().to_ascii_lowercase(),
        })
    }
}

impl TokenBankPlanItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        plan_code: &str,
        display_name: &str,
        plan_period: TokenBankPlanPeriod,
        grant_amount: CommerceMoney,
        bonus_amount: CommerceMoney,
        price_amount: CommerceMoney,
        currency_code: &str,
        renewal_policy: &str,
        status: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("plan_code", plan_code)?;
        crate::validation::require_non_empty("display_name", display_name)?;
        crate::validation::require_non_empty("currency_code", currency_code)?;
        crate::validation::require_non_empty("renewal_policy", renewal_policy)?;
        crate::validation::require_non_empty("status", status)?;

        Ok(Self {
            plan_code: plan_code.trim().to_string(),
            display_name: display_name.trim().to_string(),
            plan_period,
            grant_amount,
            bonus_amount,
            price_amount,
            currency_code: currency_code.trim().to_ascii_uppercase(),
            renewal_policy: renewal_policy.trim().to_ascii_lowercase(),
            status: status.trim().to_ascii_lowercase(),
        })
    }
}

impl AccountValueRequestView {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        request_id: &str,
        request_no: &str,
        original_order_id: Option<&str>,
        owner_user_id: &str,
        subject: AccountValueOrderSubject,
        target_asset: AccountValueAssetCode,
        amount: CommerceMoney,
        currency_code: &str,
        status: &str,
        provider_reference_id: Option<&str>,
        created_at: &str,
        updated_at: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("request_id", request_id)?;
        crate::validation::require_non_empty("request_no", request_no)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("currency_code", currency_code)?;
        crate::validation::require_non_empty("status", status)?;
        crate::validation::require_non_empty("created_at", created_at)?;
        crate::validation::require_non_empty("updated_at", updated_at)?;
        subject.validate_target_asset(target_asset)?;

        Ok(Self {
            request_id: request_id.trim().to_string(),
            request_no: request_no.trim().to_string(),
            original_order_id: optional_text(original_order_id),
            owner_user_id: owner_user_id.trim().to_string(),
            subject,
            target_asset,
            amount,
            currency_code: currency_code.trim().to_ascii_uppercase(),
            provider_amount: None,
            provider_currency_code: None,
            status: status.trim().to_ascii_lowercase(),
            provider_reference_id: optional_text(provider_reference_id),
            account_effect_reference_id: None,
            created_at: created_at.trim().to_string(),
            updated_at: updated_at.trim().to_string(),
        })
    }

    pub fn with_provider_execution_amount(
        mut self,
        amount: CommerceMoney,
        currency_code: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("provider_currency_code", currency_code)?;
        self.provider_amount = Some(amount);
        self.provider_currency_code = Some(currency_code.trim().to_ascii_uppercase());
        Ok(self)
    }

    pub fn with_account_effect_reference_id(mut self, reference_id: Option<&str>) -> Self {
        self.account_effect_reference_id = optional_text(reference_id);
        self
    }

    pub fn provider_execution_amount(&self) -> CommerceMoney {
        self.provider_amount
            .clone()
            .unwrap_or_else(|| self.amount.clone())
    }

    pub fn provider_execution_currency_code(&self) -> &str {
        self.provider_currency_code
            .as_deref()
            .unwrap_or(self.currency_code.as_str())
    }
}

impl AccountValueFulfillmentContext {
    pub fn payment_is_succeeded(&self) -> bool {
        payment_status_is_succeeded(&self.payment_attempt_status)
            || payment_status_is_succeeded(&self.payment_status)
    }

    pub fn already_fulfilled(&self) -> bool {
        self.fulfillment_status.eq_ignore_ascii_case("fulfilled")
            || self.order_status.eq_ignore_ascii_case("fulfilled")
            || self.order_status.eq_ignore_ascii_case("completed")
    }

    pub fn fulfillment_in_progress(&self) -> bool {
        self.fulfillment_status.eq_ignore_ascii_case("processing")
    }

    pub fn validate_for_fulfillment(&self) -> Result<(), CommerceServiceError> {
        if self.already_fulfilled() {
            return Ok(());
        }
        if !self.subject.is_account_value_fulfillment_subject() {
            return Err(CommerceServiceError::validation(
                "order subject does not support account value fulfillment",
            ));
        }
        self.subject.validate_target_asset(self.target_asset)?;
        if !self.payment_is_succeeded() && self.subject.requires_payment_collection() {
            return Err(CommerceServiceError::conflict(
                "account value order payment is not succeeded",
            ));
        }
        if self.grant_amount.as_str() == "0" {
            return Err(CommerceServiceError::validation(
                "account value fulfillment requires positive grant amount",
            ));
        }
        Ok(())
    }
}

impl FulfillAccountValueOrderOutcome {
    pub fn replayed(context: &AccountValueFulfillmentContext) -> Self {
        Self {
            accepted: true,
            replayed: true,
            order_id: context.order_id.clone(),
            order_no: context.order_no.clone(),
            target_asset: context.target_asset,
            amount: context.grant_amount.clone(),
            fulfillment_status: "fulfilled".to_owned(),
        }
    }

    pub fn fulfilled(context: &AccountValueFulfillmentContext) -> Self {
        Self {
            accepted: true,
            replayed: false,
            order_id: context.order_id.clone(),
            order_no: context.order_no.clone(),
            target_asset: context.target_asset,
            amount: context.grant_amount.clone(),
            fulfillment_status: "fulfilled".to_owned(),
        }
    }
}

fn payment_status_is_succeeded(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "succeeded" | "success" | "paid"
    )
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

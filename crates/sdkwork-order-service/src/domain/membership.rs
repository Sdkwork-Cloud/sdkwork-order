use sdkwork_contract_service::{CommerceMoney, CommerceServiceError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateMembershipOrderOutcome {
    pub order_id: String,
    pub order_no: String,
    pub out_trade_no: String,
    pub amount: CommerceMoney,
    pub currency_code: String,
    pub package_id: String,
    pub package_name: String,
    pub duration_days: i64,
    pub payment_method: String,
    pub status: String,
    pub cashier_url: String,
}

impl CreateMembershipOrderOutcome {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        order_id: &str,
        order_no: &str,
        out_trade_no: &str,
        amount: CommerceMoney,
        currency_code: &str,
        package_id: &str,
        package_name: &str,
        duration_days: i64,
        payment_method: &str,
        status: &str,
        cashier_url: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("order_id", order_id)?;
        crate::validation::require_non_empty("order_no", order_no)?;
        crate::validation::require_non_empty("out_trade_no", out_trade_no)?;
        crate::validation::require_non_empty("currency_code", currency_code)?;
        crate::validation::require_non_empty("package_id", package_id)?;
        crate::validation::require_non_empty("package_name", package_name)?;
        crate::validation::require_non_empty("payment_method", payment_method)?;
        crate::validation::require_non_empty("status", status)?;
        crate::validation::require_non_empty("cashier_url", cashier_url)?;
        if duration_days <= 0 {
            return Err(CommerceServiceError::validation(
                "membership package duration must be greater than zero",
            ));
        }

        Ok(Self {
            order_id: order_id.trim().to_string(),
            order_no: order_no.trim().to_string(),
            out_trade_no: out_trade_no.trim().to_string(),
            amount,
            currency_code: currency_code.trim().to_ascii_uppercase(),
            package_id: package_id.trim().to_string(),
            package_name: package_name.trim().to_string(),
            duration_days,
            payment_method: payment_method.trim().to_ascii_lowercase(),
            status: status.trim().to_string(),
            cashier_url: cashier_url.trim().to_string(),
        })
    }
}

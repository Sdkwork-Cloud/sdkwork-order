use sdkwork_contract_service::{CommerceMoney, CommerceServiceError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PointsRechargeFulfillmentContext {
    pub order_id: String,
    pub order_no: String,
    pub order_status: String,
    pub fulfillment_status: String,
    pub payment_status: String,
    pub payment_attempt_status: String,
    pub points: i64,
    pub amount: CommerceMoney,
    pub currency_code: String,
    pub billing_history_status: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FulfillPointsRechargeOrderOutcome {
    pub accepted: bool,
    pub replayed: bool,
    pub order_id: String,
    pub order_no: String,
    pub points_credited: i64,
    pub fulfillment_status: String,
}

impl PointsRechargeFulfillmentContext {
    pub fn payment_is_succeeded(&self) -> bool {
        payment_attempt_is_succeeded(&self.payment_attempt_status)
            || payment_intent_is_succeeded(&self.payment_status)
    }

    pub fn already_fulfilled(&self) -> bool {
        self.fulfillment_status.eq_ignore_ascii_case("fulfilled")
            || self.order_status.eq_ignore_ascii_case("fulfilled")
            || self.order_status.eq_ignore_ascii_case("completed")
    }

    pub fn validate_for_fulfillment(&self) -> Result<(), CommerceServiceError> {
        if self.already_fulfilled() {
            return Ok(());
        }
        if !self.payment_is_succeeded() {
            return Err(CommerceServiceError::conflict(
                "points recharge order payment is not succeeded",
            ));
        }
        if self.points <= 0 {
            return Err(CommerceServiceError::validation(
                "points recharge fulfillment requires positive points",
            ));
        }
        Ok(())
    }
}

impl FulfillPointsRechargeOrderOutcome {
    pub fn replayed(
        order_id: &str,
        order_no: &str,
        points_credited: i64,
    ) -> Self {
        Self {
            accepted: true,
            replayed: true,
            order_id: order_id.to_owned(),
            order_no: order_no.to_owned(),
            points_credited,
            fulfillment_status: "fulfilled".to_owned(),
        }
    }

    pub fn fulfilled(
        order_id: &str,
        order_no: &str,
        points_credited: i64,
    ) -> Self {
        Self {
            accepted: true,
            replayed: false,
            order_id: order_id.to_owned(),
            order_no: order_no.to_owned(),
            points_credited,
            fulfillment_status: "fulfilled".to_owned(),
        }
    }
}

fn payment_intent_is_succeeded(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "succeeded" | "success" | "paid"
    )
}

fn payment_attempt_is_succeeded(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "succeeded" | "success" | "paid"
    )
}

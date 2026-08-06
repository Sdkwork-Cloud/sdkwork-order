use sdkwork_contract_service::CommerceServiceError;
use sdkwork_utils_rust::parse_datetime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateMembershipOrderCommand {
    pub action: String,
    pub client_request_no: Option<String>,
    pub expire_at: String,
    pub idempotency_key: String,
    pub method: String,
    pub order_id: String,
    pub order_item_id: String,
    pub order_no: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub package_id: String,
    pub payment_product: String,
    pub requested_at: String,
    pub source: Option<String>,
    pub tenant_id: String,
    pub out_trade_no: String,
    /// 订阅期额度充值数量（仅 action=recharge 使用）。
    pub grant_quantity: Option<i64>,
    /// 订阅期额度充值金额（仅 action=recharge 使用，货币金额字符串）。
    pub amount: Option<String>,
}

impl CreateMembershipOrderCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        package_id: &str,
        action: &str,
        method: &str,
        payment_product: &str,
        order_id: &str,
        order_item_id: &str,
        order_no: &str,
        out_trade_no: &str,
        requested_at: &str,
        expire_at: &str,
        idempotency_key: &str,
        client_request_no: Option<&str>,
        source: Option<&str>,
        grant_quantity: Option<i64>,
        amount: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("package_id", package_id)?;
        crate::validation::require_non_empty("action", action)?;
        crate::validation::require_non_empty("method", method)?;
        crate::validation::require_non_empty("payment_product", payment_product)?;
        crate::validation::require_non_empty("order_id", order_id)?;
        crate::validation::require_non_empty("order_item_id", order_item_id)?;
        crate::validation::require_non_empty("order_no", order_no)?;
        crate::validation::require_non_empty("out_trade_no", out_trade_no)?;
        crate::validation::require_non_empty("requested_at", requested_at)?;
        crate::validation::require_non_empty("expire_at", expire_at)?;
        crate::validation::require_non_empty("idempotency_key", idempotency_key)?;

        let action = action.trim().to_ascii_lowercase();
        if !matches!(
            action.as_str(),
            "purchase" | "renew" | "upgrade" | "recharge"
        ) {
            return Err(CommerceServiceError::validation(
                "membership action must be one of: purchase, renew, upgrade, recharge",
            ));
        }
        if action == "recharge" {
            // 订阅期额度充值：数量与金额必填且为正
            if !matches!(grant_quantity, Some(quantity) if quantity > 0) {
                return Err(CommerceServiceError::validation(
                    "membership quota recharge requires a positive grantQuantity",
                ));
            }
            let amount = amount.unwrap_or_default().trim();
            if amount.is_empty() || !is_positive_money_amount(amount) {
                return Err(CommerceServiceError::validation(
                    "membership quota recharge requires a positive amount",
                ));
            }
        } else if grant_quantity.is_some() || amount.is_some() {
            return Err(CommerceServiceError::validation(
                "grantQuantity and amount are only valid for membership quota recharge",
            ));
        }
        let requested_at_value = parse_datetime(requested_at.trim(), None).ok_or_else(|| {
            CommerceServiceError::validation("requested_at must be an RFC3339 timestamp")
        })?;
        let expire_at_value = parse_datetime(expire_at.trim(), None).ok_or_else(|| {
            CommerceServiceError::validation("expire_at must be an RFC3339 timestamp")
        })?;
        if expire_at_value <= requested_at_value {
            return Err(CommerceServiceError::validation(
                "expire_at must be later than requested_at",
            ));
        }

        Ok(Self {
            action,
            client_request_no: optional_text(client_request_no),
            expire_at: expire_at.trim().to_string(),
            idempotency_key: idempotency_key.trim().to_string(),
            method: method.trim().to_ascii_lowercase(),
            order_id: order_id.trim().to_string(),
            order_item_id: order_item_id.trim().to_string(),
            order_no: order_no.trim().to_string(),
            organization_id: organization_id
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            owner_user_id: owner_user_id.trim().to_string(),
            package_id: package_id.trim().to_string(),
            payment_product: payment_product.trim().to_ascii_lowercase(),
            requested_at: requested_at.trim().to_string(),
            source: optional_text(source),
            tenant_id: tenant_id.trim().to_string(),
            out_trade_no: out_trade_no.trim().to_string(),
            grant_quantity,
            amount: amount
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
        })
    }
}

/// 校验货币金额字符串为正数（支持两位小数的十进制）。
fn is_positive_money_amount(value: &str) -> bool {
    let value = value.trim();
    let mut dot_count = 0;
    for (index, character) in value.chars().enumerate() {
        match character {
            '0'..='9' => {}
            '.' if dot_count == 0 && index > 0 && index + 1 < value.len() => dot_count += 1,
            _ => return false,
        }
    }
    value
        .parse::<f64>()
        .map(|parsed| parsed > 0.0 && parsed < 1_000_000_000.0)
        .unwrap_or(false)
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::CreateMembershipOrderCommand;

    fn command(
        action: &str,
        requested_at: &str,
        expire_at: &str,
    ) -> Result<CreateMembershipOrderCommand, sdkwork_contract_service::CommerceServiceError> {
        CreateMembershipOrderCommand::new(
            "tenant-1",
            Some("0"),
            "user-1",
            "package-1",
            action,
            "wechat_pay",
            "wechat_native",
            "order-1",
            "item-1",
            "MB1",
            "MEMBERSHIP1",
            requested_at,
            expire_at,
            "idem-1",
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn membership_time_window_compares_actual_rfc3339_instants() {
        let value = command(
            " PURCHASE ",
            "2026-07-26T08:00:00+08:00",
            "2026-07-26T00:30:00Z",
        )
        .expect("valid membership command");

        assert_eq!(value.action, "purchase");
    }

    #[test]
    fn membership_time_window_rejects_invalid_or_non_increasing_values() {
        assert!(command("purchase", "not-a-time", "2026-07-26T00:30:00Z").is_err());
        assert!(command("purchase", "2026-07-26T00:00:00Z", "invalid").is_err());
        assert!(command(
            "purchase",
            "2026-07-26T08:00:00+08:00",
            "2026-07-26T00:00:00Z"
        )
        .is_err());
    }

    #[test]
    fn membership_command_rejects_unknown_actions() {
        assert!(command("downgrade", "2026-07-26T00:00:00Z", "2026-07-26T00:30:00Z").is_err());
    }

    #[test]
    fn membership_command_accepts_recharge_with_quantity_and_amount() {
        let value = CreateMembershipOrderCommand::new(
            "tenant-1",
            Some("0"),
            "user-1",
            "membership-quota-recharge",
            "recharge",
            "wechat_pay",
            "wechat_native",
            "order-1",
            "item-1",
            "MB1",
            "MEMBERSHIP1",
            "2026-07-26T00:00:00Z",
            "2026-07-26T00:30:00Z",
            "idem-1",
            None,
            None,
            Some(1000),
            Some("10.00"),
        )
        .expect("valid recharge command");
        assert_eq!(value.action, "recharge");
        assert_eq!(value.grant_quantity, Some(1000));
        assert_eq!(value.amount.as_deref(), Some("10.00"));
    }

    #[test]
    fn membership_command_rejects_recharge_without_quantity_or_amount() {
        assert!(CreateMembershipOrderCommand::new(
            "tenant-1",
            Some("0"),
            "user-1",
            "membership-quota-recharge",
            "recharge",
            "wechat_pay",
            "wechat_native",
            "order-1",
            "item-1",
            "MB1",
            "MEMBERSHIP1",
            "2026-07-26T00:00:00Z",
            "2026-07-26T00:30:00Z",
            "idem-1",
            None,
            None,
            None,
            Some("10.00"),
        )
        .is_err());
        assert!(CreateMembershipOrderCommand::new(
            "tenant-1",
            Some("0"),
            "user-1",
            "membership-quota-recharge",
            "recharge",
            "wechat_pay",
            "wechat_native",
            "order-1",
            "item-1",
            "MB1",
            "MEMBERSHIP1",
            "2026-07-26T00:00:00Z",
            "2026-07-26T00:30:00Z",
            "idem-1",
            None,
            None,
            Some(1000),
            None,
        )
        .is_err());
    }

    #[test]
    fn membership_command_rejects_non_recharge_with_quantity_or_amount() {
        assert!(command("purchase", "2026-07-26T00:00:00Z", "2026-07-26T00:30:00Z")
            .unwrap()
            .grant_quantity
            .is_none());
        assert!(CreateMembershipOrderCommand::new(
            "tenant-1",
            Some("0"),
            "user-1",
            "package-1",
            "purchase",
            "wechat_pay",
            "wechat_native",
            "order-1",
            "item-1",
            "MB1",
            "MEMBERSHIP1",
            "2026-07-26T00:00:00Z",
            "2026-07-26T00:30:00Z",
            "idem-1",
            None,
            None,
            Some(100),
            Some("10.00"),
        )
        .is_err());
    }
}

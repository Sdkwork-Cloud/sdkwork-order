use crate::validation::write_command_hash::stable_command_request_hash;
use crate::{CreateCheckoutQuoteCommand, CreateCheckoutSessionCommand, CreateOwnerOrderCommand};

pub fn checkout_session_request_hash(command: &CreateCheckoutSessionCommand) -> String {
    let lines = command
        .lines
        .iter()
        .map(|line| format!("{}:{}", line.sku_id, line.quantity))
        .collect::<Vec<_>>()
        .join(",");
    let physical_fingerprint = command
        .physical_checkout
        .as_ref()
        .map(|checkout| {
            let address = checkout
                .shipping_address
                .snapshot_json()
                .unwrap_or_default();
            let resolved_lines = checkout
                .lines
                .iter()
                .map(|line| {
                    format!(
                        "{}:{}:{}:{}:{}:{}",
                        line.sku_id,
                        line.quantity,
                        line.product_id,
                        line.shop_id,
                        line.unit_price.as_str(),
                        line.currency_code
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{}|{}|{}|{}",
                checkout.merchant_organization_id, checkout.shop_id, address, resolved_lines
            )
        })
        .unwrap_or_else(|| "legacy".to_owned());
    stable_command_request_hash(
        "checkout.sessions.create",
        &[
            &command.tenant_id,
            command.organization_id.as_deref().unwrap_or("global"),
            &command.owner_user_id,
            &command.currency_code,
            &lines,
            &command.request_no,
            &physical_fingerprint,
        ],
    )
}

pub fn checkout_quote_request_hash(command: &CreateCheckoutQuoteCommand) -> String {
    stable_command_request_hash(
        "checkout.sessions.quotes.create",
        &[
            &command.tenant_id,
            command.organization_id.as_deref().unwrap_or("global"),
            &command.owner_user_id,
            &command.checkout_session_id,
            &command.request_no,
        ],
    )
}

pub fn checkout_owner_order_request_hash(command: &CreateOwnerOrderCommand) -> String {
    stable_command_request_hash(
        "checkout.sessions.orders.create",
        &[
            &command.tenant_id,
            command.organization_id.as_deref().unwrap_or("global"),
            &command.owner_user_id,
            &command.checkout_session_id,
            &command.request_no,
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CheckoutLineInput;

    #[test]
    fn checkout_session_request_hash_is_stable_for_same_command() {
        let lines = vec![CheckoutLineInput::new("sku-1", 1).expect("line")];
        let command = CreateCheckoutSessionCommand::new(
            "100001",
            Some("0"),
            "user-1",
            "CNY",
            lines,
            "request-1",
            "idem-1",
        )
        .expect("command");

        let first = checkout_session_request_hash(&command);
        let second = checkout_session_request_hash(&command);
        assert_eq!(first, second);
        assert!(!first.is_empty());
    }
}

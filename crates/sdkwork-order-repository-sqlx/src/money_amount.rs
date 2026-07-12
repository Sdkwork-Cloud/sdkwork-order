use sdkwork_contract_service::{CommerceMoney, CommerceServiceError};

pub(crate) fn normalize_money_amount(amount: &str) -> Result<String, CommerceServiceError> {
    parse_minor_units(amount).map(|minor_units| minor_units.to_string())
}

pub(crate) fn commerce_money(amount: &str) -> Result<CommerceMoney, CommerceServiceError> {
    let normalized = normalize_money_amount(amount)?;
    CommerceMoney::new(&normalized).map_err(CommerceServiceError::storage)
}

pub(crate) fn sum_money_amounts<'a>(
    mut amounts: impl Iterator<Item = &'a str>,
) -> Result<String, CommerceServiceError> {
    amounts
        .try_fold(0_i64, |total, amount| {
            let amount = parse_minor_units(amount)?;
            total.checked_add(amount).ok_or_else(|| {
                CommerceServiceError::validation("checkout total amount is too large")
            })
        })
        .map(|total| total.to_string())
}

pub(crate) fn multiply_money_amount(
    amount: &str,
    quantity: i64,
) -> Result<String, CommerceServiceError> {
    if quantity <= 0 {
        return Err(CommerceServiceError::validation(
            "checkout line quantity must be greater than zero",
        ));
    }

    parse_minor_units(amount)?
        .checked_mul(quantity)
        .map(|total| total.to_string())
        .ok_or_else(|| CommerceServiceError::validation("checkout line amount is too large"))
}

fn parse_minor_units(amount: &str) -> Result<i64, CommerceServiceError> {
    let money = CommerceMoney::new(amount).map_err(|error| {
        CommerceServiceError::storage(format!("invalid minor-unit money amount: {error}"))
    })?;
    money.as_str().parse::<i64>().map_err(|_| {
        CommerceServiceError::validation("money amount exceeds the supported integer range")
    })
}

#[cfg(test)]
mod tests {
    use super::{multiply_money_amount, normalize_money_amount, sum_money_amounts};

    #[test]
    fn computes_minor_unit_totals_without_major_unit_conversion() {
        assert_eq!(multiply_money_amount("6990", 2).unwrap(), "13980");
        assert_eq!(
            sum_money_amounts(["6990", "6990"].into_iter()).unwrap(),
            "13980"
        );
        assert_eq!(normalize_money_amount("0").unwrap(), "0");
    }

    #[test]
    fn rejects_decimal_and_overflowing_minor_unit_amounts() {
        let maximum = i64::MAX.to_string();

        assert!(multiply_money_amount("69.90", 2).is_err());
        assert!(multiply_money_amount(&maximum, 2).is_err());
        assert!(sum_money_amounts([maximum.as_str(), "1"].into_iter()).is_err());
    }
}

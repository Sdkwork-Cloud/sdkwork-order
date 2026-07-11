#[test]
fn recharge_create_does_not_inline_payment_or_billing_helpers() {
    let sqlite = include_str!("../src/sqlite_recharge.rs");
    let postgres = include_str!("../src/postgres_recharge.rs");
    for source in [sqlite, postgres] {
        assert!(
            !source.contains("fn insert_payment"),
            "recharge repository must not define inline payment insert helpers"
        );
        assert!(
            !source.contains("fn insert_recharge_billing_history"),
            "recharge repository must not define billing history insert helpers"
        );
    }
}

#[test]
fn recharge_payment_queries_use_standard_payment_method_columns() {
    let sqlite = include_str!("../src/sqlite_recharge.rs");
    let postgres = include_str!("../src/postgres_recharge.rs");

    for (label, source) in [("sqlite", sqlite), ("postgres", postgres)] {
        assert!(
            source.contains("SELECT method_key, provider_code\nFROM commerce_payment_method"),
            "{label} recharge repository must read provider_code from the standard payment method table"
        );
        assert!(
            source.contains("COALESCE(sort_order, 0) ASC"),
            "{label} recharge repository must order payment methods with standard sort_order"
        );
        for legacy_fragment in [
            "provider AS provider_code",
            "NULLIF(pa.provider,",
            "NULLIF(pi.provider,",
        ] {
            assert!(
                !source.contains(legacy_fragment),
                "{label} recharge repository must not read legacy payment provider fragment {legacy_fragment}",
            );
        }
    }
}

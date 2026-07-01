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

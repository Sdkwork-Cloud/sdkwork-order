#[test]
fn membership_create_does_not_inline_payment_or_billing_helpers() {
    let sqlite = include_str!("../src/sqlite_membership_order.rs");
    let postgres = include_str!("../src/postgres_membership_order.rs");
    for source in [sqlite, postgres] {
        assert!(
            !source.contains("fn insert_payment"),
            "membership repository must not define inline payment insert helpers"
        );
        assert!(
            !source.contains("commerce_payment_intent"),
            "membership repository must not insert payment intents at create time"
        );
    }
}

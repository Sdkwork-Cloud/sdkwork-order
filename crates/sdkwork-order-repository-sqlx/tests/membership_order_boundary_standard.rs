fn normalized(source: &str) -> String {
    source.replace("\r\n", "\n")
}

#[test]
fn membership_create_does_not_inline_payment_or_billing_helpers() {
    let postgres = normalized(include_str!("../src/postgres_membership_order.rs"));
    assert!(
        !postgres.contains("fn insert_payment"),
        "membership repository must not define inline payment insert helpers"
    );
    assert!(
        !postgres.contains("commerce_payment_intent"),
        "membership repository must not insert payment intents at create time"
    );
}

#[test]
fn membership_repository_has_no_server_side_sqlite() {
    let lib = normalized(include_str!("../src/lib.rs"));
    assert!(
        !lib.contains("SqliteCommerceMembershipOrderStore"),
        "server repository must not expose sqlite membership stores"
    );
    assert!(
        !lib.contains("sqlite_"),
        "server repository must not declare sqlite modules"
    );
}

use sdkwork_order_repository_sqlx::{
    order_points_recharge_e2e_sqlite_memory_pool, SqliteCommerceMembershipOrderStore,
};
use sdkwork_order_service::CreateMembershipOrderCommand;

const TENANT_ID: &str = "100001";
const ORGANIZATION_ID: &str = "0";
const OWNER_USER_ID: &str = "1";
const PACKAGE_EXTERNAL_ID: &str = "201";

fn membership_command(
    suffix: &str,
    action: &str,
    requested_at: &str,
    expire_at: &str,
    idempotency_key: &str,
) -> CreateMembershipOrderCommand {
    CreateMembershipOrderCommand::new(
        TENANT_ID,
        Some(ORGANIZATION_ID),
        OWNER_USER_ID,
        PACKAGE_EXTERNAL_ID,
        action,
        "wechat_pay",
        "wechat_native",
        &format!("membership-order-{suffix}"),
        &format!("membership-item-{suffix}"),
        &format!("MB{suffix}"),
        &format!("MEMBERSHIP{suffix}"),
        requested_at,
        expire_at,
        idempotency_key,
        None,
        Some("token-plan"),
    )
    .expect("membership create command")
}

#[tokio::test]
async fn sqlite_membership_order_create_persists_order_without_payment_intent() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    let store = SqliteCommerceMembershipOrderStore::new(pool.clone());
    let command = CreateMembershipOrderCommand::new(
        TENANT_ID,
        Some(ORGANIZATION_ID),
        OWNER_USER_ID,
        PACKAGE_EXTERNAL_ID,
        "purchase",
        "wechat_pay",
        "wechat_native",
        "550e8400-e29b-41d4-a716-446655440000",
        "550e8400-e29b-41d4-a716-446655440001",
        "MB0000000000000001",
        "MEMBERSHIP0000000000000001",
        "2026-07-07T00:00:00Z",
        "2026-07-07T00:30:00Z",
        "membership-create-idem-1",
        None,
        None,
    )
    .expect("membership create command");

    let outcome = store
        .create_membership_order(command.clone())
        .await
        .expect("create membership order");

    assert_eq!(outcome.order_id, command.order_id);
    assert_eq!(outcome.package_id, PACKAGE_EXTERNAL_ID);
    assert_eq!(outcome.amount.as_str(), "6800");
    assert_eq!(outcome.currency_code, "CNY");
    assert_eq!(outcome.duration_days, 30);
    assert_eq!(outcome.payment_method, "wechat_pay");
    assert_eq!(outcome.status, "pending_payment");
    assert!(!outcome.reused);
    assert!(outcome.cashier_url.contains("scene=virtual"));

    let payment_intent_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM commerce_payment_intent WHERE order_id = ?")
            .bind(&command.order_id)
            .fetch_one(&pool)
            .await
            .expect("count payment intents");
    assert_eq!(payment_intent_count, 0);

    let replay = store
        .create_membership_order(command)
        .await
        .expect("replay membership order");
    assert_eq!(replay.order_id, outcome.order_id);
    assert_eq!(replay.order_no, outcome.order_no);
    assert!(replay.reused);
}

#[tokio::test]
async fn sqlite_membership_order_uses_the_platform_catalog_for_another_subject_scope() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    let store = SqliteCommerceMembershipOrderStore::new(pool);
    let command = CreateMembershipOrderCommand::new(
        "200002",
        Some("42"),
        "7",
        PACKAGE_EXTERNAL_ID,
        "purchase",
        "wechat_pay",
        "wechat_native",
        "550e8400-e29b-41d4-a716-446655440010",
        "550e8400-e29b-41d4-a716-446655440011",
        "MB0000000000000010",
        "MEMBERSHIP0000000000000010",
        "2026-07-07T01:00:00Z",
        "2026-07-07T01:30:00Z",
        "membership-create-idem-platform-catalog",
        None,
        None,
    )
    .expect("membership create command");

    let outcome = store
        .create_membership_order(command)
        .await
        .expect("create membership order from platform catalog");

    assert_eq!(outcome.package_id, PACKAGE_EXTERNAL_ID);
    assert_eq!(outcome.amount.as_str(), "6800");
    assert_eq!(outcome.payment_method, "wechat_pay");
}

#[tokio::test]
async fn sqlite_membership_order_rejects_idempotency_key_payload_mismatch() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    let store = SqliteCommerceMembershipOrderStore::new(pool);
    let command = membership_command(
        "idem-a",
        "purchase",
        "2026-07-07T02:00:00Z",
        "2026-07-07T02:30:00Z",
        "membership-idem-mismatch",
    );
    store
        .create_membership_order(command.clone())
        .await
        .expect("create membership order");

    let mut mismatched = command;
    mismatched.source = Some("different-source".to_owned());
    let error = store
        .create_membership_order(mismatched)
        .await
        .expect_err("idempotency payload mismatch must fail");

    assert_eq!(error.code(), "conflict");
}

#[tokio::test]
async fn sqlite_membership_order_replay_rejects_a_missing_expiration_boundary() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    let store = SqliteCommerceMembershipOrderStore::new(pool.clone());
    let command = membership_command(
        "missing-expiry",
        "purchase",
        "2026-07-07T02:30:00Z",
        "2026-07-07T03:00:00Z",
        "membership-missing-expiry",
    );
    store
        .create_membership_order(command.clone())
        .await
        .expect("create membership order");

    sqlx::query("UPDATE commerce_order SET expired_at = NULL WHERE id = ?")
        .bind(&command.order_id)
        .execute(&pool)
        .await
        .expect("remove expiration boundary");

    store
        .create_membership_order(command)
        .await
        .expect_err("missing expiration boundary must fail closed");
}

#[tokio::test]
async fn sqlite_membership_order_reuses_purchase_intent_across_client_keys() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    let store = SqliteCommerceMembershipOrderStore::new(pool.clone());
    let first = membership_command(
        "reuse-a",
        "purchase",
        "2026-07-07T03:00:00Z",
        "2026-07-07T03:30:00Z",
        "membership-reuse-a",
    );
    let second = membership_command(
        "reuse-b",
        "purchase",
        "2026-07-07T03:01:00Z",
        "2026-07-07T03:31:00Z",
        "membership-reuse-b",
    );

    let created = store
        .create_membership_order(first)
        .await
        .expect("create membership order");
    let reused = store
        .create_membership_order(second)
        .await
        .expect("reuse membership order");

    assert_eq!(reused.order_id, created.order_id);
    assert!(reused.reused);
    let active_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM commerce_order WHERE subject = 'membership' AND status = 'pending_payment'",
    )
    .fetch_one(&pool)
    .await
    .expect("count active membership orders");
    assert_eq!(active_count, 1);
}

#[tokio::test]
async fn sqlite_membership_order_separates_actions_and_replaces_expired_intents() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    let store = SqliteCommerceMembershipOrderStore::new(pool.clone());
    let purchase = store
        .create_membership_order(membership_command(
            "expiry-a",
            "purchase",
            "2026-07-07T04:00:00Z",
            "2026-07-07T04:30:00Z",
            "membership-expiry-a",
        ))
        .await
        .expect("create purchase order");
    let renewal = store
        .create_membership_order(membership_command(
            "renew-a",
            "renew",
            "2026-07-07T04:01:00Z",
            "2026-07-07T04:31:00Z",
            "membership-renew-a",
        ))
        .await
        .expect("create renewal order");
    assert_ne!(renewal.order_id, purchase.order_id);

    let replacement = store
        .create_membership_order(membership_command(
            "expiry-b",
            "purchase",
            "2026-07-07T05:00:00Z",
            "2026-07-07T05:30:00Z",
            "membership-expiry-b",
        ))
        .await
        .expect("replace expired purchase order");
    assert_ne!(replacement.order_id, purchase.order_id);
    let old_status: String = sqlx::query_scalar("SELECT status FROM commerce_order WHERE id = ?")
        .bind(&purchase.order_id)
        .fetch_one(&pool)
        .await
        .expect("load expired order status");
    assert_eq!(old_status, "expired");
}

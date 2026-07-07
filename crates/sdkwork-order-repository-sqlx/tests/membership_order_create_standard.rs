use sdkwork_order_repository_sqlx::{
    order_points_recharge_e2e_sqlite_memory_pool, SqliteCommerceMembershipOrderStore,
};
use sdkwork_order_service::CreateMembershipOrderCommand;

const TENANT_ID: &str = "100001";
const ORGANIZATION_ID: &str = "0";
const OWNER_USER_ID: &str = "1";
const PACKAGE_EXTERNAL_ID: &str = "201";

#[tokio::test]
async fn sqlite_membership_order_create_persists_order_without_payment_intent() {
    let pool = order_points_recharge_e2e_sqlite_memory_pool().await;
    let store = SqliteCommerceMembershipOrderStore::new(pool.clone());
    let command = CreateMembershipOrderCommand::new(
        TENANT_ID,
        Some(ORGANIZATION_ID),
        OWNER_USER_ID,
        PACKAGE_EXTERNAL_ID,
        "wechat_pay",
        "550e8400-e29b-41d4-a716-446655440000",
        "550e8400-e29b-41d4-a716-446655440001",
        "MB0000000000000001",
        "MEMBERSHIP0000000000000001",
        "2026-07-07 00:00:00",
        "2026-07-07 00:30:00",
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
}

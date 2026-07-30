use sdkwork_contract_service::CommerceMoney;
use sdkwork_order_service::{
    AccountValueAssetCode, CreateCashWithdrawalRequestCommand, CreateOrderRefundRequestCommand,
};

use crate::test_sqlite_pool::order_points_recharge_e2e_sqlite_memory_pool;
use crate::SqliteCommerceRechargeStore;

#[tokio::test]
async fn refund_idempotency_replays_identical_request_and_rejects_changed_amount() {
    let store =
        SqliteCommerceRechargeStore::new(order_points_recharge_e2e_sqlite_memory_pool().await);
    let command = refund_command("2500");

    let created = store
        .create_order_refund_request(command.clone())
        .await
        .expect("create refund request");
    let replayed = store
        .create_order_refund_request(command)
        .await
        .expect("replay refund request");
    assert_eq!(created.request_id, replayed.request_id);

    let error = store
        .create_order_refund_request(refund_command("2600"))
        .await
        .expect_err("changed refund request must conflict");
    assert_eq!(error.code(), "conflict");
}

#[tokio::test]
async fn withdrawal_idempotency_rejects_changed_payout_destination() {
    let store =
        SqliteCommerceRechargeStore::new(order_points_recharge_e2e_sqlite_memory_pool().await);
    let mut command = withdrawal_command();
    command.payout_method = Some("bank_transfer".to_owned());
    command.payout_account_ref = Some("account-a".to_owned());

    store
        .create_cash_withdrawal_request(command.clone())
        .await
        .expect("create withdrawal request");
    store
        .create_cash_withdrawal_request(command.clone())
        .await
        .expect("replay withdrawal request");

    command.payout_account_ref = Some("account-b".to_owned());
    let error = store
        .create_cash_withdrawal_request(command)
        .await
        .expect_err("changed payout destination must conflict");
    assert_eq!(error.code(), "conflict");
}

fn refund_command(amount: &str) -> CreateOrderRefundRequestCommand {
    CreateOrderRefundRequestCommand::new(
        "tenant-idempotency",
        None,
        "user-idempotency",
        "refund-idempotency-1",
        "order-paid-1",
        AccountValueAssetCode::TokenBank,
        CommerceMoney::new(amount).expect("refund amount"),
        "TOKEN_BANK",
        "refund-idempotency-key-1",
    )
    .expect("refund command")
}

fn withdrawal_command() -> CreateCashWithdrawalRequestCommand {
    CreateCashWithdrawalRequestCommand::new(
        "tenant-idempotency",
        None,
        "user-idempotency",
        "withdrawal-idempotency-1",
        AccountValueAssetCode::Cash,
        CommerceMoney::new("5000").expect("withdrawal amount"),
        "CNY",
        "withdrawal-idempotency-key-1",
    )
    .expect("withdrawal command")
}

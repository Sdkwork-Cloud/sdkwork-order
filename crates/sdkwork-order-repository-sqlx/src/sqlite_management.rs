use sdkwork_contract_service::{CommerceMoney, CommerceServiceError};
use sdkwork_order_service::{
    CancelManagementOrderCommand, CloseManagementOrderCommand, OrderCancellationListQuery,
    OrderCancellationView, OrderManagementDetailQuery, OrderManagementEventListQuery,
    OrderManagementEventView, OrderManagementListQuery, OrderOwnerDetail, OrderOwnerItem,
    OrderOwnerSummary,
};
use sqlx::{Row, SqlitePool};

use crate::sqlite_order::SqliteCommerceOrderStore;

impl SqliteCommerceOrderStore {
    pub async fn list_management_orders(
        &self,
        query: OrderManagementListQuery,
    ) -> Result<Vec<OrderOwnerSummary>, CommerceServiceError> {
        let search = query
            .q
            .as_deref()
            .map(|value| format!("%{}%", value.to_ascii_lowercase()));
        let rows = sqlx::query(
            r#"
            SELECT
                o.id AS order_id,
                o.order_no AS order_sn,
                o.status,
                o.subject,
                o.created_at,
                o.paid_at AS pay_time,
                o.expired_at AS expire_time,
                COALESCE(
                    (
                        SELECT b.payable_amount
                        FROM commerce_order_amount_breakdown b
                        WHERE b.tenant_id = o.tenant_id
                          AND b.order_id = o.id
                          AND b.allocation_type = 'order_total'
                        LIMIT 1
                    ),
                    '0'
                ) AS total_amount,
                COALESCE(
                    (
                        SELECT b.discount_amount
                        FROM commerce_order_amount_breakdown b
                        WHERE b.tenant_id = o.tenant_id
                          AND b.order_id = o.id
                          AND b.allocation_type = 'order_total'
                        LIMIT 1
                    ),
                    '0'
                ) AS discount_amount,
                COALESCE(
                    (
                        SELECT SUM(oi.quantity)
                        FROM commerce_order_item oi
                        WHERE oi.tenant_id = o.tenant_id
                          AND oi.order_id = o.id
                    ),
                    1
                ) AS quantity,
                COALESCE(
                    NULLIF(pa.payment_method, ''),
                    NULLIF(pi.payment_method, '')
                ) AS payment_method
            FROM commerce_order o
            LEFT JOIN commerce_payment_intent pi
                ON pi.tenant_id = o.tenant_id
               AND (pi.organization_id IS NULL OR o.organization_id IS NULL OR pi.organization_id = o.organization_id)
               AND pi.owner_user_id = o.owner_user_id
               AND pi.order_id = o.id
            LEFT JOIN commerce_payment_attempt pa
                ON pa.tenant_id = o.tenant_id
               AND (pa.organization_id IS NULL OR o.organization_id IS NULL OR pa.organization_id = o.organization_id)
               AND pa.owner_user_id = o.owner_user_id
               AND pa.order_id = o.id
            WHERE o.tenant_id = ?
              AND ((o.organization_id = ?) OR (o.organization_id IS NULL AND ? IS NULL))
              AND (? IS NULL OR o.status = ?)
              AND (
                    ? IS NULL
                    OR LOWER(o.order_no) LIKE ?
                    OR LOWER(o.subject) LIKE ?
                    OR LOWER(o.id) LIKE ?
                  )
            ORDER BY o.created_at DESC, o.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&query.tenant_id)
        .bind(query.organization_id.as_deref())
        .bind(query.organization_id.as_deref())
        .bind(query.status.as_deref())
        .bind(query.status.as_deref())
        .bind(search.as_deref())
        .bind(search.as_deref())
        .bind(search.as_deref())
        .bind(search.as_deref())
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(self.pool())
        .await
        .map_err(|error| store_error("failed to list management orders", error))?;

        rows.iter().map(map_management_summary_row).collect()
    }

    pub async fn retrieve_management_order(
        &self,
        query: OrderManagementDetailQuery,
    ) -> Result<Option<OrderOwnerDetail>, CommerceServiceError> {
        let row = sqlx::query(
            r#"
            SELECT
                o.id AS order_id,
                o.order_no AS order_sn,
                o.status,
                o.subject,
                o.created_at,
                o.paid_at AS pay_time,
                o.expired_at AS expire_time,
                COALESCE(
                    (
                        SELECT b.payable_amount
                        FROM commerce_order_amount_breakdown b
                        WHERE b.tenant_id = o.tenant_id
                          AND b.order_id = o.id
                          AND b.allocation_type = 'order_total'
                        LIMIT 1
                    ),
                    '0'
                ) AS total_amount,
                COALESCE(
                    (
                        SELECT b.discount_amount
                        FROM commerce_order_amount_breakdown b
                        WHERE b.tenant_id = o.tenant_id
                          AND b.order_id = o.id
                          AND b.allocation_type = 'order_total'
                        LIMIT 1
                    ),
                    '0'
                ) AS discount_amount,
                COALESCE(
                    (
                        SELECT SUM(oi.quantity)
                        FROM commerce_order_item oi
                        WHERE oi.tenant_id = o.tenant_id
                          AND oi.order_id = o.id
                    ),
                    1
                ) AS quantity,
                COALESCE(
                    NULLIF(pa.payment_method, ''),
                    NULLIF(pi.payment_method, '')
                ) AS payment_method,
                COALESCE(NULLIF(pa.out_trade_no, ''), NULLIF(o.order_no, '')) AS out_trade_no,
                pa.id AS transaction_id
            FROM commerce_order o
            LEFT JOIN commerce_payment_intent pi
                ON pi.tenant_id = o.tenant_id
               AND (pi.organization_id IS NULL OR o.organization_id IS NULL OR pi.organization_id = o.organization_id)
               AND pi.owner_user_id = o.owner_user_id
               AND pi.order_id = o.id
            LEFT JOIN commerce_payment_attempt pa
                ON pa.tenant_id = o.tenant_id
               AND (pa.organization_id IS NULL OR o.organization_id IS NULL OR pa.organization_id = o.organization_id)
               AND pa.owner_user_id = o.owner_user_id
               AND pa.order_id = o.id
            WHERE o.tenant_id = ?
              AND ((o.organization_id = ?) OR (o.organization_id IS NULL AND ? IS NULL))
              AND o.id = ?
            "#,
        )
        .bind(&query.tenant_id)
        .bind(query.organization_id.as_deref())
        .bind(query.organization_id.as_deref())
        .bind(&query.order_id)
        .fetch_optional(self.pool())
        .await
        .map_err(|error| store_error("failed to retrieve management order", error))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let summary = map_management_summary_row(&row)?;
        let items =
            load_management_order_items(self.pool(), &query.tenant_id, &query.order_id).await?;
        Ok(Some(OrderOwnerDetail {
            summary,
            items,
            out_trade_no: optional_string_cell(&row, "out_trade_no"),
            transaction_id: optional_string_cell(&row, "transaction_id"),
        }))
    }

    pub async fn cancel_management_order(
        &self,
        command: CancelManagementOrderCommand,
    ) -> Result<(), CommerceServiceError> {
        let now = current_command_timestamp();
        let result = sqlx::query(
            r#"
            UPDATE commerce_order
            SET status = 'cancelled',
                payment_status = 'closed',
                cancelled_at = ?,
                updated_at = ?
            WHERE tenant_id = ?
              AND ((organization_id = ?) OR (organization_id IS NULL AND ? IS NULL))
              AND id = ?
              AND LOWER(COALESCE(status, '')) IN ('draft', 'pending', 'pending_payment', 'unpaid')
            "#,
        )
        .bind(&now)
        .bind(&now)
        .bind(&command.tenant_id)
        .bind(command.organization_id.as_deref())
        .bind(command.organization_id.as_deref())
        .bind(&command.order_id)
        .execute(self.pool())
        .await
        .map_err(|error| store_error("failed to cancel management order", error))?;

        if result.rows_affected() == 0 {
            return Err(CommerceServiceError::conflict(
                "order is not cancellable or was not found",
            ));
        }

        let _ = command.cancel_reason;
        Ok(())
    }

    pub async fn close_management_order(
        &self,
        command: CloseManagementOrderCommand,
    ) -> Result<(), CommerceServiceError> {
        let now = current_command_timestamp();
        let result = sqlx::query(
            r#"
            UPDATE commerce_order
            SET status = 'closed',
                updated_at = ?
            WHERE tenant_id = ?
              AND ((organization_id = ?) OR (organization_id IS NULL AND ? IS NULL))
              AND id = ?
              AND LOWER(COALESCE(status, '')) NOT IN ('cancelled', 'closed')
            "#,
        )
        .bind(&now)
        .bind(&command.tenant_id)
        .bind(command.organization_id.as_deref())
        .bind(command.organization_id.as_deref())
        .bind(&command.order_id)
        .execute(self.pool())
        .await
        .map_err(|error| store_error("failed to close management order", error))?;

        if result.rows_affected() == 0 {
            return Err(CommerceServiceError::conflict(
                "order is not closable or was not found",
            ));
        }

        let _ = command.close_reason;
        Ok(())
    }

    pub async fn list_management_order_events(
        &self,
        query: OrderManagementEventListQuery,
    ) -> Result<Vec<OrderManagementEventView>, CommerceServiceError> {
        let rows = sqlx::query(
            r#"
            SELECT id, event_type, from_status, to_status, actor_type, actor_id, message, created_at
            FROM commerce_order_event
            WHERE tenant_id = ?
              AND order_id = ?
            ORDER BY created_at DESC, id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&query.tenant_id)
        .bind(&query.order_id)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(self.pool())
        .await
        .map_err(|error| store_error("failed to list management order events", error))?;

        Ok(rows
            .iter()
            .map(|row| OrderManagementEventView {
                id: string_cell(row, "id"),
                event_type: string_cell(row, "event_type"),
                from_status: optional_string_cell(row, "from_status"),
                to_status: string_cell(row, "to_status"),
                actor_type: string_cell(row, "actor_type"),
                actor_id: optional_string_cell(row, "actor_id"),
                message: optional_string_cell(row, "message"),
                created_at: string_cell(row, "created_at"),
            })
            .collect())
    }

    pub async fn list_order_cancellations(
        &self,
        query: OrderCancellationListQuery,
    ) -> Result<Vec<OrderCancellationView>, CommerceServiceError> {
        let rows = sqlx::query(
            r#"
            SELECT id, order_id, status, reason_code, reason_message, created_at
            FROM commerce_order_cancellation
            WHERE tenant_id = ?
              AND (? IS NULL OR status = ?)
            ORDER BY created_at DESC, id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&query.tenant_id)
        .bind(query.status.as_deref())
        .bind(query.status.as_deref())
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(self.pool())
        .await
        .map_err(|error| store_error("failed to list order cancellations", error))?;

        Ok(rows
            .iter()
            .map(|row| OrderCancellationView {
                id: string_cell(row, "id"),
                order_id: string_cell(row, "order_id"),
                status: string_cell(row, "status"),
                reason_code: string_cell(row, "reason_code"),
                reason_message: optional_string_cell(row, "reason_message"),
                created_at: string_cell(row, "created_at"),
            })
            .collect())
    }
}

async fn load_management_order_items(
    pool: &SqlitePool,
    tenant_id: &str,
    order_id: &str,
) -> Result<Vec<OrderOwnerItem>, CommerceServiceError> {
    let rows = sqlx::query(
        r#"
        SELECT id, product_name, quantity, unit_price_amount, total_amount
        FROM commerce_order_item
        WHERE tenant_id = ?
          AND order_id = ?
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(tenant_id)
    .bind(order_id)
    .fetch_all(pool)
    .await
    .map_err(|error| store_error("failed to list management order items", error))?;

    rows.iter()
        .map(|row| {
            Ok(OrderOwnerItem {
                id: string_cell(row, "id"),
                product_name: string_cell(row, "product_name"),
                quantity: row.try_get::<i64, _>("quantity").unwrap_or(1),
                unit_price: CommerceMoney::new(&string_cell(row, "unit_price_amount"))
                    .map_err(CommerceServiceError::storage)?,
                total_amount: CommerceMoney::new(&string_cell(row, "total_amount"))
                    .map_err(CommerceServiceError::storage)?,
            })
        })
        .collect()
}

fn map_management_summary_row(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<OrderOwnerSummary, CommerceServiceError> {
    Ok(OrderOwnerSummary {
        order_id: string_cell(row, "order_id"),
        order_sn: string_cell(row, "order_sn"),
        status: string_cell(row, "status"),
        subject: string_cell(row, "subject"),
        total_amount: CommerceMoney::new(&string_cell(row, "total_amount"))
            .map_err(CommerceServiceError::storage)?,
        paid_amount: None,
        discount_amount: Some(
            CommerceMoney::new(&string_cell(row, "discount_amount"))
                .map_err(CommerceServiceError::storage)?,
        ),
        quantity: row.try_get::<i64, _>("quantity").unwrap_or(1),
        created_at: string_cell(row, "created_at"),
        pay_time: optional_string_cell(row, "pay_time"),
        expire_time: optional_string_cell(row, "expire_time"),
        payment_method: optional_string_cell(row, "payment_method"),
    })
}

fn string_cell(row: &sqlx::sqlite::SqliteRow, column: &str) -> String {
    row.try_get::<String, _>(column).unwrap_or_default()
}

fn optional_string_cell(row: &sqlx::sqlite::SqliteRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column)
        .ok()
        .flatten()
        .filter(|value| !value.is_empty())
}

fn store_error(message: &str, error: impl std::fmt::Display) -> CommerceServiceError {
    CommerceServiceError::storage(format!("{message}: {error}"))
}

fn current_command_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}

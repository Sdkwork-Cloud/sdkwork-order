use sdkwork_commerce_contract_service::{CommerceMoney, CommerceServiceError};
use sdkwork_commerce_order_service::{
    CancelOwnerOrderCommand, CreateOwnerOrderCommand, CreateOwnerOrderOutcome, OrderOwnerDetail,
    OrderOwnerDetailQuery, OrderOwnerItem, OrderOwnerListQuery, OrderOwnerStatistics,
    OrderOwnerSummary,
};
use sdkwork_commerce_payment_service::{PaymentMethodItem, PaymentMethodListQuery};
use sqlx::{PgPool, Postgres, Row, Transaction};
use std::time::{SystemTime, UNIX_EPOCH};

const LIST_OWNER_ORDERS: &str = r#"
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
WHERE o.tenant_id = CAST($1 AS TEXT)
  AND ((o.organization_id = CAST($1 AS TEXT)) OR (o.organization_id IS NULL AND $2 IS NULL))
  AND o.owner_user_id = CAST($1 AS TEXT)
  AND ($4 IS NULL OR o.status = $4)
ORDER BY o.created_at DESC, o.id DESC
LIMIT $5 OFFSET $6
"#;

const RETRIEVE_OWNER_ORDER: &str = r#"
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
    CAST(pa.id AS TEXT) AS transaction_id
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
WHERE o.tenant_id = CAST($1 AS TEXT)
  AND ((o.organization_id = CAST($1 AS TEXT)) OR (o.organization_id IS NULL AND $2 IS NULL))
  AND o.owner_user_id = CAST($1 AS TEXT)
  AND o.id = CAST($1 AS TEXT)
LIMIT 1
"#;

const LIST_ORDER_ITEMS: &str = r#"
SELECT
    id,
    title AS product_name,
    quantity,
    unit_price_amount,
    total_amount
FROM commerce_order_item
WHERE tenant_id = CAST($1 AS TEXT)
  AND order_id = CAST($1 AS TEXT)
ORDER BY created_at ASC, id ASC
"#;

const OWNER_ORDER_STATISTICS: &str = r#"
SELECT
    COUNT(*) AS total_orders,
    SUM(CASE WHEN LOWER(o.status) IN ('pending_payment', 'unpaid', 'wait_pay') THEN 1 ELSE 0 END) AS pending_payment,
    SUM(CASE WHEN LOWER(o.status) IN ('paid', 'fulfilled') THEN 1 ELSE 0 END) AS pending_shipment,
    SUM(CASE WHEN LOWER(o.status) IN ('shipped', 'delivered') THEN 1 ELSE 0 END) AS pending_receipt,
    SUM(CASE WHEN LOWER(o.status) IN ('completed', 'finished') THEN 1 ELSE 0 END) AS completed,
    COALESCE(
        SUM(
            CAST(
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
                ) AS REAL
            )
        ),
        0
    ) AS total_amount
FROM commerce_order o
WHERE o.tenant_id = CAST($1 AS TEXT)
  AND ((o.organization_id = CAST($1 AS TEXT)) OR (o.organization_id IS NULL AND $2 IS NULL))
  AND o.owner_user_id = CAST($1 AS TEXT)
"#;

const LIST_PAYMENT_METHODS: &str = r#"
SELECT
    id,
    method_key,
    display_name,
    provider_code,
    sort_order
FROM commerce_payment_method
WHERE (
        (tenant_id = CAST($1 AS TEXT) AND organization_id = CAST($1 AS TEXT))
        OR (tenant_id = CAST($1 AS TEXT) AND organization_id IS NULL)
        OR (tenant_id = '100001' AND (organization_id = '0' OR organization_id IS NULL))
      )
  AND status = 'active'
ORDER BY COALESCE(sort_order, 0) ASC, id ASC
"#;

#[derive(Debug, Clone)]
pub struct PostgresCommerceOrderStore {
    pool: PgPool,
}

impl PostgresCommerceOrderStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn list_owner_orders(
        &self,
        query: OrderOwnerListQuery,
    ) -> Result<Vec<OrderOwnerSummary>, CommerceServiceError> {
        let rows = sqlx::query(LIST_OWNER_ORDERS)
            .bind(&query.tenant_id)
            .bind(query.organization_id.as_deref())
            .bind(&query.owner_user_id)
            .bind(query.status.as_deref())
            .bind(query.status.as_deref())
            .bind(query.limit())
            .bind(query.offset())
            .fetch_all(&self.pool)
            .await
            .or_else(empty_rows_when_read_model_is_missing)
            .map_err(|error| store_error("failed to list owner orders", error))?;

        rows.iter().map(map_order_summary_row).collect()
    }

    pub async fn retrieve_owner_order(
        &self,
        query: OrderOwnerDetailQuery,
    ) -> Result<Option<OrderOwnerDetail>, CommerceServiceError> {
        let row = sqlx::query(RETRIEVE_OWNER_ORDER)
            .bind(&query.tenant_id)
            .bind(query.organization_id.as_deref())
            .bind(&query.owner_user_id)
            .bind(&query.order_id)
            .fetch_optional(&self.pool)
            .await
            .or_else(none_when_read_model_is_missing)
            .map_err(|error| store_error("failed to retrieve owner order", error))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let summary = map_order_summary_row(&row)?;
        let items = load_order_items(&self.pool, &query.tenant_id, &query.order_id).await?;
        Ok(Some(OrderOwnerDetail {
            summary,
            items,
            out_trade_no: optional_string_cell(&row, "out_trade_no"),
            transaction_id: optional_string_cell(&row, "transaction_id"),
        }))
    }

    pub async fn retrieve_owner_order_statistics(
        &self,
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
    ) -> Result<OrderOwnerStatistics, CommerceServiceError> {
        match sqlx::query(OWNER_ORDER_STATISTICS)
            .bind(tenant_id)
            .bind(organization_id)
            .bind(owner_user_id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(row) => Ok(OrderOwnerStatistics {
                total_orders: row.try_get::<i64, _>("total_orders").unwrap_or(0),
                pending_payment: row.try_get::<i64, _>("pending_payment").unwrap_or(0),
                pending_shipment: row.try_get::<i64, _>("pending_shipment").unwrap_or(0),
                pending_receipt: row.try_get::<i64, _>("pending_receipt").unwrap_or(0),
                completed: row.try_get::<i64, _>("completed").unwrap_or(0),
                total_amount: CommerceMoney::new(&format!(
                    "{:.2}",
                    row.try_get::<f64, _>("total_amount").unwrap_or(0.0)
                ))
                .map_err(CommerceServiceError::storage)?,
            }),
            Err(error) if read_model_table_is_missing(&error) => Ok(empty_order_statistics()),
            Err(error) => Err(store_error(
                "failed to retrieve owner order statistics",
                error,
            )),
        }
    }

    pub async fn list_payment_methods(
        &self,
        query: PaymentMethodListQuery,
    ) -> Result<Vec<PaymentMethodItem>, CommerceServiceError> {
        let rows = sqlx::query(LIST_PAYMENT_METHODS)
            .bind(&query.tenant_id)
            .bind(query.organization_id.as_deref())
            .fetch_all(&self.pool)
            .await
            .or_else(empty_rows_when_read_model_is_missing)
            .map_err(|error| store_error("failed to list payment methods", error))?;

        Ok(rows
            .iter()
            .map(|row| PaymentMethodItem {
                id: string_cell(row, "id"),
                method_key: string_cell(row, "method_key"),
                display_name: string_cell(row, "display_name"),
                provider_code: string_cell(row, "provider_code"),
                sort_order: row.try_get::<i64, _>("sort_order").unwrap_or(0),
            })
            .collect())
    }
    pub async fn create_owner_order(
        &self,
        command: CreateOwnerOrderCommand,
    ) -> Result<CreateOwnerOrderOutcome, CommerceServiceError> {
        let order_id = format!("order-{}", command.checkout_session_id);
        let mut tx = self.pool.begin().await.map_err(|error| {
            store_error("failed to begin create owner order transaction", error)
        })?;

        let existing = sqlx::query(
            r#"
            SELECT
                o.id AS order_id,
                o.order_no AS order_sn,
                o.status,
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
                ) AS total_amount
            FROM commerce_order o
            WHERE o.id = $1
              AND o.tenant_id = CAST($2 AS TEXT)
              AND ((o.organization_id = CAST($3 AS TEXT)) OR (o.organization_id IS NULL AND $4 IS NULL))
              AND o.owner_user_id = CAST($5 AS TEXT)
            FOR UPDATE
            "#,
        )
        .bind(&order_id)
        .bind(&command.tenant_id)
        .bind(command.organization_id.as_deref())
        .bind(command.organization_id.as_deref())
        .bind(&command.owner_user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|error| store_error("failed to lock owner order for create", error))?;

        if let Some(row) = existing {
            tx.commit().await.map_err(|error| {
                store_error("failed to commit existing owner order lookup", error)
            })?;
            let total_amount = CommerceMoney::new(&string_cell(&row, "total_amount"))
                .map_err(CommerceServiceError::storage)?;
            return Ok(CreateOwnerOrderOutcome {
                order_id: string_cell(&row, "order_id"),
                order_sn: string_cell(&row, "order_sn"),
                status: string_cell(&row, "status"),
                total_amount,
            });
        }

        let session = load_checkout_session_for_order(&mut tx, &command).await?;
        let lines = load_checkout_lines_for_order(&mut tx, &command).await?;
        if lines.is_empty() {
            return Err(CommerceServiceError::conflict(
                "checkout session has no selected lines",
            ));
        }
        let quote = load_checkout_quote_for_order(&mut tx, &command).await?;
        let now = current_command_timestamp();
        let order_sn = command.request_no.clone();
        let subject = checkout_order_subject(&lines);
        let currency_code = string_cell(&session, "currency_code");
        let payable_amount = string_cell(&quote, "payable_amount");
        let original_amount = string_cell(&quote, "original_amount");
        let discount_amount = string_cell(&quote, "discount_amount");
        let expires_at =
            optional_string_cell(&session, "expires_at").unwrap_or_else(|| now.clone());

        sqlx::query(
            r#"
            INSERT INTO commerce_order
                (id, tenant_id, organization_id, owner_user_id, order_no, status, payment_status,
                 fulfillment_status, refund_status, subject, currency_code, request_no,
                 idempotency_key, created_at, paid_at, cancelled_at, expired_at, updated_at)
            VALUES
                ($1, CAST($2 AS TEXT), CAST($3 AS TEXT), CAST($4 AS TEXT), $5, 'pending_payment',
                 'pending', 'unfulfilled', 'none', $6, $7, $8, $9, $10, NULL, NULL, $11, $12)
            "#,
        )
        .bind(&order_id)
        .bind(&command.tenant_id)
        .bind(command.organization_id.as_deref())
        .bind(&command.owner_user_id)
        .bind(&order_sn)
        .bind(&subject)
        .bind(&currency_code)
        .bind(&command.request_no)
        .bind(&command.idempotency_key)
        .bind(&now)
        .bind(&expires_at)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to insert owner order", error))?;

        for line in &lines {
            let line_id = string_cell(line, "id");
            let item_id = format!("{order_id}-item-{line_id}");
            let quantity = line.try_get::<i64, _>("quantity").unwrap_or(1).max(1);
            let unit_price = string_cell(line, "price_amount_snapshot");
            let total_amount = multiply_money_amount(&unit_price, quantity);
            sqlx::query(
                r#"
                INSERT INTO commerce_order_item
                    (id, tenant_id, order_id, product_id, shop_id, sku_id, sku_snapshot_json,
                     title, quantity, unit_price_amount, discount_amount, tax_amount,
                     total_amount, fulfillment_status, refund_status, created_at)
                VALUES
                    ($1, CAST($2 AS TEXT), $3, $4, $5, $6, $7, $8, $9, $10, '0.00', '0.00', $11,
                     'unfulfilled', 'none', $12)
                "#,
            )
            .bind(&item_id)
            .bind(&command.tenant_id)
            .bind(&order_id)
            .bind(optional_string_cell(line, "product_id"))
            .bind(optional_string_cell(line, "shop_id"))
            .bind(string_cell(line, "sku_id"))
            .bind(string_cell(line, "sku_snapshot_json"))
            .bind(checkout_line_title(line))
            .bind(quantity)
            .bind(&unit_price)
            .bind(&total_amount)
            .bind(&now)
            .execute(&mut *tx)
            .await
            .map_err(|error| store_error("failed to insert owner order item", error))?;
        }

        sqlx::query(
            r#"
            INSERT INTO commerce_order_amount_breakdown
                (id, tenant_id, organization_id, order_id, order_item_id, allocation_type,
                 original_amount, discount_amount, payable_amount, currency_code, created_at)
            VALUES
                ($1, CAST($2 AS TEXT), CAST($3 AS TEXT), $4, NULL, 'order_total', $5, $6, $7, $8, $9)
            "#,
        )
        .bind(format!("{order_id}-amount"))
        .bind(&command.tenant_id)
        .bind(command.organization_id.as_deref())
        .bind(&order_id)
        .bind(&original_amount)
        .bind(&discount_amount)
        .bind(&payable_amount)
        .bind(&currency_code)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to insert owner order amount breakdown", error))?;

        sqlx::query(
            r#"
            UPDATE commerce_checkout_session
            SET status = 'submitted', submitted_at = $1, updated_at = $2
            WHERE id = $3
              AND tenant_id = CAST($4 AS TEXT)
              AND owner_user_id = CAST($5 AS TEXT)
            "#,
        )
        .bind(&now)
        .bind(&now)
        .bind(&command.checkout_session_id)
        .bind(&command.tenant_id)
        .bind(&command.owner_user_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to submit checkout session", error))?;

        tx.commit().await.map_err(|error| {
            store_error("failed to commit create owner order transaction", error)
        })?;

        let total_amount =
            CommerceMoney::new(&payable_amount).map_err(CommerceServiceError::storage)?;
        Ok(CreateOwnerOrderOutcome {
            order_id,
            order_sn,
            status: "pending_payment".to_owned(),
            total_amount,
        })
    }

    pub async fn cancel_owner_order(
        &self,
        command: CancelOwnerOrderCommand,
    ) -> Result<(), CommerceServiceError> {
        let now = current_command_timestamp();
        let result = sqlx::query(
            r#"
            UPDATE commerce_order
            SET status = 'cancelled',
                payment_status = 'closed',
                cancelled_at = $1,
                updated_at = $2
            WHERE tenant_id = CAST($3 AS TEXT)
              AND ((organization_id = CAST($4 AS TEXT)) OR (organization_id IS NULL AND $5 IS NULL))
              AND owner_user_id = CAST($6 AS TEXT)
              AND id = CAST($7 AS TEXT)
              AND LOWER(COALESCE(status, '')) IN ('draft', 'pending', 'pending_payment', 'unpaid')
            "#,
        )
        .bind(&now)
        .bind(&now)
        .bind(&command.tenant_id)
        .bind(command.organization_id.as_deref())
        .bind(command.organization_id.as_deref())
        .bind(&command.owner_user_id)
        .bind(&command.order_id)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("failed to cancel owner order", error))?;

        if result.rows_affected() == 0 {
            return Err(CommerceServiceError::conflict(
                "order is not cancellable or was not found",
            ));
        }

        let _ = command.cancel_reason;

        Ok(())
    }
}

async fn load_order_items(
    pool: &PgPool,
    tenant_id: &str,
    order_id: &str,
) -> Result<Vec<OrderOwnerItem>, CommerceServiceError> {
    let rows = sqlx::query(LIST_ORDER_ITEMS)
        .bind(tenant_id)
        .bind(order_id)
        .fetch_all(pool)
        .await
        .or_else(empty_rows_when_read_model_is_missing)
        .map_err(|error| store_error("failed to list order items", error))?;

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

fn map_order_summary_row(
    row: &sqlx::postgres::PgRow,
) -> Result<OrderOwnerSummary, CommerceServiceError> {
    let total_amount = CommerceMoney::new(&string_cell(row, "total_amount"))
        .map_err(CommerceServiceError::storage)?;
    let discount_amount = CommerceMoney::new(&string_cell(row, "discount_amount"))
        .map_err(CommerceServiceError::storage)?;
    let status = string_cell(row, "status");
    let paid_amount = if status.eq_ignore_ascii_case("paid")
        || status.eq_ignore_ascii_case("completed")
        || status.eq_ignore_ascii_case("fulfilled")
    {
        Some(total_amount.clone())
    } else {
        None
    };

    Ok(OrderOwnerSummary {
        order_id: string_cell(row, "order_id"),
        order_sn: string_cell(row, "order_sn"),
        status,
        subject: string_cell(row, "subject"),
        total_amount,
        paid_amount,
        discount_amount: Some(discount_amount),
        quantity: row.try_get::<i64, _>("quantity").unwrap_or(1),
        created_at: string_cell(row, "created_at"),
        pay_time: optional_string_cell(row, "pay_time"),
        expire_time: optional_string_cell(row, "expire_time"),
        payment_method: optional_string_cell(row, "payment_method"),
    })
}

fn optional_string_cell(row: &sqlx::postgres::PgRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column).ok().flatten()
}

fn string_cell(row: &sqlx::postgres::PgRow, column: &str) -> String {
    optional_string_cell(row, column).unwrap_or_default()
}

fn store_error(message: &str, error: impl std::fmt::Display) -> CommerceServiceError {
    CommerceServiceError::storage(format!("{message}: {error}"))
}

fn empty_rows_when_read_model_is_missing<T>(error: sqlx::Error) -> Result<Vec<T>, sqlx::Error> {
    if read_model_table_is_missing(&error) {
        Ok(Vec::new())
    } else {
        Err(error)
    }
}

fn none_when_read_model_is_missing<T>(error: sqlx::Error) -> Result<Option<T>, sqlx::Error> {
    if read_model_table_is_missing(&error) {
        Ok(None)
    } else {
        Err(error)
    }
}

fn read_model_table_is_missing(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(ref db) if db.message().contains("does not exist"))
}


async fn load_checkout_session_for_order(
    tx: &mut Transaction<'_, Postgres>,
    command: &CreateOwnerOrderCommand,
) -> Result<sqlx::postgres::PgRow, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT currency_code, expires_at, status
        FROM commerce_checkout_session
        WHERE id = $1
          AND tenant_id = CAST($2 AS TEXT)
          AND ((organization_id = CAST($3 AS TEXT)) OR (organization_id IS NULL AND $4 IS NULL))
          AND owner_user_id = CAST($5 AS TEXT)
          AND LOWER(COALESCE(status, '')) IN ('active', 'quoted', 'open')
        "#,
    )
    .bind(&command.checkout_session_id)
    .bind(&command.tenant_id)
    .bind(command.organization_id.as_deref())
    .bind(command.organization_id.as_deref())
    .bind(&command.owner_user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load checkout session", error))?
    .ok_or_else(|| CommerceServiceError::conflict("checkout session is not orderable"))?;
    Ok(row)
}

async fn load_checkout_lines_for_order(
    tx: &mut Transaction<'_, Postgres>,
    command: &CreateOwnerOrderCommand,
) -> Result<Vec<sqlx::postgres::PgRow>, CommerceServiceError> {
    sqlx::query(
        r#"
        SELECT id, product_id, shop_id, sku_id, sku_snapshot_json, quantity, price_amount_snapshot
        FROM commerce_checkout_line
        WHERE tenant_id = CAST($1 AS TEXT)
          AND checkout_session_id = $2
          AND selected = 1
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(&command.tenant_id)
    .bind(&command.checkout_session_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load checkout lines", error))
}

async fn load_checkout_quote_for_order(
    tx: &mut Transaction<'_, Postgres>,
    command: &CreateOwnerOrderCommand,
) -> Result<sqlx::postgres::PgRow, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT original_amount, discount_amount, payable_amount
        FROM commerce_checkout_quote
        WHERE tenant_id = CAST($1 AS TEXT)
          AND checkout_session_id = $2
          AND LOWER(COALESCE(quote_status, '')) IN ('active', 'quoted', 'ready')
        ORDER BY created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(&command.tenant_id)
    .bind(&command.checkout_session_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load checkout quote", error))?
    .ok_or_else(|| CommerceServiceError::conflict("checkout quote was not found"))?;
    Ok(row)
}

fn checkout_order_subject(lines: &[sqlx::postgres::PgRow]) -> String {
    lines
        .first()
        .map(checkout_line_title)
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| "Checkout order".to_owned())
}

fn checkout_line_title(row: &sqlx::postgres::PgRow) -> String {
    let snapshot = string_cell(row, "sku_snapshot_json");
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&snapshot) {
        if let Some(title) = value.get("title").and_then(serde_json::Value::as_str) {
            if !title.trim().is_empty() {
                return title.trim().to_owned();
            }
        }
    }
    string_cell(row, "sku_id")
}

fn multiply_money_amount(amount: &str, quantity: i64) -> String {
    let Ok(cents) = money_cents(amount) else {
        return amount.to_owned();
    };
    let total_cents = cents.saturating_mul(quantity.max(1));
    format!("{}.{:02}", total_cents / 100, total_cents.rem_euclid(100))
}

fn money_cents(value: &str) -> Result<i64, CommerceServiceError> {
    CommerceMoney::new(value)
        .map(|money| {
            let parts: Vec<_> = money.as_str().split('.').collect();
            let integer = parts
                .first()
                .and_then(|part| part.parse::<i64>().ok())
                .unwrap_or(0);
            let fraction = parts
                .get(1)
                .map(|part| {
                    let padded = format!("{part:0<2}");
                    padded[..2.min(padded.len())].parse::<i64>().unwrap_or(0)
                })
                .unwrap_or(0);
            integer * 100 + fraction
        })
        .map_err(CommerceServiceError::storage)
}

fn current_command_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    format!("{seconds}")
}

fn empty_order_statistics() -> OrderOwnerStatistics {
    OrderOwnerStatistics {
        total_orders: 0,
        pending_payment: 0,
        pending_shipment: 0,
        pending_receipt: 0,
        completed: 0,
        total_amount: CommerceMoney::new("0.00").expect("zero money should be valid"),
    }
}

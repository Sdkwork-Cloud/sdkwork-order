use std::collections::HashSet;

use sdkwork_contract_service::CommerceServiceError;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_order_service::{
    PhysicalInventoryMutationOutcome, PhysicalInventoryReservationPort, PhysicalPurchaseFuture,
    ReleasePhysicalOrderInventoryRequest, ReservePhysicalOrderInventoryRequest,
};
use sqlx::{Postgres, Row, Sqlite, Transaction};

#[derive(Clone)]
pub struct PhysicalInventoryAdapter {
    pool: DatabasePool,
}

impl PhysicalInventoryAdapter {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

impl PhysicalInventoryReservationPort for PhysicalInventoryAdapter {
    fn reserve_physical_order_inventory<'a>(
        &'a self,
        request: ReservePhysicalOrderInventoryRequest,
    ) -> PhysicalPurchaseFuture<'a, PhysicalInventoryMutationOutcome> {
        Box::pin(async move {
            match &self.pool {
                DatabasePool::Sqlite(pool, _) => reserve_sqlite(pool, &request).await,
                DatabasePool::Postgres(pool, _) => reserve_postgres(pool, &request).await,
            }
        })
    }

    fn release_physical_order_inventory<'a>(
        &'a self,
        request: ReleasePhysicalOrderInventoryRequest,
    ) -> PhysicalPurchaseFuture<'a, PhysicalInventoryMutationOutcome> {
        Box::pin(async move {
            match &self.pool {
                DatabasePool::Sqlite(pool, _) => release_sqlite(pool, &request).await,
                DatabasePool::Postgres(pool, _) => release_postgres(pool, &request).await,
            }
        })
    }
}

pub(crate) async fn consume_order_inventory(
    pool: &DatabasePool,
    tenant_id: &str,
    order_id: &str,
    idempotency_key: &str,
) -> Result<PhysicalInventoryMutationOutcome, CommerceServiceError> {
    match pool {
        DatabasePool::Sqlite(pool, _) => {
            let mut tx = pool
                .begin()
                .await
                .map_err(store_error("begin inventory consume"))?;
            let rows = sqlx::query(
                "SELECT id, tenant_id, organization_id, sku_id, warehouse_id, fulfillment_node_id, quantity, status FROM commerce_inventory_reservation WHERE tenant_id = ? AND order_id = ? ORDER BY id",
            )
            .bind(tenant_id)
            .bind(order_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(store_error("load inventory reservations for consume"))?;
            if rows.is_empty() {
                return Err(CommerceServiceError::invalid_state(
                    "physical order has no inventory reservation",
                ));
            }
            let replayed = rows
                .iter()
                .all(|row| text_sqlite(row, "status").eq_ignore_ascii_case("consumed"));
            if !replayed {
                for row in &rows {
                    let status = text_sqlite(row, "status");
                    if status.eq_ignore_ascii_case("consumed") {
                        continue;
                    }
                    if !status.eq_ignore_ascii_case("reserved") {
                        return Err(CommerceServiceError::invalid_state(
                            "inventory reservation cannot be consumed",
                        ));
                    }
                    consume_stock_sqlite(&mut tx, row).await?;
                    sqlx::query("UPDATE commerce_inventory_reservation SET status = 'consumed', consumed_quantity = quantity, consumed_at = ?, updated_at = ?, idempotency_key = ? WHERE id = ? AND status = 'reserved'")
                        .bind(now_string()).bind(now_string()).bind(idempotency_key)
                        .bind(text_sqlite(row, "id"))
                        .execute(&mut *tx).await.map_err(store_error("consume inventory reservation"))?;
                }
            }
            tx.commit()
                .await
                .map_err(store_error("commit inventory consume"))?;
            Ok(PhysicalInventoryMutationOutcome {
                accepted: true,
                replayed,
            })
        }
        DatabasePool::Postgres(pool, _) => {
            let mut tx = pool
                .begin()
                .await
                .map_err(store_error("begin inventory consume"))?;
            let rows = sqlx::query(
                "SELECT id, tenant_id, organization_id, sku_id, warehouse_id, fulfillment_node_id, quantity, status FROM commerce_inventory_reservation WHERE tenant_id = $1 AND order_id = $2 ORDER BY id FOR UPDATE",
            )
            .bind(tenant_id).bind(order_id).fetch_all(&mut *tx).await
            .map_err(store_error("load inventory reservations for consume"))?;
            if rows.is_empty() {
                return Err(CommerceServiceError::invalid_state(
                    "physical order has no inventory reservation",
                ));
            }
            let replayed = rows
                .iter()
                .all(|row| text_postgres(row, "status").eq_ignore_ascii_case("consumed"));
            if !replayed {
                for row in &rows {
                    let status = text_postgres(row, "status");
                    if status.eq_ignore_ascii_case("consumed") {
                        continue;
                    }
                    if !status.eq_ignore_ascii_case("reserved") {
                        return Err(CommerceServiceError::invalid_state(
                            "inventory reservation cannot be consumed",
                        ));
                    }
                    consume_stock_postgres(&mut tx, row).await?;
                    sqlx::query("UPDATE commerce_inventory_reservation SET status = 'consumed', consumed_quantity = quantity, consumed_at = $1, updated_at = $2, idempotency_key = $3 WHERE id = $4 AND status = 'reserved'")
                        .bind(now_string()).bind(now_string()).bind(idempotency_key)
                        .bind(text_postgres(row, "id"))
                        .execute(&mut *tx).await.map_err(store_error("consume inventory reservation"))?;
                }
            }
            tx.commit()
                .await
                .map_err(store_error("commit inventory consume"))?;
            Ok(PhysicalInventoryMutationOutcome {
                accepted: true,
                replayed,
            })
        }
    }
}

async fn reserve_sqlite(
    pool: &sqlx::SqlitePool,
    request: &ReservePhysicalOrderInventoryRequest,
) -> Result<PhysicalInventoryMutationOutcome, CommerceServiceError> {
    validate_reserve_request(request)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(store_error("begin inventory reservation"))?;
    let existing: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM commerce_inventory_reservation WHERE tenant_id = ? AND order_id = ?",
    )
    .bind(&request.tenant_id)
    .bind(&request.order_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(store_error("count inventory reservations"))?;
    if existing > 0 {
        validate_reservation_replay_sqlite(&mut tx, request, existing).await?;
        tx.commit()
            .await
            .map_err(store_error("commit inventory reservation replay"))?;
        return Ok(PhysicalInventoryMutationOutcome {
            accepted: true,
            replayed: true,
        });
    }
    for line in &request.lines {
        let stock = sqlx::query("SELECT id, warehouse_id, fulfillment_node_id FROM commerce_inventory_stock WHERE tenant_id = ? AND organization_id = ? AND shop_id = ? AND sku_id = ? AND status = 'active' AND available_quantity - safety_stock_quantity >= ? ORDER BY available_quantity DESC, id LIMIT 1")
            .bind(&request.tenant_id).bind(&request.merchant_organization_id).bind(&line.shop_id)
            .bind(&line.sku_id).bind(line.quantity).fetch_optional(&mut *tx).await
            .map_err(store_error("select reservable inventory stock"))?
            .ok_or_else(|| CommerceServiceError::conflict("physical SKU inventory is insufficient"))?;
        let stock_id = text_sqlite(&stock, "id");
        let updated = sqlx::query("UPDATE commerce_inventory_stock SET available_quantity = available_quantity - ?, reserved_quantity = reserved_quantity + ?, version = version + 1, updated_at = ? WHERE id = ? AND available_quantity - safety_stock_quantity >= ?")
            .bind(line.quantity).bind(line.quantity).bind(now_string()).bind(&stock_id).bind(line.quantity)
            .execute(&mut *tx).await.map_err(store_error("reserve inventory stock"))?;
        if updated.rows_affected() != 1 {
            return Err(CommerceServiceError::conflict(
                "physical SKU inventory is insufficient",
            ));
        }
        insert_reservation_sqlite(&mut tx, request, line, &stock).await?;
    }
    tx.commit()
        .await
        .map_err(store_error("commit inventory reservation"))?;
    Ok(PhysicalInventoryMutationOutcome {
        accepted: true,
        replayed: false,
    })
}

async fn reserve_postgres(
    pool: &sqlx::PgPool,
    request: &ReservePhysicalOrderInventoryRequest,
) -> Result<PhysicalInventoryMutationOutcome, CommerceServiceError> {
    validate_reserve_request(request)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(store_error("begin inventory reservation"))?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(format!(
            "physical-inventory:{}:{}",
            request.tenant_id, request.order_id
        ))
        .execute(&mut *tx)
        .await
        .map_err(store_error("lock physical inventory reservation"))?;
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM commerce_inventory_reservation WHERE tenant_id = $1 AND order_id = $2")
        .bind(&request.tenant_id).bind(&request.order_id).fetch_one(&mut *tx).await
        .map_err(store_error("count inventory reservations"))?;
    if existing > 0 {
        validate_reservation_replay_postgres(&mut tx, request, existing).await?;
        tx.commit()
            .await
            .map_err(store_error("commit inventory reservation replay"))?;
        return Ok(PhysicalInventoryMutationOutcome {
            accepted: true,
            replayed: true,
        });
    }
    for line in &request.lines {
        let stock = sqlx::query("SELECT id, warehouse_id, fulfillment_node_id FROM commerce_inventory_stock WHERE tenant_id = $1 AND organization_id = $2 AND shop_id = $3 AND sku_id = $4 AND status = 'active' AND available_quantity - safety_stock_quantity >= $5 ORDER BY available_quantity DESC, id LIMIT 1 FOR UPDATE")
            .bind(&request.tenant_id).bind(&request.merchant_organization_id).bind(&line.shop_id)
            .bind(&line.sku_id).bind(line.quantity).fetch_optional(&mut *tx).await
            .map_err(store_error("select reservable inventory stock"))?
            .ok_or_else(|| CommerceServiceError::conflict("physical SKU inventory is insufficient"))?;
        let stock_id = text_postgres(&stock, "id");
        let updated = sqlx::query("UPDATE commerce_inventory_stock SET available_quantity = available_quantity - $1, reserved_quantity = reserved_quantity + $1, version = version + 1, updated_at = $2 WHERE id = $3 AND available_quantity - safety_stock_quantity >= $1")
            .bind(line.quantity).bind(now_string()).bind(&stock_id)
            .execute(&mut *tx).await.map_err(store_error("reserve inventory stock"))?;
        if updated.rows_affected() != 1 {
            return Err(CommerceServiceError::conflict(
                "physical SKU inventory is insufficient",
            ));
        }
        insert_reservation_postgres(&mut tx, request, line, &stock).await?;
    }
    tx.commit()
        .await
        .map_err(store_error("commit inventory reservation"))?;
    Ok(PhysicalInventoryMutationOutcome {
        accepted: true,
        replayed: false,
    })
}

async fn release_sqlite(
    pool: &sqlx::SqlitePool,
    request: &ReleasePhysicalOrderInventoryRequest,
) -> Result<PhysicalInventoryMutationOutcome, CommerceServiceError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(store_error("begin inventory release"))?;
    let rows = sqlx::query("SELECT id, tenant_id, organization_id, sku_id, warehouse_id, fulfillment_node_id, quantity, status FROM commerce_inventory_reservation WHERE tenant_id = ? AND order_id = ? ORDER BY id")
        .bind(&request.tenant_id).bind(&request.order_id).fetch_all(&mut *tx).await.map_err(store_error("load inventory reservations for release"))?;
    if rows.is_empty() {
        return Ok(PhysicalInventoryMutationOutcome {
            accepted: true,
            replayed: true,
        });
    }
    let replayed = rows
        .iter()
        .all(|row| text_sqlite(row, "status").eq_ignore_ascii_case("released"));
    if !replayed {
        for row in &rows {
            let status = text_sqlite(row, "status");
            if status.eq_ignore_ascii_case("released") {
                continue;
            }
            if !status.eq_ignore_ascii_case("reserved") {
                return Err(CommerceServiceError::invalid_state(
                    "consumed inventory cannot be released",
                ));
            }
            release_stock_sqlite(&mut tx, row).await?;
            sqlx::query("UPDATE commerce_inventory_reservation SET status = 'released', released_quantity = quantity, release_reason_code = ?, released_at = ?, updated_at = ?, idempotency_key = ? WHERE id = ? AND status = 'reserved'")
                .bind(&request.reason_code).bind(now_string()).bind(now_string()).bind(&request.idempotency_key).bind(text_sqlite(row, "id"))
                .execute(&mut *tx).await.map_err(store_error("release inventory reservation"))?;
        }
    }
    tx.commit()
        .await
        .map_err(store_error("commit inventory release"))?;
    Ok(PhysicalInventoryMutationOutcome {
        accepted: true,
        replayed,
    })
}

async fn release_postgres(
    pool: &sqlx::PgPool,
    request: &ReleasePhysicalOrderInventoryRequest,
) -> Result<PhysicalInventoryMutationOutcome, CommerceServiceError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(store_error("begin inventory release"))?;
    let rows = sqlx::query("SELECT id, tenant_id, organization_id, sku_id, warehouse_id, fulfillment_node_id, quantity, status FROM commerce_inventory_reservation WHERE tenant_id = $1 AND order_id = $2 ORDER BY id FOR UPDATE")
        .bind(&request.tenant_id).bind(&request.order_id).fetch_all(&mut *tx).await.map_err(store_error("load inventory reservations for release"))?;
    if rows.is_empty() {
        return Ok(PhysicalInventoryMutationOutcome {
            accepted: true,
            replayed: true,
        });
    }
    let replayed = rows
        .iter()
        .all(|row| text_postgres(row, "status").eq_ignore_ascii_case("released"));
    if !replayed {
        for row in &rows {
            let status = text_postgres(row, "status");
            if status.eq_ignore_ascii_case("released") {
                continue;
            }
            if !status.eq_ignore_ascii_case("reserved") {
                return Err(CommerceServiceError::invalid_state(
                    "consumed inventory cannot be released",
                ));
            }
            release_stock_postgres(&mut tx, row).await?;
            sqlx::query("UPDATE commerce_inventory_reservation SET status = 'released', released_quantity = quantity, release_reason_code = $1, released_at = $2, updated_at = $3, idempotency_key = $4 WHERE id = $5 AND status = 'reserved'")
                .bind(&request.reason_code).bind(now_string()).bind(now_string()).bind(&request.idempotency_key).bind(text_postgres(row, "id"))
                .execute(&mut *tx).await.map_err(store_error("release inventory reservation"))?;
        }
    }
    tx.commit()
        .await
        .map_err(store_error("commit inventory release"))?;
    Ok(PhysicalInventoryMutationOutcome {
        accepted: true,
        replayed,
    })
}

fn validate_reserve_request(
    request: &ReservePhysicalOrderInventoryRequest,
) -> Result<(), CommerceServiceError> {
    if request.tenant_id.trim().is_empty()
        || request.merchant_organization_id.trim().is_empty()
        || request.order_id.trim().is_empty()
        || request.request_no.trim().is_empty()
        || request.idempotency_key.trim().is_empty()
        || request.lines.is_empty()
        || request.lines.iter().any(|line| {
            line.quantity <= 0 || line.sku_id.trim().is_empty() || line.shop_id.trim().is_empty()
        })
    {
        return Err(CommerceServiceError::validation(
            "physical inventory reservation lines are invalid",
        ));
    }
    let mut sku_ids = HashSet::with_capacity(request.lines.len());
    if request
        .lines
        .iter()
        .any(|line| !sku_ids.insert(line.sku_id.trim()))
    {
        return Err(CommerceServiceError::validation(
            "physical inventory reservation contains duplicate SKU lines",
        ));
    }
    Ok(())
}

async fn validate_reservation_replay_sqlite(
    tx: &mut Transaction<'_, Sqlite>,
    request: &ReservePhysicalOrderInventoryRequest,
    existing: i64,
) -> Result<(), CommerceServiceError> {
    let rows = sqlx::query("SELECT organization_id, sku_id, quantity, idempotency_key, status FROM commerce_inventory_reservation WHERE tenant_id = ? AND order_id = ? ORDER BY sku_id")
        .bind(&request.tenant_id).bind(&request.order_id).fetch_all(&mut **tx).await.map_err(store_error("validate inventory reservation replay"))?;
    if !reservation_replay_matches_sqlite(&rows, request, existing) {
        return Err(CommerceServiceError::conflict(
            "inventory reservation replay does not match the original order",
        ));
    }
    Ok(())
}

async fn validate_reservation_replay_postgres(
    tx: &mut Transaction<'_, Postgres>,
    request: &ReservePhysicalOrderInventoryRequest,
    existing: i64,
) -> Result<(), CommerceServiceError> {
    let rows = sqlx::query("SELECT organization_id, sku_id, quantity, idempotency_key, status FROM commerce_inventory_reservation WHERE tenant_id = $1 AND order_id = $2 ORDER BY sku_id FOR UPDATE")
        .bind(&request.tenant_id).bind(&request.order_id).fetch_all(&mut **tx).await.map_err(store_error("validate inventory reservation replay"))?;
    if !reservation_replay_matches_postgres(&rows, request, existing) {
        return Err(CommerceServiceError::conflict(
            "inventory reservation replay does not match the original order",
        ));
    }
    Ok(())
}

async fn insert_reservation_sqlite(
    tx: &mut Transaction<'_, Sqlite>,
    request: &ReservePhysicalOrderInventoryRequest,
    line: &sdkwork_order_service::PhysicalInventoryLine,
    stock: &sqlx::sqlite::SqliteRow,
) -> Result<(), CommerceServiceError> {
    let id = reservation_id(&request.order_id, &line.sku_id);
    sqlx::query("INSERT INTO commerce_inventory_reservation (id, tenant_id, organization_id, reservation_no, order_id, reservation_source_type, reservation_source_id, reservation_type, sku_id, warehouse_id, fulfillment_node_id, quantity, reserved_quantity, consumed_quantity, released_quantity, status, request_no, idempotency_key, expires_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'order', ?, 'sale', ?, ?, ?, ?, ?, 0, 0, 'reserved', ?, ?, ?, ?, ?)")
        .bind(&id).bind(&request.tenant_id).bind(&request.merchant_organization_id).bind(&id).bind(&request.order_id).bind(&request.order_id)
        .bind(&line.sku_id).bind(optional_text_sqlite(stock, "warehouse_id")).bind(optional_text_sqlite(stock, "fulfillment_node_id"))
        .bind(line.quantity).bind(line.quantity).bind(&request.request_no).bind(&request.idempotency_key).bind(expires_at()).bind(now_string()).bind(now_string())
        .execute(&mut **tx).await.map_err(store_error("insert inventory reservation"))?;
    Ok(())
}

async fn insert_reservation_postgres(
    tx: &mut Transaction<'_, Postgres>,
    request: &ReservePhysicalOrderInventoryRequest,
    line: &sdkwork_order_service::PhysicalInventoryLine,
    stock: &sqlx::postgres::PgRow,
) -> Result<(), CommerceServiceError> {
    let id = reservation_id(&request.order_id, &line.sku_id);
    sqlx::query("INSERT INTO commerce_inventory_reservation (id, tenant_id, organization_id, reservation_no, order_id, reservation_source_type, reservation_source_id, reservation_type, sku_id, warehouse_id, fulfillment_node_id, quantity, reserved_quantity, consumed_quantity, released_quantity, status, request_no, idempotency_key, expires_at, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'order', $6, 'sale', $7, $8, $9, $10, $10, 0, 0, 'reserved', $11, $12, $13, $14, $14)")
        .bind(&id).bind(&request.tenant_id).bind(&request.merchant_organization_id).bind(&id).bind(&request.order_id).bind(&request.order_id)
        .bind(&line.sku_id).bind(optional_text_postgres(stock, "warehouse_id")).bind(optional_text_postgres(stock, "fulfillment_node_id"))
        .bind(line.quantity).bind(&request.request_no).bind(&request.idempotency_key).bind(expires_at()).bind(now_string())
        .execute(&mut **tx).await.map_err(store_error("insert inventory reservation"))?;
    Ok(())
}

async fn consume_stock_sqlite(
    tx: &mut Transaction<'_, Sqlite>,
    row: &sqlx::sqlite::SqliteRow,
) -> Result<(), CommerceServiceError> {
    mutate_stock_sqlite(tx, row, false).await
}
async fn release_stock_sqlite(
    tx: &mut Transaction<'_, Sqlite>,
    row: &sqlx::sqlite::SqliteRow,
) -> Result<(), CommerceServiceError> {
    mutate_stock_sqlite(tx, row, true).await
}
async fn mutate_stock_sqlite(
    tx: &mut Transaction<'_, Sqlite>,
    row: &sqlx::sqlite::SqliteRow,
    release: bool,
) -> Result<(), CommerceServiceError> {
    let quantity = row.try_get::<i64, _>("quantity").unwrap_or(0);
    let sql = if release {
        "UPDATE commerce_inventory_stock SET available_quantity = available_quantity + ?, reserved_quantity = reserved_quantity - ?, version = version + 1, updated_at = ? WHERE tenant_id = ? AND organization_id = ? AND sku_id = ? AND ((warehouse_id = ?) OR (warehouse_id IS NULL AND ? IS NULL)) AND ((fulfillment_node_id = ?) OR (fulfillment_node_id IS NULL AND ? IS NULL)) AND reserved_quantity >= ?"
    } else {
        "UPDATE commerce_inventory_stock SET reserved_quantity = reserved_quantity - ?, sold_quantity = sold_quantity + ?, version = version + 1, updated_at = ? WHERE tenant_id = ? AND organization_id = ? AND sku_id = ? AND ((warehouse_id = ?) OR (warehouse_id IS NULL AND ? IS NULL)) AND ((fulfillment_node_id = ?) OR (fulfillment_node_id IS NULL AND ? IS NULL)) AND reserved_quantity >= ?"
    };
    let warehouse = optional_text_sqlite(row, "warehouse_id");
    let fulfillment_node = optional_text_sqlite(row, "fulfillment_node_id");
    let result = sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(quantity)
        .bind(quantity)
        .bind(now_string())
        .bind(text_sqlite(row, "tenant_id"))
        .bind(text_sqlite(row, "organization_id"))
        .bind(text_sqlite(row, "sku_id"))
        .bind(warehouse.as_deref())
        .bind(warehouse.as_deref())
        .bind(fulfillment_node.as_deref())
        .bind(fulfillment_node.as_deref())
        .bind(quantity)
        .execute(&mut **tx)
        .await
        .map_err(store_error("mutate reserved inventory stock"))?;
    if result.rows_affected() != 1 {
        return Err(CommerceServiceError::invalid_state(
            "reserved inventory stock is inconsistent",
        ));
    }
    Ok(())
}
async fn consume_stock_postgres(
    tx: &mut Transaction<'_, Postgres>,
    row: &sqlx::postgres::PgRow,
) -> Result<(), CommerceServiceError> {
    mutate_stock_postgres(tx, row, false).await
}
async fn release_stock_postgres(
    tx: &mut Transaction<'_, Postgres>,
    row: &sqlx::postgres::PgRow,
) -> Result<(), CommerceServiceError> {
    mutate_stock_postgres(tx, row, true).await
}
async fn mutate_stock_postgres(
    tx: &mut Transaction<'_, Postgres>,
    row: &sqlx::postgres::PgRow,
    release: bool,
) -> Result<(), CommerceServiceError> {
    let quantity = row.try_get::<i64, _>("quantity").unwrap_or(0);
    let sql = if release {
        "UPDATE commerce_inventory_stock SET available_quantity = available_quantity + $1, reserved_quantity = reserved_quantity - $1, version = version + 1, updated_at = $2 WHERE tenant_id = $3 AND organization_id = $4 AND sku_id = $5 AND ((warehouse_id = $6) OR (warehouse_id IS NULL AND $6 IS NULL)) AND ((fulfillment_node_id = $7) OR (fulfillment_node_id IS NULL AND $7 IS NULL)) AND reserved_quantity >= $1"
    } else {
        "UPDATE commerce_inventory_stock SET reserved_quantity = reserved_quantity - $1, sold_quantity = sold_quantity + $1, version = version + 1, updated_at = $2 WHERE tenant_id = $3 AND organization_id = $4 AND sku_id = $5 AND ((warehouse_id = $6) OR (warehouse_id IS NULL AND $6 IS NULL)) AND ((fulfillment_node_id = $7) OR (fulfillment_node_id IS NULL AND $7 IS NULL)) AND reserved_quantity >= $1"
    };
    let result = sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(quantity)
        .bind(now_string())
        .bind(text_postgres(row, "tenant_id"))
        .bind(text_postgres(row, "organization_id"))
        .bind(text_postgres(row, "sku_id"))
        .bind(optional_text_postgres(row, "warehouse_id"))
        .bind(optional_text_postgres(row, "fulfillment_node_id"))
        .execute(&mut **tx)
        .await
        .map_err(store_error("mutate reserved inventory stock"))?;
    if result.rows_affected() != 1 {
        return Err(CommerceServiceError::invalid_state(
            "reserved inventory stock is inconsistent",
        ));
    }
    Ok(())
}

fn reservation_id(order_id: &str, sku_id: &str) -> String {
    format!("inventory-reservation-{order_id}-{sku_id}")
}

fn reservation_replay_matches_sqlite(
    rows: &[sqlx::sqlite::SqliteRow],
    request: &ReservePhysicalOrderInventoryRequest,
    existing: i64,
) -> bool {
    reservation_replay_matches(
        rows.iter().map(|row| {
            (
                text_sqlite(row, "organization_id"),
                text_sqlite(row, "sku_id"),
                row.try_get::<i64, _>("quantity").unwrap_or(0),
                text_sqlite(row, "idempotency_key"),
                text_sqlite(row, "status"),
            )
        }),
        request,
        existing,
    )
}

fn reservation_replay_matches_postgres(
    rows: &[sqlx::postgres::PgRow],
    request: &ReservePhysicalOrderInventoryRequest,
    existing: i64,
) -> bool {
    reservation_replay_matches(
        rows.iter().map(|row| {
            (
                text_postgres(row, "organization_id"),
                text_postgres(row, "sku_id"),
                row.try_get::<i64, _>("quantity").unwrap_or(0),
                text_postgres(row, "idempotency_key"),
                text_postgres(row, "status"),
            )
        }),
        request,
        existing,
    )
}

fn reservation_replay_matches<I>(
    stored: I,
    request: &ReservePhysicalOrderInventoryRequest,
    existing: i64,
) -> bool
where
    I: Iterator<Item = (String, String, i64, String, String)>,
{
    if existing != request.lines.len() as i64 {
        return false;
    }
    let mut stored = stored.collect::<Vec<_>>();
    stored.sort_by(|left, right| left.1.cmp(&right.1));
    let mut requested = request
        .lines
        .iter()
        .map(|line| (line.sku_id.trim(), line.quantity))
        .collect::<Vec<_>>();
    requested.sort_unstable_by(|left, right| left.0.cmp(right.0));

    stored.len() == requested.len()
        && stored.iter().zip(requested).all(
            |((organization_id, sku_id, quantity, idempotency_key, status), requested)| {
                organization_id == &request.merchant_organization_id
                    && sku_id == requested.0
                    && *quantity == requested.1
                    && idempotency_key == &request.idempotency_key
                    && matches!(status.as_str(), "reserved" | "consumed")
            },
        )
}
fn now_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|v| v.as_secs())
        .unwrap_or(0)
        .to_string()
}
fn expires_at() -> String {
    (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|v| v.as_secs())
        .unwrap_or(0)
        + 1800)
        .to_string()
}
fn text_sqlite(row: &sqlx::sqlite::SqliteRow, column: &str) -> String {
    optional_text_sqlite(row, column).unwrap_or_default()
}
fn optional_text_sqlite(row: &sqlx::sqlite::SqliteRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column).ok().flatten()
}
fn text_postgres(row: &sqlx::postgres::PgRow, column: &str) -> String {
    optional_text_postgres(row, column).unwrap_or_default()
}
fn optional_text_postgres(row: &sqlx::postgres::PgRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column).ok().flatten()
}
fn store_error(message: &'static str) -> impl FnOnce(sqlx::Error) -> CommerceServiceError {
    move |error| CommerceServiceError::storage(format!("{message}: {error}"))
}

#[cfg(test)]
mod tests;

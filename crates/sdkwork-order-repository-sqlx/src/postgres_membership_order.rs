use sdkwork_contract_service::{CommerceMoney, CommerceServiceError};
use sdkwork_order_service::{CreateMembershipOrderCommand, CreateMembershipOrderOutcome};
use sdkwork_utils_rust::{build_commerce_cashier_url, commerce_cashier_scene};
use sqlx::{PgPool, Postgres, Row, Transaction};

const PLATFORM_ORGANIZATION_SCOPE_SENTINEL: &str = "0";

const LOAD_MEMBERSHIP_PACKAGE_BY_EXTERNAL_ID: &str = r#"
SELECT
    CAST(p.external_id AS TEXT) AS package_external_id,
    p.name AS package_name,
    CAST(p.price_amount AS TEXT) AS price_amount,
    COALESCE(NULLIF(p.currency_code, ''), 'CNY') AS currency_code,
    CAST(p.duration_days AS BIGINT) AS duration_days,
    p.sku_id,
    COALESCE(NULLIF(s.name, ''), NULLIF(s.title, ''), NULLIF(pr.title, ''), p.name) AS product_name
FROM membership_package p
JOIN membership_package_group g
    ON g.id = p.package_group_id
LEFT JOIN commerce_product_sku s
    ON s.id = p.sku_id
   AND s.sales_status = 'active'
LEFT JOIN commerce_product_spu pr
    ON pr.id = s.spu_id
   AND pr.sales_status = 'active'
WHERE (
        (p.tenant_id = CAST($1 AS TEXT) AND p.organization_id = CAST($2 AS TEXT))
        OR (p.tenant_id = CAST($1 AS TEXT) AND p.organization_id IS NULL)
      )
  AND (
        (g.tenant_id = CAST($1 AS TEXT) AND g.organization_id = CAST($2 AS TEXT))
        OR (g.tenant_id = CAST($1 AS TEXT) AND g.organization_id IS NULL)
      )
  AND CAST(p.external_id AS TEXT) = $3
  AND p.status = 'active'
  AND g.status = 'active'
ORDER BY
    CASE
        WHEN p.tenant_id = CAST($1 AS TEXT) AND p.organization_id = CAST($2 AS TEXT) THEN 0
        WHEN p.tenant_id = CAST($1 AS TEXT) AND p.organization_id IS NULL THEN 1
        ELSE 2
    END ASC,
    COALESCE(p.sort_weight, 0) ASC,
    p.id ASC
LIMIT 1
"#;

const LOAD_MEMBERSHIP_PACKAGE_BY_EXTERNAL_ID_PUBLIC: &str = r#"
SELECT
    CAST(p.external_id AS TEXT) AS package_external_id,
    p.name AS package_name,
    CAST(p.price_amount AS TEXT) AS price_amount,
    COALESCE(NULLIF(p.currency_code, ''), 'CNY') AS currency_code,
    CAST(p.duration_days AS BIGINT) AS duration_days,
    p.sku_id,
    COALESCE(NULLIF(s.name, ''), NULLIF(s.title, ''), NULLIF(pr.title, ''), p.name) AS product_name
FROM membership_package p
JOIN membership_package_group g
    ON g.id = p.package_group_id
LEFT JOIN commerce_product_sku s
    ON s.id = p.sku_id
   AND s.sales_status = 'active'
LEFT JOIN commerce_product_spu pr
    ON pr.id = s.spu_id
   AND pr.sales_status = 'active'
WHERE p.tenant_id = '__PLATFORM_TENANT__'
  AND (p.organization_id = '0' OR p.organization_id IS NULL)
  AND (g.tenant_id = '__PLATFORM_TENANT__' OR g.tenant_id IS NULL)
  AND (g.organization_id = '0' OR g.organization_id IS NULL)
  AND CAST(p.external_id AS TEXT) = $1
  AND p.status = 'active'
  AND g.status = 'active'
ORDER BY COALESCE(p.sort_weight, 0) ASC, p.id ASC
LIMIT 1
"#;

const LOAD_MEMBERSHIP_PAYMENT_METHOD: &str = r#"
SELECT method_key, provider AS provider_code
FROM commerce_payment_method
WHERE (
        (tenant_id = CAST($1 AS TEXT) AND organization_id = CAST($2 AS TEXT))
        OR (tenant_id = CAST($1 AS TEXT) AND organization_id IS NULL)
        OR (tenant_id = '__PLATFORM_TENANT__' AND (organization_id = '0' OR organization_id IS NULL))
      )
  AND status = 'active'
  AND LOWER(method_key) = $3
ORDER BY
    CASE
        WHEN tenant_id = CAST($1 AS TEXT) AND organization_id = CAST($2 AS TEXT) THEN 0
        WHEN tenant_id = CAST($1 AS TEXT) AND organization_id IS NULL THEN 1
        ELSE 2
    END ASC,
    COALESCE(sort_weight, 0) ASC,
    id ASC
LIMIT 1
"#;

#[derive(Debug, Clone)]
pub struct PostgresCommerceMembershipOrderStore {
    pool: PgPool,
}

#[derive(Debug, Clone)]
struct MembershipPackageCatalog {
    package_external_id: String,
    package_name: String,
    price_amount: CommerceMoney,
    currency_code: String,
    duration_days: i64,
    sku_id: String,
    product_name: String,
}

impl PostgresCommerceMembershipOrderStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_membership_order(
        &self,
        command: CreateMembershipOrderCommand,
    ) -> Result<CreateMembershipOrderOutcome, CommerceServiceError> {
        if let Some(outcome) = self
            .load_membership_order_by_idempotency_key(&command)
            .await?
        {
            return Ok(outcome);
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("failed to begin membership order transaction", error))?;

        let package = load_membership_package(&mut tx, &command).await?;
        let method_key = load_membership_payment_method(&mut tx, &command).await?;

        insert_membership_order(&mut tx, &command, &package).await?;
        insert_membership_order_item(&mut tx, &command, &package, &method_key).await?;
        insert_membership_order_amount_breakdown(&mut tx, &command, &package).await?;

        tx.commit()
            .await
            .map_err(|error| store_error("failed to commit membership order transaction", error))?;

        Ok(build_membership_order_outcome(&command, &package, &method_key))
    }

    async fn load_membership_order_by_idempotency_key(
        &self,
        command: &CreateMembershipOrderCommand,
    ) -> Result<Option<CreateMembershipOrderOutcome>, CommerceServiceError> {
        let organization_id = normalize_organization_scope(command.organization_id.as_deref());
        let row = sqlx::query(
            r#"
            SELECT
                o.id AS order_id,
                o.order_no,
                COALESCE(NULLIF(o.request_no, ''), o.order_no) AS out_trade_no,
                CAST(COALESCE(ab.payable_amount, oi.total_amount, '0') AS TEXT) AS amount,
                COALESCE(NULLIF(ab.currency_code, ''), 'CNY') AS currency_code,
                COALESCE(
                    NULLIF(COALESCE(oi.sku_snapshot_json, '{}')::jsonb ->> 'packageId', ''),
                    $1
                ) AS package_id,
                COALESCE(
                    NULLIF(COALESCE(oi.sku_snapshot_json, '{}')::jsonb ->> 'productName', ''),
                    oi.title,
                    'Membership package'
                ) AS package_name,
                CAST(COALESCE(
                    NULLIF(COALESCE(oi.sku_snapshot_json, '{}')::jsonb ->> 'durationDays', ''),
                    '0'
                ) AS BIGINT) AS duration_days,
                COALESCE(
                    NULLIF(COALESCE(oi.sku_snapshot_json, '{}')::jsonb ->> 'paymentMethod', ''),
                    '-'
                ) AS payment_method,
                o.status AS order_status
            FROM commerce_order o
            LEFT JOIN commerce_order_item oi
                ON oi.tenant_id = o.tenant_id
               AND oi.order_id = o.id
            LEFT JOIN commerce_order_amount_breakdown ab
                ON ab.tenant_id = o.tenant_id
               AND ab.order_id = o.id
            WHERE o.tenant_id = CAST($2 AS TEXT)
              AND ((o.organization_id = CAST($3 AS TEXT)) OR (o.organization_id IS NULL AND $3 IS NULL))
              AND o.owner_user_id = CAST($4 AS TEXT)
              AND o.idempotency_key = CAST($5 AS TEXT)
              AND o.subject = 'membership'
            ORDER BY oi.created_at ASC NULLS LAST, oi.id ASC
            LIMIT 1
            "#,
        )
        .bind(&command.package_id)
        .bind(&command.tenant_id)
        .bind(&organization_id)
        .bind(&command.owner_user_id)
        .bind(&command.idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("failed to load membership order idempotency replay", error))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let order_no = string_cell(&row, "order_no");
        let out_trade_no = string_cell(&row, "out_trade_no");
        let amount = commerce_money_cell(&row, "amount", "membership order amount")?;
        let duration_days = required_positive_integer_cell(&row, "duration_days")?;

        CreateMembershipOrderOutcome::new(
            &string_cell(&row, "order_id"),
            &order_no,
            &out_trade_no,
            amount,
            &string_cell(&row, "currency_code"),
            &string_cell(&row, "package_id"),
            &string_cell(&row, "package_name"),
            duration_days,
            &string_cell(&row, "payment_method"),
            membership_order_status_label(&string_cell(&row, "order_status")),
            &membership_cashier_url(&order_no, &out_trade_no),
        )
        .map(Some)
    }
}

async fn load_membership_package(
    tx: &mut Transaction<'_, Postgres>,
    command: &CreateMembershipOrderCommand,
) -> Result<MembershipPackageCatalog, CommerceServiceError> {
    let organization_id = normalize_organization_scope(command.organization_id.as_deref());
    let row = if command.tenant_id.trim().is_empty() {
        sqlx::query(LOAD_MEMBERSHIP_PACKAGE_BY_EXTERNAL_ID_PUBLIC)
            .bind(&command.package_id)
            .fetch_optional(&mut **tx)
            .await
    } else {
        let scoped_row = sqlx::query(LOAD_MEMBERSHIP_PACKAGE_BY_EXTERNAL_ID)
            .bind(&command.tenant_id)
            .bind(&organization_id)
            .bind(&command.package_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|error| store_error("failed to load membership package", error))?;
        if scoped_row.is_some() {
            Ok(scoped_row)
        } else {
            sqlx::query(LOAD_MEMBERSHIP_PACKAGE_BY_EXTERNAL_ID_PUBLIC)
                .bind(&command.package_id)
                .fetch_optional(&mut **tx)
                .await
        }
    }
    .map_err(|error| store_error("failed to load membership package", error))?
    .ok_or_else(|| CommerceServiceError::conflict("membership package is unavailable"))?;

    map_membership_package_row(&row)
}

fn map_membership_package_row(
    row: &sqlx::postgres::PgRow,
) -> Result<MembershipPackageCatalog, CommerceServiceError> {
    let sku_id = string_cell(row, "sku_id");
    if sku_id.trim().is_empty() {
        return Err(CommerceServiceError::conflict(
            "membership package product sku is unavailable",
        ));
    }

    Ok(MembershipPackageCatalog {
        package_external_id: string_cell(row, "package_external_id"),
        package_name: string_cell(row, "package_name"),
        price_amount: commerce_money_cell(row, "price_amount", "membership package price amount")?,
        currency_code: string_cell(row, "currency_code")
            .trim()
            .to_ascii_uppercase(),
        duration_days: required_positive_integer_cell(row, "duration_days")?,
        sku_id,
        product_name: string_cell(row, "product_name"),
    })
}

async fn load_membership_payment_method(
    tx: &mut Transaction<'_, Postgres>,
    command: &CreateMembershipOrderCommand,
) -> Result<String, CommerceServiceError> {
    let requested_method = normalize_method_key(&command.method);
    let organization_id = normalize_organization_scope(command.organization_id.as_deref());
    let row = sqlx::query(LOAD_MEMBERSHIP_PAYMENT_METHOD)
        .bind(&command.tenant_id)
        .bind(&organization_id)
        .bind(&requested_method)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|error| store_error("failed to load membership payment method", error))?
        .ok_or_else(|| {
            CommerceServiceError::conflict("membership payment method is unavailable")
        })?;

    Ok(normalize_method_key(&string_cell(&row, "method_key")))
}

async fn insert_membership_order(
    tx: &mut Transaction<'_, Postgres>,
    command: &CreateMembershipOrderCommand,
    package: &MembershipPackageCatalog,
) -> Result<(), CommerceServiceError> {
    let organization_id = normalize_organization_scope(command.organization_id.as_deref());
    sqlx::query(
        r#"
        INSERT INTO commerce_order
            (id, tenant_id, organization_id, owner_user_id, order_no, status, payment_status, fulfillment_status, refund_status, subject, currency_code, request_no, idempotency_key, created_at, paid_at, cancelled_at, expired_at, updated_at)
        VALUES
            ($1, CAST($2 AS TEXT), CAST($3 AS TEXT), CAST($4 AS TEXT), $5, 'pending_payment', 'pending', 'unfulfilled', 'none', 'membership', $6, $7, $8, $9, NULL, NULL, $10, $9)
        "#,
    )
    .bind(&command.order_id)
    .bind(&command.tenant_id)
    .bind(&organization_id)
    .bind(&command.owner_user_id)
    .bind(&command.order_no)
    .bind(&package.currency_code)
    .bind(&command.order_no)
    .bind(&command.idempotency_key)
    .bind(&command.requested_at)
    .bind(&command.expire_at)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to insert membership order", error))?;
    Ok(())
}

async fn insert_membership_order_item(
    tx: &mut Transaction<'_, Postgres>,
    command: &CreateMembershipOrderCommand,
    package: &MembershipPackageCatalog,
    payment_method: &str,
) -> Result<(), CommerceServiceError> {
    sqlx::query(
        r#"
        INSERT INTO commerce_order_item
            (id, tenant_id, order_id, sku_id, sku_snapshot_json, title, quantity, unit_price_amount, total_amount, fulfillment_status, refund_status, created_at)
        VALUES
            ($1, CAST($2 AS TEXT), $3, $4, $5, $6, 1, $7, $7, 'unfulfilled', 'none', $8)
        "#,
    )
    .bind(&command.order_item_id)
    .bind(&command.tenant_id)
    .bind(&command.order_id)
    .bind(&package.sku_id)
    .bind(membership_order_item_snapshot_json(
        package,
        command,
        payment_method,
    ))
    .bind(&package.package_name)
    .bind(package.price_amount.as_str())
    .bind(&command.requested_at)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to insert membership order item", error))?;
    Ok(())
}

async fn insert_membership_order_amount_breakdown(
    tx: &mut Transaction<'_, Postgres>,
    command: &CreateMembershipOrderCommand,
    package: &MembershipPackageCatalog,
) -> Result<(), CommerceServiceError> {
    sqlx::query(
        r#"
        INSERT INTO commerce_order_amount_breakdown
            (id, tenant_id, order_id, original_amount, discount_amount, payable_amount, currency_code, created_at)
        VALUES
            ($1, CAST($2 AS TEXT), $3, $4, '0.00', $4, $5, $6)
        "#,
    )
    .bind(format!("{}-amount", command.order_id))
    .bind(&command.tenant_id)
    .bind(&command.order_id)
    .bind(package.price_amount.as_str())
    .bind(&package.currency_code)
    .bind(&command.requested_at)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to insert membership order amount breakdown", error))?;
    Ok(())
}

fn membership_order_item_snapshot_json(
    package: &MembershipPackageCatalog,
    command: &CreateMembershipOrderCommand,
    payment_method: &str,
) -> String {
    serde_json::json!({
        "skuId": package.sku_id,
        "productName": package.product_name,
        "packageId": package.package_external_id,
        "durationDays": package.duration_days,
        "clientRequestNo": command.client_request_no,
        "source": command.source,
        "paymentMethod": payment_method,
    })
    .to_string()
}

fn build_membership_order_outcome(
    command: &CreateMembershipOrderCommand,
    package: &MembershipPackageCatalog,
    payment_method: &str,
) -> CreateMembershipOrderOutcome {
    CreateMembershipOrderOutcome::new(
        &command.order_id,
        &command.order_no,
        &command.out_trade_no,
        package.price_amount.clone(),
        &package.currency_code,
        &package.package_external_id,
        &package.package_name,
        package.duration_days,
        payment_method,
        "pending_payment",
        &membership_cashier_url(&command.order_no, &command.out_trade_no),
    )
    .expect("membership order outcome should be valid")
}

fn membership_cashier_url(order_no: &str, out_trade_no: &str) -> String {
    build_commerce_cashier_url(
        commerce_cashier_scene(Some("membership")),
        order_no,
        out_trade_no,
    )
}

fn membership_order_status_label(status: &str) -> &str {
    match status.trim().to_ascii_lowercase().as_str() {
        "paid" | "fulfilled" | "completed" => "paid",
        "cancelled" | "canceled" | "expired" => "closed",
        _ => "pending_payment",
    }
}

fn normalize_organization_scope(organization_id: Option<&str>) -> String {
    organization_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(PLATFORM_ORGANIZATION_SCOPE_SENTINEL)
        .to_owned()
}

fn normalize_method_key(method: &str) -> String {
    match method.trim().to_ascii_lowercase().as_str() {
        "wechat" => "wechat_pay".to_string(),
        other => other.to_string(),
    }
}

fn commerce_money_cell(
    row: &sqlx::postgres::PgRow,
    column: &str,
    field_name: &str,
) -> Result<CommerceMoney, CommerceServiceError> {
    let value = string_cell(row, column);
    let cents = money_cents(&value)
        .map_err(|_| CommerceServiceError::storage(format!("invalid {field_name}: {value}")))?;
    CommerceMoney::new(&format_money_minor(cents))
        .map_err(|message| CommerceServiceError::storage(format!("{message}: {value}")))
}

fn money_cents(amount: &str) -> Result<i64, CommerceServiceError> {
    let value = amount.trim();
    let mut parts = value.split('.');
    let whole = parts
        .next()
        .unwrap_or_default()
        .parse::<i64>()
        .map_err(|_| {
            CommerceServiceError::storage(format!("invalid commerce money amount: {value}"))
        })?;
    let fraction = parts.next().unwrap_or_default();
    if parts.next().is_some() || fraction.len() > 2 {
        return Err(CommerceServiceError::storage(format!(
            "invalid commerce money amount: {value}"
        )));
    }
    let mut padded = fraction.to_string();
    while padded.len() < 2 {
        padded.push('0');
    }
    let cents = if padded.is_empty() {
        0
    } else {
        padded.parse::<i64>().map_err(|_| {
            CommerceServiceError::storage(format!("invalid commerce money amount: {value}"))
        })?
    };
    whole
        .checked_mul(100)
        .and_then(|amount| amount.checked_add(cents))
        .ok_or_else(|| {
            CommerceServiceError::storage(format!("invalid commerce money amount: {value}"))
        })
}

fn format_money_minor(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.abs();
    format!("{sign}{}.{:02}", abs / 100, abs % 100)
}

fn optional_string_cell(row: &sqlx::postgres::PgRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column).ok().flatten()
}

fn string_cell(row: &sqlx::postgres::PgRow, column: &str) -> String {
    optional_string_cell(row, column).unwrap_or_default()
}

fn required_positive_integer_cell(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<i64, CommerceServiceError> {
    let value = row
        .try_get::<Option<i64>, _>(column)
        .ok()
        .flatten()
        .or_else(|| {
            row.try_get::<Option<i32>, _>(column)
                .ok()
                .flatten()
                .map(i64::from)
        })
        .or_else(|| {
            optional_string_cell(row, column).and_then(|value| value.trim().parse::<i64>().ok())
        })
        .ok_or_else(|| CommerceServiceError::storage(format!("invalid integer column {column}")))?;
    if value <= 0 {
        return Err(CommerceServiceError::storage(format!(
            "integer column {column} must be greater than zero"
        )));
    }
    Ok(value)
}

fn store_error(context: &str, error: sqlx::Error) -> CommerceServiceError {
    crate::sql_store_error::map_sqlx_store_error(context, error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postgres_membership_cashier_url_uses_virtual_scene() {
        let url = membership_cashier_url("MB123", "MEMBERSHIP123");
        assert!(url.contains("scene=virtual"));
    }
}

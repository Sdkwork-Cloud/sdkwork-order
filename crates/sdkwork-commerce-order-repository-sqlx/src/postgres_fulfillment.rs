use sdkwork_commerce_contract_service::CommerceServiceError;
use sdkwork_commerce_order_service::{
    FulfillmentDetailQuery, FulfillmentListQuery, FulfillmentView,
};
use sqlx::Row;

use crate::postgres_order::PostgresCommerceOrderStore;

impl PostgresCommerceOrderStore {
    pub async fn list_owner_fulfillments(
        &self,
        query: FulfillmentListQuery,
    ) -> Result<Vec<FulfillmentView>, CommerceServiceError> {
        let rows = sqlx::query(
            r#"
            SELECT f.id, f.fulfillment_no, f.order_id, f.fulfillment_type, f.status
            FROM commerce_fulfillment_order f
            INNER JOIN commerce_order o
                ON o.tenant_id = f.tenant_id
               AND o.id = f.order_id
            WHERE f.tenant_id = CAST($1 AS TEXT)
              AND ((f.organization_id = CAST($2 AS TEXT)) OR (f.organization_id IS NULL AND $3 IS NULL))
              AND o.owner_user_id = CAST($4 AS TEXT)
              AND ($5 IS NULL OR f.order_id = CAST($6 AS TEXT))
              AND ($7 IS NULL OR LOWER(f.status) = LOWER(CAST($8 AS TEXT)))
            ORDER BY f.created_at DESC, f.id DESC
           "#,
        )
        .bind(&query.tenant_id)
        .bind(query.organization_id.as_deref())
        .bind(query.organization_id.as_deref())
        .bind(&query.owner_user_id)
        .bind(query.order_id.as_deref())
        .bind(query.order_id.as_deref())
        .bind(query.status.as_deref())
        .bind(query.status.as_deref())
        .fetch_all(self.pool())
        .await
        .map_err(|error| store_error("failed to list owner fulfillments", error))?;

        Ok(rows
            .into_iter()
            .map(|row| FulfillmentView {
                fulfillment_id: string_cell(&row, "id"),
                fulfillment_no: string_cell(&row, "fulfillment_no"),
                order_id: string_cell(&row, "order_id"),
                fulfillment_type: string_cell(&row, "fulfillment_type"),
                status: string_cell(&row, "status"),
            })
            .collect())
    }

    pub async fn retrieve_owner_fulfillment(
        &self,
        query: FulfillmentDetailQuery,
    ) -> Result<Option<FulfillmentView>, CommerceServiceError> {
        let row = sqlx::query(
            r#"
            SELECT f.id, f.fulfillment_no, f.order_id, f.fulfillment_type, f.status
            FROM commerce_fulfillment_order f
            INNER JOIN commerce_order o
                ON o.tenant_id = f.tenant_id
               AND o.id = f.order_id
            WHERE f.tenant_id = CAST($1 AS TEXT)
              AND ((f.organization_id = CAST($2 AS TEXT)) OR (f.organization_id IS NULL AND $3 IS NULL))
              AND o.owner_user_id = CAST($4 AS TEXT)
              AND f.id = CAST($5 AS TEXT)
            LIMIT 1
           "#,
        )
        .bind(&query.tenant_id)
        .bind(query.organization_id.as_deref())
        .bind(query.organization_id.as_deref())
        .bind(&query.owner_user_id)
        .bind(&query.fulfillment_id)
        .fetch_optional(self.pool())
        .await
        .map_err(|error| store_error("failed to retrieve owner fulfillment", error))?;

        Ok(row.map(|row| FulfillmentView {
            fulfillment_id: string_cell(&row, "id"),
            fulfillment_no: string_cell(&row, "fulfillment_no"),
            order_id: string_cell(&row, "order_id"),
            fulfillment_type: string_cell(&row, "fulfillment_type"),
            status: string_cell(&row, "status"),
        }))
    }
}

fn store_error(message: &str, error: impl std::fmt::Display) -> CommerceServiceError {
    CommerceServiceError::storage(format!("{message}: {error}"))
}

fn optional_string_cell(row: &sqlx::postgres::PgRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column).ok().flatten()
}

fn string_cell(row: &sqlx::postgres::PgRow, column: &str) -> String {
    optional_string_cell(row, column).unwrap_or_default()
}

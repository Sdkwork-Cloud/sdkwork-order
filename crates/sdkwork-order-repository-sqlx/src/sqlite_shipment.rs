#![allow(clippy::too_many_arguments)]

use sdkwork_contract_service::CommerceServiceError;
use sdkwork_order_service::{
    ShipmentDetailQuery, ShipmentPackageListQuery, ShipmentPackageView,
    ShipmentTrackingEventListQuery, ShipmentTrackingEventView, ShipmentView,
};
use sqlx::Row;

use crate::sqlite_order::SqliteCommerceOrderStore;

impl SqliteCommerceOrderStore {
    pub async fn retrieve_owner_shipment(
        &self,
        query: ShipmentDetailQuery,
    ) -> Result<Option<ShipmentView>, CommerceServiceError> {
        let row = sqlx::query(
            r#"
            SELECT s.id, s.shipment_no, s.fulfillment_id, s.carrier_code, s.tracking_no, s.status
            FROM commerce_shipment s
            INNER JOIN commerce_fulfillment_order f
                ON f.tenant_id = s.tenant_id
               AND f.id = s.fulfillment_id
            INNER JOIN commerce_order o
                ON o.tenant_id = f.tenant_id
               AND o.id = f.order_id
            WHERE s.tenant_id = CAST(? AS TEXT)
              AND ((s.organization_id = CAST(? AS TEXT)) OR (s.organization_id IS NULL AND ? IS NULL))
              AND o.owner_user_id = CAST(? AS TEXT)
              AND s.id = CAST(? AS TEXT)
            LIMIT 1
            "#,
        )
        .bind(&query.tenant_id)
        .bind(query.organization_id.as_deref())
        .bind(query.organization_id.as_deref())
        .bind(&query.owner_user_id)
        .bind(&query.shipment_id)
        .fetch_optional(self.pool())
        .await
        .map_err(|error| store_error("failed to retrieve owner shipment", error))?;

        row.map(map_shipment_row).transpose()
    }

    pub async fn list_owner_shipment_packages(
        &self,
        query: ShipmentPackageListQuery,
    ) -> Result<Vec<ShipmentPackageView>, CommerceServiceError> {
        if self
            .retrieve_owner_shipment(ShipmentDetailQuery {
                tenant_id: query.tenant_id.clone(),
                organization_id: query.organization_id.clone(),
                owner_user_id: query.owner_user_id.clone(),
                shipment_id: query.shipment_id.clone(),
            })
            .await?
            .is_none()
        {
            return Err(CommerceServiceError::not_found("shipment was not found"));
        }

        let rows = sqlx::query(
            r#"
            SELECT id, shipment_id, package_no, package_type, tracking_no, status
            FROM commerce_shipment_package
            WHERE tenant_id = CAST(? AS TEXT)
              AND shipment_id = CAST(? AS TEXT)
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(&query.tenant_id)
        .bind(&query.shipment_id)
        .fetch_all(self.pool())
        .await
        .map_err(|error| store_error("failed to list shipment packages", error))?;

        Ok(rows
            .into_iter()
            .map(|row| ShipmentPackageView {
                package_id: string_cell(&row, "id"),
                shipment_id: string_cell(&row, "shipment_id"),
                package_no: string_cell(&row, "package_no"),
                package_type: string_cell(&row, "package_type"),
                tracking_no: optional_string_cell(&row, "tracking_no"),
                status: string_cell(&row, "status"),
            })
            .collect())
    }

    pub async fn list_owner_shipment_tracking_events(
        &self,
        query: ShipmentTrackingEventListQuery,
    ) -> Result<Vec<ShipmentTrackingEventView>, CommerceServiceError> {
        if self
            .retrieve_owner_shipment(ShipmentDetailQuery {
                tenant_id: query.tenant_id.clone(),
                organization_id: query.organization_id.clone(),
                owner_user_id: query.owner_user_id.clone(),
                shipment_id: query.shipment_id.clone(),
            })
            .await?
            .is_none()
        {
            return Err(CommerceServiceError::not_found("shipment was not found"));
        }

        let rows = sqlx::query(
            r#"
            SELECT id, shipment_id, tracking_event_no, event_type, event_status, event_time, location_text
            FROM commerce_shipment_tracking_event
            WHERE tenant_id = CAST(? AS TEXT)
              AND shipment_id = CAST(? AS TEXT)
            ORDER BY event_time ASC, id ASC
            "#,
        )
        .bind(&query.tenant_id)
        .bind(&query.shipment_id)
        .fetch_all(self.pool())
        .await
        .map_err(|error| store_error("failed to list shipment tracking events", error))?;

        Ok(rows
            .into_iter()
            .map(|row| ShipmentTrackingEventView {
                event_id: string_cell(&row, "id"),
                shipment_id: string_cell(&row, "shipment_id"),
                tracking_event_no: string_cell(&row, "tracking_event_no"),
                event_type: string_cell(&row, "event_type"),
                event_status: optional_string_cell(&row, "event_status"),
                event_time: string_cell(&row, "event_time"),
                location_text: optional_string_cell(&row, "location_text"),
            })
            .collect())
    }
}

fn map_shipment_row(row: sqlx::sqlite::SqliteRow) -> Result<ShipmentView, CommerceServiceError> {
    Ok(ShipmentView {
        shipment_id: string_cell(&row, "id"),
        shipment_no: string_cell(&row, "shipment_no"),
        fulfillment_id: string_cell(&row, "fulfillment_id"),
        carrier_code: string_cell(&row, "carrier_code"),
        tracking_no: optional_string_cell(&row, "tracking_no"),
        status: string_cell(&row, "status"),
    })
}

fn store_error(message: &str, error: impl std::fmt::Display) -> CommerceServiceError {
    CommerceServiceError::storage(format!("{message}: {error}"))
}

fn optional_string_cell(row: &sqlx::sqlite::SqliteRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column).ok().flatten()
}

fn string_cell(row: &sqlx::sqlite::SqliteRow, column: &str) -> String {
    optional_string_cell(row, column).unwrap_or_default()
}

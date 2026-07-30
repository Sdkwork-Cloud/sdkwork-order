use std::collections::HashSet;

use sdkwork_contract_service::{CommerceMoney, CommerceServiceError};
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_merchandise_repository_sqlx::{
    PostgresCommerceCatalogStore, SqliteCommerceCatalogStore,
};
use sdkwork_merchandise_service::{ProductSkuRetrieveQuery, ProductSpuRetrieveQuery, SkuRecord};
use sdkwork_order_service::{
    PhysicalCheckoutResolverPort, PhysicalPurchaseFuture, ResolvePhysicalCheckoutRequest,
    ResolvedPhysicalCheckout, ResolvedPhysicalCheckoutLine,
};
use sdkwork_shop_repository_sqlx::{PostgresCommerceShopStore, SqliteCommerceShopStore};
use sdkwork_shop_service::{ShopScopeQuery, ShopSummaryView};

enum CatalogStore {
    Sqlite(SqliteCommerceCatalogStore),
    Postgres(PostgresCommerceCatalogStore),
}

enum ShopStore {
    Sqlite(SqliteCommerceShopStore),
    Postgres(PostgresCommerceShopStore),
}

pub struct PhysicalCheckoutAdapter {
    catalog: CatalogStore,
    shops: ShopStore,
}

impl PhysicalCheckoutAdapter {
    pub fn new(merchandise_pool: DatabasePool, shop_pool: DatabasePool) -> Self {
        let catalog = match merchandise_pool {
            DatabasePool::Sqlite(pool, _) => {
                CatalogStore::Sqlite(SqliteCommerceCatalogStore::new(pool))
            }
            DatabasePool::Postgres(pool, _) => {
                CatalogStore::Postgres(PostgresCommerceCatalogStore::new(pool))
            }
        };
        let shops = match shop_pool {
            DatabasePool::Sqlite(pool, _) => ShopStore::Sqlite(SqliteCommerceShopStore::new(pool)),
            DatabasePool::Postgres(pool, _) => {
                ShopStore::Postgres(PostgresCommerceShopStore::new(pool))
            }
        };
        Self { catalog, shops }
    }

    async fn resolve(
        &self,
        request: ResolvePhysicalCheckoutRequest,
    ) -> Result<ResolvedPhysicalCheckout, CommerceServiceError> {
        if request.lines.is_empty() {
            return Err(CommerceServiceError::validation(
                "physical checkout requires at least one line",
            ));
        }

        let mut merchant_organization_id: Option<String> = None;
        let mut resolved_lines = Vec::with_capacity(request.lines.len());
        let mut requested_sku_ids = HashSet::with_capacity(request.lines.len());
        for requested in &request.lines {
            if requested.quantity <= 0 {
                return Err(CommerceServiceError::validation(
                    "physical checkout quantity must be greater than zero",
                ));
            }
            if !requested_sku_ids.insert(requested.sku_id.trim()) {
                return Err(CommerceServiceError::validation(
                    "physical checkout contains duplicate SKU lines",
                ));
            }
            let sku = self
                .retrieve_sku(&request.tenant_id, &requested.sku_id)
                .await?
                .ok_or_else(|| CommerceServiceError::not_found("checkout SKU was not found"))?;
            validate_sku(&sku, &request.currency_code)?;
            let seller = sku
                .organization_id
                .clone()
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    CommerceServiceError::conflict("physical SKU has no merchant organization")
                })?;
            if merchant_organization_id
                .as_deref()
                .is_some_and(|existing| existing != seller)
            {
                return Err(CommerceServiceError::conflict(
                    "cross-shop checkout is not supported",
                ));
            }
            merchant_organization_id = Some(seller.clone());

            let spu = self
                .retrieve_spu(&request.tenant_id, &sku.spu_id)
                .await?
                .ok_or_else(|| CommerceServiceError::not_found("checkout product was not found"))?;
            if !spu.status.eq_ignore_ascii_case("active") {
                return Err(CommerceServiceError::conflict(
                    "checkout product is not active",
                ));
            }
            if !matches!(
                spu.product_type.trim().to_ascii_lowercase().as_str(),
                "physical" | "physical_goods" | "physical_shipment"
            ) {
                return Err(CommerceServiceError::conflict(
                    "checkout product is not a physical product",
                ));
            }

            resolved_lines.push((sku, requested.quantity));
        }

        let merchant_organization_id = merchant_organization_id.ok_or_else(|| {
            CommerceServiceError::conflict("physical checkout merchant is unavailable")
        })?;
        let shop = self
            .retrieve_current_shop(&request.tenant_id, &merchant_organization_id)
            .await?
            .ok_or_else(|| CommerceServiceError::not_found("merchant shop was not found"))?;
        validate_shop(&shop, &request.currency_code)?;

        let shop_snapshot_json = serde_json::json!({
            "shopId": shop.shop_id,
            "shopNo": shop.shop_no,
            "shopName": shop.shop_name,
            "merchantOrganizationId": shop.organization_id,
            "storefrontStatus": shop.storefront_status,
            "operationStatus": shop.operation_status,
            "reviewStatus": shop.review_status,
            "currencyCode": shop.default_currency_code,
            "version": shop.version,
        })
        .to_string();

        let lines = resolved_lines
            .into_iter()
            .map(|(sku, quantity)| {
                let title = if sku.title.trim().is_empty() {
                    sku.name.clone()
                } else {
                    sku.title.clone()
                };
                let sku_snapshot_json = serde_json::json!({
                    "skuId": sku.id,
                    "skuNo": sku.sku_no,
                    "productId": sku.spu_id,
                    "title": title,
                    "merchantOrganizationId": merchant_organization_id,
                    "shopId": shop.shop_id,
                    "priceAmount": sku.price_amount,
                    "currencyCode": sku.currency_code,
                    "fulfillmentType": sku.fulfillment_type,
                    "inventoryTracking": sku.inventory_tracking,
                    "spec": sku.spec_json,
                    "publishedAt": sku.published_at,
                    "versionAt": sku.updated_at,
                })
                .to_string();
                Ok(ResolvedPhysicalCheckoutLine {
                    sku_id: sku.id,
                    product_id: sku.spu_id,
                    merchant_organization_id: merchant_organization_id.clone(),
                    shop_id: shop.shop_id.clone(),
                    title,
                    unit_price: CommerceMoney::new(&sku.price_amount).map_err(|error| {
                        CommerceServiceError::validation(format!(
                            "physical SKU price is invalid: {error}"
                        ))
                    })?,
                    currency_code: sku.currency_code.to_ascii_uppercase(),
                    fulfillment_type: sku.fulfillment_type,
                    quantity,
                    inventory_tracking: sku.inventory_tracking,
                    sku_snapshot_json,
                })
            })
            .collect::<Result<Vec<_>, CommerceServiceError>>()?;

        Ok(ResolvedPhysicalCheckout {
            merchant_organization_id,
            shop_id: shop.shop_id,
            shop_snapshot_json,
            shipping_address: request.shipping_address,
            lines,
        })
    }

    async fn retrieve_sku(
        &self,
        tenant_id: &str,
        sku_id: &str,
    ) -> Result<Option<SkuRecord>, CommerceServiceError> {
        let query = ProductSkuRetrieveQuery {
            tenant_id: tenant_id.to_owned(),
            sku_id: sku_id.to_owned(),
        };
        match &self.catalog {
            CatalogStore::Sqlite(store) => store.retrieve_sku(&query).await,
            CatalogStore::Postgres(store) => store.retrieve_sku(&query).await,
        }
    }

    async fn retrieve_spu(
        &self,
        tenant_id: &str,
        spu_id: &str,
    ) -> Result<Option<sdkwork_merchandise_service::SpuRecord>, CommerceServiceError> {
        let query = ProductSpuRetrieveQuery {
            tenant_id: tenant_id.to_owned(),
            spu_id: spu_id.to_owned(),
        };
        match &self.catalog {
            CatalogStore::Sqlite(store) => store.retrieve_spu(&query).await,
            CatalogStore::Postgres(store) => store.retrieve_spu(&query).await,
        }
    }

    async fn retrieve_current_shop(
        &self,
        tenant_id: &str,
        organization_id: &str,
    ) -> Result<Option<ShopSummaryView>, CommerceServiceError> {
        let scope = ShopScopeQuery::new(tenant_id, Some(organization_id))?;
        match &self.shops {
            ShopStore::Sqlite(store) => store.retrieve_current_shop(scope).await,
            ShopStore::Postgres(store) => store.retrieve_current_shop(scope).await,
        }
    }
}

impl PhysicalCheckoutResolverPort for PhysicalCheckoutAdapter {
    fn resolve_physical_checkout<'a>(
        &'a self,
        request: ResolvePhysicalCheckoutRequest,
    ) -> PhysicalPurchaseFuture<'a, ResolvedPhysicalCheckout> {
        Box::pin(async move { self.resolve(request).await })
    }
}

fn validate_sku(sku: &SkuRecord, currency_code: &str) -> Result<(), CommerceServiceError> {
    if !sku.status.eq_ignore_ascii_case("active") {
        return Err(CommerceServiceError::conflict("checkout SKU is not active"));
    }
    if !matches!(
        sku.fulfillment_type.trim().to_ascii_lowercase().as_str(),
        "physical" | "physical_shipment"
    ) {
        return Err(CommerceServiceError::conflict(
            "checkout SKU is not physically shippable",
        ));
    }
    if matches!(
        sku.inventory_tracking.trim().to_ascii_lowercase().as_str(),
        "none" | "disabled" | "untracked"
    ) {
        return Err(CommerceServiceError::conflict(
            "physical SKU must enable inventory tracking",
        ));
    }
    if !sku.currency_code.eq_ignore_ascii_case(currency_code) {
        return Err(CommerceServiceError::conflict(
            "checkout SKU currency does not match checkout currency",
        ));
    }
    CommerceMoney::new(&sku.price_amount).map_err(|error| {
        CommerceServiceError::validation(format!("physical SKU price is invalid: {error}"))
    })?;
    Ok(())
}

fn validate_shop(shop: &ShopSummaryView, currency_code: &str) -> Result<(), CommerceServiceError> {
    if !shop.operation_status.eq_ignore_ascii_case("active") {
        return Err(CommerceServiceError::conflict(
            "merchant shop is not operational",
        ));
    }
    if !matches!(
        shop.storefront_status.trim().to_ascii_lowercase().as_str(),
        "active" | "open" | "published"
    ) {
        return Err(CommerceServiceError::conflict(
            "merchant storefront is not open",
        ));
    }
    if !matches!(
        shop.review_status.trim().to_ascii_lowercase().as_str(),
        "approved" | "passed" | "active"
    ) {
        return Err(CommerceServiceError::conflict(
            "merchant shop has not passed review",
        ));
    }
    if !shop
        .default_currency_code
        .eq_ignore_ascii_case(currency_code)
    {
        return Err(CommerceServiceError::conflict(
            "merchant shop currency does not match checkout currency",
        ));
    }
    Ok(())
}

use sdkwork_commerce_promotion_repository_sqlx::{
    PostgresCommercePromotionStore, SqliteCommercePromotionStore,
};
use sdkwork_commerce_promotion_service::{
    PromotionCodeRedemptionCommand, PromotionOrderCouponBenefit,
};
use sdkwork_contract_service::{CommerceMoney, CommerceServiceError};
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_order_service::{
    AccountValueAssetCode, AccountValueFuture, CouponRedemptionOutcome, CouponRedemptionPort,
    CouponRedemptionRequest,
};
use std::sync::Arc;

#[derive(Clone)]
enum PromotionStore {
    Postgres(PostgresCommercePromotionStore),
    Sqlite(SqliteCommercePromotionStore),
}

#[derive(Clone)]
pub struct PromotionCouponRedemptionAdapter {
    store: PromotionStore,
}

impl PromotionCouponRedemptionAdapter {
    pub fn from_database_pool(pool: &DatabasePool) -> Self {
        let store = match pool {
            DatabasePool::Postgres(pool, _) => {
                PromotionStore::Postgres(PostgresCommercePromotionStore::new(pool.clone()))
            }
            DatabasePool::Sqlite(pool, _) => {
                PromotionStore::Sqlite(SqliteCommercePromotionStore::new(pool.clone()))
            }
        };
        Self { store }
    }

    async fn preview(
        &self,
        request: CouponRedemptionRequest,
    ) -> Result<CouponRedemptionOutcome, CommerceServiceError> {
        let command = promotion_command(&request)?;
        let benefit = match &self.store {
            PromotionStore::Postgres(store) => {
                store.preview_promotion_code_for_order(command).await?
            }
            PromotionStore::Sqlite(store) => {
                store.preview_promotion_code_for_order(command).await?
            }
        };
        map_benefit(benefit)
    }

    async fn redeem(
        &self,
        request: CouponRedemptionRequest,
    ) -> Result<CouponRedemptionOutcome, CommerceServiceError> {
        let command = promotion_command(&request)?;
        let benefit = match &self.store {
            PromotionStore::Postgres(store) => {
                store.redeem_promotion_code_for_order(command).await?
            }
            PromotionStore::Sqlite(store) => store.redeem_promotion_code_for_order(command).await?,
        };
        map_benefit(benefit)
    }
}

impl CouponRedemptionPort for PromotionCouponRedemptionAdapter {
    fn preview_coupon<'a>(
        &'a self,
        request: CouponRedemptionRequest,
    ) -> AccountValueFuture<'a, CouponRedemptionOutcome> {
        Box::pin(async move { self.preview(request).await })
    }

    fn redeem_coupon<'a>(
        &'a self,
        request: CouponRedemptionRequest,
    ) -> AccountValueFuture<'a, CouponRedemptionOutcome> {
        Box::pin(async move { self.redeem(request).await })
    }
}

pub fn promotion_coupon_redemption_port_from_database_pool(
    pool: &DatabasePool,
) -> Arc<dyn CouponRedemptionPort> {
    Arc::new(PromotionCouponRedemptionAdapter::from_database_pool(pool))
}

fn promotion_command(
    request: &CouponRedemptionRequest,
) -> Result<PromotionCodeRedemptionCommand, CommerceServiceError> {
    PromotionCodeRedemptionCommand::new(
        &request.tenant_id,
        request.organization_id.as_deref(),
        &request.owner_user_id,
        &request.coupon_code,
        &request.order_id,
        &request.idempotency_key,
    )
}

fn map_benefit(
    benefit: PromotionOrderCouponBenefit,
) -> Result<CouponRedemptionOutcome, CommerceServiceError> {
    let grant_amount = CommerceMoney::new(&benefit.grant_units.to_string())
        .map_err(CommerceServiceError::validation)?;
    Ok(CouponRedemptionOutcome {
        accepted: true,
        replayed: benefit.replayed,
        target_asset: AccountValueAssetCode::TokenBank,
        grant_amount,
    })
}

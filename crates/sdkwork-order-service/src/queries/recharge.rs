use sdkwork_contract_service::CommerceServiceError;

use crate::{AccountValueAssetCode, AccountValueOrderSubject};

/// 充值套餐分页结果。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RechargePackageListPage {
    pub items: Vec<crate::RechargePackageItem>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

impl RechargePackageListPage {
    pub fn empty_for(query: &RechargePackageListQuery) -> Self {
        Self {
            items: Vec::new(),
            page: query.page,
            page_size: query.page_size,
            total: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RechargePackageListQuery {
    pub organization_id: Option<String>,
    pub tenant_id: String,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RechargeSettingsQuery {
    pub organization_id: Option<String>,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckoutStatusQuery {
    pub order_no: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountValueCatalogListQuery {
    pub organization_id: Option<String>,
    pub tenant_id: String,
    pub target_asset: Option<AccountValueAssetCode>,
    pub status: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountValueRequestListQuery {
    pub organization_id: Option<String>,
    pub owner_user_id: Option<String>,
    pub tenant_id: String,
    pub subject: Option<AccountValueOrderSubject>,
    pub status: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountValueRequestDetailQuery {
    pub organization_id: Option<String>,
    pub owner_user_id: Option<String>,
    pub tenant_id: String,
    pub subject: Option<AccountValueOrderSubject>,
    pub request_id: String,
}

impl RechargePackageListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        let (page, page_size) = crate::validation::offset_list_params(page, page_size)?;
        Ok(Self {
            organization_id: optional_text(organization_id),
            tenant_id: tenant_id.trim().to_string(),
            page,
            page_size,
        })
    }

    pub fn limit(&self) -> i64 {
        self.page_size
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }
}

impl RechargeSettingsQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        Ok(Self {
            organization_id: optional_text(organization_id),
            tenant_id: tenant_id.trim().to_string(),
        })
    }
}

impl CheckoutStatusQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        order_no: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("order_no", order_no)?;
        Ok(Self {
            order_no: order_no.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
        })
    }
}

impl AccountValueCatalogListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        target_asset: Option<AccountValueAssetCode>,
        status: Option<&str>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        let (page, page_size) = crate::validation::offset_list_params(page, page_size)?;
        Ok(Self {
            organization_id: optional_text(organization_id),
            tenant_id: tenant_id.trim().to_string(),
            target_asset,
            status: optional_text(status).map(|value| value.to_ascii_lowercase()),
            page,
            page_size,
        })
    }

    pub fn limit(&self) -> i64 {
        self.page_size
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }
}

impl AccountValueRequestListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: Option<&str>,
        subject: Option<AccountValueOrderSubject>,
        status: Option<&str>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        let (page, page_size) = crate::validation::offset_list_params(page, page_size)?;
        Ok(Self {
            organization_id: optional_text(organization_id),
            owner_user_id: optional_text(owner_user_id),
            tenant_id: tenant_id.trim().to_string(),
            subject,
            status: optional_text(status).map(|value| value.to_ascii_lowercase()),
            page,
            page_size,
        })
    }

    pub fn limit(&self) -> i64 {
        self.page_size
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }
}

impl AccountValueRequestDetailQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: Option<&str>,
        subject: Option<AccountValueOrderSubject>,
        request_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("request_id", request_id)?;
        Ok(Self {
            organization_id: optional_text(organization_id),
            owner_user_id: optional_text(owner_user_id),
            tenant_id: tenant_id.trim().to_string(),
            subject,
            request_id: request_id.trim().to_string(),
        })
    }
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_list_query_rejects_invalid_page_size() {
        assert!(RechargePackageListQuery::new("tenant-1", None, Some(1), Some(1000)).is_err());
        assert!(RechargePackageListQuery::new("tenant-1", None, Some(1), Some(0)).is_err());
    }

    #[test]
    fn package_list_query_rejects_empty_tenant() {
        assert!(RechargePackageListQuery::new("", None, None, None).is_err());
    }

    #[test]
    fn settings_query_rejects_empty_tenant() {
        assert!(RechargeSettingsQuery::new("", None).is_err());
    }
}

export interface ShopSummary {
  id: string;
  tenantId: string;
  organizationId: string;
  shopNo: string;
  shopName: string;
  shopType: string;
  businessModel: string;
  storefrontStatus: string;
  operationStatus: string;
  reviewStatus: string;
  dataScope: string;
  logoMediaResourceId?: string;
  coverMediaResourceId?: string;
  defaultCurrencyCode: string;
  defaultLocale?: string;
  timezone?: string;
  version: number;
  createdAt: string;
  updatedAt: string;
}

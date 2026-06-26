export interface ShopFulfillmentProfile {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  fulfillmentMode: string;
  shippingOriginRegionCode?: string;
  serviceLevelCode?: string;
  afterSalesPolicy: Record<string, unknown>;
  serviceConfig: Record<string, unknown>;
  createdAt: string;
  updatedAt: string;
}

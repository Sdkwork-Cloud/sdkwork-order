export interface UpdateShopFulfillmentProfileRequest {
  fulfillmentMode?: string;
  shippingOriginRegionCode?: string;
  serviceLevelCode?: string;
  afterSalesPolicy?: Record<string, unknown>;
  serviceConfig?: Record<string, unknown>;
}

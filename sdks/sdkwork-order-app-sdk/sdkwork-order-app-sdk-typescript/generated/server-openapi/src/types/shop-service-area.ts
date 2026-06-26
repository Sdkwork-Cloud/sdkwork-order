export interface ShopServiceArea {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  areaType: string;
  countryCode: string;
  regionCode?: string;
  cityCode?: string;
  postalCodePattern?: string;
  deliveryRadiusMeters?: number;
  serviceStatus: string;
  serviceConfig: Record<string, unknown>;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

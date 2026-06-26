export interface CreateShopServiceAreaRequest {
  areaType: string;
  countryCode: string;
  regionCode?: string;
  cityCode?: string;
  postalCodePattern?: string;
  deliveryRadiusMeters?: number;
  serviceStatus: string;
  serviceConfig?: Record<string, unknown>;
  sortOrder?: number;
}

export interface UpsertShopReturnAddressRequest {
  addressUsage: string;
  addressKey: string;
  receiverName: string;
  phoneHash?: string;
  countryCode: string;
  regionCode?: string;
  cityCode?: string;
  districtCode?: string;
  addressLine1: string;
  postalCode?: string;
  isDefault: boolean;
  addressStatus: string;
  addressSnapshot: Record<string, unknown>;
}

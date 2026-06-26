export interface UpsertShopBrandAuthorizationRequest {
  brandCode: string;
  brandName: string;
  authorizationType: string;
  authorizationStatus: string;
  brandOwnerName?: string;
  trademarkHash?: string;
  trademarkMediaResourceId?: string;
  authorizationMediaResourceId?: string;
  authorizationSnapshot: Record<string, unknown>;
  validFrom?: string;
  validTo?: string;
}

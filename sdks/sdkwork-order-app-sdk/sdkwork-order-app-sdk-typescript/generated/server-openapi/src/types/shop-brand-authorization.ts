export interface ShopBrandAuthorization {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
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
  reviewedBy?: string;
  reviewedAt?: string;
  createdAt: string;
  updatedAt: string;
}

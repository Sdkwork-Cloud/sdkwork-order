export interface ShopVerification {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  verificationType: string;
  verificationStatus: string;
  legalEntityName?: string;
  credentialNoHash?: string;
  credentialMediaResourceId?: string;
  verificationSnapshot: Record<string, unknown>;
  expiresAt?: string;
  reviewedBy?: string;
  reviewedAt?: string;
  createdAt: string;
  updatedAt: string;
}

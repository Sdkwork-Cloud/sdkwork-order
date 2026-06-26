export interface ShopPolicy {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  policyType: string;
  policyStatus: string;
  policyVersion: number;
  policy: Record<string, unknown>;
  publishedAt?: string;
  reviewedBy?: string;
  reviewedAt?: string;
  createdAt: string;
  updatedAt: string;
}

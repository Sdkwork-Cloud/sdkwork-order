export interface ShopQualification {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  qualificationType: string;
  qualificationStatus: string;
  subjectType: string;
  subjectId: string;
  credentialName?: string;
  credentialHash?: string;
  credentialMediaResourceId?: string;
  qualificationSnapshot: Record<string, unknown>;
  issuedAt?: string;
  expiresAt?: string;
  reviewedBy?: string;
  reviewedAt?: string;
  createdAt: string;
  updatedAt: string;
}

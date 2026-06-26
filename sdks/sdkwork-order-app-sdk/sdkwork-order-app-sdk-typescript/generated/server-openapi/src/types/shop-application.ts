export interface ShopApplication {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  applicationNo: string;
  applicationType: string;
  reviewStatus: string;
  legalEntitySnapshot: Record<string, unknown>;
  contactSnapshot: Record<string, unknown>;
  qualificationSnapshot: Record<string, unknown>;
  submittedBy: string;
  submittedAt: string;
  reviewedBy?: string;
  reviewedAt?: string;
  reviewComment?: string;
  createdAt: string;
  updatedAt: string;
}

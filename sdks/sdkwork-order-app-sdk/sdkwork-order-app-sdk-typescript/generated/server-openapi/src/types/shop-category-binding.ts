export interface ShopCategoryBinding {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  shopCategoryCode: string;
  platformCategoryCode?: string;
  platformCategoryName?: string;
  categoryPath?: string;
  categoryLevel: number;
  categoryStatus: string;
  qualificationRequired: boolean;
  qualificationSnapshot: Record<string, unknown>;
  reviewStatus: string;
  reviewedBy?: string;
  reviewedAt?: string;
  effectiveFrom?: string;
  effectiveTo?: string;
  createdAt: string;
  updatedAt: string;
}

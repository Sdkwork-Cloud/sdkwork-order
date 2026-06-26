export interface UpsertShopCategoryBindingRequest {
  shopCategoryCode: string;
  platformCategoryCode?: string;
  platformCategoryName?: string;
  categoryPath?: string;
  categoryLevel?: number;
  categoryStatus: string;
  qualificationRequired: boolean;
  qualificationSnapshot: Record<string, unknown>;
  reviewStatus: string;
  effectiveFrom?: string;
  effectiveTo?: string;
}

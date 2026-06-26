import type { ShopCategoryBinding } from './shop-category-binding';

export interface ShopCategoryBindingListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

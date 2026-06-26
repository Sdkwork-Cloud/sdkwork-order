import type { ShopCategoryBinding } from './shop-category-binding';

export interface ShopCategoryBindingResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopCategoryBinding;
}

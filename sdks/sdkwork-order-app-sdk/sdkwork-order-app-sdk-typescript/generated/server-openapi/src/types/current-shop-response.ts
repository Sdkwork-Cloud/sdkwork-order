import type { ShopDetail } from './shop-detail';

export interface CurrentShopResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopDetail;
}

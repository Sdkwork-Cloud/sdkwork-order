import type { ShopServiceArea } from './shop-service-area';

export interface ShopServiceAreaResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopServiceArea;
}

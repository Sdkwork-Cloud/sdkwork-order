import type { ShopServiceArea } from './shop-service-area';

export interface ShopServiceAreaListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

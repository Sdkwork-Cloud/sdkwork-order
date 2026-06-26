import type { ShopReadiness } from './shop-readiness';

export interface ShopReadinessResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopReadiness;
}

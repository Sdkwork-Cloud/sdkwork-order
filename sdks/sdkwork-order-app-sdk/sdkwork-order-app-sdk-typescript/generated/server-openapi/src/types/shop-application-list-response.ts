import type { ShopApplication } from './shop-application';

export interface ShopApplicationListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

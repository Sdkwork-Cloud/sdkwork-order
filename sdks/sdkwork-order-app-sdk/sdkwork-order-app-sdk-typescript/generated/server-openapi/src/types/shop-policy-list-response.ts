import type { ShopPolicy } from './shop-policy';

export interface ShopPolicyListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

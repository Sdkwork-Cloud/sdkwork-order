import type { ShopPolicy } from './shop-policy';

export interface ShopPolicyResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopPolicy;
}

import type { ShopVerification } from './shop-verification';

export interface ShopVerificationListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

import type { ShopQualification } from './shop-qualification';

export interface ShopQualificationListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

import type { ShopQualification } from './shop-qualification';

export interface ShopQualificationResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopQualification;
}

import type { ShopApplication } from './shop-application';

export interface ShopApplicationResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopApplication;
}

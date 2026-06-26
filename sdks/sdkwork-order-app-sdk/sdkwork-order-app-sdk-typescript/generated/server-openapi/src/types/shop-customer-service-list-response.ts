import type { ShopCustomerService } from './shop-customer-service';

export interface ShopCustomerServiceListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

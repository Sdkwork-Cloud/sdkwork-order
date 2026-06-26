import type { ShopCustomerService } from './shop-customer-service';

export interface ShopCustomerServiceResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopCustomerService;
}

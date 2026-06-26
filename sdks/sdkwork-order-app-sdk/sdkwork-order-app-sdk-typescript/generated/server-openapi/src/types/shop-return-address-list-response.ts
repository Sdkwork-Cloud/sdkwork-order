import type { ShopReturnAddress } from './shop-return-address';

export interface ShopReturnAddressListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

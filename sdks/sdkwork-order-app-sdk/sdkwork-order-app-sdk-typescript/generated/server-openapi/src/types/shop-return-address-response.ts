import type { ShopReturnAddress } from './shop-return-address';

export interface ShopReturnAddressResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopReturnAddress;
}

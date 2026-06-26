import type { ShopChannel } from './shop-channel';

export interface ShopChannelResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopChannel;
}

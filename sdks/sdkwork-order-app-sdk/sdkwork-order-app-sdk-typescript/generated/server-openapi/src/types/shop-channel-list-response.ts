import type { ShopChannel } from './shop-channel';

export interface ShopChannelListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

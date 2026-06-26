import type { ShopStatusEvent } from './shop-status-event';

export interface ShopStatusEventListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

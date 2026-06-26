import type { ShopRiskSignal } from './shop-risk-signal';

export interface ShopRiskSignalListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

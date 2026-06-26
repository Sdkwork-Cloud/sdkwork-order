import type { ShopMetricSnapshot } from './shop-metric-snapshot';
import type { ShopSummary } from './shop-summary';

export interface ShopDashboardResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

import type { OrderSummary } from './order-summary';
import type { PageInfo } from './page-info';

export interface OrdersAdminListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}

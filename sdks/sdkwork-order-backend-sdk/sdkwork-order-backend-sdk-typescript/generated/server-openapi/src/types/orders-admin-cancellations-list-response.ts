import type { OrderCancellation } from './order-cancellation';
import type { PageInfo } from './page-info';

export interface OrdersAdminCancellationsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}

import type { OrderEvent } from './order-event';
import type { PageInfo } from './page-info';

export interface OrdersAdminEventsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}

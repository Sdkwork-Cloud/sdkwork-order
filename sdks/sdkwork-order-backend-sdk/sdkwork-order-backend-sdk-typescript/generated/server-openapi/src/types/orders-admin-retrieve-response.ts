import type { OrderDetail } from './order-detail';

export interface OrdersAdminRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}

import type { AfterSalesEvent } from './after-sales-event';

export interface AfterSalesEventListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

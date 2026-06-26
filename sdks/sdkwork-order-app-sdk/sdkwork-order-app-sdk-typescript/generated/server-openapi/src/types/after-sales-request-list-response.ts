import type { AfterSalesRequest } from './after-sales-request';

export interface AfterSalesRequestListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

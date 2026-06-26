import type { AfterSalesRequest } from './after-sales-request';

export interface AfterSalesRequestResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: AfterSalesRequest;
}

import type { AfterSalesRequestResponse } from './after-sales-request-response';

export interface AfterSalesRequestsCreateResponse201 {
  code: 0;
  data: unknown & { item: AfterSalesRequestResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}

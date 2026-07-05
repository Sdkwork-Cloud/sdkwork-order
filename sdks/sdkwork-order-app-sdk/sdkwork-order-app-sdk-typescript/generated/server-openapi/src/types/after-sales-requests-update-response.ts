import type { AfterSalesRequestResponse } from './after-sales-request-response';

export interface AfterSalesRequestsUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}

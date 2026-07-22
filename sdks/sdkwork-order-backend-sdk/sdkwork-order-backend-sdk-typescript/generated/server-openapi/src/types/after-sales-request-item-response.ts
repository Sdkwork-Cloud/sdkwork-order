import type { AfterSalesRequestSummary } from './after-sales-request-summary';

export interface AfterSalesRequestItemResponse {
  code: 0;
  data: unknown & { item: AfterSalesRequestSummary; };
  /** Server-owned request correlation id. */
  traceId: string;
}

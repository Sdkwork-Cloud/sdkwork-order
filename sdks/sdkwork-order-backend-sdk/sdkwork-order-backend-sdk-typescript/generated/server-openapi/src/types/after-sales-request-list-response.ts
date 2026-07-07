import type { AfterSalesRequestSummary } from './after-sales-request-summary';
import type { PageInfo } from './page-info';

export interface AfterSalesRequestListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}

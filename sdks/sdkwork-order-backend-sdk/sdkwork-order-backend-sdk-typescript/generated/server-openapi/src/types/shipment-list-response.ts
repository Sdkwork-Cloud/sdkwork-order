import type { PageInfo } from './page-info';
import type { ShipmentSummary } from './shipment-summary';

export interface ShipmentListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}

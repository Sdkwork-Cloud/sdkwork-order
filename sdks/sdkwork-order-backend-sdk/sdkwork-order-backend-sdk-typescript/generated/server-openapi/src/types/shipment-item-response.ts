import type { ShipmentSummary } from './shipment-summary';

export interface ShipmentItemResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}

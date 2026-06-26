import type { AfterSalesReturnShipment } from './after-sales-return-shipment';

export interface AfterSalesReturnShipmentListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

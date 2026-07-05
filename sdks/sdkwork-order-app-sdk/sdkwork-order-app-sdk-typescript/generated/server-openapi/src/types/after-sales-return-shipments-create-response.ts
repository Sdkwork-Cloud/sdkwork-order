import type { AfterSalesReturnShipmentResponse } from './after-sales-return-shipment-response';

export interface AfterSalesReturnShipmentsCreateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}

import type { AfterSalesReturnShipmentResponse } from './after-sales-return-shipment-response';

export interface AfterSalesReturnShipmentsCreateResponse201 {
  code: 0;
  data: unknown & { item: AfterSalesReturnShipmentResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}

import type { AfterSalesReturnShipment } from './after-sales-return-shipment';

export interface AfterSalesReturnShipmentResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: AfterSalesReturnShipment;
}

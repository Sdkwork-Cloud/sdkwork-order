import type { ShipmentPackageSummary } from './shipment-package-summary';

export interface ShipmentPackageItemResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}

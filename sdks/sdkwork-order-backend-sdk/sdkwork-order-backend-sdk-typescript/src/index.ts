import {
  createClient as createGeneratedBackendClient,
  SdkworkOrderBackendClient as GeneratedSdkworkOrderBackendClient,
} from '../generated/server-openapi/src/index';
import type { SdkworkBackendConfig } from '../generated/server-openapi/src/types/common';

export {
  createGeneratedBackendClient,
};
export type { SdkworkBackendConfig };
export * from '../generated/server-openapi/src/types';
export * from '../generated/server-openapi/src/api';
export * from '../generated/server-openapi/src/http';
export * from '../generated/server-openapi/src/auth';

export class SdkworkOrderBackendClient extends GeneratedSdkworkOrderBackendClient {
  public readonly afterSales: GeneratedSdkworkOrderBackendClient["orderAdminAfterSales"]["afterSales"];
  public readonly backend: GeneratedSdkworkOrderBackendClient["orderAdminBackend"]["backend"];
  public readonly orders: GeneratedSdkworkOrderBackendClient["orderAdminOrders"]["orders"];
  public readonly shipments: GeneratedSdkworkOrderBackendClient["orderAdminShipments"]["shipments"];

  constructor(config: SdkworkBackendConfig) {
    super(config);
    this.afterSales = this.orderAdminAfterSales.afterSales;
    this.backend = this.orderAdminBackend.backend;
    this.orders = this.orderAdminOrders.orders;
    this.shipments = this.orderAdminShipments.shipments;
  }
}

export { SdkworkOrderBackendClient as SdkworkBackendClient };

export function createClient(config: SdkworkBackendConfig): SdkworkOrderBackendClient {
  return new SdkworkOrderBackendClient(config);
}

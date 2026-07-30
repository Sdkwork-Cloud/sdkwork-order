import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkBackendConfig } from './types/common';
import type { AuthTokenManager } from '@sdkwork/sdk-common';

import { OrderAdminOrdersApi, createOrderAdminOrdersApi } from './api/order-admin-orders';
import { OrderAdminAfterSalesApi, createOrderAdminAfterSalesApi } from './api/order-admin-after-sales';
import { OrderAdminShipmentsApi, createOrderAdminShipmentsApi } from './api/order-admin-shipments';
import { OrderAdminBackendApi, createOrderAdminBackendApi } from './api/order-admin-backend';

export class SdkworkOrderBackendClient {
  private httpClient: HttpClient;

  public readonly orderAdminOrders: OrderAdminOrdersApi;
  public readonly orderAdminAfterSales: OrderAdminAfterSalesApi;
  public readonly orderAdminShipments: OrderAdminShipmentsApi;
  public readonly orderAdminBackend: OrderAdminBackendApi;

  constructor(config: SdkworkBackendConfig) {
    this.httpClient = createHttpClient(config);
    this.orderAdminOrders = createOrderAdminOrdersApi(this.httpClient);

    this.orderAdminAfterSales = createOrderAdminAfterSalesApi(this.httpClient);

    this.orderAdminShipments = createOrderAdminShipmentsApi(this.httpClient);

    this.orderAdminBackend = createOrderAdminBackendApi(this.httpClient);
  }
  setAuthToken(token: string): this {
    this.httpClient.setAuthToken(token);
    return this;
  }

  setAccessToken(token: string): this {
    this.httpClient.setAccessToken(token);
    return this;
  }

  setTokenManager(manager: AuthTokenManager): this {
    this.httpClient.setTokenManager(manager);
    return this;
  }

  get http(): HttpClient {
    return this.httpClient;
  }
}

export function createClient(config: SdkworkBackendConfig): SdkworkOrderBackendClient {
  return new SdkworkOrderBackendClient(config);
}

export default SdkworkOrderBackendClient;

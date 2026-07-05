import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkBackendConfig } from './types/common';
import type { AuthTokenManager } from '@sdkwork/sdk-common';

import { OrdersApi, createOrdersApi } from './api/orders';
import { FulfillmentsApi, createFulfillmentsApi } from './api/fulfillments';

export class SdkworkOrderBackendClient {
  private httpClient: HttpClient;

  public readonly orders: OrdersApi;
  public readonly fulfillments: FulfillmentsApi;

  constructor(config: SdkworkBackendConfig) {
    this.httpClient = createHttpClient(config);
    this.orders = createOrdersApi(this.httpClient);

    this.fulfillments = createFulfillmentsApi(this.httpClient);
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

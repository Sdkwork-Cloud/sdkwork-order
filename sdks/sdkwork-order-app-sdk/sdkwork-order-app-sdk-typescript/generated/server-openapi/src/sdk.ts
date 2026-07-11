import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkAppConfig } from './types/common';
import type { AuthTokenManager } from '@sdkwork/sdk-common';

import { CheckoutApi, createCheckoutApi } from './api/checkout';
import { OrdersApi, createOrdersApi } from './api/orders';
import { PaymentsApi, createPaymentsApi } from './api/payments';
import { AfterSalesApi, createAfterSalesApi } from './api/after-sales';
import { FulfillmentsApi, createFulfillmentsApi } from './api/fulfillments';
import { ShipmentsApi, createShipmentsApi } from './api/shipments';
import { RechargesApi, createRechargesApi } from './api/recharges';
import { MembershipsApi, createMembershipsApi } from './api/memberships';
import { WithdrawalsApi, createWithdrawalsApi } from './api/withdrawals';

export class SdkworkAppClient {
  private httpClient: HttpClient;

  public readonly checkout: CheckoutApi;
  public readonly orders: OrdersApi;
  public readonly payments: PaymentsApi;
  public readonly afterSales: AfterSalesApi;
  public readonly fulfillments: FulfillmentsApi;
  public readonly shipments: ShipmentsApi;
  public readonly recharges: RechargesApi;
  public readonly memberships: MembershipsApi;
  public readonly withdrawals: WithdrawalsApi;

  constructor(config: SdkworkAppConfig) {
    this.httpClient = createHttpClient(config);
    this.checkout = createCheckoutApi(this.httpClient);

    this.orders = createOrdersApi(this.httpClient);

    this.payments = createPaymentsApi(this.httpClient);

    this.afterSales = createAfterSalesApi(this.httpClient);

    this.fulfillments = createFulfillmentsApi(this.httpClient);

    this.shipments = createShipmentsApi(this.httpClient);

    this.recharges = createRechargesApi(this.httpClient);

    this.memberships = createMembershipsApi(this.httpClient);

    this.withdrawals = createWithdrawalsApi(this.httpClient);
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

export function createClient(config: SdkworkAppConfig): SdkworkAppClient {
  return new SdkworkAppClient(config);
}

export default SdkworkAppClient;

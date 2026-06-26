import type { ShopFulfillmentProfile } from './shop-fulfillment-profile';

export interface ShopFulfillmentProfileResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopFulfillmentProfile;
}

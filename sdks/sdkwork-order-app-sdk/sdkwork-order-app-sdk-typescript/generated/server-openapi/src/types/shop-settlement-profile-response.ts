import type { ShopSettlementProfile } from './shop-settlement-profile';

export interface ShopSettlementProfileResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopSettlementProfile;
}

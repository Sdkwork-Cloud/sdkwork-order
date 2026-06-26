import type { ShopBusinessHour } from './shop-business-hour';

export interface ShopBusinessHourResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopBusinessHour;
}

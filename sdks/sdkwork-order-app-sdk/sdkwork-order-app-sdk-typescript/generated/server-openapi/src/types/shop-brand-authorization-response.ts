import type { ShopBrandAuthorization } from './shop-brand-authorization';

export interface ShopBrandAuthorizationResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopBrandAuthorization;
}

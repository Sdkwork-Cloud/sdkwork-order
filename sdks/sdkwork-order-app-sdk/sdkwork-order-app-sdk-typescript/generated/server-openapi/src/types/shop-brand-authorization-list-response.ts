import type { ShopBrandAuthorization } from './shop-brand-authorization';

export interface ShopBrandAuthorizationListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

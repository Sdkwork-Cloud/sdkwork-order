import type { ShopShippingTemplate } from './shop-shipping-template';

export interface ShopShippingTemplateListResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: Record<string, unknown>;
}

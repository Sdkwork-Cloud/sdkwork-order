import type { ShopShippingTemplate } from './shop-shipping-template';

export interface ShopShippingTemplateResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopShippingTemplate;
}

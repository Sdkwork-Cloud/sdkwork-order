import type { CheckoutLineRequest } from './checkout-line-request';

export interface CreateCheckoutSessionRequest {
  items: CheckoutLineRequest[];
  currencyCode?: string;
}

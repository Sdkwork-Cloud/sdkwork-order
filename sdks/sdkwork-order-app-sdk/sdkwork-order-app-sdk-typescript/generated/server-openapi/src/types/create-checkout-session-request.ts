import type { CheckoutLineRequest } from './checkout-line-request';
import type { ShippingAddressRequest } from './shipping-address-request';

export interface CreateCheckoutSessionRequest {
  items: CheckoutLineRequest[];
  currencyCode?: string;
  shippingAddress: ShippingAddressRequest;
}

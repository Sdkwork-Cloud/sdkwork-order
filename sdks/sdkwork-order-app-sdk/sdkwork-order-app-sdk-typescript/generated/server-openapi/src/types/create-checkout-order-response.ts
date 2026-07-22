import type { CheckoutOrder } from './checkout-order';

export interface CreateCheckoutOrderResponse {
  code: 0;
  data: unknown & { item: CheckoutOrder; };
  /** Server-owned request correlation id. */
  traceId: string;
}

export interface CheckoutQuote {
  checkoutSessionId: string;
  quoteId: string;
  currencyCode: string;
  originalAmount: string;
  discountAmount: string;
  payableAmount: string;
}

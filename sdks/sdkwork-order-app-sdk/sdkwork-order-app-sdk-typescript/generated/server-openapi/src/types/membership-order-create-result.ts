export interface MembershipOrderCreateResult {
  orderId: string;
  orderNo: string;
  outTradeNo: string;
  amount: string;
  currencyCode: string;
  packageId: string;
  packageName: string;
  durationDays: string;
  paymentMethod: string;
  paymentProduct: 'mobile_cashier_h5' | 'wechat_native' | 'alipay_native';
  qrCode: string;
  qrCodeType: 'cashier_url' | 'provider_native';
  paymentId?: string | null;
  paymentParams: Record<string, string>;
  status: string;
  cashierUrl: string;
}

export interface MembershipOrderCreateCommand {
  /** Membership lifecycle action. It participates in business purchase-intent identity. */
  action: 'purchase' | 'renew' | 'upgrade';
  packageId: string;
  paymentMethod: string;
  /** QR payment product. H5 returns the order-bound cashierUrl; native products create a provider payment intent. */
  paymentProduct?: 'mobile_cashier_h5' | 'wechat_native' | 'alipay_native';
  clientRequestNo?: string;
  source?: string;
}

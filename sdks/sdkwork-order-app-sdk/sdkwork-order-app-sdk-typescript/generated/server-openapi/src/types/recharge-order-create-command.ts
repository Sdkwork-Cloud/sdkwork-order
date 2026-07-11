export interface RechargeOrderCreateCommand {
  subject?: 'points_recharge' | 'token_bank_recharge' | 'token_bank_plan_purchase' | 'token_bank_plan_renewal' | 'account_recharge_package' | 'coupon_recharge';
  targetAsset?: 'points' | 'token_bank' | 'cash';
  amount?: string | number;
  grantAmount?: string | number;
  clientRequestNo?: string;
  currencyCode?: string;
  packageId?: string;
  planCode?: string;
  planPeriod?: 'monthly' | 'quarterly' | 'yearly' | 'continuous_monthly' | 'continuous_yearly';
  couponCode?: string;
  source?: string;
  paymentMethod?: string;
  paymentPassword?: string;
}

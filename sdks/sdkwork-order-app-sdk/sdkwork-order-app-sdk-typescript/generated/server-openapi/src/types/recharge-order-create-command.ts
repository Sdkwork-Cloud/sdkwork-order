export interface RechargeOrderCreateCommand {
  amount?: string | number;
  clientRequestNo?: string;
  currencyCode?: string;
  packageId?: string;
  source?: string;
  paymentMethod?: string;
  paymentPassword?: string;
}

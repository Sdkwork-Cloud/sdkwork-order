export interface WithdrawalRequestCreateCommand {
  asset?: 'cash';
  amount: string | number;
  currencyCode: string;
  payoutMethod?: string;
  payoutAccountRef?: string;
  reasonCode?: string;
}

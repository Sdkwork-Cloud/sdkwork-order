export interface RefundRequestCreateCommand {
  originalOrderId: string;
  targetAsset: 'points' | 'token_bank' | 'cash';
  amount: string | number;
  currencyCode: string;
  reasonCode?: string;
  reasonDetail?: string;
}

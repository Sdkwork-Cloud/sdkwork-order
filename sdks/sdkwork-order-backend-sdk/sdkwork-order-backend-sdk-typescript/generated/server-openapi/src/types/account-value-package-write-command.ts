export interface AccountValuePackageWriteCommand {
  packageCode: string;
  displayName: string;
  targetAsset: 'points' | 'token_bank' | 'cash';
  grantAmount: string;
  bonusAmount?: string;
  priceAmount: string;
  currencyCode: string;
  status?: string;
  sortWeight?: number;
  validFrom?: string;
  validTo?: string;
}

export interface ShopDepositAccount {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  depositStatus: string;
  currencyCode: string;
  requiredAmount: string;
  paidAmount: string;
  frozenAmount: string;
  accountRef?: string;
  dueAt?: string;
  reviewedBy?: string;
  reviewedAt?: string;
  createdAt: string;
  updatedAt: string;
}

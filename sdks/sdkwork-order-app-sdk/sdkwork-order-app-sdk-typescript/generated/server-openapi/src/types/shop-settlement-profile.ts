export interface ShopSettlementProfile {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  settlementStatus: string;
  settlementCycle: string;
  settlementCurrencyCode: string;
  accountRef?: string;
  riskHoldDays: number;
  settlementConfig: Record<string, unknown>;
  reviewedBy?: string;
  reviewedAt?: string;
  createdAt: string;
  updatedAt: string;
}

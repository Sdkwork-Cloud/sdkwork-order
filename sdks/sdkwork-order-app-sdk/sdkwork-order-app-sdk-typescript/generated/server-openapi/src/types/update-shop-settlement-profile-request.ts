export interface UpdateShopSettlementProfileRequest {
  settlementStatus?: string;
  settlementCycle?: string;
  settlementCurrencyCode?: string;
  accountRef?: string;
  riskHoldDays?: number;
  settlementConfig?: Record<string, unknown>;
}

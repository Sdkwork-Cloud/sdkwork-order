export interface ShopRiskSignal {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  signalNo: string;
  signalType: string;
  riskLevel: string;
  signalStatus: string;
  sourceType?: string;
  sourceId?: string;
  riskScore: number;
  payload: Record<string, unknown>;
  detectedAt: string;
  resolvedAt?: string;
  createdAt: string;
  updatedAt: string;
}

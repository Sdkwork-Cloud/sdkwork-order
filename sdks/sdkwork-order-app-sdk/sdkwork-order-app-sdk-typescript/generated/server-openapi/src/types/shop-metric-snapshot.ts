export interface ShopMetricSnapshot {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  snapshotDate: string;
  grossSalesAmount: string;
  currencyCode: string;
  paidOrderCount: number;
  refundOrderCount: number;
  fulfillmentPendingCount: number;
  settlementPendingAmount: string;
  createdAt: string;
}

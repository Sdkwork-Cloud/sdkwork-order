export interface CreateAfterSalesRequestItem {
  orderItemId: string;
  skuId?: string;
  requestedQuantity: number;
  refundAmount?: string;
  replacementSkuId?: string;
  reasonCode?: string;
  evidenceSnapshot?: Record<string, unknown>[];
}

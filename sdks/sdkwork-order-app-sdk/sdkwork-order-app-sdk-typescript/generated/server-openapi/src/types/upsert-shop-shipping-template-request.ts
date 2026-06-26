export interface UpsertShopShippingTemplateRequest {
  templateCode: string;
  templateName: string;
  templateStatus: string;
  pricingMode: string;
  deliveryMethod: string;
  baseQuantity: number;
  baseFeeAmount: string;
  currencyCode: string;
  isDefault: boolean;
  regionRule: Record<string, unknown>[];
  freeShippingRule: Record<string, unknown>;
}

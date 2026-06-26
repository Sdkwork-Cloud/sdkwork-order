export interface ShopChannel {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  channelCode: string;
  storefrontStatus: string;
  domainName?: string;
  pathPrefix?: string;
  themeCode?: string;
  channelConfig: Record<string, unknown>;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

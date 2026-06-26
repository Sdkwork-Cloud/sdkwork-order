export interface UpdateShopChannelRequest {
  channelCode?: string;
  storefrontStatus?: string;
  domainName?: string;
  pathPrefix?: string;
  themeCode?: string;
  channelConfig?: Record<string, unknown>;
  sortOrder?: number;
}

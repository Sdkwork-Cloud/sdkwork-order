export interface UpsertShopCustomerServiceRequest {
  serviceChannel: string;
  serviceStatus: string;
  contactRef: string;
  contactLabel?: string;
  serviceWindow: Record<string, unknown>;
  serviceConfig: Record<string, unknown>;
  isDefault: boolean;
  sortOrder?: number;
}

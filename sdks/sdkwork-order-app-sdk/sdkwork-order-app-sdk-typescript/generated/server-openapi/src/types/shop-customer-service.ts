export interface ShopCustomerService {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  serviceChannel: string;
  serviceStatus: string;
  contactRef: string;
  contactLabel?: string;
  serviceWindow: Record<string, unknown>;
  serviceConfig: Record<string, unknown>;
  isDefault: boolean;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

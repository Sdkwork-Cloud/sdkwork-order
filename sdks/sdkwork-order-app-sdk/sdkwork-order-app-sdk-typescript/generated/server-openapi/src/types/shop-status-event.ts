export interface ShopStatusEvent {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  eventNo: string;
  eventType: string;
  fromStatus?: string;
  toStatus: string;
  reasonCode?: string;
  reasonDetail?: string;
  actorType: string;
  actorId?: string;
  createdAt: string;
}

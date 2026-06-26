export interface AfterSalesEvent {
  id: string;
  tenantId: string;
  organizationId?: string;
  afterSalesId: string;
  eventNo: string;
  eventType: string;
  fromStatus?: string;
  toStatus: string;
  actorType: string;
  actorId?: string;
  reasonCode?: string;
  payload: Record<string, unknown>;
  idempotencyKey: string;
  createdAt: string;
}

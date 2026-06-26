export interface ShopBusinessHour {
  id: string;
  tenantId: string;
  organizationId: string;
  shopId: string;
  scheduleType: string;
  timezone: string;
  weeklySchedule: Record<string, unknown>;
  holidaySchedule: Record<string, unknown>;
  effectiveFrom?: string;
  effectiveTo?: string;
  status: string;
  version: number;
  createdAt: string;
  updatedAt: string;
}

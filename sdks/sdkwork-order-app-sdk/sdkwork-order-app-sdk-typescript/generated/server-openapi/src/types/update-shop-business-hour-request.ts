export interface UpdateShopBusinessHourRequest {
  scheduleType?: string;
  timezone?: string;
  weeklySchedule?: Record<string, unknown>;
  holidaySchedule?: Record<string, unknown>;
  effectiveFrom?: string;
  effectiveTo?: string;
  status?: string;
  version?: number;
}

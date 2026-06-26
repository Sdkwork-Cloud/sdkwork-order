export interface ShopReadinessItem {
  code: string;
  title: string;
  status: string;
  severity: string;
  sourceType?: string;
  sourceId?: string;
  blocking: boolean;
  message?: string;
  actionHint?: string;
  evaluatedAt: string;
}

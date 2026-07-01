export type SdkworkOrderStatus =
  | "cancelled"
  | "completed"
  | "expired"
  | "paid"
  | "pending-payment"
  | "refunded"
  | "refunding"
  | "unknown";

export interface SdkworkOrderRouteIntent {
  focusWindow: boolean;
  orderId?: string;
  route: string;
  source: "order-workspace";
  type: "order-route-intent";
}

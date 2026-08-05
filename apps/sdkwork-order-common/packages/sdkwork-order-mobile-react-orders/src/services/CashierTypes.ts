import type { Order, PaymentStatus } from "./OrderService";

/**
 * Shared cashier types. `CashierPhase` is the page state machine; it stays
 * independent of React so the transitions can be unit tested.
 */
export type CashierPhase =
  | "loading"
  | "creating"
  /** WeChat OAuth authorization in flight (payer sent to WeChat). */
  | "oauth_waiting"
  | "pending"
  | "paid"
  | "cancelled"
  | "expired"
  | "failed"
  | "not_payable";

export type { Order, PaymentStatus };

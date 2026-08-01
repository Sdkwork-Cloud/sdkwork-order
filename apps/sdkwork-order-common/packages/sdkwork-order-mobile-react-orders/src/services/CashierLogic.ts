import type {
  CashierPhase,
  Order,
  PaymentStatus,
} from "./CashierTypes";

/**
 * Pure cashier state-machine helpers shared by the cashier page and its
 * unit tests. Keep transitions explicit so frontend behavior stays aligned
 * with the backend order/payment status semantics.
 */

export const CASHIER_POLL_INTERVAL_MS = 3000;

/** Backend payment/order TTL applied at cashier creation (15 minutes). */
export const CASHIER_TTL_SECONDS = 15 * 60;

/** Maps a `payment_success` response + order status onto the cashier phase. */
export function resolveCashierPhaseFromPaymentStatus(
  status: PaymentStatus,
  orderStatus: Order["status"],
): CashierPhase {
  if (status.paid) {
    return "paid";
  }
  const normalized = String(orderStatus).toLowerCase();
  if (normalized === "cancelled" || normalized === "closed") {
    return "cancelled";
  }
  if (normalized === "expired" || normalized === "timeout") {
    return "expired";
  }
  if (normalized === "refunding" || normalized === "refunded") {
    return "paid";
  }
  if (
    normalized === "paid" ||
    normalized === "fulfilled" ||
    normalized === "completed" ||
    normalized === "shipped" ||
    normalized === "delivered"
  ) {
    return "paid";
  }
  return "pending";
}

/** Seconds remaining until the cashier closes, floored at zero. */
export function computeCashierRemainingSeconds(
  orderExpireTime: string | undefined,
  paymentCreatedAtMs: number,
  nowMs: number,
): number {
  const orderExpireMs = orderExpireTime ? Date.parse(orderExpireTime) : Number.NaN;
  const candidates: number[] = [];
  if (Number.isFinite(orderExpireMs)) {
    candidates.push(orderExpireMs);
  }
  candidates.push(paymentCreatedAtMs + CASHIER_TTL_SECONDS * 1000);
  const deadline = Math.min(...candidates);
  return Math.max(0, Math.floor((deadline - nowMs) / 1000));
}

export function formatCashierCountdown(remainingSeconds: number): string {
  const minutes = Math.floor(remainingSeconds / 60);
  const seconds = remainingSeconds % 60;
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

export function cashierPaymentMethodLabelKey(method: string): string {
  return `orders.payment_method_${method}`;
}

export function isCashierRetryablePhase(phase: CashierPhase): boolean {
  return phase === "failed" || phase === "expired";
}

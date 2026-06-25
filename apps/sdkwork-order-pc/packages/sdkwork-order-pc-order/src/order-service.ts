import {
  getSdkworkOrderService,
  hasSdkworkOrderSession,
  requireSdkworkOrderSession,
  toNullableSdkworkOrderNumber,
  toSdkworkOrderNumber,
  toSdkworkOrderOptionalString,
  unwrapSdkworkOrderResponse,
  readSdkworkMediaResource,
  type SdkworkOrderAppService,
  type SdkworkMediaResource,
} from "@sdkwork/order-service";
import {
  createSdkworkOrderMessages,
  type SdkworkOrderMessages,
  type SdkworkOrderMessagesOverrides,
} from "./order-copy";

export type SdkworkOrderStatus =
  | "cancelled"
  | "completed"
  | "expired"
  | "paid"
  | "pending-payment"
  | "refunded"
  | "refunding"
  | "unknown";

export interface SdkworkOrderSummary {
  createdAt: string;
  discountAmountCny: number | null;
  expireTime?: string;
  id: string;
  orderSn?: string;
  paidAmountCny: number | null;
  payTime?: string;
  paymentMethod?: string;
  paymentProvider?: string;
  productImage?: SdkworkMediaResource;
  quantity: number;
  remark?: string;
  status: SdkworkOrderStatus;
  statusLabel: string;
  subject: string;
  totalAmountCny: number | null;
}

export interface SdkworkOrderStatistics {
  completed: number;
  pendingPayment: number;
  pendingReceipt: number;
  pendingShipment: number;
  totalAmountCny: number | null;
  totalOrders: number;
}

export interface SdkworkOrderItem {
  id: string;
  image?: SdkworkMediaResource;
  name: string;
  quantity: number;
  totalAmountCny: number | null;
  unitPriceCny: number | null;
}

export interface SdkworkOrderTimelineEvent {
  label: string;
  occurredAt?: string;
  tone: "danger" | "default" | "success" | "warning";
}

export interface SdkworkOrderDetail {
  createdAt: string;
  id: string;
  items: SdkworkOrderItem[];
  orderSn?: string;
  outTradeNo?: string;
  paidAmountCny: number | null;
  payTime?: string;
  paymentMethod?: string;
  productImage?: SdkworkMediaResource;
  quantity: number;
  remark?: string;
  status: SdkworkOrderStatus;
  statusLabel: string;
  subject: string;
  timeline: SdkworkOrderTimelineEvent[];
  totalAmountCny: number | null;
  transactionId?: string;
}

export interface SdkworkOrderDashboardData {
  orders: SdkworkOrderSummary[];
  statistics: SdkworkOrderStatistics;
}

export interface SdkworkOrderPaymentInput {
  orderId: string;
  paymentMethod?: string;
  paymentPassword?: string;
}

export interface SdkworkOrderPaymentResult {
  amountCny: number | null;
  orderId: string;
  outTradeNo?: string;
  paymentId?: string;
  paymentMethod?: string;
  paymentParams: Record<string, unknown>;
}

export interface SdkworkOrderCancelInput {
  cancelReason?: string;
  cancelType?: string;
  orderId: string;
}

export interface SdkworkOrderCancelResult {
  cancelled: true;
  orderId: string;
}

export interface CreateSdkworkOrderServiceOptions {
  orderAppService?: SdkworkOrderAppService;
  locale?: string | null;
  messages?: SdkworkOrderMessagesOverrides;
}

export interface SdkworkOrderService {
  cancelOrder(input: SdkworkOrderCancelInput): Promise<SdkworkOrderCancelResult>;
  getDashboard(): Promise<SdkworkOrderDashboardData>;
  getEmptyDashboard(): SdkworkOrderDashboardData;
  getOrderDetail(orderId: string): Promise<SdkworkOrderDetail>;
  payOrder(input: SdkworkOrderPaymentInput): Promise<SdkworkOrderPaymentResult>;
}

interface RemoteOrder {
  createdAt?: string;
  discountAmount?: number | string;
  expireTime?: string;
  orderId?: string;
  orderSn?: string;
  paidAmount?: number | string;
  payTime?: string;
  paymentMethod?: string;
  paymentProvider?: string;
  productImage?: unknown;
  quantity?: number | string;
  remark?: string;
  status?: string;
  statusName?: string;
  subject?: string;
  totalAmount?: number | string;
}

interface RemoteOrderItem {
  id?: string;
  productImage?: unknown;
  productName?: string;
  quantity?: number | string;
  totalAmount?: number | string;
  unitPrice?: number | string;
}

interface RemoteOrderDetail extends RemoteOrder {
  items?: RemoteOrderItem[];
  outTradeNo?: string;
  transactionId?: string;
}

interface RemoteOrderStatistics {
  completed?: number | string;
  pendingPayment?: number | string;
  pendingReceipt?: number | string;
  pendingShipment?: number | string;
  totalAmount?: number | string;
  totalOrders?: number | string;
}

interface RemoteOrderStatus {
  status?: string;
  statusName?: string;
}

interface RemoteOrderPaymentSuccess {
  paid?: boolean;
  status?: string;
  statusName?: string;
}

interface RemotePaymentParams {
  amount?: number | string;
  orderId?: string;
  outTradeNo?: string;
  paymentId?: string;
  paymentMethod?: string;
  paymentParams?: Record<string, unknown>;
}

type SdkworkOrderCopyContext = Pick<SdkworkOrderMessages, "status" | "timeline">;
type SdkworkOrderServiceCopy = SdkworkOrderMessages["service"];

function mapOrderStatus(status: string | undefined): SdkworkOrderStatus {
  const normalized = (status || "").trim().toUpperCase();
  if (normalized === "PENDING_PAYMENT" || normalized === "UNPAID" || normalized === "WAIT_PAY") {
    return "pending-payment";
  }

  if (normalized === "PAID") {
    return "paid";
  }

  if (normalized === "COMPLETED" || normalized === "FINISHED") {
    return "completed";
  }

  if (normalized === "CANCELLED" || normalized === "CANCELED" || normalized === "CLOSED") {
    return "cancelled";
  }

  if (normalized === "EXPIRED" || normalized === "TIMEOUT") {
    return "expired";
  }

  if (normalized === "REFUNDING") {
    return "refunding";
  }

  if (normalized === "REFUNDED") {
    return "refunded";
  }

  return "unknown";
}

function formatStatusLabel(
  status: SdkworkOrderStatus,
  messages: SdkworkOrderCopyContext,
): string {
  if (status === "cancelled") {
    return messages.status.cancelled;
  }

  if (status === "completed") {
    return messages.status.completed;
  }

  if (status === "expired") {
    return messages.status.expired;
  }

  if (status === "paid") {
    return messages.status.paid;
  }

  if (status === "pending-payment") {
    return messages.status.pendingPayment;
  }

  if (status === "refunded") {
    return messages.status.refunded;
  }

  if (status === "refunding") {
    return messages.status.refunding;
  }

  return messages.status.unknown;
}

function createEmptyDashboard(): SdkworkOrderDashboardData {
  return {
    orders: [],
    statistics: {
      completed: 0,
      pendingPayment: 0,
      pendingReceipt: 0,
      pendingShipment: 0,
      totalAmountCny: 0,
      totalOrders: 0,
    },
  };
}

function mapOrderSummary(
  order: RemoteOrder,
  messages: SdkworkOrderCopyContext,
  copy: SdkworkOrderServiceCopy,
): SdkworkOrderSummary {
  const status = mapOrderStatus(order.status);

  return {
    createdAt: toSdkworkOrderOptionalString(order.createdAt) || new Date(0).toISOString(),
    discountAmountCny: toNullableSdkworkOrderNumber(order.discountAmount),
    expireTime: toSdkworkOrderOptionalString(order.expireTime),
    id: toSdkworkOrderOptionalString(order.orderId) || "unknown-order",
    orderSn: toSdkworkOrderOptionalString(order.orderSn),
    paidAmountCny: toNullableSdkworkOrderNumber(order.paidAmount),
    payTime: toSdkworkOrderOptionalString(order.payTime),
    paymentMethod: toSdkworkOrderOptionalString(order.paymentMethod),
    paymentProvider: toSdkworkOrderOptionalString(order.paymentProvider),
    productImage: readSdkworkMediaResource(order.productImage),
    quantity: toSdkworkOrderNumber(order.quantity, 1),
    remark: toSdkworkOrderOptionalString(order.remark),
    status,
    statusLabel: toSdkworkOrderOptionalString(order.statusName) || formatStatusLabel(status, messages),
    subject: toSdkworkOrderOptionalString(order.subject) || copy.summaryFallbackSubject,
    totalAmountCny: toNullableSdkworkOrderNumber(order.totalAmount),
  };
}

function mapStatistics(statistics: RemoteOrderStatistics | null | undefined): SdkworkOrderStatistics {
  return {
    completed: toSdkworkOrderNumber(statistics?.completed),
    pendingPayment: toSdkworkOrderNumber(statistics?.pendingPayment),
    pendingReceipt: toSdkworkOrderNumber(statistics?.pendingReceipt),
    pendingShipment: toSdkworkOrderNumber(statistics?.pendingShipment),
    totalAmountCny: toNullableSdkworkOrderNumber(statistics?.totalAmount),
    totalOrders: toSdkworkOrderNumber(statistics?.totalOrders),
  };
}

function mapItems(items: RemoteOrderItem[] | undefined, copy: SdkworkOrderServiceCopy): SdkworkOrderItem[] {
  return (items ?? []).map((item, index) => ({
    id: toSdkworkOrderOptionalString(item.id) || `order-item-${index + 1}`,
    image: readSdkworkMediaResource(item.productImage),
    name: toSdkworkOrderOptionalString(item.productName) || copy.itemFallbackName,
    quantity: toSdkworkOrderNumber(item.quantity, 1),
    totalAmountCny: toNullableSdkworkOrderNumber(item.totalAmount),
    unitPriceCny: toNullableSdkworkOrderNumber(item.unitPrice),
  }));
}

function createTimeline(
  detail: RemoteOrderDetail,
  status: RemoteOrderStatus | null,
  paymentSuccess: RemoteOrderPaymentSuccess | null,
  messages: SdkworkOrderCopyContext,
): SdkworkOrderTimelineEvent[] {
  const resolvedStatus = mapOrderStatus(status?.status || detail.status);
  const events: SdkworkOrderTimelineEvent[] = [
    {
      label: messages.timeline.created,
      occurredAt: toSdkworkOrderOptionalString(detail.createdAt),
      tone: "default",
    },
  ];

  const paid = Boolean(paymentSuccess?.paid || resolvedStatus === "paid" || resolvedStatus === "completed");
  if (paid) {
    events.push({
      label: messages.timeline.paid,
      occurredAt: toSdkworkOrderOptionalString(detail.payTime),
      tone: "success",
    });
  }

  const statusLabel = toSdkworkOrderOptionalString(status?.statusName)
    || toSdkworkOrderOptionalString(paymentSuccess?.statusName)
    || toSdkworkOrderOptionalString(detail.statusName)
    || formatStatusLabel(resolvedStatus, messages);
  events.push({
    label: statusLabel,
    tone:
      resolvedStatus === "cancelled" || resolvedStatus === "expired"
        ? "danger"
        : resolvedStatus === "pending-payment"
          ? "warning"
          : "default",
  });

  return events;
}

function mapDetail(
  detail: RemoteOrderDetail | null | undefined,
  status: RemoteOrderStatus | null,
  paymentSuccess: RemoteOrderPaymentSuccess | null,
  messages: SdkworkOrderCopyContext,
  copy: SdkworkOrderServiceCopy,
): SdkworkOrderDetail {
  const summary = mapOrderSummary(detail ?? {}, messages, copy);
  const resolvedStatus = mapOrderStatus(status?.status || detail?.status);

  return {
    createdAt: summary.createdAt,
    id: summary.id,
    items: mapItems(detail?.items, copy),
    orderSn: summary.orderSn,
    outTradeNo: toSdkworkOrderOptionalString(detail?.outTradeNo),
    paidAmountCny: summary.paidAmountCny,
    payTime: summary.payTime,
    paymentMethod: summary.paymentMethod,
    productImage: summary.productImage,
    quantity: summary.quantity,
    remark: summary.remark,
    status: resolvedStatus,
    statusLabel:
      toSdkworkOrderOptionalString(status?.statusName)
      || toSdkworkOrderOptionalString(detail?.statusName)
      || formatStatusLabel(resolvedStatus, messages),
    subject: summary.subject,
    timeline: createTimeline(detail ?? {}, status, paymentSuccess, messages),
    totalAmountCny: summary.totalAmountCny,
    transactionId: toSdkworkOrderOptionalString(detail?.transactionId),
  };
}

function mapPaymentResult(result: RemotePaymentParams | null | undefined): SdkworkOrderPaymentResult {
  return {
    amountCny: toNullableSdkworkOrderNumber(result?.amount),
    orderId: toSdkworkOrderOptionalString(result?.orderId) || "",
    outTradeNo: toSdkworkOrderOptionalString(result?.outTradeNo),
    paymentId: toSdkworkOrderOptionalString(result?.paymentId),
    paymentMethod: toSdkworkOrderOptionalString(result?.paymentMethod),
    paymentParams: (result?.paymentParams ?? {}) as Record<string, unknown>,
  };
}

export function createSdkworkOrderService(
  options: CreateSdkworkOrderServiceOptions = {},
): SdkworkOrderService {
  const messages = createSdkworkOrderMessages(options.locale, options.messages);
  const copy = messages.service;
  const getOrderAppService = () => options.orderAppService ?? getSdkworkOrderService();

  return {
    async cancelOrder(input) {
      requireSdkworkOrderSession(copy.signInRequired);
      await unwrapSdkworkOrderResponse<void>(
        await getOrderAppService().orders.cancel(input.orderId, {
          cancelReason: toSdkworkOrderOptionalString(input.cancelReason),
          cancelType: toSdkworkOrderOptionalString(input.cancelType),
        }),
        copy.cancelFailed,
      );

      return {
        cancelled: true,
        orderId: input.orderId,
      };
    },

    async getDashboard() {
      if (!hasSdkworkOrderSession()) {
        return createEmptyDashboard();
      }

      const [orderPagePayload, statisticsPayload] = await Promise.all([
        getOrderAppService().orders.list({
            page: 1,
            pageSize: 20,
            sortDirection: "desc",
            sortField: "createdAt",
        }),
        getOrderAppService().orders.statistics.retrieve(),
      ]);
      const orderPage = unwrapSdkworkOrderResponse<{ content?: RemoteOrder[] }>(
        orderPagePayload,
        copy.requestFailed,
      );
      const statistics = unwrapSdkworkOrderResponse<RemoteOrderStatistics | null>(
        statisticsPayload,
        copy.requestFailed,
      );

      return {
        orders: (orderPage.content ?? [])
          .map((order) => mapOrderSummary(order, messages, copy))
          .sort((left, right) => new Date(right.createdAt).getTime() - new Date(left.createdAt).getTime()),
        statistics: mapStatistics(statistics),
      };
    },

    getEmptyDashboard() {
      return createEmptyDashboard();
    },

    async getOrderDetail(orderId) {
      requireSdkworkOrderSession(copy.signInRequired);
      const [detailPayload, statusPayload, paymentSuccessPayload] = await Promise.all([
        getOrderAppService().orders.retrieve(orderId),
        getOrderAppService().orders.status.retrieve(orderId),
        getOrderAppService().orders.paymentSuccess.retrieve(orderId),
      ]);
      const detail = unwrapSdkworkOrderResponse<RemoteOrderDetail | null>(detailPayload, copy.requestFailed);
      const status = unwrapSdkworkOrderResponse<RemoteOrderStatus | null>(statusPayload, copy.requestFailed);
      const paymentSuccess = unwrapSdkworkOrderResponse<RemoteOrderPaymentSuccess | null>(
        paymentSuccessPayload,
        copy.requestFailed,
      );

      return mapDetail(detail, status, paymentSuccess, messages, copy);
    },

    async payOrder(input) {
      requireSdkworkOrderSession(copy.signInRequired);
      const result = unwrapSdkworkOrderResponse<RemotePaymentParams>(
        await getOrderAppService().orders.pay(input.orderId, {
          paymentMethod: toSdkworkOrderOptionalString(input.paymentMethod),
          paymentPassword: toSdkworkOrderOptionalString(input.paymentPassword),
        }),
        copy.payFailed,
      );

      return mapPaymentResult(result);
    },
  };
}

export const sdkworkOrderService = createSdkworkOrderService();

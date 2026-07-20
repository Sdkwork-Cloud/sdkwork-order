import {
  APP_ORDER_METHOD_TREE,
  type ClientFromMethodTree,
  type OrderAppSdkClient,
  type OrderSdkMethod,
} from "@sdkwork/order-sdk-ports";
import { formatCurrency as formatSdkworkCurrency } from "@sdkwork/utils";
import {
  createOrderAppSdkClientFromTransport,
  createOrderAppTransportClient,
  type BootstrapSdkworkOrderAppServiceInput,
} from "./transport.ts";
import { createSdkworkWriteCommandHeaders } from "./write-command-headers.ts";

type ServiceTemplate = { readonly [key: string]: true | ServiceTemplate };

export type SdkworkOrderOrdersService = ClientFromMethodTree<(typeof APP_ORDER_METHOD_TREE)["orders"]>;
export type SdkworkOrderRechargesService = ClientFromMethodTree<(typeof APP_ORDER_METHOD_TREE)["recharges"]>;
export type SdkworkOrderMembershipsService = ClientFromMethodTree<(typeof APP_ORDER_METHOD_TREE)["memberships"]>;
export type SdkworkOrderWithdrawalsService = ClientFromMethodTree<(typeof APP_ORDER_METHOD_TREE)["withdrawals"]>;

export type SdkworkOrderAppService = {
  memberships: SdkworkOrderMembershipsService;
  orders: SdkworkOrderOrdersService;
  recharges: SdkworkOrderRechargesService;
  withdrawals: SdkworkOrderWithdrawalsService;
};

export type SdkworkMembershipCheckoutAction = "purchase" | "renew" | "upgrade";

export interface SdkworkMembershipCheckoutInput {
  action: SdkworkMembershipCheckoutAction;
  packageId: number;
  paymentMethod?: string;
  paymentProduct?: "alipay_native" | "mobile_cashier_h5" | "wechat_native";
}

export interface SdkworkMembershipCheckoutPayment {
  amountCny: number | null;
  cashierUrl?: string;
  durationDays: number | null;
  orderId?: string;
  packageId: number | null;
  packageName?: string;
  qrCode?: string;
  status: "completed" | "failed" | "pending";
  targetLevelName?: string;
}

export interface SdkworkMembershipCheckoutService {
  createCheckout(input: SdkworkMembershipCheckoutInput): Promise<SdkworkMembershipCheckoutPayment>;
  getCheckoutStatus(orderId: string): Promise<SdkworkMembershipCheckoutPayment>;
}

export interface CreateSdkworkMembershipCheckoutServiceOptions {
  appService?: SdkworkOrderAppService;
}

export interface SdkworkPointsRechargePackage {
  id: string;
  priceAmount: number;
  currencyCode: string;
  bonusPoints: number;
  grantAmount: number;
  points: number;
}

export interface SdkworkPointsRechargePayment {
  amountCny: number | null;
  cashierUrl?: string;
  orderId?: string;
  orderNo?: string;
  points: number;
  qrCode?: string;
  status: "completed" | "failed" | "pending";
}

export interface SdkworkPointsRechargeOrderInput {
  packageId: number | string;
  paymentMethod?: string;
  source?: string;
}

export interface SdkworkPointsRechargeService {
  listPackages(): Promise<SdkworkPointsRechargePackage[]>;
  createOrder(input: SdkworkPointsRechargeOrderInput): Promise<SdkworkPointsRechargePayment>;
  getOrderStatus(orderId: string): Promise<SdkworkPointsRechargePayment>;
}

export interface CreateSdkworkPointsRechargeServiceOptions {
  appService?: SdkworkOrderAppService;
}

export interface SdkworkCouponRechargeResult {
  grantAmount: number;
  orderId: string;
  orderNo?: string;
  replayed: boolean;
  status: "completed" | "pending";
  targetAsset: "token_bank";
}

export interface SdkworkCouponRechargeService {
  redeem(code: string): Promise<SdkworkCouponRechargeResult>;
}

export interface CreateSdkworkCouponRechargeServiceOptions {
  appService?: SdkworkOrderAppService;
}

export type SdkworkOrderAppServiceProvider = () => SdkworkOrderAppService;

let sdkworkOrderAppServiceProvider: SdkworkOrderAppServiceProvider | null = null;

export interface SdkworkOrderSessionTokens {
  accessToken?: string;
  authToken?: string;
  refreshToken?: string;
}

export type SdkworkOrderSessionTokenProvider = () => SdkworkOrderSessionTokens;

let sdkworkOrderSessionTokenProvider: SdkworkOrderSessionTokenProvider = () => ({});

export interface CreateSdkworkOrderAppServiceInput {
  appClient: OrderAppSdkClient;
}

export interface SdkworkOrderResponseEnvelope<T> {
  code?: number | string;
  data?: T;
  message?: string;
  msg?: string;
}

export type SdkworkMediaKind =
  | "archive"
  | "audio"
  | "document"
  | "image"
  | "model"
  | "other"
  | "video";

export type SdkworkMediaSource =
  | "data_url"
  | "external_url"
  | "generated"
  | "object_storage"
  | "provider_asset";

export interface SdkworkMediaResource {
  kind: SdkworkMediaKind;
  publicUrl?: string;
  source: SdkworkMediaSource;
  url?: string;
  [key: string]: unknown;
}

export function configureSdkworkOrderAppServiceProvider(provider: SdkworkOrderAppServiceProvider | null): void {
  sdkworkOrderAppServiceProvider = provider;
}

export function configureSdkworkOrderSessionTokenProvider(provider: SdkworkOrderSessionTokenProvider | null): void {
  sdkworkOrderSessionTokenProvider = provider ?? (() => ({}));
}

export function getSdkworkOrderService(): SdkworkOrderAppService {
  if (!sdkworkOrderAppServiceProvider) {
    throw new Error(
      "SDKWork order service provider is not configured. Call configureSdkworkOrderAppServiceProvider() from order PC bootstrap.",
    );
  }
  return sdkworkOrderAppServiceProvider();
}

export function getSdkworkOrderSessionTokens(): SdkworkOrderSessionTokens {
  const tokens = sdkworkOrderSessionTokenProvider();
  return {
    accessToken: normalizeSessionToken(tokens.accessToken),
    authToken: normalizeSessionToken(tokens.authToken),
    refreshToken: normalizeSessionToken(tokens.refreshToken),
  };
}

export function hasSdkworkOrderSession(): boolean {
  const tokens = getSdkworkOrderSessionTokens();
  return Boolean(normalizeSessionToken(tokens.authToken) || normalizeSessionToken(tokens.accessToken));
}

export function requireSdkworkOrderSession(message = "Authentication required"): void {
  if (!hasSdkworkOrderSession()) {
    throw new Error(message);
  }
}

export function createSdkworkOrderAppService(input: CreateSdkworkOrderAppServiceInput): SdkworkOrderAppService {
  return {
    memberships: buildServiceTree<SdkworkOrderMembershipsService>(
      APP_ORDER_METHOD_TREE.memberships,
      input.appClient.commerce.memberships,
      ["commerce", "memberships"],
    ),
    orders: buildServiceTree<SdkworkOrderOrdersService>(
      APP_ORDER_METHOD_TREE.orders,
      input.appClient.commerce.orders,
      ["commerce", "orders"],
    ),
    recharges: buildServiceTree<SdkworkOrderRechargesService>(
      APP_ORDER_METHOD_TREE.recharges,
      input.appClient.commerce.recharges,
      ["commerce", "recharges"],
    ),
    withdrawals: buildServiceTree<SdkworkOrderWithdrawalsService>(
      APP_ORDER_METHOD_TREE.withdrawals,
      input.appClient.commerce.withdrawals,
      ["commerce", "withdrawals"],
    ),
  };
}

export function createSdkworkPointsRechargeService(
  options: CreateSdkworkPointsRechargeServiceOptions = {},
): SdkworkPointsRechargeService {
  const resolveAppService = () => options.appService ?? getSdkworkOrderService();

  return {
    async listPackages() {
      const response = await resolveAppService().recharges.packages.list({ page: 1, pageSize: 200 });
      const page = unwrapSdkworkOrderListPage<unknown>(response, "Unable to load recharge packages.");
      return page.items.map(normalizePointsRechargePackage).filter((item): item is SdkworkPointsRechargePackage => item !== null);
    },

    async createOrder(input) {
      const packageId = String(input.packageId).trim();
      if (!packageId) {
        throw new Error("A recharge package is required.");
      }
      const packages = await this.listPackages();
      const selectedPackage = packages.find((item) => item.id === packageId);
      if (!selectedPackage) {
        throw new Error("The selected recharge package is unavailable.");
      }

      const body = {
        amount: selectedPackage.priceAmount,
        currencyCode: selectedPackage.currencyCode,
        packageId: selectedPackage.id,
        paymentMethod: input.paymentMethod ?? "wechat_pay",
        source: input.source ?? "membership-token-plan",
        subject: "points_recharge" as const,
        targetAsset: "points" as const,
      };
      const headers = createSdkworkWriteCommandHeaders("recharges.orders.create", body);
      const response = await resolveAppService().recharges.orders.create(body, headers);
      return normalizePointsRechargePayment(
        unwrapSdkworkOrderResource<unknown>(response, "Unable to create points recharge order."),
      );
    },

    async getOrderStatus(orderId) {
      const normalizedOrderId = orderId.trim();
      if (!normalizedOrderId) {
        throw new Error("A recharge order id is required.");
      }
      const response = await resolveAppService().recharges.orders.retrieve(normalizedOrderId);
      return normalizePointsRechargePayment(
        unwrapSdkworkOrderResource<unknown>(response, "Unable to retrieve points recharge order."),
      );
    },
  };
}

export function createSdkworkCouponRechargeService(
  options: CreateSdkworkCouponRechargeServiceOptions = {},
): SdkworkCouponRechargeService {
  const resolveAppService = () => options.appService ?? getSdkworkOrderService();

  return {
    async redeem(code) {
      requireSdkworkOrderSession();
      const couponCode = code.trim();
      if (!couponCode) {
        throw new Error("A coupon code is required.");
      }
      const body = {
        amount: 0,
        couponCode,
        currencyCode: "CNY",
        subject: "coupon_recharge" as const,
        targetAsset: "token_bank" as const,
      };
      const headers = createSdkworkWriteCommandHeaders(
        "recharges.orders.create",
        body,
      );
      const response = await resolveAppService().recharges.orders.create(body, headers);
      return normalizeCouponRechargeResult(
        unwrapSdkworkOrderResource<unknown>(response, "Unable to redeem this coupon."),
      );
    },
  };
}

export function createSdkworkMembershipCheckoutService(
  options: CreateSdkworkMembershipCheckoutServiceOptions = {},
): SdkworkMembershipCheckoutService {
  const resolveAppService = () => options.appService ?? getSdkworkOrderService();

  return {
    async createCheckout(input) {
      requireSdkworkOrderSession();
      const packageId = String(input.packageId).trim();
      if (!packageId || input.packageId <= 0) {
        throw new Error("A valid membership package is required.");
      }

      const paymentProduct = input.paymentProduct ?? "mobile_cashier_h5";
      const paymentMethod = normalizeMembershipPaymentMethod(input.paymentMethod, paymentProduct);
      const body = {
        packageId,
        paymentMethod,
        paymentProduct,
      };
      const headers = createSdkworkWriteCommandHeaders(
        "memberships.orders.create",
        body,
        `membership-checkout:${packageId}:${input.action}`,
      );
      const response = await resolveAppService().memberships.orders.create(body, headers);
      return normalizeMembershipCheckoutPayment(
        unwrapSdkworkOrderResource<unknown>(response, "Unable to create membership order."),
        input.packageId,
        paymentProduct,
      );
    },

    async getCheckoutStatus(orderId) {
      requireSdkworkOrderSession();
      const normalizedOrderId = orderId.trim();
      if (!normalizedOrderId) {
        throw new Error("A membership order id is required.");
      }
      const response = await resolveAppService().orders.paymentSuccess.retrieve(normalizedOrderId);
      const record = unwrapSdkworkOrderResource<Record<string, unknown>>(
        response,
        "Unable to retrieve membership order status.",
      );
      return {
        amountCny: null,
        durationDays: null,
        orderId: normalizedOrderId,
        packageId: null,
        status: record?.paid === true ? "completed" : normalizePointsRechargeStatus(record?.status),
      };
    },
  };
}

export function unwrapSdkworkOrderResource<T>(
  value: unknown,
  fallbackMessage = "Request failed.",
): T {
  const data = unwrapSdkworkOrderResponse<{ item?: T } | T>(value, fallbackMessage);
  if (data && typeof data === "object" && "item" in (data as Record<string, unknown>)) {
    return (data as { item?: T }).item as T;
  }
  return data as T;
}

export interface SdkworkOffsetPageInfo {
  hasMore?: boolean;
  mode?: string;
  page?: number;
  pageSize?: number;
  totalItems?: number;
  totalPages?: number;
}

export interface SdkworkOffsetListPage<T> {
  items: T[];
  pageInfo?: SdkworkOffsetPageInfo;
}

export function unwrapSdkworkOrderPage<T>(
  value: unknown,
  fallbackMessage = "Request failed.",
): T[] {
  return unwrapSdkworkOrderListPage<T>(value, fallbackMessage).items;
}

export function unwrapSdkworkOrderListPage<T>(
  value: unknown,
  fallbackMessage = "Request failed.",
): SdkworkOffsetListPage<T> {
  const data = unwrapSdkworkOrderResponse<SdkworkOffsetListPage<T> | T[]>(value, fallbackMessage);
  if (Array.isArray(data)) {
    return { items: data };
  }
  return {
    items: Array.isArray(data?.items) ? data.items : [],
    pageInfo: data?.pageInfo,
  };
}

export function resolveSdkworkOffsetPagination(
  pageInfo: SdkworkOffsetPageInfo | null | undefined,
  fallbackPage: number,
  fallbackPageSize: number,
): {
  hasMore: boolean;
  page: number;
  pageSize: number;
  total: number;
  totalPages: number;
} {
  const pageSize = toSdkworkOrderNumber(pageInfo?.pageSize, fallbackPageSize) || fallbackPageSize;
  const page = toSdkworkOrderNumber(pageInfo?.page, fallbackPage) || fallbackPage;
  const total = toSdkworkOrderNumber(pageInfo?.totalItems);
  const totalPages = pageInfo?.totalPages === undefined
    ? (pageSize > 0 ? Math.ceil(total / pageSize) : 0)
    : toSdkworkOrderNumber(pageInfo?.totalPages);
  return {
    page,
    pageSize,
    total,
    hasMore: Boolean(pageInfo?.hasMore ?? page * pageSize < total),
    totalPages,
  };
}

/** Maps UI kebab-case order status filters to API snake_case wire values. */
export function toApiOrderStatusWire(status: string): string {
  const normalized = status.trim();
  if (!normalized || normalized === "all") {
    return normalized;
  }
  return normalized.replace(/-/g, "_").toLowerCase();
}

export function unwrapSdkworkOrderResponse<T>(value: unknown, fallbackMessage = "Request failed."): T {
  if (!value || typeof value !== "object") {
    return value as T;
  }
  if (!("data" in value) && !("code" in value)) {
    return value as T;
  }
  const envelope = value as SdkworkOrderResponseEnvelope<T>;
  if (!isSuccessCode(envelope.code)) {
    throw new Error(String(envelope.message || envelope.msg || fallbackMessage).trim());
  }
  return (envelope.data ?? null) as T;
}

export function toSdkworkOrderOptionalString(value: unknown): string | undefined {
  const normalized = typeof value === "string" ? value.trim() : String(value ?? "").trim();
  return normalized || undefined;
}

export function toNullableSdkworkOrderNumber(value: unknown): number | null {
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }
  if (typeof value === "string" && value.trim()) {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : null;
  }
  return null;
}

export function toSdkworkOrderNumber(value: unknown, fallback = 0): number {
  return toNullableSdkworkOrderNumber(value) ?? fallback;
}

export function formatSdkworkOrderCurrencyCny(value: number | null | undefined, language = "en-US"): string {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return "--";
  }
  return formatSdkworkCurrency(value, "CNY", language) ?? "--";
}

export function readSdkworkMediaResource(value: unknown): SdkworkMediaResource | undefined {
  if (!value || typeof value !== "object") {
    return undefined;
  }
  const record = value as Record<string, unknown>;
  const kind = typeof record.kind === "string" ? record.kind : undefined;
  const source = typeof record.source === "string" ? record.source : undefined;
  if (!kind || !source) {
    return undefined;
  }
  return { ...record, kind, source } as SdkworkMediaResource;
}

function normalizePointsRechargePackage(value: unknown): SdkworkPointsRechargePackage | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  const record = value as Record<string, unknown>;
  const id = toSdkworkOrderOptionalString(record.id ?? record.packageId);
  if (!id) {
    return null;
  }
  return {
    id,
    priceAmount: toSdkworkOrderNumber(record.priceAmount ?? record.price),
    currencyCode: toSdkworkOrderOptionalString(record.currencyCode) ?? "CNY",
    bonusPoints: toSdkworkOrderNumber(record.bonusPoints),
    grantAmount: toSdkworkOrderNumber(record.grantAmount),
    points: toSdkworkOrderNumber(record.points ?? record.grantAmount ?? record.bonusPoints),
  };
}

function normalizePointsRechargePayment(value: unknown): SdkworkPointsRechargePayment {
  const record = value && typeof value === "object" ? value as Record<string, unknown> : {};
  const status = normalizePointsRechargeStatus(
    record.status ?? record.rechargeStatus ?? record.paymentStatus ?? record.orderStatus,
  );
  return {
    amountCny: toNullableSdkworkOrderNumber(record.amountCny ?? record.amount),
    cashierUrl: toSdkworkOrderOptionalString(record.cashierUrl),
    orderId: toSdkworkOrderOptionalString(record.orderId ?? record.id),
    orderNo: toSdkworkOrderOptionalString(record.orderNo ?? record.outTradeNo),
    points: toSdkworkOrderNumber(record.points ?? record.grantAmount),
    qrCode: toSdkworkOrderOptionalString(record.qrCode ?? record.qrCodePayload ?? record.providerQrCode ?? record.cashierUrl),
    status,
  };
}

function normalizeCouponRechargeResult(value: unknown): SdkworkCouponRechargeResult {
  const record = value && typeof value === "object" ? value as Record<string, unknown> : {};
  const orderId = toSdkworkOrderOptionalString(record.orderId ?? record.id);
  if (!orderId) {
    throw new Error("Coupon redemption did not return an order id.");
  }
  const grantAmount = toSdkworkOrderNumber(record.grantAmount);
  if (grantAmount <= 0) {
    throw new Error("Coupon redemption did not return a Token Bank grant.");
  }
  const status = normalizePointsRechargeStatus(
    record.status ?? record.fulfillmentStatus ?? record.orderStatus,
  );
  return {
    grantAmount,
    orderId,
    orderNo: toSdkworkOrderOptionalString(record.orderNo ?? record.outTradeNo),
    replayed: record.replayed === true,
    status: status === "completed" ? "completed" : "pending",
    targetAsset: "token_bank",
  };
}

function normalizeMembershipCheckoutPayment(
  value: unknown,
  fallbackPackageId: number,
  paymentProduct: SdkworkMembershipCheckoutInput["paymentProduct"],
): SdkworkMembershipCheckoutPayment {
  const record = value && typeof value === "object" ? value as Record<string, unknown> : {};
  const paymentParams = record.paymentParams && typeof record.paymentParams === "object"
    ? record.paymentParams as Record<string, unknown>
    : {};
  const cashierUrl = toSdkworkOrderOptionalString(record.cashierUrl ?? paymentParams.cashierUrl);
  const providerQrCode = toSdkworkOrderOptionalString(
    paymentParams.qrCodeUrl
      ?? paymentParams.qrCode
      ?? paymentParams.qrCodePayload
      ?? paymentParams.codeUrl
      ?? record.qrCode
      ?? record.qrCodePayload
      ?? record.codeUrl,
  );
  return {
    amountCny: toNullableSdkworkOrderNumber(record.amountCny ?? record.amount),
    cashierUrl,
    durationDays: toNullableSdkworkOrderNumber(record.durationDays),
    orderId: toSdkworkOrderOptionalString(record.orderId ?? record.id),
    packageId: toNullableSdkworkOrderNumber(record.packageId) ?? fallbackPackageId,
    packageName: toSdkworkOrderOptionalString(record.packageName),
    qrCode: paymentProduct === "mobile_cashier_h5" ? cashierUrl : providerQrCode ?? cashierUrl,
    status: normalizePointsRechargeStatus(record.status ?? record.paymentStatus ?? record.orderStatus),
    targetLevelName: toSdkworkOrderOptionalString(record.targetLevelName ?? record.targetPlanName),
  };
}

function normalizeMembershipPaymentMethod(
  value: string | undefined,
  paymentProduct: NonNullable<SdkworkMembershipCheckoutInput["paymentProduct"]>,
): string {
  const normalized = value?.trim().toLowerCase().replace(/-/gu, "_");
  if (normalized) {
    return normalized === "wechat" ? "wechat_pay" : normalized;
  }
  return paymentProduct === "alipay_native" ? "alipay" : "wechat_pay";
}

function normalizePointsRechargeStatus(value: unknown): SdkworkPointsRechargePayment["status"] {
  const status = String(value ?? "").trim().toLowerCase();
  if (["completed", "complete", "paid", "success", "succeeded", "fulfilled"].includes(status)) {
    return "completed";
  }
  if (["failed", "cancelled", "canceled", "closed", "expired", "rejected"].includes(status)) {
    return "failed";
  }
  return "pending";
}

function buildServiceTree<TService>(
  template: ServiceTemplate,
  client: unknown,
  missingPathPrefix: readonly string[],
  servicePath: readonly string[] = [],
): TService {
  const service: Record<string, unknown> = {};
  for (const [key, marker] of Object.entries(template)) {
    const nextServicePath = [...servicePath, key];
    if (marker === true) {
      const missingPath = [...missingPathPrefix, ...nextServicePath].join(".");
      service[key] = (...args: Parameters<OrderSdkMethod>) =>
        callOrder(readMethod(client, nextServicePath), missingPath, ...args);
    } else {
      service[key] = buildServiceTree<Record<string, unknown>>(
        marker,
        client,
        missingPathPrefix,
        nextServicePath,
      );
    }
  }
  return service as TService;
}

function readMethod(root: unknown, path: readonly string[]): OrderSdkMethod | undefined {
  let node: unknown = root;
  for (const segment of path) {
    if (!node || typeof node !== "object") {
      return undefined;
    }
    const parent = node;
    node = (parent as Record<string, unknown>)[segment];
    if (typeof node === "function") {
      return node.bind(parent) as OrderSdkMethod;
    }
  }
  return typeof node === "function" ? (node as OrderSdkMethod) : undefined;
}

async function callOrder(
  method: OrderSdkMethod | undefined,
  name: string,
  ...args: Parameters<OrderSdkMethod>
): Promise<unknown> {
  if (!method) {
    throw new Error(`Missing SDKWork order SDK resource: ${name}`);
  }
  return method(...args);
}

function normalizeSessionToken(value: unknown): string | undefined {
  const normalized = typeof value === "string" ? value.trim() : "";
  return normalized || undefined;
}

function isSuccessCode(code: number | string | undefined): boolean {
  if (code === undefined || code === null || code === "") {
    return true;
  }
  if (typeof code === "number") {
    return code === 0;
  }
  return String(code).trim() === "0";
}

export function bootstrapSdkworkOrderAppService(
  input: BootstrapSdkworkOrderAppServiceInput,
): SdkworkOrderAppService {
  const transport = createOrderAppTransportClient(input);
  const service = createSdkworkOrderAppService({
    appClient: createOrderAppSdkClientFromTransport(transport),
  });
  configureSdkworkOrderAppServiceProvider(() => service);
  return service;
}

export {
  createOrderAppSdkClientFromTransport,
  createOrderAppTransportClient,
  resolveOrderAppApiOrigin,
  type BootstrapSdkworkOrderAppServiceInput,
} from "./transport.ts";

export {
  bootstrapSdkworkOrderBackendSdk,
  createOrderBackendTransportClient,
  getSdkworkOrderBackendSdkClient,
  resetSdkworkOrderBackendSdkClient,
  resolveOrderBackendApiOrigin,
  type BootstrapSdkworkOrderBackendSdkInput,
} from "./backend-transport.ts";

export {
  checkoutOwnerOrderRequestHash,
  checkoutQuoteRequestHash,
  checkoutSessionRequestHash,
  createCheckoutOwnerOrderWriteHeaders,
  createCheckoutQuoteWriteHeaders,
  createCheckoutSessionWriteHeaders,
  createSdkworkWriteCommandHeaders,
  stableCommandRequestHash,
  stableJsonRequestHash,
  writePayloadWithRouteParam,
  type CheckoutOwnerOrderHashInput,
  type CheckoutQuoteHashInput,
  type CheckoutSessionHashInput,
  type SdkworkWriteCommandHeaders,
} from "./write-command-headers.ts";

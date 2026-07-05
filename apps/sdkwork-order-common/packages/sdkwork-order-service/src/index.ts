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

type ServiceTemplate = { readonly [key: string]: true | ServiceTemplate };

type OrderServiceMethod = (...args: Parameters<OrderSdkMethod>) => Promise<unknown>;

export type SdkworkOrderOrdersService = ClientFromMethodTree<(typeof APP_ORDER_METHOD_TREE)["orders"]>;
export type SdkworkOrderRechargesService = ClientFromMethodTree<(typeof APP_ORDER_METHOD_TREE)["recharges"]>;

export type SdkworkOrderAppService = {
  orders: SdkworkOrderOrdersService;
  recharges: SdkworkOrderRechargesService;
};

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

import type {
  AccountValuePackageResponse,
  AccountValueRequestResponse,
  AfterSalesRequestSummary,
  ShipmentSummary,
  SdkworkOrderBackendClient,
  TokenBankPlanResponse,
} from "@sdkwork/order-pc-admin-core";
import { createSdkworkWriteCommandHeaders } from "@sdkwork/order-service";

export interface TradeOperationsQuery {
  page?: number;
  pageSize?: number;
  status?: string;
}

export interface TradeOperationsPage<T> {
  items: T[];
  page: number;
  pageSize: number;
  totalItems: number;
  totalPages: number;
}

export type TradeRequestAction = "approve" | "reject" | "retry";

export interface TradeOperationsService {
  listAfterSales(query?: TradeOperationsQuery): Promise<TradeOperationsPage<AfterSalesRequestSummary>>;
  listShipments(query?: TradeOperationsQuery): Promise<TradeOperationsPage<ShipmentSummary>>;
  listAccountValuePackages(query?: TradeOperationsQuery): Promise<TradeOperationsPage<AccountValuePackageResponse>>;
  listTokenBankPlans(query?: TradeOperationsQuery): Promise<TradeOperationsPage<TokenBankPlanResponse>>;
  listRefundRequests(query?: TradeOperationsQuery): Promise<TradeOperationsPage<AccountValueRequestResponse>>;
  reviewRefundRequest(id: string, action: TradeRequestAction): Promise<void>;
  listWithdrawalRequests(query?: TradeOperationsQuery): Promise<TradeOperationsPage<AccountValueRequestResponse>>;
  reviewWithdrawalRequest(id: string, action: TradeRequestAction): Promise<void>;
}

function unwrapPage<T>(value: unknown, page: number, pageSize: number): TradeOperationsPage<T> {
  const record = value && typeof value === "object" ? value as Record<string, unknown> : {};
  const data = record.data && typeof record.data === "object" ? record.data as Record<string, unknown> : record;
  const pageInfo = data.pageInfo && typeof data.pageInfo === "object" ? data.pageInfo as Record<string, unknown> : {};
  return {
    items: Array.isArray(data.items) ? data.items as T[] : [],
    page: Number(pageInfo.page ?? page),
    pageSize: Number(pageInfo.pageSize ?? pageSize),
    totalItems: Number(pageInfo.totalItems ?? 0),
    totalPages: Number(pageInfo.totalPages ?? 0),
  };
}

function listParams(query: TradeOperationsQuery = {}) {
  return { page: query.page ?? 1, pageSize: query.pageSize ?? 20, status: query.status };
}

function stringListParams(query: TradeOperationsQuery = {}) {
  return { page: String(query.page ?? 1), pageSize: String(query.pageSize ?? 20), status: query.status };
}

export function createTradeOperationsService(client: SdkworkOrderBackendClient): TradeOperationsService {
  const page = async <T>(query: TradeOperationsQuery, loader: () => Promise<unknown>) =>
    unwrapPage<T>(await loader(), query.page ?? 1, query.pageSize ?? 20);
  const review = async (
    scope: string,
    id: string,
    action: TradeRequestAction,
    api: { approve: Function; reject: Function; retry: Function },
  ) => {
    const body = { reviewComment: "manager trade operation" };
    const { idempotencyKey } = createSdkworkWriteCommandHeaders(`${scope}.${action}`, { id, ...body });
    await api[action](id, body, { idempotencyKey });
  };

  return {
    listAfterSales: (query = {}) => page<AfterSalesRequestSummary>(query, () => client.afterSales.management.list(stringListParams(query))),
    listShipments: (query = {}) => page<ShipmentSummary>(query, () => client.shipments.list(stringListParams(query))),
    listAccountValuePackages: (query = {}) => page<AccountValuePackageResponse>(query, () => client.backend.accountValuePackages.list(listParams(query))),
    listTokenBankPlans: (query = {}) => page<TokenBankPlanResponse>(query, () => client.backend.tokenBankPlans.list(listParams(query))),
    listRefundRequests: (query = {}) => page<AccountValueRequestResponse>(query, () => client.backend.refundRequests.list(listParams(query))),
    reviewRefundRequest: (id, action) => review("backend.refundRequests", id, action, client.backend.refundRequests),
    listWithdrawalRequests: (query = {}) => page<AccountValueRequestResponse>(query, () => client.backend.withdrawalRequests.list(listParams(query))),
    reviewWithdrawalRequest: (id, action) => review("backend.withdrawalRequests", id, action, client.backend.withdrawalRequests),
  };
}

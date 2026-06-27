import {
  useMemo,
  useSyncExternalStore,
} from "react";
import {
  createSdkworkOrderMessages,
  type SdkworkOrderMessagesOverrides,
} from "./order-copy";
import {
  createSdkworkOrderService,
  type SdkworkOrderCancelInput,
  type SdkworkOrderCancelResult,
  type SdkworkOrderDashboardData,
  type SdkworkOrderDashboardQuery,
  type SdkworkOrderDetail,
  type SdkworkOrderPaymentInput,
  type SdkworkOrderPaymentResult,
  type SdkworkOrderService,
  type SdkworkOrderStatus,
  type SdkworkOrderSummary,
} from "./order-service";

export type SdkworkOrderFilter = "all" | SdkworkOrderStatus;

export interface SdkworkOrderControllerState {
  activeFilter: SdkworkOrderFilter;
  currentPage: number;
  dashboard: SdkworkOrderDashboardData;
  detail?: SdkworkOrderDetail;
  isBootstrapped: boolean;
  isDetailLoading: boolean;
  isDetailOpen: boolean;
  isLoading: boolean;
  isMutating: boolean;
  lastError?: string;
  pageSize: number;
  selectedOrderId?: string;
  visibleOrders: SdkworkOrderSummary[];
}

export interface SdkworkOrderController {
  bootstrap(): Promise<SdkworkOrderControllerState>;
  cancelOrder(input: SdkworkOrderCancelInput): Promise<SdkworkOrderCancelResult>;
  closeDetail(): void;
  getState(): SdkworkOrderControllerState;
  openDetail(orderId: string): Promise<SdkworkOrderControllerState>;
  payOrder(input: SdkworkOrderPaymentInput): Promise<SdkworkOrderPaymentResult>;
  refresh(): Promise<SdkworkOrderControllerState>;
  service: SdkworkOrderService;
  setFilter(filter: SdkworkOrderFilter): Promise<SdkworkOrderControllerState>;
  setPage(page: number): Promise<SdkworkOrderControllerState>;
  subscribe(listener: () => void): () => void;
}

export interface CreateSdkworkOrderControllerOptions {
  initialState?: Partial<SdkworkOrderControllerState>;
  locale?: string | null;
  messages?: SdkworkOrderMessagesOverrides;
  service?: Partial<SdkworkOrderService>;
}

function deriveVisibleOrders(
  dashboard: SdkworkOrderDashboardData,
  activeFilter: SdkworkOrderFilter,
): SdkworkOrderSummary[] {
  if (activeFilter === "all") {
    return dashboard.orders;
  }

  return dashboard.orders.filter((order) => order.status === activeFilter);
}

export function createSdkworkOrderController(
  options: CreateSdkworkOrderControllerOptions = {},
): SdkworkOrderController {
  const messages = createSdkworkOrderMessages(options.locale, options.messages);
  const copy = messages.controller;
  const fallbackDashboard = (
    options.service?.getEmptyDashboard
    ?? createSdkworkOrderService({
      locale: options.locale,
      messages: options.messages,
    }).getEmptyDashboard
  )();
  const service: SdkworkOrderService = options.service
    ? {
        ...createSdkworkOrderService({
          locale: options.locale,
          messages: options.messages,
        }),
        ...options.service,
      }
    : createSdkworkOrderService({
        locale: options.locale,
        messages: options.messages,
      });
  const listeners = new Set<() => void>();
  let state: SdkworkOrderControllerState = {
    activeFilter: "all",
    currentPage: 1,
    dashboard: fallbackDashboard,
    isBootstrapped: false,
    isDetailLoading: false,
    isDetailOpen: false,
    isLoading: false,
    isMutating: false,
    pageSize: 20,
    visibleOrders: [],
    ...options.initialState,
  };
  state.visibleOrders = deriveVisibleOrders(state.dashboard, state.activeFilter);

  function emit(): void {
    listeners.forEach((listener) => listener());
  }

  function setState(
    next:
      | Partial<SdkworkOrderControllerState>
      | ((currentState: SdkworkOrderControllerState) => Partial<SdkworkOrderControllerState>),
  ): void {
    const partial = typeof next === "function" ? next(state) : next;
    state = {
      ...state,
      ...partial,
    };
    state.visibleOrders = deriveVisibleOrders(state.dashboard, state.activeFilter);
    emit();
  }

  function dashboardQuery(): SdkworkOrderDashboardQuery {
    const filter = state.activeFilter;
    return {
      page: state.currentPage,
      pageSize: state.pageSize,
      status: filter === "all" ? "all" : filter,
    };
  }

  async function loadDashboard(): Promise<SdkworkOrderDashboardData> {
    return service.getDashboard(dashboardQuery());
  }

  async function reload(): Promise<SdkworkOrderControllerState> {
    setState({ isLoading: true, lastError: undefined });
    try {
      const dashboard = await loadDashboard();
      setState({
        dashboard,
        isBootstrapped: true,
        isLoading: false,
      });
      return state;
    } catch (error) {
      const message = error instanceof Error ? error.message : copy.bootstrapFailed;
      setState({ isLoading: false, lastError: message });
      return state;
    }
  }

  return {
    async bootstrap() {
      // Controller owns the error surface — UI reads `state.lastError`.
      // Returning the state instead of re-throwing prevents unhandled
      // promise rejections when callers use `void controller.bootstrap()`.
      return reload();
    },

    async cancelOrder(input) {
      setState({
        isMutating: true,
        lastError: undefined,
      });

      try {
        const result = await service.cancelOrder(input);
        const dashboard = await loadDashboard();
        setState({
          dashboard,
          isBootstrapped: true,
          isMutating: false,
        });
        return result;
      } catch (error) {
        const message = error instanceof Error ? error.message : copy.cancelFailed;
        setState({
          isMutating: false,
          lastError: message,
        });
        return {
          cancelled: true,
          orderId: input.orderId,
          success: false,
          message,
        } satisfies SdkworkOrderCancelResult;
      }
    },

    closeDetail() {
      setState({
        detail: undefined,
        isDetailOpen: false,
        selectedOrderId: undefined,
      });
    },

    getState() {
      return state;
    },

    async openDetail(orderId) {
      setState({
        isDetailLoading: true,
        isDetailOpen: true,
        lastError: undefined,
        selectedOrderId: orderId,
      });

      try {
        const detail = await service.getOrderDetail(orderId);
        setState({
          detail,
          isDetailLoading: false,
          isDetailOpen: true,
          selectedOrderId: orderId,
        });
        return state;
      } catch (error) {
        const message = error instanceof Error ? error.message : copy.detailFailed;
        setState({
          isDetailLoading: false,
          isDetailOpen: false,
          lastError: message,
        });
        return state;
      }
    },

    async payOrder(input) {
      setState({
        isMutating: true,
        lastError: undefined,
      });

      try {
        const result = await service.payOrder(input);
        const dashboard = await loadDashboard();
        setState({
          dashboard,
          isBootstrapped: true,
          isMutating: false,
        });
        return result;
      } catch (error) {
        const message = error instanceof Error ? error.message : copy.payFailed;
        setState({
          isMutating: false,
          lastError: message,
        });
        return {
          amountCny: null,
          orderId: input.orderId,
          paymentParams: {},
          success: false,
          message,
        } satisfies SdkworkOrderPaymentResult;
      }
    },

    async refresh() {
      return reload();
    },

    service,

    async setFilter(filter) {
      if (filter === state.activeFilter) {
        return state;
      }
      // Switching the filter changes the underlying row set, so reset to the
      // first page to keep the visible page aligned with the new filter scope.
      // The dashboard is reloaded immediately so the server returns the
      // filtered subset with correct pagination metadata.
      setState({
        activeFilter: filter,
        currentPage: 1,
      });
      return reload();
    },

    async setPage(page) {
      const nextPage = Math.max(1, Math.floor(page));
      if (nextPage === state.currentPage) {
        return state;
      }
      setState({ currentPage: nextPage });
      return reload();
    },

    subscribe(listener) {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };
}

export function useSdkworkOrderController(
  controller?: SdkworkOrderController,
  options?: Pick<CreateSdkworkOrderControllerOptions, "locale" | "messages" | "service">,
): SdkworkOrderController {
  return useMemo(
    () => controller ?? createSdkworkOrderController({
      ...(options?.locale ? { locale: options.locale } : {}),
      ...(options?.messages ? { messages: options.messages } : {}),
      ...(options?.service ? { service: options.service } : {}),
    }),
    [controller, options?.locale, options?.messages, options?.service],
  );
}

export function useSdkworkOrderControllerState(
  controller: SdkworkOrderController,
): SdkworkOrderControllerState {
  return useSyncExternalStore(
    controller.subscribe,
    controller.getState,
    controller.getState,
  );
}

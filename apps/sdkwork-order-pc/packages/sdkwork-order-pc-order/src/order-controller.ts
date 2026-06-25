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
  dashboard: SdkworkOrderDashboardData;
  detail?: SdkworkOrderDetail;
  isBootstrapped: boolean;
  isDetailLoading: boolean;
  isDetailOpen: boolean;
  isLoading: boolean;
  isMutating: boolean;
  lastError?: string;
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
  setFilter(filter: SdkworkOrderFilter): void;
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
    dashboard: fallbackDashboard,
    isBootstrapped: false,
    isDetailLoading: false,
    isDetailOpen: false,
    isLoading: false,
    isMutating: false,
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

  async function loadDashboard(): Promise<SdkworkOrderDashboardData> {
    return service.getDashboard();
  }

  return {
    async bootstrap() {
      setState({
        isLoading: true,
        lastError: undefined,
      });

      try {
        const dashboard = await loadDashboard();
        setState({
          dashboard,
          isBootstrapped: true,
          isLoading: false,
        });
        return state;
      } catch (error) {
        setState({
          isLoading: false,
          lastError: error instanceof Error ? error.message : copy.bootstrapFailed,
        });
        throw error;
      }
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
        setState({
          isMutating: false,
          lastError: error instanceof Error ? error.message : copy.cancelFailed,
        });
        throw error;
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
        setState({
          isDetailLoading: false,
          lastError: error instanceof Error ? error.message : copy.detailFailed,
        });
        throw error;
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
        setState({
          isMutating: false,
          lastError: error instanceof Error ? error.message : copy.payFailed,
        });
        throw error;
      }
    },

    async refresh() {
      const dashboard = await loadDashboard();
      setState({
        dashboard,
        isBootstrapped: true,
        isLoading: false,
      });
      return state;
    },

    service,

    setFilter(filter) {
      setState({
        activeFilter: filter,
      });
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

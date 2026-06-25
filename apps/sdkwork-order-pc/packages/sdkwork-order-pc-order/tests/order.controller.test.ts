import { describe, expect, it, vi } from "vitest";
import {
  createSdkworkOrderController,
  type CreateSdkworkOrderControllerOptions,
  type SdkworkOrderMessagesOverrides,
} from "../src";

describe("sdkwork-order-pc-order controller", () => {
  it("bootstraps, filters orders, opens details, and refreshes after cancellation", async () => {
    const firstDashboard = {
      orders: [
        {
          createdAt: "2026-04-03T09:00:00.000Z",
          id: "ORDER-3",
          paidAmountCny: 0,
          status: "pending-payment" as const,
          statusLabel: "Pending payment",
          subject: "Pro Monthly",
          totalAmountCny: 199,
        },
        {
          createdAt: "2026-04-02T08:00:00.000Z",
          id: "ORDER-2",
          paidAmountCny: 699,
          status: "paid" as const,
          statusLabel: "Paid",
          subject: "Pro Annual",
          totalAmountCny: 699,
        },
      ],
      statistics: {
        completed: 8,
        pendingPayment: 1,
        pendingReceipt: 0,
        pendingShipment: 0,
        totalAmountCny: 2999,
        totalOrders: 9,
      },
    };
    const secondDashboard = {
      ...firstDashboard,
      orders: [
        {
          ...firstDashboard.orders[1],
        },
      ],
      statistics: {
        ...firstDashboard.statistics,
        pendingPayment: 0,
      },
    };
    const detail = {
      createdAt: "2026-04-03T09:00:00.000Z",
      id: "ORDER-3",
      items: [],
      status: "pending-payment" as const,
      statusLabel: "Pending payment",
      subject: "Pro Monthly",
      timeline: [],
      totalAmountCny: 199,
    };
    const service = {
      cancelOrder: vi.fn().mockResolvedValue({
        cancelled: true,
        orderId: "ORDER-3",
      }),
      getDashboard: vi
        .fn()
        .mockResolvedValueOnce(firstDashboard)
        .mockResolvedValueOnce(secondDashboard),
      getEmptyDashboard: vi.fn().mockReturnValue({
        orders: [],
        statistics: {
          completed: 0,
          pendingPayment: 0,
          pendingReceipt: 0,
          pendingShipment: 0,
          totalAmountCny: 0,
          totalOrders: 0,
        },
      }),
      getOrderDetail: vi.fn().mockResolvedValue(detail),
      payOrder: vi.fn(),
    };

    const controller = createSdkworkOrderController({
      service,
    });

    await controller.bootstrap();
    expect(controller.getState().visibleOrders).toHaveLength(2);

    controller.setFilter("pending-payment");
    expect(controller.getState().visibleOrders).toHaveLength(1);

    await controller.openDetail("ORDER-3");
    expect(controller.getState().detail?.id).toBe("ORDER-3");

    await controller.cancelOrder({
      cancelReason: "Changed package",
      orderId: "ORDER-3",
    });

    expect(service.cancelOrder).toHaveBeenCalledWith({
      cancelReason: "Changed package",
      orderId: "ORDER-3",
    });
    expect(controller.getState().dashboard.statistics.pendingPayment).toBe(0);
  });

  it("uses controller copy overrides when mutations fail without an Error instance", async () => {
    const controller = createSdkworkOrderController({
      messages: {
        controller: {
          cancelFailed: "Cancel fallback from overrides",
        },
      } satisfies SdkworkOrderMessagesOverrides,
      service: {
        cancelOrder: vi.fn().mockRejectedValue("bad"),
        getDashboard: vi.fn().mockResolvedValue({
          orders: [],
          statistics: {
            completed: 0,
            pendingPayment: 0,
            pendingReceipt: 0,
            pendingShipment: 0,
            totalAmountCny: 0,
            totalOrders: 0,
          },
        }),
        getEmptyDashboard: vi.fn().mockReturnValue({
          orders: [],
          statistics: {
            completed: 0,
            pendingPayment: 0,
            pendingReceipt: 0,
            pendingShipment: 0,
            totalAmountCny: 0,
            totalOrders: 0,
          },
        }),
        getOrderDetail: vi.fn(),
        payOrder: vi.fn(),
      },
    } satisfies CreateSdkworkOrderControllerOptions);

    await expect(
      controller.cancelOrder({
        orderId: "ORDER-3",
      }),
    ).rejects.toBe("bad");
    expect(controller.getState().lastError).toBe("Cancel fallback from overrides");
  });
});

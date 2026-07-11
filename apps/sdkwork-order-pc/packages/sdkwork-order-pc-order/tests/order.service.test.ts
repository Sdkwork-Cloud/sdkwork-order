import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  configureOrderServiceMockSession,
  createOrderAppServiceMock,
  resetOrderServiceMockSession,
} from "../../../tests/test-utils/order-service-mock";
import {
  createSdkworkOrderService,
  type CreateSdkworkOrderServiceOptions,
  type SdkworkOrderMessagesOverrides,
} from "../src";

const productImage = {
  kind: "image",
  publicUrl: "https://cdn.sdkwork.ai/pro-annual.png",
  source: "external_url",
  url: "https://cdn.sdkwork.ai/pro-annual.png",
} as const;

describe("sdkwork-order-pc-order service", () => {
  beforeEach(() => {
    configureOrderServiceMockSession({ authToken: "order-auth-token" });
  });

  afterEach(() => {
    resetOrderServiceMockSession();
  });

  it("maps orders, statistics, details, and payment actions into a reusable order center", async () => {
    const orderAppService = createOrderAppServiceMock({
      orders: {
        cancel: vi.fn().mockResolvedValue({
          code: 0,
          data: { accepted: true },
        }),
        retrieve: vi.fn().mockResolvedValue({
          createdAt: "2026-04-02T08:00:00.000Z",
          items: [
            {
              id: "ITEM-1",
              productImage,
              productName: "Pro Annual",
              quantity: 1,
              totalAmount: "699",
            },
          ],
          orderId: "ORDER-2",
          orderSn: "SN-ORDER-2",
          outTradeNo: "OUT-ORDER-2",
          paidAmount: "699",
          payTime: "2026-04-02T08:03:00.000Z",
          paymentMethod: "WECHAT",
          productImage,
          quantity: 1,
          remark: "Annual renewal",
          status: "paid",
          statusName: "Paid",
          subject: "Pro Annual",
          totalAmount: "699",
          transactionId: "TXN-ORDER-2",
        }),
        paymentSuccess: {
          retrieve: vi.fn().mockResolvedValue({
            orderId: "ORDER-2",
            paid: true,
            status: "paid",
            statusName: "Paid",
          }),
        },
        statistics: {
          retrieve: vi.fn().mockResolvedValue({
            completed: 8,
            pendingPayment: 1,
            pendingReceipt: 0,
            pendingShipment: 0,
            totalAmount: "2999",
            totalOrders: 9,
          }),
        },
        status: {
          retrieve: vi.fn().mockResolvedValue({
            orderId: "ORDER-2",
            status: "paid",
            statusName: "Paid",
          }),
        },
        list: vi.fn().mockResolvedValue({
          items: [
            {
              createdAt: "2026-04-03T09:00:00.000Z",
              discountAmount: "0",
              expireTime: "2026-04-03T09:30:00.000Z",
              orderId: "ORDER-3",
              orderSn: "SN-ORDER-3",
              paidAmount: "0",
              paymentMethod: "ALIPAY",
              quantity: 1,
              status: "pending_payment",
              statusName: "Pending payment",
              subject: "Pro Monthly",
              totalAmount: "199",
            },
            {
              createdAt: "2026-04-02T08:00:00.000Z",
              discountAmount: "0",
              orderId: "ORDER-2",
              orderSn: "SN-ORDER-2",
              paidAmount: "699",
              payTime: "2026-04-02T08:03:00.000Z",
              paymentMethod: "WECHAT",
              quantity: 1,
              status: "paid",
              statusName: "Paid",
              subject: "Pro Annual",
              totalAmount: "699",
            },
          ],
          pageInfo: {
            mode: "offset",
            page: 1,
            pageSize: 20,
            totalItems: 2,
            hasMore: false,
            totalPages: 1,
          },
        }),
        payments: {
          create: vi.fn().mockResolvedValue({
            amount: "199",
            orderId: "ORDER-3",
            outTradeNo: "OUT-ORDER-3",
            paymentId: "PAY-ORDER-3",
            paymentMethod: "ALIPAY",
            paymentParams: {
              payUrl: "https://pay.sdkwork.ai/alipay/ORDER-3",
            },
          }),
        },
      },
    });

    const service = createSdkworkOrderService({
      orderAppService,
    });

    const dashboard = await service.getDashboard();
    const detail = await service.getOrderDetail("ORDER-2");

    expect(dashboard.statistics).toMatchObject({
      completed: 8,
      pendingPayment: 1,
      totalAmountCny: 2999,
      totalOrders: 9,
    });
    expect(dashboard.orders[0]).toMatchObject({
      id: "ORDER-3",
      status: "pending-payment",
      subject: "Pro Monthly",
    });
    expect(dashboard.pagination).toMatchObject({
      hasMore: false,
      page: 1,
      pageSize: 20,
      total: 2,
      totalPages: 1,
    });
    expect(detail).toMatchObject({
      id: "ORDER-2",
      productImage,
      status: "paid",
      subject: "Pro Annual",
    });
    expect(detail.items[0]).toMatchObject({
      image: productImage,
      name: "Pro Annual",
    });
    expect(detail.timeline).toHaveLength(3);

    await expect(
      service.payOrder({
        orderId: "ORDER-3",
        paymentMethod: "ALIPAY",
      }),
    ).resolves.toMatchObject({
      amountCny: 199,
      orderId: "ORDER-3",
      paymentId: "PAY-ORDER-3",
      paymentMethod: "ALIPAY",
    });

    await expect(
      service.cancelOrder({
        cancelReason: "Switched plan",
        orderId: "ORDER-3",
      }),
    ).resolves.toEqual({
      cancelled: true,
      orderId: "ORDER-3",
    });

    const cancelMock = orderAppService.orders.cancel as ReturnType<typeof vi.fn>;
    const payMock = orderAppService.orders.payments.create as ReturnType<typeof vi.fn>;
    expect(cancelMock).toHaveBeenCalledWith(
      "ORDER-3",
      expect.objectContaining({
        idempotencyKey: expect.any(String),
        sdkworkRequestHash: expect.any(String),
        xIdempotencyFingerprint: expect.any(String),
      }),
      expect.objectContaining({ cancelReason: "Switched plan" }),
    );
    expect(payMock).toHaveBeenCalledWith(
      "ORDER-3",
      expect.objectContaining({ paymentMethod: "ALIPAY" }),
      expect.objectContaining({
        idempotencyKey: expect.any(String),
        sdkworkRequestHash: expect.any(String),
        xIdempotencyFingerprint: expect.any(String),
      }),
    );
  });

  it("returns a guest-safe empty order dashboard without creating a client", async () => {
    resetOrderServiceMockSession();
    const service = createSdkworkOrderService();

    const dashboard = await service.getDashboard();

    expect(dashboard.orders).toEqual([]);
    expect(dashboard.statistics.totalOrders).toBe(0);
  });

  it("uses copy overrides for order fallbacks, auth errors, and payment failure messages", async () => {
    const orderAppService = createOrderAppServiceMock({
      orders: {
        retrieve: vi.fn().mockResolvedValue({
          createdAt: "2026-04-02T08:00:00.000Z",
          items: [
            {
              quantity: 1,
              totalAmount: "199",
            },
          ],
          orderId: "ORDER-9",
          status: "pending_payment",
          totalAmount: "199",
        }),
        paymentSuccess: {
          retrieve: vi.fn().mockResolvedValue(null),
        },
        statistics: {
          retrieve: vi.fn().mockResolvedValue({
            totalOrders: 1,
          }),
        },
        status: {
          retrieve: vi.fn().mockResolvedValue({
            status: "pending_payment",
          }),
        },
        list: vi.fn().mockResolvedValue({
          items: [
            {
              createdAt: "2026-04-02T08:00:00.000Z",
              orderId: "ORDER-9",
              status: "pending_payment",
              totalAmount: "199",
            },
          ],
          pageInfo: {
            mode: "offset",
            page: 1,
            pageSize: 20,
            totalItems: 1,
            hasMore: false,
            totalPages: 1,
          },
        }),
      },
    });

    const service = createSdkworkOrderService({
      orderAppService,
      messages: {
        service: {
          itemFallbackName: "Fallback order item",
          payFailed: "Unable to restart payment from overrides",
          signInRequired: "Override order auth required",
          summaryFallbackSubject: "Fallback order",
        },
        status: {
          pendingPayment: "Needs payment",
        },
      } satisfies SdkworkOrderMessagesOverrides,
    } satisfies CreateSdkworkOrderServiceOptions);

    const dashboard = await service.getDashboard();
    const detail = await service.getOrderDetail("ORDER-9");

    expect(dashboard.orders[0]).toMatchObject({
      statusLabel: "Needs payment",
      subject: "Fallback order",
    });
    expect(detail.items[0]?.name).toBe("Fallback order item");
    expect(detail.statusLabel).toBe("Needs payment");

    resetOrderServiceMockSession();
    const guestService = createSdkworkOrderService({
      messages: {
        service: {
          signInRequired: "Override order auth required",
        },
      } satisfies SdkworkOrderMessagesOverrides,
    } satisfies CreateSdkworkOrderServiceOptions);

    await expect(guestService.getOrderDetail("ORDER-9")).rejects.toThrow("Override order auth required");

    configureOrderServiceMockSession({ authToken: "order-auth-token" });
    const failingService = createSdkworkOrderService({
      orderAppService: createOrderAppServiceMock({
        orders: {
          payments: {
            create: vi.fn().mockResolvedValue({
              code: 50001,
            }),
          },
        },
      }),
      messages: {
        service: {
          payFailed: "Unable to restart payment from overrides",
        },
      } satisfies SdkworkOrderMessagesOverrides,
    } satisfies CreateSdkworkOrderServiceOptions);

    await expect(
      failingService.payOrder({
        orderId: "ORDER-9",
      }),
    ).rejects.toThrow("Unable to restart payment from overrides");
  });
});

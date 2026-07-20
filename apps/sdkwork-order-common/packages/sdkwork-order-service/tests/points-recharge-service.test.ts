import { describe, expect, it, vi } from "vitest";
import {
  createSdkworkPointsRechargeService,
  createSdkworkCouponRechargeService,
  configureSdkworkOrderSessionTokenProvider,
  type SdkworkOrderAppService,
} from "../src/index.ts";

function createAppService(overrides: {
  packages?: unknown;
  create?: unknown;
  retrieve?: unknown;
} = {}): { appService: SdkworkOrderAppService; create: ReturnType<typeof vi.fn> } {
  const create = vi.fn().mockResolvedValue(overrides.create ?? {
    item: {
      amount: "90",
      orderId: "order-900",
      points: 900,
      qrCodePayload: "weixin://pay/order-900",
      status: "pending",
    },
  });
  return {
    appService: {
      memberships: {} as SdkworkOrderAppService["memberships"],
      orders: {} as SdkworkOrderAppService["orders"],
      recharges: {
        packages: {
          list: vi.fn().mockResolvedValue(overrides.packages ?? {
            items: [
              { id: "recharge-500", priceAmount: "50", currencyCode: "CNY", points: 500 },
              { id: "recharge-900", priceAmount: "90", currencyCode: "CNY", points: 900 },
            ],
          }),
        },
        orders: {
          create,
          retrieve: vi.fn().mockResolvedValue(overrides.retrieve ?? {
            item: { orderId: "order-1", points: 500, status: "paid" },
          }),
          list: vi.fn(),
          cancel: vi.fn(),
        },
        settings: { retrieve: vi.fn() },
      },
      withdrawals: {} as SdkworkOrderAppService["withdrawals"],
    },
    create,
  };
}

describe("createSdkworkPointsRechargeService", () => {
  it("resolves the selected package and creates the canonical points recharge order", async () => {
    const { appService, create } = createAppService();
    const service = createSdkworkPointsRechargeService({ appService });

    await expect(service.createOrder({ packageId: "recharge-900" })).resolves.toEqual({
      amountCny: 90,
      cashierUrl: undefined,
      orderId: "order-900",
      orderNo: undefined,
      points: 900,
      qrCode: "weixin://pay/order-900",
      status: "pending",
    });
    expect(create).toHaveBeenCalledWith(
      {
        amount: 90,
        currencyCode: "CNY",
        packageId: "recharge-900",
        paymentMethod: "wechat_pay",
        source: "membership-token-plan",
        subject: "points_recharge",
        targetAsset: "points",
      },
      expect.objectContaining({
        idempotencyKey: expect.any(String),
        sdkworkRequestHash: expect.stringContaining("recharges.orders.create"),
      }),
    );
  });

  it("rejects an unavailable package before creating an order", async () => {
    const { appService, create } = createAppService();
    const service = createSdkworkPointsRechargeService({ appService });

    await expect(service.createOrder({ packageId: "missing" })).rejects.toThrow(
      "selected recharge package is unavailable",
    );
    expect(create).not.toHaveBeenCalled();
  });

  it("maps paid checkout status to completed", async () => {
    const { appService } = createAppService();
    const service = createSdkworkPointsRechargeService({ appService });

    await expect(service.getOrderStatus("order-1")).resolves.toEqual(
      expect.objectContaining({ orderId: "order-1", status: "completed" }),
    );
  });
});

describe("createSdkworkCouponRechargeService", () => {
  it("sends only the coupon code and server-controlled Token Bank target", async () => {
    const { appService, create } = createAppService({
      create: {
        item: {
          grantAmount: "50",
          orderId: "order-coupon-1",
          orderNo: "CP1001",
          status: "fulfilled",
          targetAsset: "token_bank",
        },
      },
    });
    configureSdkworkOrderSessionTokenProvider(() => ({ accessToken: "session-token" }));
    const service = createSdkworkCouponRechargeService({ appService });

    await expect(service.redeem("  WELCOME  ")).resolves.toEqual({
      grantAmount: 50,
      orderId: "order-coupon-1",
      orderNo: "CP1001",
      replayed: false,
      status: "completed",
      targetAsset: "token_bank",
    });
    expect(create).toHaveBeenCalledWith(
      {
        amount: 0,
        couponCode: "WELCOME",
        currencyCode: "CNY",
        subject: "coupon_recharge",
        targetAsset: "token_bank",
      },
      expect.objectContaining({
        idempotencyKey: expect.any(String),
        sdkworkRequestHash: expect.stringContaining("recharges.orders.create"),
      }),
    );
    expect(create.mock.calls[0]?.[0]).not.toHaveProperty("grantAmount");
    configureSdkworkOrderSessionTokenProvider(null);
  });
});

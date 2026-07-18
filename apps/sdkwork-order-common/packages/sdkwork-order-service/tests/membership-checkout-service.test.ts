import { describe, expect, it, vi } from "vitest";

import {
  configureSdkworkOrderSessionTokenProvider,
  createSdkworkMembershipCheckoutService,
  type SdkworkOrderAppService,
} from "../src/index.ts";

function createAppService() {
  const create = vi.fn().mockResolvedValue({
    item: {
      amount: "58",
      cashierUrl: "http://127.0.0.1:3901/cashier/membership-order-58",
      orderId: "membership-order-58",
      packageId: "58",
      paymentParams: { qrCodeUrl: "weixin://pay/membership-order-58" },
      status: "pending",
    },
  });
  const retrieve = vi.fn().mockResolvedValue({ item: { paid: true, status: "paid" } });
  const appService = {
    memberships: { orders: { create } },
    orders: { paymentSuccess: { retrieve } },
    recharges: {},
  } as unknown as SdkworkOrderAppService;
  return { appService, create, retrieve };
}

describe("createSdkworkMembershipCheckoutService", () => {
  it("defaults to the provider-independent H5 cashier", async () => {
    configureSdkworkOrderSessionTokenProvider(() => ({ accessToken: "access-token" }));
    const { appService, create } = createAppService();
    const service = createSdkworkMembershipCheckoutService({ appService });

    await expect(service.createCheckout({
      action: "purchase",
      packageId: 58,
    })).resolves.toMatchObject({
      amountCny: 58,
      orderId: "membership-order-58",
      packageId: 58,
      qrCode: "http://127.0.0.1:3901/cashier/membership-order-58",
      status: "pending",
    });
    expect(create).toHaveBeenCalledWith(
      {
        packageId: "58",
        paymentMethod: "wechat_pay",
        paymentProduct: "mobile_cashier_h5",
      },
      expect.objectContaining({
        idempotencyKey: "membership-checkout:58:purchase",
        sdkworkRequestHash: expect.stringContaining("memberships.orders.create"),
      }),
    );
  });

  it("creates membership-subject orders through the order app service", async () => {
    configureSdkworkOrderSessionTokenProvider(() => ({ accessToken: "access-token" }));
    const { appService, create } = createAppService();
    const service = createSdkworkMembershipCheckoutService({ appService });

    await expect(service.createCheckout({
      action: "purchase",
      packageId: 58,
      paymentProduct: "wechat_native",
    })).resolves.toMatchObject({
      amountCny: 58,
      orderId: "membership-order-58",
      packageId: 58,
      qrCode: "weixin://pay/membership-order-58",
      status: "pending",
    });
    expect(create).toHaveBeenCalledWith(
      {
        packageId: "58",
        paymentMethod: "wechat_pay",
        paymentProduct: "wechat_native",
      },
      expect.objectContaining({
        idempotencyKey: "membership-checkout:58:purchase",
        sdkworkRequestHash: expect.stringContaining("memberships.orders.create"),
      }),
    );
  });

  it("reads membership payment completion through the order app service", async () => {
    configureSdkworkOrderSessionTokenProvider(() => ({ accessToken: "access-token" }));
    const { appService, retrieve } = createAppService();
    const service = createSdkworkMembershipCheckoutService({ appService });

    await expect(service.getCheckoutStatus("membership-order-58")).resolves.toEqual({
      amountCny: null,
      durationDays: null,
      orderId: "membership-order-58",
      packageId: null,
      status: "completed",
    });
    expect(retrieve).toHaveBeenCalledWith("membership-order-58");
  });
});

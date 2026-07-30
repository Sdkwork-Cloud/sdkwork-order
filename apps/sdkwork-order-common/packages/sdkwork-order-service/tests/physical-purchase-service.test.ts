import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  configureSdkworkOrderSessionTokenProvider,
  createSdkworkPhysicalPurchaseService,
  type SdkworkOrderAppService,
} from "../src/index";

function appServiceFixture() {
  const createSession = vi.fn().mockResolvedValue({
    checkoutSessionId: "checkout-1",
    currencyCode: "CNY",
    discountAmount: "0",
    originalAmount: "19800",
    payableAmount: "19800",
    status: "quoted",
  });
  const createQuote = vi.fn().mockResolvedValue({
    checkoutSessionId: "checkout-1",
    currencyCode: "CNY",
    discountAmount: "0",
    originalAmount: "19800",
    payableAmount: "19800",
    quoteId: "quote-1",
  });
  const createOrder = vi.fn().mockResolvedValue({
    orderId: "order-1",
    orderNo: "order-no-1",
    orderSn: "order-sn-1",
    status: "pending_payment",
    totalAmount: "19800",
  });
  const appService = {
    checkout: {
      sessions: {
        create: createSession,
        orders: { create: createOrder },
        quotes: { create: createQuote },
      },
    },
    memberships: {},
    orders: {},
    recharges: {},
    withdrawals: {},
  } as unknown as SdkworkOrderAppService;
  return { appService, createOrder, createQuote, createSession };
}

describe("createSdkworkPhysicalPurchaseService", () => {
  beforeEach(() => {
    configureSdkworkOrderSessionTokenProvider(() => ({ accessToken: "access-token" }));
  });

  it("creates an authoritative checkout snapshot and quote before order placement", async () => {
    const { appService, createOrder, createQuote, createSession } = appServiceFixture();
    const service = createSdkworkPhysicalPurchaseService({ appService });

    await expect(service.prepareCheckout({
      currencyCode: "cny",
      items: [{ quantity: 2, skuId: " sku-1 " }],
      shippingAddress: {
        city: "Hangzhou",
        countryCode: "cn",
        detailAddress: "No. 1 Market Road",
        district: "Xihu",
        province: "Zhejiang",
        receiverName: "Buyer",
        receiverPhone: "13800000000",
      },
    })).resolves.toEqual({
      checkoutSessionId: "checkout-1",
      currencyCode: "CNY",
      discountAmount: "0",
      originalAmount: "19800",
      payableAmount: "19800",
      quoteId: "quote-1",
      status: "quoted",
    });

    expect(createSession).toHaveBeenCalledWith(
      {
        currencyCode: "CNY",
        items: [{ quantity: "2", skuId: "sku-1" }],
        shippingAddress: {
          city: "Hangzhou",
          countryCode: "CN",
          detailAddress: "No. 1 Market Road",
          district: "Xihu",
          postalCode: undefined,
          province: "Zhejiang",
          receiverName: "Buyer",
          receiverPhone: "13800000000",
        },
      },
      { idempotencyKey: expect.any(String) },
    );
    expect(createQuote).toHaveBeenCalledWith(
      "checkout-1",
      { idempotencyKey: expect.any(String) },
    );

    await expect(service.placeOrder("checkout-1")).resolves.toMatchObject({
      orderId: "order-1",
      status: "pending_payment",
    });
    expect(createOrder).toHaveBeenCalledWith(
      "checkout-1",
      { idempotencyKey: expect.any(String) },
    );
  });

  it("rejects duplicate SKU lines before dispatch", async () => {
    const { appService, createSession } = appServiceFixture();
    const service = createSdkworkPhysicalPurchaseService({ appService });

    await expect(service.prepareCheckout({
      items: [
        { quantity: 1, skuId: "sku-1" },
        { quantity: 2, skuId: "sku-1" },
      ],
      shippingAddress: {
        city: "Hangzhou",
        countryCode: "CN",
        detailAddress: "No. 1 Market Road",
        province: "Zhejiang",
        receiverName: "Buyer",
        receiverPhone: "13800000000",
      },
    })).rejects.toThrow("Duplicate physical SKU lines");
    expect(createSession).not.toHaveBeenCalled();
  });
});

import { describe, expect, it } from "vitest";
import {
  checkoutSessionRequestHash,
  createSdkworkWriteCommandHeaders,
  stableJsonRequestHash,
} from "../src/write-command-headers.ts";

describe("write-command-headers", () => {
  it("matches Rust checkout session command digest scopes", () => {
    const hash = checkoutSessionRequestHash({
      tenantId: "100001",
      organizationId: "0",
      ownerUserId: "user-1",
      currencyCode: "CNY",
      lines: [{ skuId: "sku-1", quantity: 1 }],
      requestNo: "request-1",
    });

    expect(hash).toBe("checkout.sessions.create-100001-0-user-1-CNY-sku-1-1-request-1");
  });

  it("uses operationId scope for JSON body writes", () => {
    const hash = stableJsonRequestHash("recharges.orders.create", {
      amount: "100",
      currencyCode: "CNY",
      paymentMethod: "WECHAT",
    });
    expect(hash.startsWith("recharges.orders.create-")).toBe(true);
  });

  it("createSdkworkWriteCommandHeaders returns both header fields", () => {
    const headers = createSdkworkWriteCommandHeaders("orders.cancel", {
      orderId: "o-1",
      cancelReason: "test",
    });
    expect(headers.idempotencyKey).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
    );
    expect(headers.sdkworkRequestHash.startsWith("orders.cancel-")).toBe(true);
  });
});

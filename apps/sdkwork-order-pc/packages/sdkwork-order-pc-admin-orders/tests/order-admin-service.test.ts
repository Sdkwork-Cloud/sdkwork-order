import { describe, expect, it, vi } from "vitest";
import { createOrderAdminService } from "../src/order-admin-service";

describe("createOrderAdminService", () => {
  it("lists orders with v3 envelope unwrapping", async () => {
    const client = {
      orders: {
        admin: {
          list: vi.fn().mockResolvedValue({
            code: 0,
            data: {
              items: [{ orderId: "o-1", subject: "Test", status: "pending_payment" }],
              pageInfo: { mode: "offset", page: 1, pageSize: 20, totalItems: 1, totalPages: 1 },
            },
            traceId: "trace-1",
          }),
          retrieve: vi.fn(),
          cancel: vi.fn(),
          close: vi.fn(),
        },
      },
    };

    const service = createOrderAdminService(client as never);
    const page = await service.listOrders({ page: 1, pageSize: 20 });

    expect(page.items).toHaveLength(1);
    expect(page.items[0]?.orderId).toBe("o-1");
    expect(page.totalItems).toBe(1);
  });
});

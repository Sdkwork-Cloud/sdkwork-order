import { describe, expect, it, vi } from "vitest";

import { createTradeOperationsService } from "../src/service";

function clientFixture() {
  return {
    afterSales: { management: { list: vi.fn() } },
    shipments: { list: vi.fn() },
    backend: {
      accountValuePackages: { list: vi.fn() },
      tokenBankPlans: { list: vi.fn() },
      refundRequests: { list: vi.fn(), approve: vi.fn(), reject: vi.fn(), retry: vi.fn() },
      withdrawalRequests: { list: vi.fn(), approve: vi.fn(), reject: vi.fn(), retry: vi.fn() },
    },
  };
}

describe("createTradeOperationsService", () => {
  it("unwraps refund pagination", async () => {
    const client = clientFixture();
    client.backend.refundRequests.list.mockResolvedValue({
      items: [{ accountValueRequestId: "refund-1", status: "pending" }],
      pageInfo: { page: 2, pageSize: 20, totalItems: "21", totalPages: 2 },
    });

    const service = createTradeOperationsService(client as never);
    const result = await service.listRefundRequests({ page: 2, pageSize: 20 });

    expect(result.items[0]?.accountValueRequestId).toBe("refund-1");
    expect(result.totalItems).toBe(21);
    expect(result.totalPages).toBe(2);
  });

  it("sends the generated idempotency parameter when approving a refund", async () => {
    const client = clientFixture();
    client.backend.refundRequests.approve.mockResolvedValue({ accepted: true });

    const service = createTradeOperationsService(client as never);
    await service.reviewRefundRequest("refund-1", "approve");

    expect(client.backend.refundRequests.approve).toHaveBeenCalledWith(
      "refund-1",
      { reviewComment: "manager trade operation" },
      {
        idempotencyKey: expect.any(String),
      },
    );
  });
});

import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { SdkworkOrderAdminOrdersPage } from "../src/pages/AdminOrdersPage";

describe("order admin capabilities", () => {
  it("keeps detail access but hides cancel and close for read-only operators", async () => {
    const service = {
      cancelOrder: vi.fn(),
      closeOrder: vi.fn(),
      getOrder: vi.fn(),
      listOrders: vi.fn().mockResolvedValue({
        items: [{
          orderId: "order-1",
          orderNo: "ORDER-1",
          subject: "Commercial order",
          status: "pending_payment",
          statusName: "Pending payment",
          totalAmount: "99.00",
          currencyCode: "CNY",
          createdAt: "2026-07-17T00:00:00.000Z",
        }],
        page: 1,
        pageSize: 20,
        totalItems: 1,
        totalPages: 1,
      }),
    };

    render(
      <SdkworkOrderAdminOrdersPage
        capabilities={{ canManageOrders: false }}
        service={service as never}
      />,
    );

    await waitFor(() => expect(screen.getByText("Commercial order")).toBeInTheDocument());
    expect(screen.getByRole("button", { name: "详情" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "取消" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "关闭" })).not.toBeInTheDocument();
  });
});

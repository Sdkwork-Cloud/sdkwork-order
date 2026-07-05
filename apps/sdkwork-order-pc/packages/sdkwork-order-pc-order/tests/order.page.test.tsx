import { fireEvent, render, screen } from "@testing-library/react";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it, vi } from "vitest";
import { SdkworkThemeProvider } from "@sdkwork/ui-pc-react/theme";
import { SdkworkOrderPage, createSdkworkOrderController } from "../src";

describe("sdkwork-order-pc-order page", () => {
  it("renders the reusable order center and opens the detail drawer", async () => {
    const controller = createSdkworkOrderController({
      service: {
        cancelOrder: vi.fn(),
        getDashboard: vi.fn().mockResolvedValue({
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
          ],
          pagination: {
            hasMore: false,
            page: 1,
            pageSize: 20,
            total: 1,
            totalPages: 1,
          },
          statistics: {
            completed: 8,
            pendingPayment: 1,
            pendingReceipt: 0,
            pendingShipment: 0,
            totalAmountCny: 2999,
            totalOrders: 9,
          },
        }),
        getEmptyDashboard: vi.fn().mockReturnValue({
          orders: [],
          pagination: {
            hasMore: false,
            page: 1,
            pageSize: 20,
            total: 0,
            totalPages: 0,
          },
          statistics: {
            completed: 0,
            pendingPayment: 0,
            pendingReceipt: 0,
            pendingShipment: 0,
            totalAmountCny: 0,
            totalOrders: 0,
          },
        }),
        getOrderDetail: vi.fn().mockResolvedValue({
          createdAt: "2026-04-03T09:00:00.000Z",
          id: "ORDER-3",
          items: [
            {
              id: "ITEM-3",
              name: "Pro Monthly",
              quantity: 1,
              totalAmountCny: 199,
            },
          ],
          status: "pending-payment" as const,
          statusLabel: "Pending payment",
          subject: "Pro Monthly",
          timeline: [],
          totalAmountCny: 199,
        }),
        payOrder: vi.fn(),
      },
    });

    render(
      <SdkworkThemeProvider defaultTheme="light">
        <SdkworkOrderPage controller={controller} />
      </SdkworkThemeProvider>,
    );

    expect(
      await screen.findByRole("heading", {
        name: "Bills & Reports",
        level: 1,
      }),
    ).toBeInTheDocument();
    expect(screen.getByText("Pro Monthly")).toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", {
        name: /view details/i,
      }),
    );

    expect(await screen.findByText(/order detail/i)).toBeInTheDocument();
  });

  it("keeps the order hero free of raw white utility styling", () => {
    const pageSource = readFileSync(
      resolve(import.meta.dirname, "../src/pages/OrderPage.tsx"),
      "utf8",
    );

    expect(pageSource).not.toContain("border-white/10");
    expect(pageSource).not.toContain("bg-white/10");
    expect(pageSource).not.toContain("text-white/72");
    expect(pageSource).not.toContain("text-white/60");
  });
});

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SdkworkThemeProvider } from "@sdkwork/ui-pc-react/theme";
import {
  SdkworkPointsRechargeDialog,
  type SdkworkPointsRechargeService,
} from "../src";

vi.mock("qrcode", () => ({
  toDataURL: vi.fn().mockResolvedValue("data:image/png;base64,recharge-qr"),
}));

describe("SdkworkPointsRechargeDialog", () => {
  it("selects a recharge package id and completes order creation inside the dialog", async () => {
    const createOrder = vi.fn().mockResolvedValue({
      amountCny: 75,
      orderId: "recharge-order-750",
      points: 750,
      qrCode: "weixin://pay/recharge-order-750",
      status: "pending" as const,
    });
    const service: SdkworkPointsRechargeService = {
      listPackages: vi.fn().mockResolvedValue([
        { id: "recharge-500", bonusPoints: 0, currencyCode: "CNY", grantAmount: 500, points: 500, priceAmount: 50 },
        { id: "recharge-750", bonusPoints: 0, currencyCode: "CNY", grantAmount: 750, points: 750, priceAmount: 75 },
      ]),
      createOrder,
      getOrderStatus: vi.fn().mockResolvedValue({
        amountCny: 75,
        orderId: "recharge-order-750",
        points: 750,
        status: "pending" as const,
      }),
    };

    const view = render(
      <SdkworkThemeProvider defaultTheme="light">
        <SdkworkPointsRechargeDialog isOpen onClose={vi.fn()} service={service} />
      </SdkworkThemeProvider>,
    );

    fireEvent.click(await screen.findByRole("button", { name: /750/ }));
    fireEvent.click(screen.getByRole("button", { name: "同意并支付" }));

    await waitFor(() => expect(createOrder).toHaveBeenCalledWith({
      packageId: "recharge-750",
      paymentMethod: "wechat_pay",
    }));
    expect(await screen.findByRole("img", { name: "请扫码完成支付" })).toHaveAttribute(
      "src",
      "data:image/png;base64,recharge-qr",
    );
    expect(screen.getByText("您已同意《积分充值服务协议》")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "同意并支付" })).not.toBeInTheDocument();
    view.unmount();
  });
});

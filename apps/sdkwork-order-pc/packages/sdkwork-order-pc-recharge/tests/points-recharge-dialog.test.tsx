import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SdkworkThemeProvider } from "@sdkwork/ui-pc-react/theme";
import {
  SdkworkPointsRechargeDialog,
  SdkworkPointsRechargeInline,
  type SdkworkPointsRechargeService,
} from "../src";

vi.mock("qrcode", () => ({
  toDataURL: vi.fn().mockResolvedValue("data:image/png;base64,recharge-qr"),
}));

describe("SDKWork points recharge surfaces", () => {
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
    const qrAgreement = screen.getByText("您已同意《积分充值服务协议》");
    expect(qrAgreement).toBeInTheDocument();
    expect(qrAgreement.querySelector(".sdkwork-points-recharge-dialog__agreement-check svg")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "同意并支付" })).not.toBeInTheDocument();
    view.unmount();
  });

  it("keeps agreement for the current dialog session and pays a newly selected package automatically", async () => {
    let resolveFirstOrder!: (payment: {
      amountCny: number;
      orderId: string;
      points: number;
      qrCode: string;
      status: "pending";
    }) => void;
    const firstOrder = new Promise<{
      amountCny: number;
      orderId: string;
      points: number;
      qrCode: string;
      status: "pending";
    }>((resolve) => {
      resolveFirstOrder = resolve;
    });
    const createOrder = vi.fn()
      .mockReturnValueOnce(firstOrder)
      .mockResolvedValueOnce({
        amountCny: 50,
        orderId: "recharge-order-500",
        points: 500,
        qrCode: "weixin://pay/recharge-order-500",
        status: "pending" as const,
      });
    const service: SdkworkPointsRechargeService = {
      listPackages: vi.fn().mockResolvedValue([
        { id: "recharge-500", bonusPoints: 0, currencyCode: "CNY", grantAmount: 500, points: 500, priceAmount: 50 },
        { id: "recharge-750", bonusPoints: 0, currencyCode: "CNY", grantAmount: 750, points: 750, priceAmount: 75 },
      ]),
      createOrder,
      getOrderStatus: vi.fn().mockImplementation(async (orderId: string) => ({
        amountCny: orderId.endsWith("500") ? 50 : 75,
        orderId,
        points: orderId.endsWith("500") ? 500 : 750,
        status: "pending" as const,
      })),
    };

    const view = render(
      <SdkworkThemeProvider defaultTheme="light">
        <SdkworkPointsRechargeDialog isOpen onClose={vi.fn()} service={service} />
      </SdkworkThemeProvider>,
    );

    const package750 = await screen.findByRole("button", { name: /750/ });
    const package500 = screen.getByRole("button", { name: /500/ });
    fireEvent.click(package750);
    const confirmPayment = screen.getByRole("button", { name: "同意并支付" });
    fireEvent.click(confirmPayment);
    fireEvent.click(confirmPayment);

    await waitFor(() => {
      expect(createOrder).toHaveBeenCalledTimes(1);
      expect(package750).toBeDisabled();
      expect(package500).toBeDisabled();
    });
    expect(screen.getByText("您已同意《积分充值服务协议》")
      .querySelector(".sdkwork-points-recharge-dialog__agreement-check svg")).toBeInTheDocument();
    fireEvent.click(package500);
    expect(package750).toHaveAttribute("aria-pressed", "true");

    await act(async () => {
      resolveFirstOrder({
        amountCny: 75,
        orderId: "recharge-order-750",
        points: 750,
        qrCode: "weixin://pay/recharge-order-750",
        status: "pending",
      });
      await firstOrder;
    });

    expect(await screen.findByRole("img", { name: "请扫码完成支付" })).toBeInTheDocument();
    fireEvent.click(package500);

    expect(screen.queryByRole("img", { name: "请扫码完成支付" })).not.toBeInTheDocument();
    const acceptedAgreement = screen.getByText("您已同意《积分充值服务协议》");
    expect(acceptedAgreement).toBeInTheDocument();
    expect(acceptedAgreement.querySelector(".sdkwork-points-recharge-dialog__agreement-check svg")).toBeInTheDocument();
    await waitFor(() => expect(createOrder).toHaveBeenNthCalledWith(2, {
      packageId: "recharge-500",
      paymentMethod: "wechat_pay",
    }));
    view.unmount();
  });

  it("requires agreement again after the dialog is closed and reopened", async () => {
    const createOrder = vi.fn().mockResolvedValue({
      amountCny: 50,
      orderId: "recharge-order-500",
      points: 500,
      qrCode: "weixin://pay/recharge-order-500",
      status: "pending" as const,
    });
    const service: SdkworkPointsRechargeService = {
      listPackages: vi.fn().mockResolvedValue([
        { id: "recharge-500", bonusPoints: 0, currencyCode: "CNY", grantAmount: 500, points: 500, priceAmount: 50 },
      ]),
      createOrder,
      getOrderStatus: vi.fn().mockResolvedValue({
        amountCny: 50,
        orderId: "recharge-order-500",
        points: 500,
        status: "pending" as const,
      }),
    };
    const renderDialog = (isOpen: boolean) => (
      <SdkworkThemeProvider defaultTheme="light">
        <SdkworkPointsRechargeDialog isOpen={isOpen} onClose={vi.fn()} service={service} />
      </SdkworkThemeProvider>
    );
    const view = render(renderDialog(true));

    fireEvent.click(await screen.findByRole("button", { name: "同意并支付" }));
    await waitFor(() => expect(createOrder).toHaveBeenCalledTimes(1));

    view.rerender(renderDialog(false));
    view.rerender(renderDialog(true));

    expect(await screen.findByRole("button", { name: "同意并支付" })).toBeInTheDocument();
    expect(createOrder).toHaveBeenCalledTimes(1);
    view.unmount();
  });

  it("ignores a stale completion poll after the customer switches packages", async () => {
    let resolveStatus!: (payment: {
      amountCny: number;
      orderId: string;
      points: number;
      status: "completed";
    }) => void;
    const statusResult = new Promise<{
      amountCny: number;
      orderId: string;
      points: number;
      status: "completed";
    }>((resolve) => {
      resolveStatus = resolve;
    });
    const onCompleted = vi.fn();
    const service: SdkworkPointsRechargeService = {
      listPackages: vi.fn().mockResolvedValue([
        { id: "recharge-500", bonusPoints: 0, currencyCode: "CNY", grantAmount: 500, points: 500, priceAmount: 50 },
        { id: "recharge-750", bonusPoints: 0, currencyCode: "CNY", grantAmount: 750, points: 750, priceAmount: 75 },
      ]),
      createOrder: vi.fn()
        .mockResolvedValueOnce({
          amountCny: 75,
          orderId: "recharge-order-750",
          points: 750,
          qrCode: "weixin://pay/recharge-order-750",
          status: "pending" as const,
        })
        .mockResolvedValueOnce({
          amountCny: 50,
          orderId: "recharge-order-500",
          points: 500,
          qrCode: "weixin://pay/recharge-order-500",
          status: "pending" as const,
        }),
      getOrderStatus: vi.fn().mockImplementation((orderId: string) => orderId === "recharge-order-750"
        ? statusResult
        : Promise.resolve({
            amountCny: 50,
            orderId,
            points: 500,
            status: "pending" as const,
          })),
    };

    const view = render(
      <SdkworkThemeProvider defaultTheme="light">
        <SdkworkPointsRechargeDialog
          isOpen
          onClose={vi.fn()}
          onCompleted={onCompleted}
          service={service}
        />
      </SdkworkThemeProvider>,
    );

    fireEvent.click(await screen.findByRole("button", { name: /750/ }));
    fireEvent.click(screen.getByRole("button", { name: "同意并支付" }));
    expect(await screen.findByRole("img", { name: "请扫码完成支付" })).toBeInTheDocument();
    await waitFor(() => expect(service.getOrderStatus).toHaveBeenCalledWith("recharge-order-750"));

    fireEvent.click(screen.getByRole("button", { name: /500/ }));
    await waitFor(() => expect(service.createOrder).toHaveBeenLastCalledWith({
      packageId: "recharge-500",
      paymentMethod: "wechat_pay",
    }));
    await act(async () => {
      resolveStatus({
        amountCny: 75,
        orderId: "recharge-order-750",
        points: 750,
        status: "completed",
      });
      await statusResult;
    });

    expect(screen.getByRole("button", { name: /500/ })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByText("您已同意《积分充值服务协议》")).toBeInTheDocument();
    expect(screen.queryByText("支付完成，积分已到账")).not.toBeInTheDocument();
    expect(onCompleted).not.toHaveBeenCalled();
    view.unmount();
  });

  it("provides the dialog checkout flow as an inline surface", async () => {
    const service: SdkworkPointsRechargeService = {
      listPackages: vi.fn().mockResolvedValue([
        { id: "recharge-500", bonusPoints: 0, currencyCode: "CNY", grantAmount: 500, points: 500, priceAmount: 50 },
      ]),
      createOrder: vi.fn().mockResolvedValue({
        amountCny: 50,
        orderId: "recharge-order-inline",
        points: 500,
        qrCode: "weixin://pay/recharge-order-inline",
        status: "pending" as const,
      }),
      getOrderStatus: vi.fn().mockResolvedValue({
        amountCny: 50,
        orderId: "recharge-order-inline",
        points: 500,
        status: "pending" as const,
      }),
    };

    const view = render(
      <SdkworkThemeProvider defaultTheme="light">
        <SdkworkPointsRechargeInline currentPoints={1250} service={service} />
      </SdkworkThemeProvider>,
    );

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(screen.getByRole("region", { name: "积分账户" })).toHaveClass("sdkwork-points-recharge-inline");
    expect(await screen.findByRole("button", { name: /500/ })).toHaveAttribute("aria-pressed", "true");

    fireEvent.click(screen.getByRole("button", { name: "同意并支付" }));

    expect(await screen.findByRole("img", { name: "请扫码完成支付" })).toBeInTheDocument();
    expect(service.createOrder).toHaveBeenCalledWith({
      packageId: "recharge-500",
      paymentMethod: "wechat_pay",
    });
    view.unmount();
  });
});

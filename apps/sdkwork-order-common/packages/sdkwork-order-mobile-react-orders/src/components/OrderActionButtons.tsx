import React from "react";
import { useTranslation } from "react-i18next";
import { showToast } from "@sdkwork/ui-mobile-react";
import { OrderService, type Order } from "../services/OrderService";

interface OrderActionButtonsProps {
  order: Order;
  onRefresh: () => void;
  onPay: (order: Order) => void;
}

export const OrderActionButtons: React.FC<OrderActionButtonsProps> = ({
  order,
  onRefresh,
  onPay,
}) => {
  const { t } = useTranslation();

  const handleAction = async (
    e: React.MouseEvent,
    action: () => Promise<void>,
    successMsg: string
  ) => {
    e.stopPropagation();
    try {
      await action();
      showToast(successMsg);
      onRefresh();
    } catch (err) {
      showToast(t("orders.operation_failed", "操作失败"));
    }
  };

  switch (order.status) {
    case "pending_payment":
      return (
        <>
          <button
            onClick={(e) =>
              handleAction(
                e,
                () => OrderService.cancelOrder(order.id),
                t("orders.cancelled_toast", "订单已取消")
              )
            }
            className="px-4 py-1.5 rounded-full border border-border-color text-[13px] text-text-main font-medium active:bg-active-bg transition-colors"
          >
            {t("orders.cancel_order", "取消订单")}
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onPay(order);
            }}
            className="px-4 py-1.5 rounded-full border border-primary-blue bg-primary-blue text-white text-[13px] font-medium active:opacity-80 transition-opacity"
          >
            {t("orders.pay_now", "付款")}
          </button>
        </>
      );
    default:
      return null;
  }
};

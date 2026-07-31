import React from "react";
import { useTranslation } from "react-i18next";
import { showToast } from "@sdkwork/ui-mobile-react";
import { OrderService, type Order } from "../services/OrderService";

interface OrderActionButtonsProps {
  order: Order;
  onRefresh: () => void;
  onReview: (order: Order) => void;
}

export const OrderActionButtons: React.FC<OrderActionButtonsProps> = ({
  order,
  onRefresh,
  onReview,
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
      showToast(t("orders.auto_fn_2f078e83", "操作失败"));
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
                () => OrderService.modifyAddress(order.id),
                "地址修改成功"
              )
            }
            className="px-4 py-1.5 rounded-full border border-border-color text-[13px] text-text-main font-medium active:bg-active-bg transition-colors"
          >
            {t("orders.auto_25dc625b", "修改地址")}
          </button>
          <button
            onClick={(e) =>
              handleAction(
                e,
                () => OrderService.cancelOrder(order.id),
                "订单已取消"
              )
            }
            className="px-4 py-1.5 rounded-full border border-border-color text-[13px] text-text-main font-medium active:bg-active-bg transition-colors"
          >
            {t("orders.auto_27c87be5", "取消订单")}
          </button>
          <button
            onClick={(e) =>
              handleAction(
                e,
                () => OrderService.payOrder(order.id),
                "支付成功"
              )
            }
            className="px-4 py-1.5 rounded-full border border-primary-blue bg-primary-blue text-white text-[13px] font-medium active:opacity-80 transition-opacity"
          >
            {t("orders.auto_9f766", "付款")}
          </button>
        </>
      );
    case "to_ship":
      return (
        <>
          <button
            onClick={(e) =>
              handleAction(
                e,
                () => OrderService.modifyAddress(order.id),
                "地址修改成功"
              )
            }
            className="px-4 py-1.5 rounded-full border border-border-color text-[13px] text-text-main font-medium active:bg-active-bg transition-colors"
          >
            {t("orders.auto_25dc625b", "修改地址")}
          </button>
          <button
            onClick={(e) =>
              handleAction(
                e,
                () => OrderService.remindShipping(order.id),
                "已提醒卖家发货"
              )
            }
            className="px-4 py-1.5 rounded-full border border-border-color text-[13px] text-text-main font-medium active:bg-active-bg transition-colors"
          >
            {t("orders.auto_1398922", "催发货")}
          </button>
        </>
      );
    case "to_receive":
      return (
        <>
          <button
            onClick={(e) => {
              e.stopPropagation();
              showToast(t("orders.auto_fn_4fa0eb64", "目前物流状态：运送中"));
            }}
            className="px-4 py-1.5 rounded-full border border-border-color text-[13px] text-text-main font-medium active:bg-active-bg transition-colors"
          >
            {t("orders.auto_31077a3e", "查看物流")}
          </button>
          <button
            onClick={(e) =>
              handleAction(
                e,
                () => OrderService.confirmReceipt(order.id),
                "已确认收货"
              )
            }
            className="px-4 py-1.5 rounded-full border border-primary-blue text-primary-blue text-[13px] font-medium active:bg-primary-blue/10 transition-colors"
          >
            {t("orders.auto_38d78a27", "确认收货")}
          </button>
        </>
      );
    case "to_review":
      return (
        <>
          <button
            onClick={(e) =>
              handleAction(
                e,
                () => OrderService.applyRefund(order.id),
                "已提交售后申请"
              )
            }
            className="px-4 py-1.5 rounded-full border border-border-color text-[13px] text-text-main font-medium active:bg-active-bg transition-colors"
          >
            {t("orders.auto_375ea8c4", "申请售后")}
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onReview(order);
            }}
            className="px-4 py-1.5 rounded-full border border-primary-blue text-primary-blue text-[13px] font-medium active:bg-primary-blue/10 transition-colors"
          >
            {t("orders.auto_113bb3", "评价")}
          </button>
        </>
      );
    case "cancelled":
    case "completed":
    case "refunded":
      return (
        <button
          onClick={(e) =>
            handleAction(
              e,
              () => OrderService.deleteOrder(order.id),
              "订单已删除"
            )
          }
          className="px-4 py-1.5 rounded-full border border-border-color text-[13px] text-text-main font-medium active:bg-active-bg transition-colors"
        >
          {t("orders.auto_279ac337", "删除订单")}
        </button>
      );
    default:
      return null;
  }
};

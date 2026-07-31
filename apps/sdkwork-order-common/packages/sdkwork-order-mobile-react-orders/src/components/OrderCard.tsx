import { useTranslation } from "react-i18next";
import React from "react";
import { Store, ChevronRight } from "lucide-react";
import { cn } from "@sdkwork/ui-mobile-react";
import { motion } from "motion/react";
import type { Order } from "../services/OrderService";

interface OrderCardProps {
  order: Order;
  onClick: () => void;
  renderActionButtons: (order: Order) => React.ReactNode;
}

export const OrderCard: React.FC<OrderCardProps> = ({
  order,
  onClick,
  renderActionButtons,
}) => {
  const { t } = useTranslation();
return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.95 }}
      transition={{ duration: 0.2 }}
      onClick={onClick}
      className="bg-white dark:bg-[#1E1E1E] rounded-xl p-3 flex flex-col gap-3 cursor-pointer active:scale-[0.98] transition-transform"
    >
      {/* Shop Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1.5 cursor-pointer active:opacity-70">
          <Store className="w-4 h-4 text-text-main" />
          <span className="text-[14px] font-semibold text-text-main">
            {order.shopName}
          </span>
          <ChevronRight className="w-4 h-4 text-text-sub" />
        </div>
        <span
          className={cn(
            "text-[13px] font-medium",
            order.status === "pending_payment"
              ? "text-primary-blue"
              : "text-text-sub",
          )}
        >
          {order.statusText}
        </span>
      </div>

      {/* Items */}
      <div className="flex flex-col gap-3">
        {order.items.map((item) => (
          <div key={item.id} className="flex gap-2.5">
            <img
              src={item.image}
              alt={item.title}
              className="w-20 h-20 rounded-lg object-cover bg-black/5 dark:bg-white/5 shrink-0"
            />
            <div className="flex-1 min-w-0 flex flex-col">
              <div className="flex justify-between gap-2 items-start">
                <h4 className="text-[13px] text-text-main leading-[1.4] line-clamp-2 font-medium">
                  {item.title}
                </h4>
                <div className="flex flex-col items-end shrink-0">
                  <span className="text-[13px] font-medium text-text-main">
                    ¥{item.price.toFixed(2)}
                  </span>
                  {item.originalPrice && (
                    <span className="text-[11px] text-text-sub line-through mt-0.5">
                      ¥{item.originalPrice.toFixed(2)}
                    </span>
                  )}
                  <span className="text-[12px] text-text-sub mt-0.5">
                    x{item.quantity}
                  </span>
                </div>
              </div>
              {item.specs && (
                <div className="mt-1.5">
                  <span className="inline-block px-1.5 py-0.5 bg-black/5 dark:bg-white/5 rounded text-[11px] text-text-sub truncate max-w-full">
                    {item.specs}
                  </span>
                </div>
              )}
              {/* Tags like 7天无理由退换 */}
              <div className="mt-auto pt-1 flex gap-1">
                <span className="text-[10px] text-orange-500 border border-orange-500/30 px-1 rounded-sm">{t('orders.auto_n43d78b59', '七天无理由退换')}</span>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Total */}
      <div className="flex justify-end items-center gap-1 mt-1">
        <span className="text-[13px] text-text-sub">{t('orders.auto_e1c31af', '共 {order.items.reduce((acc, item) => acc + item.quantity, 0)} 件商品')}</span>
        <span className="text-[13px] text-text-main ml-1">{t('orders.auto_161e384', '实付款')}</span>
        <span className="text-[15px] font-bold text-text-main">
          ¥{order.totalAmount.toFixed(2)}
        </span>
      </div>

      {/* Actions */}
      <div className="flex justify-end items-center gap-2 mt-1 pt-3 border-t border-border-color/50">
        {renderActionButtons(order)}
      </div>
    </motion.div>
  );
};

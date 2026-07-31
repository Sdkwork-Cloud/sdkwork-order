import { useTranslation } from "react-i18next";
import React from "react";
import { useNavigate } from "react-router";
import { ChevronRight, Store, MessageCircle, Phone, QrCode } from "lucide-react";
import { showToast } from "@sdkwork/ui-mobile-react";
import type { Order } from "../services/OrderService";

interface OrderItemsCardProps {
  order: Order;
}

export const OrderItemsCard: React.FC<OrderItemsCardProps> = ({ order }) => {
  const { t } = useTranslation();
const navigate = useNavigate();

  return (
    <div className="bg-white dark:bg-[#1E1E1E] rounded-xl p-4 shadow-sm flex flex-col gap-4">
      <div className="flex items-center gap-1.5 cursor-pointer active:opacity-70">
          <Store className="w-4 h-4 text-text-main" />
          <span className="text-[14px] font-semibold text-text-main">
            {order.shopName}
          </span>
          <ChevronRight className="w-4 h-4 text-text-sub" />
        </div>

        <div className="flex flex-col gap-4">
          {order.items.map((item) => (
            <div key={item.id} className="flex flex-col gap-3">
              <div className="flex gap-3">
                <img
                  src={item.image}
                  alt={item.title}
                  className="w-20 h-20 rounded-lg object-cover bg-black/5 dark:bg-white/5 shrink-0"
                />
                <div className="flex-1 min-w-0 flex flex-col">
                  <div className="flex justify-between gap-2 items-start">
                    <h4 className="text-[14px] text-text-main leading-[1.4] line-clamp-2 font-medium">
                      {item.title}
                    </h4>
                    <div className="flex flex-col items-end shrink-0">
                      <span className="text-[14px] font-medium text-text-main">
                        ¥{item.price.toFixed(2)}
                      </span>
                      {item.originalPrice && (
                        <span className="text-[12px] text-text-sub line-through mt-0.5">
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
                      <span className="inline-block px-1.5 py-0.5 bg-black/5 dark:bg-white/5 rounded text-[12px] text-text-sub truncate max-w-full">
                        {item.specs}
                      </span>
                    </div>
                  )}
                  {(!item.voucherCodes || item.voucherCodes.length === 0) && (
                    <div className="mt-auto pt-2 flex justify-start">
                      {item.virtualType === 'group_chat' ? (
                        <button 
                           onClick={() => navigate('/')} 
                           className="px-3 py-1 rounded-full border border-primary-blue text-[12px] text-primary-blue active:bg-primary-blue/10 transition-colors">进入群聊</button>
                      ) : (
                        <button className="px-3 py-1 rounded-full border border-border-color text-[12px] text-text-main active:bg-active-bg transition-colors">申请退款</button>
                      )}
                    </div>
                  )}
                </div>
              </div>
              
              {item.voucherCodes && item.voucherCodes.length > 0 && (
                <div className="mt-3 w-full">
                  <div className="text-[13px] font-medium text-text-main mb-2 flex items-center justify-between">
                    <span>电子凭证</span>
                    <span className="text-text-sub font-normal text-[12px]">共 {item.voucherCodes.length} 份</span>
                  </div>
                  <div className="flex flex-col gap-2">
                    {item.voucherCodes.map((voucher, idx) => (
                      <div 
                        key={idx} 
                        onClick={() => {
                          if (voucher.status !== 'used') {
                            navigate(`/me/orders/${order.id}/voucher/${voucher.code}`);
                          }
                        }}
                        className={`flex items-center justify-between bg-black/[0.03] dark:bg-white/[0.05] p-3 rounded-xl border border-black/5 dark:border-white/5 ${voucher.status !== 'used' ? 'cursor-pointer active:scale-[0.98] transition-transform' : ''}`}
                      >
                        <div className="flex flex-col gap-1 min-w-0 flex-1 pr-3">
                          <span className={`text-[15px] font-mono font-semibold truncate ${voucher.status === 'used' ? 'text-text-sub line-through decoration-text-sub/40' : 'text-text-main'}`}>
                            {voucher.code}
                          </span>
                          <div className="flex items-center gap-2 mt-0.5">
                            <span className={`text-[11px] px-1.5 py-0.5 rounded-sm ${voucher.status === 'used' ? 'bg-black/5 dark:bg-white/10 text-text-sub' : 'bg-[#FA5151]/10 text-[#FA5151]'}`}>{voucher.status === 'used' ? '已使用' : '待核销'}</span>
                            <button
                              onClick={async (e) => {
                                e.stopPropagation();
                                try {
                                  await navigator.clipboard.writeText(voucher.code);
                                  showToast("复制成功");
                                } catch (e) {
                                  showToast("复制失败");
                                }
                              }}
                              className="text-[11px] text-primary-blue active:opacity-70"
                            >复制</button>
                          </div>
                        </div>

                        {voucher.status !== 'used' && (
                          <div 
                            className="flex items-center justify-center w-10 h-10 bg-white dark:bg-[#2A2A2A] rounded-full shadow-sm border border-border-color shrink-0"
                          >
                            <QrCode className="w-5 h-5 text-text-main" />
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>

        <div className="flex items-center justify-center gap-8 pt-4 border-t border-border-color/50">
          <div className="flex items-center gap-1.5 text-text-main active:opacity-70 cursor-pointer">
            <MessageCircle className="w-4 h-4 text-primary-blue" />
            <span className="text-[13px] font-medium">联系卖家</span>
          </div>
          <div className="w-[1px] h-4 bg-border-color" />
          <div className="flex items-center gap-1.5 text-text-main active:opacity-70 cursor-pointer">
            <Phone className="w-4 h-4 text-primary-blue" />
            <span className="text-[13px] font-medium">拨打电话</span>
          </div>
        </div>
      </div>
  );
};

import { useTranslation } from "react-i18next";
import React from "react";
import {
  ChevronRight,
  MapPin,
  Store,
  MessageCircle,
  Phone,
} from "lucide-react";
import type { Order } from "../services/OrderService";

interface OrderAddressCardProps {
  address: Order["address"];
}

export const OrderAddressCard: React.FC<OrderAddressCardProps> = ({
  address,
}) => {
  const { t } = useTranslation();
return (
    <div className="bg-white dark:bg-[#1E1E1E] rounded-xl p-4 flex items-center gap-3 shadow-sm">
      <div className="w-8 h-8 rounded-full bg-primary-blue/10 flex items-center justify-center shrink-0">
        <MapPin className="w-4 h-4 text-primary-blue" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <span className="text-[16px] font-bold text-text-main">
            {address.name}
          </span>
          <span className="text-[14px] text-text-sub">{address.phone}</span>
        </div>
        <p className="text-[13px] text-text-main leading-relaxed">
          {address.detail}
        </p>
      </div>
      <ChevronRight className="w-5 h-5 text-text-sub opacity-50 shrink-0" />
    </div>
  );
};

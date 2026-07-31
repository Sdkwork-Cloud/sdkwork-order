import { useTranslation } from "react-i18next";
import React, { useState } from "react";
import { Star, X } from "lucide-react";
import { IconButton, cn } from "@sdkwork/ui-mobile-react";
import { motion, AnimatePresence } from "motion/react";
import type { Order } from "../services/OrderService";

interface OrderReviewModalProps {
  order: Order | null;
  onClose: () => void;
  onSubmit: (rating: number, text: string) => Promise<void>;
}

export const OrderReviewModal: React.FC<OrderReviewModalProps> = ({
  order,
  onClose,
  onSubmit,
}) => {
  const { t } = useTranslation();
const [rating, setRating] = useState(0);
  const [reviewText, setReviewText] = useState("");

  const handleClose = () => {
  setRating(0);
    setReviewText("");
    onClose();
  };

  const handleSubmit = async () => {
    await onSubmit(rating, reviewText);
    handleClose();
  };

  return (
    <AnimatePresence>
      {order && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 z-[100] bg-black/40 flex items-center justify-center px-4"
          onClick={handleClose}
        >
          <motion.div
            initial={{ scale: 0.95 }}
            animate={{ scale: 1 }}
            exit={{ scale: 0.95 }}
            onClick={(e) => e.stopPropagation()}
            className="bg-bg-color w-full max-w-[320px] rounded-2xl p-5 shadow-xl flex flex-col pt-6"
          >
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-[17px] font-bold text-text-main">{t('orders.auto_282b18aa', '发表评价')}</h3>
              <IconButton
                icon={<X className="w-5 h-5 text-text-sub" />}
                onClick={handleClose}
              />
            </div>
            <div className="flex gap-2 justify-center mb-6">
              {[1, 2, 3, 4, 5].map((star) => (
                <Star
                  key={star}
                  className={cn(
                    "w-8 h-8 cursor-pointer transition-colors",
                    rating >= star
                      ? "text-yellow-400 fill-yellow-400"
                      : "text-border-color",
                  )}
                  onClick={() => setRating(star)}
                />
              ))}
            </div>
            <textarea
              className="w-full bg-chat-other-bg rounded-xl p-3 text-[14px] text-text-main outline-none min-h-[100px] mb-4 border border-border-color/50 placeholder:text-text-sub"
              placeholder={t('orders.auto_prop_n539d9fee', '写下你的购物体验吧...')}
              value={reviewText}
              onChange={(e) => setReviewText(e.target.value)}
            />
            <button
              disabled={rating === 0}
              className="w-full py-3 bg-primary-blue text-white font-bold rounded-full disabled:opacity-50 transition-opacity active:opacity-80"
              onClick={handleSubmit}
            >{t('orders.auto_2e97bb87', '提交评价')}</button>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};

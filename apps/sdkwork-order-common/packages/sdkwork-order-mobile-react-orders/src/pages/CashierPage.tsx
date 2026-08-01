import React, { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate, useParams } from "react-router";
import {
  AlertCircle,
  CheckCircle2,
  Clock,
  ExternalLink,
  Loader2,
  QrCode,
  Store,
  XCircle,
} from "lucide-react";
import QRCode from "qrcode";
import { PageLayout, showToast } from "@sdkwork/ui-mobile-react";

import {
  formatAmountCny,
  OrderService,
  ORDER_PAYMENT_METHODS,
  type Order,
  type OrderPaymentMethod,
  type PaymentSession,
} from "../services/OrderService";
import {
  CASHIER_POLL_INTERVAL_MS,
  computeCashierRemainingSeconds,
  formatCashierCountdown,
  resolveCashierPhaseFromPaymentStatus,
} from "../services/CashierLogic";
import type { CashierPhase } from "../services/CashierTypes";
import {
  ORDER_MOBILE_ROUTE_DEFINITIONS,
  resolveOrderRoutePath,
} from "../routes";

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function CashierPage() {
  const { orderId } = useParams<{ orderId: string }>();
  const navigate = useNavigate();
  const { t } = useTranslation("orders");

  const [phase, setPhase] = useState<CashierPhase>("loading");
  const [order, setOrder] = useState<Order | null>(null);
  const [paymentMethod, setPaymentMethod] = useState<OrderPaymentMethod>("wechat_pay");
  const [paymentSession, setPaymentSession] = useState<PaymentSession | null>(null);
  const [qrDataUrl, setQrDataUrl] = useState<string | null>(null);
  const [remainingSeconds, setRemainingSeconds] = useState(0);
  const [errorMessageText, setErrorMessageText] = useState<string | null>(null);

  const paymentCreatedAtRef = useRef(0);
  const pollTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const countdownTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const phaseRef = useRef<CashierPhase>("loading");
  phaseRef.current = phase;
  const orderRef = useRef<Order | null>(null);

  const clearTimers = useCallback(() => {
    if (pollTimerRef.current) {
      clearInterval(pollTimerRef.current);
      pollTimerRef.current = null;
    }
    if (countdownTimerRef.current) {
      clearInterval(countdownTimerRef.current);
      countdownTimerRef.current = null;
    }
  }, []);

  const stopCashier = useCallback((nextPhase: CashierPhase) => {
    clearTimers();
    setPhase(nextPhase);
  }, [clearTimers]);

  const pollPaymentStatus = useCallback(async (targetOrderId: string) => {
    try {
      const status = await OrderService.getPaymentStatus(targetOrderId);
      const currentOrder = await OrderService.getOrderById(targetOrderId);
      const resolved = resolveCashierPhaseFromPaymentStatus(
        status,
        currentOrder?.status ?? status.status,
      );
      if (resolved !== "pending") {
        stopCashier(resolved);
        if (resolved === "paid") {
          setOrder(currentOrder);
        }
      }
    } catch {
      // Transient network errors keep the cashier pending; the next poll
      // retries. Permanent failures surface through the countdown expiry.
    }
  }, [stopCashier]);

  const startPolling = useCallback((targetOrderId: string) => {
    clearTimers();
    pollTimerRef.current = setInterval(() => {
      void pollPaymentStatus(targetOrderId);
    }, CASHIER_POLL_INTERVAL_MS);
    countdownTimerRef.current = setInterval(() => {
      const now = Date.now();
      setRemainingSeconds((previous) => {
        const next = computeCashierRemainingSeconds(
          orderRef.current?.expireTime,
          paymentCreatedAtRef.current || now,
          now,
        );
        if (next <= 0 && previous > 0 && phaseRef.current === "pending") {
          // One final authoritative check before declaring expiry.
          void pollPaymentStatus(targetOrderId).then(() => {
            if (phaseRef.current === "pending") {
              stopCashier("expired");
            }
          });
        }
        return next;
      });
    }, 1000);
  }, [clearTimers, pollPaymentStatus, stopCashier]);

  const createPayment = useCallback(async (targetOrderId: string, method: OrderPaymentMethod) => {
    setPhase("creating");
    setErrorMessageText(null);
    try {
      const session = await OrderService.payOrder(targetOrderId, method);
      setPaymentSession(session);
      paymentCreatedAtRef.current = Date.now();
      setRemainingSeconds(computeCashierRemainingSeconds(
        orderRef.current?.expireTime,
        paymentCreatedAtRef.current,
        Date.now(),
      ));
      const payload = session.paymentParams.qrCodePayload
        ?? session.paymentParams.cashierUrl;
      if (payload) {
        setQrDataUrl(await QRCode.toDataURL(payload, { width: 220, margin: 1 }));
      } else {
        setQrDataUrl(null);
      }
      setPhase("pending");
      startPolling(targetOrderId);
    } catch (error) {
      setErrorMessageText(errorMessage(error));
      stopCashier("failed");
    }
  }, [startPolling, stopCashier]);

  useEffect(() => {
    if (!orderId) {
      setPhase("not_payable");
      return undefined;
    }
    let cancelled = false;
    void (async () => {
      try {
        const loaded = await OrderService.getOrderById(orderId);
        if (cancelled) {
          return;
        }
        if (!loaded) {
          setPhase("not_payable");
          return;
        }
        orderRef.current = loaded;
        setOrder(loaded);
        if (loaded.status === "pending_payment") {
          await createPayment(orderId, "wechat_pay");
        } else if (
          ["paid", "fulfilled", "completed", "refunding", "refunded"].includes(String(loaded.status))
        ) {
          setPhase("paid");
        } else if (loaded.status === "cancelled") {
          setPhase("cancelled");
        } else if (loaded.status === "expired") {
          setPhase("expired");
        } else {
          setPhase("not_payable");
        }
      } catch (error) {
        if (!cancelled) {
          setErrorMessageText(errorMessage(error));
          setPhase("failed");
        }
      }
    })();
    return () => {
      cancelled = true;
      clearTimers();
    };
  }, [orderId, clearTimers, createPayment]);

  const changePaymentMethod = (method: OrderPaymentMethod) => {
    if (phase !== "pending" || !orderId) {
      return;
    }
    setPaymentMethod(method);
    void createPayment(orderId, method);
  };

  const retryPayment = () => {
    if (orderId) {
      void createPayment(orderId, paymentMethod);
    }
  };

  const cancelPayment = async () => {
    if (!orderId) {
      return;
    }
    try {
      await OrderService.cancelOrder(orderId);
      showToast(t("cancelled_toast", "订单已取消"));
      stopCashier("cancelled");
    } catch (error) {
      showToast(errorMessage(error));
    }
  };

  const openExternalCashier = () => {
    const url = paymentSession?.paymentParams.cashierUrl;
    if (url) {
      window.open(url, "_blank", "noopener,noreferrer");
    }
  };

  const goToOrderDetail = () => {
    if (orderId) {
      navigate(resolveOrderRoutePath(ORDER_MOBILE_ROUTE_DEFINITIONS.orderDetail, { orderId }));
    }
  };

  const goToOrderCenter = () => {
    navigate(ORDER_MOBILE_ROUTE_DEFINITIONS.orderCenter.path);
  };

  const renderResult = (
    title: string,
    description: string,
    icon: React.ReactNode,
    extraActions?: React.ReactNode,
  ) => (
    <div className="flex flex-col items-center justify-center flex-1 px-8 text-center">
      <div className="mb-5">{icon}</div>
      <h2 className="text-[18px] font-bold text-text-main mb-2">{title}</h2>
      <p className="text-[13px] text-text-sub leading-relaxed">{description}</p>
      <div className="flex flex-col gap-3 w-full max-w-[280px] mt-8">
        {extraActions}
        <button
          onClick={goToOrderDetail}
          className="w-full bg-primary-blue active:opacity-80 text-white font-medium text-[15px] py-3 rounded-xl transition-opacity"
        >
          {t("cashier_view_order", "查看订单")}
        </button>
        <button
          onClick={goToOrderCenter}
          className="w-full border border-border-color active:bg-active-bg text-text-main font-medium text-[15px] py-3 rounded-xl transition-colors"
        >
          {t("cashier_back_center", "返回订单中心")}
        </button>
      </div>
    </div>
  );

  return (
    <PageLayout title={t("cashier_title", "收银台")}>
      <div className="flex flex-col h-full bg-[#f5f6f8] dark:bg-[#1a1b1c]">
        {phase === "loading" || phase === "creating" ? (
          <div className="flex flex-col items-center justify-center flex-1 text-text-sub">
            <Loader2 className="w-8 h-8 animate-spin mb-3" />
            <p className="text-[14px]">
              {phase === "loading" ? t("loading", "加载中...") : t("cashier_creating", "正在创建支付...")}
            </p>
          </div>
        ) : phase === "pending" && order && paymentSession ? (
          <>
            <div className="bg-white dark:bg-[#1E1E1E] m-4 rounded-xl shadow-sm flex flex-col items-center py-6 px-4">
              <span className="text-[13px] text-text-sub mb-1">
                {t("cashier_amount_label", "支付金额")}
              </span>
              <span className="text-[30px] font-bold text-text-main mb-4">
                {formatAmountCny(paymentSession.amount, order.currencyCode)}
              </span>

              <div className="flex items-center gap-1 text-text-sub mb-4">
                <Store className="w-4 h-4" />
                <span className="text-[13px]">{order.subject}</span>
              </div>

              {qrDataUrl ? (
                <div className="bg-white p-3 rounded-lg border border-border-color/60">
                  <img src={qrDataUrl} alt={t("cashier_qr_alt", "收银台二维码")} className="w-[220px] h-[220px]" />
                </div>
              ) : (
                <div className="flex flex-col items-center gap-2 py-6">
                  <QrCode className="w-12 h-12 text-text-sub/50" />
                  <span className="text-[13px] text-text-sub">
                    {t("cashier_no_qr", "收银台暂未生成二维码")}
                  </span>
                </div>
              )}

              <p className="text-[13px] text-text-sub mt-4">
                {t("cashier_scan_pay", "请使用微信或支付宝扫一扫完成支付")}
              </p>

              <div className="flex items-center gap-1 mt-3 text-[#FA5151]">
                <Clock className="w-4 h-4" />
                <span className="text-[13px] font-medium">
                  {t("cashier_auto_close", "剩余 {countdown} 自动关闭", {
                    countdown: formatCashierCountdown(remainingSeconds),
                  })}
                </span>
              </div>
            </div>

            <div className="bg-white dark:bg-[#1E1E1E] mx-4 rounded-xl shadow-sm p-4">
              <h3 className="text-[14px] font-bold text-text-main mb-3">
                {t("cashier_pay_method", "支付方式")}
              </h3>
              <div className="flex flex-col gap-2">
                {ORDER_PAYMENT_METHODS.map((method) => (
                  <label
                    key={method}
                    className="flex items-center justify-between px-3 py-3 rounded-lg border border-border-color cursor-pointer active:bg-active-bg"
                  >
                    <span className="text-[14px] text-text-main">
                      {t(`payment_method_${method}`, method)}
                    </span>
                    <input
                      type="radio"
                      name="payment-method"
                      checked={paymentMethod === method}
                      onChange={() => changePaymentMethod(method)}
                      className="accent-[#FA5151]"
                    />
                  </label>
                ))}
              </div>
              <button
                onClick={openExternalCashier}
                className="mt-4 w-full flex items-center justify-center gap-1.5 border border-border-color active:bg-active-bg text-text-main text-[14px] py-2.5 rounded-lg transition-colors"
              >
                <ExternalLink className="w-4 h-4" />
                {t("cashier_open_external", "打开外部收银台")}
              </button>
            </div>

            <div className="p-4 pb-6">
              <button
                onClick={() => void cancelPayment()}
                className="w-full border border-border-color active:bg-active-bg text-text-sub text-[14px] py-3 rounded-xl transition-colors"
              >
                {t("cashier_cancel_pay", "取消支付")}
              </button>
            </div>
          </>
        ) : phase === "paid" ? (
          renderResult(
            t("cashier_paid_title", "支付成功"),
            t("cashier_paid_desc", "支付已完成，订单将尽快为您处理。"),
            <CheckCircle2 className="w-16 h-16 text-green-500" />,
          )
        ) : phase === "expired" ? (
          renderResult(
            t("cashier_expired_title", "支付超时"),
            t("cashier_expired_desc", "支付已超时，订单已自动关闭。"),
            <Clock className="w-16 h-16 text-text-sub" />,
          )
        ) : phase === "cancelled" ? (
          renderResult(
            t("cashier_cancelled_title", "订单已取消"),
            t("cashier_cancelled_desc", "订单已取消，如有疑问请联系商家。"),
            <XCircle className="w-16 h-16 text-text-sub" />,
          )
        ) : phase === "not_payable" ? (
          renderResult(
            t("cashier_not_payable_title", "订单不可支付"),
            t("cashier_not_payable_desc", "该订单当前状态不支持支付。"),
            <AlertCircle className="w-16 h-16 text-text-sub" />,
          )
        ) : (
          renderResult(
            t("cashier_failed_title", "支付创建失败"),
            errorMessageText ?? t("cashier_failed_desc", "请重试或稍后再试。"),
            <XCircle className="w-16 h-16 text-[#FA5151]" />,
            <button
              onClick={retryPayment}
              className="w-full bg-primary-blue active:opacity-80 text-white font-medium text-[15px] py-3 rounded-xl transition-opacity"
            >
              {t("cashier_retry", "重新支付")}
            </button>,
          )
        )}
      </div>
    </PageLayout>
  );
}

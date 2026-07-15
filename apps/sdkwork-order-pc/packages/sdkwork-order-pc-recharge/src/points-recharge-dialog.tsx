/// <reference path="../styles.d.ts" />

import { useEffect, useMemo, useRef, useState } from "react";
import { CheckCircle2, QrCode, Sparkles, X } from "lucide-react";
import { toDataURL } from "qrcode";
import {
  Button,
  Modal,
  ModalBody,
  ModalClose,
  ModalContent,
  ModalHeader,
  ModalTitle,
  StatusNotice,
} from "@sdkwork/ui-pc-react";
import {
  createSdkworkPointsRechargeService,
  type SdkworkPointsRechargePackage,
  type SdkworkPointsRechargePayment,
  type SdkworkPointsRechargeService,
} from "@sdkwork/order-service";
import "./points-recharge-dialog.css";

export interface SdkworkPointsRechargeDialogCopy {
  account: string;
  agreement: string;
  agreementAccepted: string;
  agreementRequired: string;
  close: string;
  completed: string;
  confirmPayment: string;
  creatingPayment: string;
  emptyPackages: string;
  loadFailed: string;
  loadingPackages: string;
  myPoints: string;
  notice: string;
  paymentUnavailable: string;
  paymentUnavailableDescription: string;
  pointsUnit: string;
  retry: string;
  scanPrompt: string;
  title: string;
}

export interface SdkworkPointsRechargeDialogProps {
  copy?: Partial<SdkworkPointsRechargeDialogCopy>;
  currentPoints?: number | null;
  isOpen: boolean;
  onClose: () => void;
  onCompleted?: (payment: SdkworkPointsRechargePayment) => Promise<void> | void;
  paymentMethod?: string;
  service?: SdkworkPointsRechargeService;
}

const DEFAULT_COPY: SdkworkPointsRechargeDialogCopy = {
  account: "积分账户",
  agreement: "我已阅读并同意《积分充值服务协议》",
  agreementAccepted: "您已同意《积分充值服务协议》",
  agreementRequired: "请先同意积分充值服务协议",
  close: "关闭",
  completed: "支付完成，积分已到账",
  confirmPayment: "同意并支付",
  creatingPayment: "正在生成支付二维码...",
  emptyPackages: "暂无可用充值套餐",
  loadFailed: "充值套餐加载失败",
  loadingPackages: "正在加载充值套餐...",
  myPoints: "我的积分",
  notice: "积分不可转赠、不可提现，充值后有效期以平台规则为准。",
  paymentUnavailable: "支付暂不可用",
  paymentUnavailableDescription: "暂时无法生成支付二维码，请稍后重试。",
  pointsUnit: "积分",
  retry: "重新加载",
  scanPrompt: "请扫码完成支付",
  title: "积分购买",
};

export function SdkworkPointsRechargeDialog({
  copy: copyOverrides,
  currentPoints,
  isOpen,
  onClose,
  onCompleted,
  paymentMethod = "wechat_pay",
  service: serviceProp,
}: SdkworkPointsRechargeDialogProps) {
  const copy = useMemo(() => ({ ...DEFAULT_COPY, ...copyOverrides }), [copyOverrides]);
  const service = useMemo(
    () => serviceProp ?? createSdkworkPointsRechargeService(),
    [serviceProp],
  );
  const [packages, setPackages] = useState<SdkworkPointsRechargePackage[]>([]);
  const [selectedPackageId, setSelectedPackageId] = useState<string | null>(null);
  const [payment, setPayment] = useState<SdkworkPointsRechargePayment | null>(null);
  const [qrImageUrl, setQrImageUrl] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isPaying, setIsPaying] = useState(false);
  const [loadAttempt, setLoadAttempt] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const completedOrderRef = useRef<string | null>(null);

  const selectedPackage = packages.find((item) => item.id === selectedPackageId) ?? null;
  const hasActivePayment = payment !== null && payment.status !== "failed";

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    let active = true;
    setIsLoading(true);
    setError(null);
    setPayment(null);
    setQrImageUrl(null);
    completedOrderRef.current = null;
    void service.listPackages()
      .then((items) => {
        if (!active) return;
        setPackages(items);
        setSelectedPackageId((current) => current && items.some((item) => item.id === current)
          ? current
          : items[0]?.id ?? null);
      })
      .catch((cause) => {
        if (active) setError(cause instanceof Error ? cause.message : copy.loadFailed);
      })
      .finally(() => {
        if (active) setIsLoading(false);
      });
    return () => {
      active = false;
    };
  }, [copy.loadFailed, isOpen, loadAttempt, service]);

  useEffect(() => {
    if (!payment?.qrCode) {
      setQrImageUrl(null);
      return;
    }
    if (payment.qrCode.startsWith("data:image/")) {
      setQrImageUrl(payment.qrCode);
      return;
    }
    let active = true;
    void toDataURL(payment.qrCode, { errorCorrectionLevel: "M", margin: 1, width: 252 })
      .then((value) => {
        if (active) setQrImageUrl(value);
      })
      .catch(() => {
        if (active) setError(copy.paymentUnavailableDescription);
      });
    return () => {
      active = false;
    };
  }, [copy.paymentUnavailableDescription, payment?.qrCode]);

  useEffect(() => {
    if (!isOpen || payment?.status !== "pending" || !payment.orderId) {
      return undefined;
    }
    const orderId = payment.orderId;
    let active = true;
    const poll = async () => {
      try {
        const next = await service.getOrderStatus(orderId);
        if (!active) return;
        setPayment((current) => current ? { ...current, ...next } : next);
        if (next.status === "completed" && completedOrderRef.current !== orderId) {
          completedOrderRef.current = orderId;
          await onCompleted?.(next);
        }
        if (next.status === "failed") setError(copy.paymentUnavailableDescription);
      } catch {
        // Keep the current QR code visible while a transient status request fails.
      }
    };
    void poll();
    const interval = window.setInterval(() => void poll(), 2_500);
    return () => {
      active = false;
      window.clearInterval(interval);
    };
  }, [copy.paymentUnavailableDescription, isOpen, onCompleted, payment?.orderId, payment?.status, service]);

  async function createPayment() {
    if (!selectedPackage) return;
    setIsPaying(true);
    setError(null);
    try {
      const result = await service.createOrder({ packageId: selectedPackage.id, paymentMethod });
      setPayment(result);
      if (result.status === "completed") {
        const key = result.orderId ?? selectedPackage.id;
        if (completedOrderRef.current !== key) {
          completedOrderRef.current = key;
          await onCompleted?.(result);
        }
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : copy.paymentUnavailableDescription);
    } finally {
      setIsPaying(false);
    }
  }

  return (
    <Modal onOpenChange={(open) => !open && onClose()} open={isOpen}>
      <ModalContent aria-describedby={undefined} aria-labelledby="sdkwork-points-recharge-title" className="sdkwork-points-recharge-dialog" showCloseButton={false} size="lg">
        <ModalHeader className="sdkwork-points-recharge-dialog__header">
          <div className="sdkwork-points-recharge-dialog__identity">
            <Sparkles aria-hidden="true" />
            <ModalTitle id="sdkwork-points-recharge-title">{copy.account}</ModalTitle>
          </div>
          <div className="sdkwork-points-recharge-dialog__header-actions">
            <div className="sdkwork-points-recharge-dialog__balance">
              {copy.myPoints} <strong>{currentPoints ?? "--"}</strong>
            </div>
            <ModalClose aria-label={copy.close} className="sdkwork-points-recharge-dialog__close"><X aria-hidden="true" /></ModalClose>
          </div>
        </ModalHeader>
        <ModalBody className="sdkwork-points-recharge-dialog__body">
          <section className="sdkwork-points-recharge-dialog__packages" aria-label={copy.title}>
            <div className="sdkwork-points-recharge-dialog__section-title"><span />{copy.title}<span /></div>
            {isLoading ? <p className="sdkwork-points-recharge-dialog__muted">{copy.loadingPackages}</p> : null}
            {!isLoading && packages.length === 0 && !error ? <p className="sdkwork-points-recharge-dialog__muted">{copy.emptyPackages}</p> : null}
            {!isLoading && packages.length === 0 && error ? (
              <div className="sdkwork-points-recharge-dialog__load-error">
                <StatusNotice tone="danger" title={copy.loadFailed}>{error}</StatusNotice>
                <Button onClick={() => setLoadAttempt((current) => current + 1)} type="button" variant="secondary">{copy.retry}</Button>
              </div>
            ) : null}
            {!isLoading && packages.length > 0 ? (
              <div className="sdkwork-points-recharge-dialog__grid">
                {packages.map((item) => {
                  const selected = item.id === selectedPackageId;
                  return (
                    <button className={`sdkwork-points-recharge-dialog__package ${selected ? "is-selected" : ""}`} key={item.id} onClick={() => { setSelectedPackageId(item.id); setPayment(null); setError(null); }} type="button">
                      <span className="sdkwork-points-recharge-dialog__points"><Sparkles aria-hidden="true" />{item.points.toLocaleString()} <small>{copy.pointsUnit}</small></span>
                      <span className="sdkwork-points-recharge-dialog__price">{item.currencyCode} {item.priceAmount.toFixed(2)}</span>
                    </button>
                  );
                })}
              </div>
            ) : null}
            <p className="sdkwork-points-recharge-dialog__hint">{copy.notice}</p>
          </section>
          <aside className="sdkwork-points-recharge-dialog__payment">
            {error && packages.length > 0 ? <StatusNotice tone="danger" title={copy.paymentUnavailable}>{error}</StatusNotice> : null}
            {!payment || isPaying ? (
              <div className="sdkwork-points-recharge-dialog__payment-empty">
                <QrCode aria-hidden="true" />
                <p>{copy.agreement}</p>
                <Button
                  disabled={!selectedPackage || isPaying || isLoading || hasActivePayment}
                  loading={isPaying}
                  onClick={() => void createPayment()}
                  type="button"
                >
                  {isPaying ? copy.creatingPayment : copy.confirmPayment}
                </Button>
              </div>
            ) : null}
            {payment?.status === "pending" && qrImageUrl ? (
              <div className="sdkwork-points-recharge-dialog__qr">
                <img alt={copy.scanPrompt} src={qrImageUrl} />
                <p>{copy.scanPrompt}</p>
                <span>{copy.agreementAccepted}</span>
              </div>
            ) : null}
            {payment?.status === "completed" ? <div className="sdkwork-points-recharge-dialog__completed"><CheckCircle2 aria-hidden="true" /><strong>{copy.completed}</strong><Button onClick={onClose} type="button">{copy.close}</Button></div> : null}
            {payment?.status === "failed" && !error ? <StatusNotice tone="danger" title={copy.paymentUnavailable}>{copy.paymentUnavailableDescription}</StatusNotice> : null}
          </aside>
        </ModalBody>
      </ModalContent>
    </Modal>
  );
}

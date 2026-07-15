/// <reference path="../styles.d.ts" />

import { useCallback, useEffect, useRef, useState } from "react";
import "./order-checkout-dialog.css";
import {
  CheckCircle2,
  QrCode,
  ShieldCheck,
  Smartphone,
  Sparkles,
  X,
} from "lucide-react";
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

export type SdkworkOrderCheckoutPaymentStatus = "completed" | "failed" | "pending";

export interface SdkworkOrderCheckoutPayment {
  amountCny?: number | null;
  cashierUrl?: string;
  orderId?: string;
  qrCode?: string;
  status: SdkworkOrderCheckoutPaymentStatus;
}

export interface SdkworkOrderCheckoutSummary {
  id: string;
  name: string;
  originalPriceLabel?: string;
  periodLabel?: string;
  priceLabel: string;
}

export interface SdkworkOrderCheckoutDialogCopy {
  activationDescription: string;
  activationTitle: string;
  close: string;
  completed: string;
  creatingPayment: string;
  paymentUnavailable: string;
  paymentUnavailableDescription: string;
  payByQr: string;
  price: string;
  retry: string;
  scanPrompt: string;
  secureDescription: string;
  secureTitle: string;
  selectedItem: string;
}

export interface SdkworkOrderCheckoutDriver {
  createPayment(): Promise<SdkworkOrderCheckoutPayment>;
  getPaymentStatus?(payment: SdkworkOrderCheckoutPayment): Promise<SdkworkOrderCheckoutPayment>;
  onPaymentCompleted?(payment: SdkworkOrderCheckoutPayment): Promise<void> | void;
  pollIntervalMs?: number;
}

export interface SdkworkOrderCheckoutDialogProps {
  copy: SdkworkOrderCheckoutDialogCopy;
  driver: SdkworkOrderCheckoutDriver;
  isOpen: boolean;
  onClose: () => void;
  summary: SdkworkOrderCheckoutSummary | null;
}

function isImageDataUrl(value: string | undefined): value is string {
  return Boolean(value?.startsWith("data:image/"));
}

/**
 * Domain-neutral order checkout UI. Product features provide the item summary
 * and checkout driver; this component owns payment QR presentation only.
 */
export function SdkworkOrderCheckoutDialog({
  copy,
  driver,
  isOpen,
  onClose,
  summary,
}: SdkworkOrderCheckoutDialogProps) {
  const createPaymentRef = useRef(driver.createPayment);
  const getPaymentStatusRef = useRef(driver.getPaymentStatus);
  const onPaymentCompletedRef = useRef(driver.onPaymentCompleted);
  const pollIntervalRef = useRef(driver.pollIntervalMs);
  const copyRef = useRef(copy);
  const completedPaymentKeyRef = useRef<string | null>(null);
  const summaryId = summary?.id;
  const [attempt, setAttempt] = useState(0);
  const [isCreatingPayment, setIsCreatingPayment] = useState(false);
  const [payment, setPayment] = useState<SdkworkOrderCheckoutPayment | null>(null);
  const [paymentError, setPaymentError] = useState<string | null>(null);
  const [qrImageUrl, setQrImageUrl] = useState<string | null>(null);
  const [qrImagePayload, setQrImagePayload] = useState<string | null>(null);

  createPaymentRef.current = driver.createPayment;
  getPaymentStatusRef.current = driver.getPaymentStatus;
  onPaymentCompletedRef.current = driver.onPaymentCompleted;
  pollIntervalRef.current = driver.pollIntervalMs;
  copyRef.current = copy;

  const notifyPaymentCompleted = useCallback((result: SdkworkOrderCheckoutPayment) => {
    const paymentKey = result.orderId ?? summaryId;
    if (!paymentKey || completedPaymentKeyRef.current === paymentKey) {
      return;
    }

    completedPaymentKeyRef.current = paymentKey;
    const onPaymentCompleted = onPaymentCompletedRef.current;
    if (onPaymentCompleted) {
      void Promise.resolve(onPaymentCompleted(result)).catch(() => undefined);
    }
  }, [summaryId]);

  useEffect(() => {
    completedPaymentKeyRef.current = null;
    if (!isOpen) {
      setAttempt(0);
      setIsCreatingPayment(false);
      setPayment(null);
      setPaymentError(null);
      setQrImageUrl(null);
      setQrImagePayload(null);
    }
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen || !summary) {
      return undefined;
    }

    let active = true;
    setIsCreatingPayment(true);
    setPayment(null);
    setPaymentError(null);
    setQrImageUrl(null);
    setQrImagePayload(null);

    void createPaymentRef.current()
      .then((result) => {
        if (!active) {
          return;
        }

        const qrCode = result.qrCode?.trim() || result.cashierUrl?.trim();
        const normalizedResult = qrCode
          ? { ...result, qrCode }
          : result;
        setPayment(normalizedResult);
        if (normalizedResult.status === "failed") {
          setPaymentError(copyRef.current.paymentUnavailableDescription);
        } else if (!normalizedResult.qrCode && normalizedResult.status !== "completed") {
          setPaymentError(copyRef.current.paymentUnavailableDescription);
        } else if (normalizedResult.status === "completed") {
          notifyPaymentCompleted(normalizedResult);
        }
      })
      .catch((error) => {
        if (active) {
          // Provider details are not safe or locale-stable UI copy. Keep the
          // checkout surface on the package-owned localized error message.
          void error;
          setPaymentError(copyRef.current.paymentUnavailableDescription);
        }
      })
      .finally(() => {
        if (active) {
          setIsCreatingPayment(false);
        }
      });

    return () => {
      active = false;
    };
  }, [attempt, isOpen, notifyPaymentCompleted, summaryId]);

  useEffect(() => {
    if (
      !isOpen
      || payment?.status !== "pending"
      || !payment.orderId
      || !getPaymentStatusRef.current
    ) {
      return undefined;
    }

    let active = true;
    let isPolling = false;
    const currentPayment = payment;
    const poll = async () => {
      if (isPolling) {
        return;
      }

      const getPaymentStatus = getPaymentStatusRef.current;
      if (!getPaymentStatus) {
        return;
      }

      isPolling = true;
      try {
        const update = await getPaymentStatus(currentPayment);
        if (!active) {
          return;
        }

        const nextPayment = {
          ...currentPayment,
          ...update,
          orderId: update.orderId ?? currentPayment.orderId,
        };
        setPayment((current) => (
          current?.orderId === currentPayment.orderId ? nextPayment : current
        ));

        if (nextPayment.status === "completed") {
          notifyPaymentCompleted(nextPayment);
        } else if (nextPayment.status === "failed") {
          setPaymentError(copyRef.current.paymentUnavailableDescription);
        }
      } catch {
        // Keep the valid QR code visible and retry a transient status read.
      } finally {
        isPolling = false;
      }
    };

    void poll();
    const interval = window.setInterval(
      () => {
        void poll();
      },
      Math.max(1_000, Math.round(pollIntervalRef.current ?? 2_500)),
    );

    return () => {
      active = false;
      window.clearInterval(interval);
    };
  }, [isOpen, notifyPaymentCompleted, payment?.orderId, payment?.status]);

  useEffect(() => {
    if (!payment?.qrCode) {
      setQrImageUrl(null);
      setQrImagePayload(null);
      return undefined;
    }

    if (payment.status === "pending") {
      setPaymentError(null);
    }

    if (isImageDataUrl(payment.qrCode)) {
      setQrImageUrl(payment.qrCode);
      setQrImagePayload(payment.qrCode);
      return undefined;
    }

    let active = true;
    const qrCodePayload = payment.qrCode;
    setQrImageUrl(null);
    setQrImagePayload(null);
    void toDataURL(payment.qrCode, {
      errorCorrectionLevel: "M",
      margin: 1,
      width: 256,
    })
      .then((value) => {
        if (active) {
          setQrImageUrl(value);
          setQrImagePayload(qrCodePayload);
        }
      })
      .catch(() => {
        if (active) {
          setPaymentError(copyRef.current.paymentUnavailableDescription);
        }
      });

    return () => {
      active = false;
    };
  }, [payment?.qrCode]);

  if (!isOpen || !summary) {
    return null;
  }

  const isCompleted = payment?.status === "completed";
  const isPreparingQr = (
    payment?.status === "pending"
    && Boolean(payment.qrCode)
    && qrImagePayload !== payment.qrCode
    && !paymentError
  );
  const canScan = (
    payment?.status === "pending"
    && Boolean(qrImageUrl)
    && qrImagePayload === payment.qrCode
  );

  return (
    <Modal
      onOpenChange={(open) => {
        if (!open) {
          onClose();
        }
      }}
      open={isOpen}
    >
      <ModalContent
        aria-describedby={undefined}
        aria-labelledby="sdkwork-order-checkout-title"
        className="sdkwork-order-checkout-dialog"
        showCloseButton={false}
        size="lg"
      >
        <ModalHeader className="sdkwork-order-checkout-dialog__header">
          <ModalTitle className="sdkwork-order-checkout-dialog__title" id="sdkwork-order-checkout-title">
            {copy.payByQr} {summary.priceLabel}
          </ModalTitle>
          <ModalClose
            aria-label={copy.close}
            className="sdkwork-order-checkout-dialog__close"
          >
            <X aria-hidden="true" className="sdkwork-order-checkout-dialog__close-icon" />
          </ModalClose>
        </ModalHeader>
        <ModalBody className="sdkwork-order-checkout-dialog__body">
          <div
            className="sdkwork-order-checkout-dialog__summary"
            data-sdk-region="order-checkout-summary"
          >
            <div className="sdkwork-order-checkout-dialog__summary-card">
              <div className="sdkwork-order-checkout-dialog__summary-header">
                <div>
                  <div className="sdkwork-order-checkout-dialog__label">{copy.selectedItem}</div>
                  <div className="sdkwork-order-checkout-dialog__plan-name">{summary.name}</div>
                  {summary.periodLabel ? (
                    <div className="sdkwork-order-checkout-dialog__period">{summary.periodLabel}</div>
                  ) : null}
                </div>
                <div className="sdkwork-order-checkout-dialog__price-block">
                  <div className="sdkwork-order-checkout-dialog__label">{copy.price}</div>
                  <div className="sdkwork-order-checkout-dialog__price">
                    {summary.priceLabel}
                  </div>
                  {summary.originalPriceLabel ? (
                    <div className="sdkwork-order-checkout-dialog__original-price">
                      {summary.originalPriceLabel}
                    </div>
                  ) : null}
                </div>
              </div>
            </div>
            <div className="sdkwork-order-checkout-dialog__benefit-grid">
              <div className="sdkwork-order-checkout-dialog__benefit-card">
                <div className="sdkwork-order-checkout-dialog__benefit-title">
                  <ShieldCheck aria-hidden="true" className="sdkwork-order-checkout-dialog__benefit-icon sdkwork-order-checkout-dialog__benefit-icon--secure" />
                  {copy.secureTitle}
                </div>
                <p className="sdkwork-order-checkout-dialog__benefit-description">{copy.secureDescription}</p>
              </div>
              <div className="sdkwork-order-checkout-dialog__benefit-card">
                <div className="sdkwork-order-checkout-dialog__benefit-title">
                  <Sparkles aria-hidden="true" className="sdkwork-order-checkout-dialog__benefit-icon sdkwork-order-checkout-dialog__benefit-icon--activation" />
                  {copy.activationTitle}
                </div>
                <p className="sdkwork-order-checkout-dialog__benefit-description">{copy.activationDescription}</p>
              </div>
            </div>
          </div>
          <aside
            className="sdkwork-order-checkout-dialog__payment-panel"
            data-sdk-region="order-checkout-payment"
          >
            <p className="sdkwork-order-checkout-dialog__payment-label">{copy.payByQr}</p>
            <p className="sdkwork-order-checkout-dialog__payment-price">{summary.priceLabel}</p>
            {isCreatingPayment || isPreparingQr ? (
              <div className="sdkwork-order-checkout-dialog__pending">
                <QrCode aria-hidden="true" className="sdkwork-order-checkout-dialog__pending-icon" />
                <span>{copy.creatingPayment}</span>
              </div>
            ) : null}
            {!isCreatingPayment && !isPreparingQr && canScan ? (
              <div className="sdkwork-order-checkout-dialog__qr-wrap">
                <img
                  alt={copy.scanPrompt}
                  className="sdkwork-order-checkout-dialog__qr-image"
                  src={qrImageUrl ?? undefined}
                />
                <p className="sdkwork-order-checkout-dialog__scan-prompt">
                  <Smartphone aria-hidden="true" className="sdkwork-order-checkout-dialog__scan-icon" />
                  {copy.scanPrompt}
                </p>
              </div>
            ) : null}
            {!isCreatingPayment && !isPreparingQr && isCompleted ? (
              <div className="sdkwork-order-checkout-dialog__completed">
                <CheckCircle2 aria-hidden="true" className="sdkwork-order-checkout-dialog__completed-icon" />
                <span className="sdkwork-order-checkout-dialog__completed-label">{copy.completed}</span>
                <Button onClick={onClose} type="button">
                  {copy.close}
                </Button>
              </div>
            ) : null}
            {!isCreatingPayment && !isPreparingQr && !canScan && !isCompleted ? (
              <div className="sdkwork-order-checkout-dialog__unavailable">
                <StatusNotice tone="danger" title={copy.paymentUnavailable}>
                  <span className="sdkwork-order-checkout-dialog__error-copy">
                    {paymentError ?? copy.paymentUnavailableDescription}
                  </span>
                </StatusNotice>
                <Button
                  className="sdkwork-order-checkout-dialog__retry"
                  onClick={() => setAttempt((current) => current + 1)}
                  type="button"
                  variant="secondary"
                >
                  {copy.retry}
                </Button>
              </div>
            ) : null}
          </aside>
        </ModalBody>
      </ModalContent>
    </Modal>
  );
}

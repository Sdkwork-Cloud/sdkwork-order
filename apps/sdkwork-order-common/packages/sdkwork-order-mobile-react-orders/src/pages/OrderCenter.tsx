import { ReceiptText } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router";

import { CapabilityUnavailablePage } from "@sdkwork/ui-mobile-react";

export function OrderCenter() {
  const { t } = useTranslation("orders");
  const navigate = useNavigate();

  return (
    <CapabilityUnavailablePage
      icon={ReceiptText}
      message={t("unavailable")}
      onBack={() => navigate(-1)}
      title={t("title")}
    />
  );
}

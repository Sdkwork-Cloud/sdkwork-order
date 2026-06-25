import {
  CheckCircle2,
  Clock3,
  Package,
  ReceiptText,
} from "lucide-react";
import { StatCard } from "@sdkwork/ui-pc-react";
import type { SdkworkOrderStatistics } from "../order-service";
import { useSdkworkOrderIntl } from "../order-intl";

export interface SdkworkOrderStatGridProps {
  statistics: SdkworkOrderStatistics;
}

export function SdkworkOrderStatGrid({
  statistics,
}: SdkworkOrderStatGridProps) {
  const { copy, formatCurrencyCny } = useSdkworkOrderIntl();

  return (
    <div className="grid gap-5 sm:grid-cols-2 xl:grid-cols-4">
      <StatCard
        icon={<ReceiptText className="h-5 w-5" />}
        label={copy.stats.totalOrders}
        value={String(statistics.totalOrders)}
      />
      <StatCard
        changeTone="warning"
        icon={<Clock3 className="h-5 w-5" />}
        label={copy.stats.pendingPayment}
        value={String(statistics.pendingPayment)}
      />
      <StatCard
        changeTone="success"
        icon={<CheckCircle2 className="h-5 w-5" />}
        label={copy.stats.completed}
        value={String(statistics.completed)}
      />
      <StatCard
        icon={<Package className="h-5 w-5" />}
        label={copy.stats.totalAmount}
        value={formatCurrencyCny(statistics.totalAmountCny)}
      />
    </div>
  );
}

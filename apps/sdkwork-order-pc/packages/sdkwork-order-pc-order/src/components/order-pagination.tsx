import { ChevronLeft, ChevronRight } from "lucide-react";
import { Button } from "@sdkwork/ui-pc-react";
import type { SdkworkOrderController } from "../order-controller";
import { useSdkworkOrderControllerState } from "../order-controller";
import { useSdkworkOrderIntl } from "../order-intl";

export interface SdkworkOrderPaginationProps {
  controller: SdkworkOrderController;
}

/**
 * Server-driven pagination for the owner order list.
 *
 * The component reads pagination metadata straight from the dashboard payload
 * (computed server-side via `COUNT(*) OVER()`) and dispatches `setPage` calls
 * back to the controller. It is intentionally compact: prev / page indicator /
 * next, plus a one-line summary. Anything more elaborate (jump-to-page,
 * page-size selector) belongs in a dedicated toolbar and is tracked as a
 * follow-up rather than expanding this leaf component.
 */
export function SdkworkOrderPagination({
  controller,
}: SdkworkOrderPaginationProps) {
  const {
    copy,
    formatPaginationPageLabel,
    formatPaginationSummary,
  } = useSdkworkOrderIntl();
  const state = useSdkworkOrderControllerState(controller);
  const pagination = state.dashboard.pagination;
  const totalPages = pagination.totalPages ?? 0;
  const page = pagination.page;
  const shown = state.visibleOrders.length;
  const total = pagination.total;
  const hasPrev = page > 1;
  const hasNext = pagination.hasMore || page < totalPages;
  const isLoading = state.isLoading;

  if (total === 0 && !isLoading) {
    // Avoid rendering a pager for empty result sets — the empty state panel
    // already communicates that there is nothing to paginate.
    return null;
  }

  return (
    <div className="flex flex-col gap-3 border-t border-[var(--sdk-color-border-subtle)] px-6 py-4 sm:flex-row sm:items-center sm:justify-between">
      <div className="text-xs text-[var(--sdk-color-text-secondary)]">
        {formatPaginationSummary(shown, total)}
      </div>
      <div className="flex items-center gap-2">
        <Button
          aria-label={copy.pagination.prev}
          disabled={!hasPrev || isLoading}
          onClick={() => void controller.setPage(page - 1)}
          size="sm"
          type="button"
          variant="outline"
        >
          <ChevronLeft className="h-4 w-4" />
          <span>{copy.pagination.prev}</span>
        </Button>
        <span
          aria-live="polite"
          className="min-w-[7.5rem] text-center text-xs font-medium text-[var(--sdk-color-text-primary)]"
        >
          {formatPaginationPageLabel(page, Math.max(totalPages, page))}
        </span>
        <Button
          aria-label={copy.pagination.next}
          disabled={!hasNext || isLoading}
          onClick={() => void controller.setPage(page + 1)}
          size="sm"
          type="button"
          variant="outline"
        >
          <span>{copy.pagination.next}</span>
          <ChevronRight className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}

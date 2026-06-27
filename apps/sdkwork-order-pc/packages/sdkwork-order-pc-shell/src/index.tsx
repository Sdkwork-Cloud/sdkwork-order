import { useMemo } from "react";

import { sdkworkOrderPcRuntimeIdentity } from "@sdkwork/order-pc-core";
import { SdkworkOrderPage } from "@sdkwork/order-pc-order";
import { SdkworkThemeProvider } from "@sdkwork/ui-pc-react";

export interface OrderAppShellProps {
  /**
   * Optional order page controller override. When omitted, the shell creates a
   * default controller bound to the runtime identity and the
   * `@sdkwork/order-service` SDK.
   */
  orderController?: React.ComponentProps<typeof SdkworkOrderPage>["controller"];
  /**
   * Initial locale for the order surface. Defaults to `zh-CN` to match the
   * document language set by `index.html`.
   */
  locale?: string;
  /**
   * Copy overrides forwarded to the order page i18n provider.
   */
  messages?: React.ComponentProps<typeof SdkworkOrderPage>["messages"];
  /**
   * Theme variant. Defaults to `light` to match the design system baseline.
   */
  theme?: "light" | "dark";
}

/**
 * Application shell for the standalone `sdkwork-order-pc` build.
 *
 * Composes:
 * - `SdkworkThemeProvider` — provides the design tokens via CSS variables.
 * - `SdkworkOrderPage` — the order capability PC surface.
 *
 * The shell intentionally stays thin: routing, global navigation, and session
 * provisioning are owned by composition hosts (`sdkwork-mall`, …), not by this
 * standalone capability build.
 */
export function OrderAppShell({
  orderController,
  locale = "zh-CN",
  messages,
  theme = "light",
}: OrderAppShellProps = {}) {
  const page = useMemo(
    () => (
      <SdkworkOrderPage
        controller={orderController}
        locale={locale}
        messages={messages}
      />
    ),
    [orderController, locale, messages],
  );

  return (
    <SdkworkThemeProvider defaultTheme={theme}>
      <a href="#order-content" className="order-shell-skip-link">
        跳到主内容
      </a>
      <main id="order-content" className="order-shell">
        <header className="order-shell-header" role="banner">
          <span className="order-shell-mark">SDKWork</span>
          <h1 className="order-shell-title">{sdkworkOrderPcRuntimeIdentity.appKey}</h1>
          <p className="order-shell-subtitle">
            Order capability PC surface — aligned with sdkwork-specs building-block model.
          </p>
        </header>
        <section className="order-shell-body" role="main">
          {page}
        </section>
      </main>
    </SdkworkThemeProvider>
  );
}

export default OrderAppShell;

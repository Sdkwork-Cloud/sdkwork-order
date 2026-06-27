import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import "@sdkwork/ui-pc-react/styles.css";
import "./app.css";

import { OrderAppShell } from "@sdkwork/order-pc-shell";

const initialTheme = (() => {
  if (typeof window === "undefined") {
    return "light" as const;
  }
  const stored = window.localStorage.getItem("sdkwork-order-theme");
  if (stored === "dark" || stored === "light") {
    return stored;
  }
  const prefersDark = window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false;
  return prefersDark ? "dark" : "light";
})();

const initialLocale = (() => {
  if (typeof document === "undefined") {
    return "zh-CN";
  }
  return document.documentElement.lang || "zh-CN";
})();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <OrderAppShell theme={initialTheme} locale={initialLocale} />
  </StrictMode>,
);

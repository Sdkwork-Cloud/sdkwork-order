import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { OrderAppShell } from "@sdkwork/order-pc-shell";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <OrderAppShell />
  </StrictMode>,
);

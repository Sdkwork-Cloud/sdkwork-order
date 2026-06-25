import type { SdkworkOrderPcRouteContribution } from "@sdkwork/order-pc-core";

export const sdkworkOrderPcOrderRoutes = [
  {
    auth: "required",
    capability: "order",
    domain: "commerce",
    id: "app.commerce.order.dashboard",
    packageName: "@sdkwork/order-pc-order",
    path: "/app/order",
    screen: "dashboard",
    surface: "app",
    title: "Orders",
    titleKey: "order.routes.dashboard.title",
  },
] as const satisfies readonly SdkworkOrderPcRouteContribution[];

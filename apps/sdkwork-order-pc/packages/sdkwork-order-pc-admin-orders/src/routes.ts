import type { SdkworkOrderPcRouteContribution } from "@sdkwork/order-pc-core";

export const sdkworkOrderPcAdminOrdersRoutes = [
  {
    auth: "required",
    capability: "admin-orders",
    domain: "commerce",
    id: "admin.commerce.orders",
    packageName: "@sdkwork/order-pc-admin-orders",
    path: "/admin/orders",
    permissionHint: "commerce.orders.read",
    screen: "orders",
    surface: "backend-admin",
    title: "订单监管",
    titleKey: "adminOrders.routes.orders.title",
  },
] as const satisfies readonly SdkworkOrderPcRouteContribution[];

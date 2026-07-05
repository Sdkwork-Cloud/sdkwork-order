import { createSdkworkOrderPcRouteRegistry } from "@sdkwork/order-pc-core";
import { sdkworkOrderPcAdminOrdersRoutes } from "@sdkwork/order-pc-admin-orders";
import { sdkworkOrderPcOrderRoutes } from "@sdkwork/order-pc-order";

export const sdkworkOrderPcRoutes = createSdkworkOrderPcRouteRegistry(
  sdkworkOrderPcOrderRoutes,
  sdkworkOrderPcAdminOrdersRoutes,
);

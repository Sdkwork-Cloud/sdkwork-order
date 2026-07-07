import { SDKWORK_ORDER_PC_SDK_PACKAGES } from "../sdk/index";

export function listSdkworkCoreSdkInventory() {
  return [
    {
      packageName: SDKWORK_ORDER_PC_SDK_PACKAGES.app,
      role: "app-api",
      capability: "order",
    },
    {
      packageName: SDKWORK_ORDER_PC_SDK_PACKAGES.backend,
      role: "backend-api",
      capability: "order",
    },
  ] as const;
}

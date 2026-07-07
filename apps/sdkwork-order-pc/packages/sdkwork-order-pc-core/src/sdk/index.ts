export const SDKWORK_ORDER_PC_SDK_PACKAGES = {
  app: "@sdkwork/order-app-sdk",
  backend: "@sdkwork/order-backend-sdk",
} as const;

export type SdkworkOrderPcSdkPackageRole = keyof typeof SDKWORK_ORDER_PC_SDK_PACKAGES;

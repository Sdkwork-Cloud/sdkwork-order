export type SdkworkOrderPcRouteSurface = "app" | "backend-admin";

export interface SdkworkOrderPcRouteContribution {
  readonly auth: "public" | "required";
  readonly capability: string;
  readonly domain: "commerce";
  readonly id: string;
  readonly packageName: string;
  readonly path: string;
  readonly permissionHint?: string;
  readonly screen: string;
  readonly surface: SdkworkOrderPcRouteSurface;
  readonly title: string;
  readonly titleKey: string;
}

export const sdkworkOrderPcRuntimeIdentity = {
  appKey: "sdkwork-order-pc",
  architecture: "pc-react",
  domain: "commerce",
  capability: "order",
  runtimeFamily: "web",
} as const;

export function createSdkworkOrderPcRouteRegistry(
  ...routeGroups: readonly (readonly SdkworkOrderPcRouteContribution[])[]
): readonly SdkworkOrderPcRouteContribution[] {
  return routeGroups.flat();
}

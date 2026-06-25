import {
  createSdkworkAppCapabilityManifest,
  type CreateSdkworkAppCapabilityManifestOptions,
  type SdkworkAppCapabilityManifest,
} from "@sdkwork/appbase-pc-react";
import {
  createSdkworkOrderMessages,
  type SdkworkOrderMessagesOverrides,
} from "./order-copy";

export interface SdkworkOrderWorkspaceManifest extends SdkworkAppCapabilityManifest {
  capability: "order";
  routePath: string;
}

export interface CreateOrderWorkspaceManifestOptions
  extends Partial<
    Pick<CreateSdkworkAppCapabilityManifestOptions, "description" | "host" | "id" | "packageNames" | "theme" | "title">
  > {
  locale?: string | null;
  messages?: SdkworkOrderMessagesOverrides;
  routePath?: string;
}

export interface SdkworkOrderRouteIntent {
  focusWindow: boolean;
  orderId?: string;
  route: string;
  source: "order-workspace";
  type: "order-route-intent";
}

export interface CreateOrderRouteIntentOptions {
  basePath?: string;
  focusWindow?: boolean;
  orderId?: string;
}

function normalizeBasePath(basePath: string | undefined): string {
  const normalized = (basePath ?? "/orders").trim();
  if (!normalized || normalized === "/") {
    return "/orders";
  }

  return normalized.endsWith("/") ? normalized.slice(0, -1) : normalized;
}

export function createOrderWorkspaceManifest({
  description,
  host,
  id = "sdkwork-order",
  locale,
  messages,
  packageNames = ["@sdkwork/order-pc-order"],
  routePath = "/orders",
  theme,
  title,
}: CreateOrderWorkspaceManifestOptions = {}): SdkworkOrderWorkspaceManifest {
  const copy = createSdkworkOrderMessages(locale, messages).manifest;

  return {
    ...createSdkworkAppCapabilityManifest({
      description: description ?? copy.description,
      host,
      id,
      packageNames,
      theme,
      title: title ?? copy.title,
    }),
    capability: "order",
    routePath: normalizeBasePath(routePath),
  };
}

export function createOrderRouteIntent(
  options: CreateOrderRouteIntentOptions = {},
): SdkworkOrderRouteIntent {
  const basePath = normalizeBasePath(options.basePath);
  const queryParams = new URLSearchParams();

  if (options.orderId) {
    queryParams.set("orderId", options.orderId);
  }

  const querySuffix = queryParams.toString() ? `?${queryParams.toString()}` : "";

  return {
    focusWindow: options.focusWindow !== false,
    ...(options.orderId ? { orderId: options.orderId } : {}),
    route: `${basePath}${querySuffix}`,
    source: "order-workspace",
    type: "order-route-intent",
  };
}

export const orderPackageMeta = {
  architecture: "pc-react",
  domain: "commerce",
  package: "@sdkwork/order-pc-order",
  status: "ready",
} as const;

export type OrderPackageMeta = typeof orderPackageMeta;

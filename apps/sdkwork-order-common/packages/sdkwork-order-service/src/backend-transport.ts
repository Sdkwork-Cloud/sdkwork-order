import {
  createClient,
  type SdkworkOrderBackendClient,
  type SdkworkBackendConfig,
} from "@sdkwork/order-backend-sdk";

const BACKEND_API_SUFFIX = "/backend/v3/api";

export function resolveOrderBackendApiOrigin(baseUrl: string): string {
  const trimmed = baseUrl.trim().replace(/\/+$/u, "");
  if (trimmed.endsWith(BACKEND_API_SUFFIX)) {
    return trimmed.slice(0, -BACKEND_API_SUFFIX.length);
  }
  return trimmed;
}

export interface BootstrapSdkworkOrderBackendSdkInput {
  baseUrl: string;
  authToken?: string;
  accessToken?: string;
  tenantId?: string;
  organizationId?: string;
  platform?: string;
}

export function createOrderBackendTransportClient(
  input: BootstrapSdkworkOrderBackendSdkInput,
): SdkworkOrderBackendClient {
  const config: SdkworkBackendConfig = {
    authMode: "dual-token",
    baseUrl: resolveOrderBackendApiOrigin(input.baseUrl),
    authToken: input.authToken,
    accessToken: input.accessToken,
    tenantId: input.tenantId,
    organizationId: input.organizationId,
    platform: input.platform ?? "pc",
  };
  return createClient(config);
}

let backendClient: SdkworkOrderBackendClient | null = null;

export function bootstrapSdkworkOrderBackendSdk(
  input: BootstrapSdkworkOrderBackendSdkInput,
): SdkworkOrderBackendClient {
  backendClient = createOrderBackendTransportClient(input);
  return backendClient;
}

export function getSdkworkOrderBackendSdkClient(): SdkworkOrderBackendClient {
  if (!backendClient) {
    throw new Error(
      "SDKWork order backend SDK is not configured. Call bootstrapSdkworkOrderBackendSdk() from order PC bootstrap.",
    );
  }
  return backendClient;
}

export function resetSdkworkOrderBackendSdkClient(): void {
  backendClient = null;
}

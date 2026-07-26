import {
  createClient as createGeneratedAppClient,
  SdkworkAppClient as GeneratedSdkworkAppClient,
} from '../generated/server-openapi/src/index';
import type { SdkworkAppConfig } from '../generated/server-openapi/src/types/common';
import { applySdkworkIdempotencyRequestFingerprint } from './idempotency-request-fingerprint';

export { createGeneratedAppClient };
export type { SdkworkAppConfig };
export * from '../generated/server-openapi/src/types';
export * from '../generated/server-openapi/src/api';
export * from '../generated/server-openapi/src/http';
export * from '../generated/server-openapi/src/auth';

export class SdkworkAppClient extends GeneratedSdkworkAppClient {
  constructor(config: SdkworkAppConfig) {
    super(config);
    this.http.addRequestInterceptor(applySdkworkIdempotencyRequestFingerprint);
  }
}

export function createClient(config: SdkworkAppConfig): SdkworkAppClient {
  return new SdkworkAppClient(config);
}

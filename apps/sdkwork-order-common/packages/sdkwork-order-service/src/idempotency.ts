import { uuid } from "@sdkwork/utils";

export interface SdkworkIdempotencyParams {
  idempotencyKey: string;
}

export function createSdkworkIdempotencyParams(
  idempotencyKey?: string,
): SdkworkIdempotencyParams {
  return { idempotencyKey: idempotencyKey ?? uuid() };
}

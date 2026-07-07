import { uuid } from "@sdkwork/utils";

export interface SdkworkWriteCommandHeaders {
  idempotencyKey: string;
  sdkworkRequestHash: string;
}

function normalizeRequestHashPart(part: string): string {
  return part
    .split("")
    .map((character) => {
      if (/^[a-zA-Z0-9]$/.test(character) || character === "-" || character === "_" || character === ".") {
        return character;
      }
      return "-";
    })
    .join("");
}

export function stableCommandRequestHash(scope: string, parts: readonly string[]): string {
  return [scope, ...parts].map(normalizeRequestHashPart).join("-");
}

function canonicalJsonString(value: unknown): string {
  if (value === null || value === undefined) {
    return "null";
  }
  if (typeof value === "boolean") {
    return String(value);
  }
  if (typeof value === "number") {
    return String(value);
  }
  if (typeof value === "string") {
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) {
    return `[${value.map((entry) => canonicalJsonString(entry)).join(",")}]`;
  }
  if (typeof value === "object") {
    const record = value as Record<string, unknown>;
    const keys = Object.keys(record).sort();
    const items = keys
      .filter((key) => record[key] !== null && record[key] !== undefined)
      .map((key) => `${JSON.stringify(key)}:${canonicalJsonString(record[key])}`);
    return `{${items.join(",")}}`;
  }
  return JSON.stringify(value);
}

export function stableJsonRequestHash(scope: string, payload: unknown): string {
  return stableCommandRequestHash(scope, [canonicalJsonString(payload)]);
}

export function writePayloadWithRouteParam(
  routeParamKey: string,
  routeParamValue: string,
  body: unknown,
): Record<string, unknown> {
  const payload =
    typeof body === "object" && body !== null && !Array.isArray(body)
      ? { ...(body as Record<string, unknown>) }
      : {};
  payload[routeParamKey] = routeParamValue;
  return payload;
}

export function createSdkworkWriteCommandHeaders(
  scope: string,
  payload: unknown,
  idempotencyKey?: string,
): SdkworkWriteCommandHeaders {
  return {
    idempotencyKey: idempotencyKey ?? uuid(),
    sdkworkRequestHash: stableJsonRequestHash(scope, payload),
  };
}

export interface CheckoutSessionHashInput {
  tenantId: string;
  organizationId?: string | null;
  ownerUserId: string;
  currencyCode: string;
  lines: ReadonlyArray<{ skuId: string; quantity: number }>;
  requestNo: string;
}

export interface CheckoutQuoteHashInput {
  tenantId: string;
  organizationId?: string | null;
  ownerUserId: string;
  checkoutSessionId: string;
  requestNo: string;
}

export interface CheckoutOwnerOrderHashInput {
  tenantId: string;
  organizationId?: string | null;
  ownerUserId: string;
  checkoutSessionId: string;
  requestNo: string;
}

export function checkoutSessionRequestHash(input: CheckoutSessionHashInput): string {
  const lines = input.lines.map((line) => `${line.skuId}:${line.quantity}`).join(",");
  return stableCommandRequestHash("checkout.sessions.create", [
    input.tenantId,
    input.organizationId ?? "global",
    input.ownerUserId,
    input.currencyCode,
    lines,
    input.requestNo,
  ]);
}

export function checkoutQuoteRequestHash(input: CheckoutQuoteHashInput): string {
  return stableCommandRequestHash("checkout.sessions.quotes.create", [
    input.tenantId,
    input.organizationId ?? "global",
    input.ownerUserId,
    input.checkoutSessionId,
    input.requestNo,
  ]);
}

export function checkoutOwnerOrderRequestHash(input: CheckoutOwnerOrderHashInput): string {
  return stableCommandRequestHash("checkout.sessions.orders.create", [
    input.tenantId,
    input.organizationId ?? "global",
    input.ownerUserId,
    input.checkoutSessionId,
    input.requestNo,
  ]);
}

export function createCheckoutSessionWriteHeaders(
  input: CheckoutSessionHashInput,
  idempotencyKey?: string,
): SdkworkWriteCommandHeaders {
  return {
    idempotencyKey: idempotencyKey ?? uuid(),
    sdkworkRequestHash: checkoutSessionRequestHash(input),
  };
}

export function createCheckoutQuoteWriteHeaders(
  input: CheckoutQuoteHashInput,
  idempotencyKey?: string,
): SdkworkWriteCommandHeaders {
  return {
    idempotencyKey: idempotencyKey ?? uuid(),
    sdkworkRequestHash: checkoutQuoteRequestHash(input),
  };
}

export function createCheckoutOwnerOrderWriteHeaders(
  input: CheckoutOwnerOrderHashInput,
  idempotencyKey?: string,
): SdkworkWriteCommandHeaders {
  return {
    idempotencyKey: idempotencyKey ?? uuid(),
    sdkworkRequestHash: checkoutOwnerOrderRequestHash(input),
  };
}

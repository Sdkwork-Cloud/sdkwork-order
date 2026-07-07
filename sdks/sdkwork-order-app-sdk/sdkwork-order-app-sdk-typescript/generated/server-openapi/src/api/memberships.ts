import { appApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { MembershipOrderCreateCommand } from '../types';


export interface MembershipsOrdersCreateParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
  xIdempotencyFingerprint: string;
}

export class MembershipsOrdersApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Memberships orders create. */
  async create(body: MembershipOrderCreateCommand, params: MembershipsOrdersCreateParams): Promise<Record<string, unknown>> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
        'X-Idempotency-Fingerprint': { value: params.xIdempotencyFingerprint, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<Record<string, unknown>>(appApiPath(`/memberships/orders`), body, undefined, requestHeaders, 'application/json');
  }
}

export class MembershipsApi {
  private client: HttpClient;
  public readonly orders: MembershipsOrdersApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.orders = new MembershipsOrdersApi(client);
  }

}

export function createMembershipsApi(client: HttpClient): MembershipsApi {
  return new MembershipsApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}



function buildRequestHeaders(
  headers: Record<string, HeaderParameterSpec | undefined>,
  cookies: Record<string, HeaderParameterSpec | undefined> = {},
): Record<string, string> | undefined {
  const requestHeaders: Record<string, string> = {};

  for (const [name, parameter] of Object.entries(headers)) {
    const serialized = serializeParameterValue(parameter);
    if (serialized !== undefined) {
      requestHeaders[name] = serialized;
    }
  }

  const cookieHeader = buildCookieHeader(cookies);
  if (cookieHeader) {
    requestHeaders.Cookie = requestHeaders.Cookie
      ? `${requestHeaders.Cookie}; ${cookieHeader}`
      : cookieHeader;
  }

  return Object.keys(requestHeaders).length > 0 ? requestHeaders : undefined;
}

interface HeaderParameterSpec {
  value: unknown;
  style: string;
  explode: boolean;
  contentType?: string;
}

function buildCookieHeader(cookies: Record<string, HeaderParameterSpec | undefined>): string | undefined {
  const pairs: string[] = [];
  for (const [name, parameter] of Object.entries(cookies)) {
    const serialized = serializeParameterValue(parameter);
    if (serialized !== undefined) {
      pairs.push(`${encodeURIComponent(name)}=${encodeURIComponent(serialized)}`);
    }
  }
  return pairs.length > 0 ? pairs.join('; ') : undefined;
}

function serializeParameterValue(parameter: HeaderParameterSpec | undefined): string | undefined {
  const value = parameter?.value;
  if (value === undefined || value === null) {
    return undefined;
  }
  if (parameter?.contentType) {
    return JSON.stringify(value);
  }
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (Array.isArray(value)) {
    return value.map((item) => serializeHeaderPrimitive(item)).join(',');
  }
  if (typeof value === 'object' && value !== null) {
    return serializeHeaderObject(value as Record<string, unknown>, parameter?.explode === true);
  }
  return serializeHeaderPrimitive(value);
}

function serializeHeaderObject(value: Record<string, unknown>, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (explode) {
    return entries.map(([key, entryValue]) => `${key}=${serializeHeaderPrimitive(entryValue)}`).join(',');
  }
  return entries.flatMap(([key, entryValue]) => [key, serializeHeaderPrimitive(entryValue)]).join(',');
}

function serializeHeaderPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  return String(value);
}

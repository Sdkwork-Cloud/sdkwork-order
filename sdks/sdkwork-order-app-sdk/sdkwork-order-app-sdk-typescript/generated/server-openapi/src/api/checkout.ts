import { appApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { CommerceOperationCommand } from '../types';


export interface CheckoutSessionsOrdersCreateParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
}

export class CheckoutSessionsOrdersApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Checkout sessions orders create. */
  async create(checkoutSessionId: string, body: CommerceOperationCommand, params: CheckoutSessionsOrdersCreateParams): Promise<Record<string, unknown>> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<Record<string, unknown>>(appApiPath(`/checkout/sessions/${serializePathParameter(checkoutSessionId, { name: 'checkoutSessionId', style: 'simple', explode: false })}/orders`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface CheckoutSessionsQuotesCreateParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
}

export class CheckoutSessionsQuotesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Checkout sessions quotes create. */
  async create(checkoutSessionId: string, body: CommerceOperationCommand, params: CheckoutSessionsQuotesCreateParams): Promise<Record<string, unknown>> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<Record<string, unknown>>(appApiPath(`/checkout/sessions/${serializePathParameter(checkoutSessionId, { name: 'checkoutSessionId', style: 'simple', explode: false })}/quotes`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface CheckoutSessionsCreateParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
}

export class CheckoutSessionsApi {
  private client: HttpClient;
  public readonly quotes: CheckoutSessionsQuotesApi;
  public readonly orders: CheckoutSessionsOrdersApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.quotes = new CheckoutSessionsQuotesApi(client);
    this.orders = new CheckoutSessionsOrdersApi(client);
  }


/** Checkout sessions create. */
  async create(body: CommerceOperationCommand, params: CheckoutSessionsCreateParams): Promise<Record<string, unknown>> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<Record<string, unknown>>(appApiPath(`/checkout/sessions`), body, undefined, requestHeaders, 'application/json');
  }

/** Checkout sessions retrieve. */
  async retrieve(checkoutSessionId: string): Promise<Record<string, unknown>> {
    return this.client.get<Record<string, unknown>>(appApiPath(`/checkout/sessions/${serializePathParameter(checkoutSessionId, { name: 'checkoutSessionId', style: 'simple', explode: false })}`));
  }
}

export class CheckoutApi {
  private client: HttpClient;
  public readonly sessions: CheckoutSessionsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.sessions = new CheckoutSessionsApi(client);
  }

}

export function createCheckoutApi(client: HttpClient): CheckoutApi {
  return new CheckoutApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}

interface PathParameterSpec {
  name: string;
  style: string;
  explode: boolean;
}

function serializePathParameter(value: unknown, spec: PathParameterSpec): string {
  if (value === undefined || value === null) {
    return '';
  }

  const style = spec.style || 'simple';
  if (Array.isArray(value)) {
    return serializePathArray(spec.name, value, style, spec.explode);
  }
  if (typeof value === 'object') {
    return serializePathObject(spec.name, value as Record<string, unknown>, style, spec.explode);
  }
  return pathPrefix(spec.name, style, false) + encodePathValue(serializePathPrimitive(value));
}

function serializePathArray(name: string, values: unknown[], style: string, explode: boolean): string {
  const serialized = values
    .filter((item) => item !== undefined && item !== null)
    .map((item) => encodePathValue(serializePathPrimitive(item)));
  if (serialized.length === 0) {
    return pathPrefix(name, style, false);
  }
  if (style === 'matrix') {
    return explode
      ? serialized.map((item) => `;${name}=${item}`).join('')
      : `;${name}=${serialized.join(',')}`;
  }
  return pathPrefix(name, style, false) + serialized.join(explode ? '.' : ',');
}

function serializePathObject(name: string, value: Record<string, unknown>, style: string, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return pathPrefix(name, style, true);
  }
  if (style === 'matrix') {
    return explode
      ? entries.map(([key, entryValue]) => `;${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join('')
      : `;${name}=${entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',')}`;
  }
  const serialized = explode
    ? entries.map(([key, entryValue]) => `${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join(style === 'label' ? '.' : ',')
    : entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',');
  return pathPrefix(name, style, true) + serialized;
}

function pathPrefix(name: string, style: string, _objectValue: boolean): string {
  if (style === 'label') return '.';
  if (style === 'matrix') return `;${name}`;
  return '';
}

function encodePathValue(value: string): string {
  return encodeURIComponent(value);
}

function serializePathPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
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

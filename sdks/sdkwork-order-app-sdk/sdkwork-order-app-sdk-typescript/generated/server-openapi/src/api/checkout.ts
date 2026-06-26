import { appApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { CommerceApiResult, CommerceOperationCommand } from '../types';


export class CheckoutSessionsOrdersApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Checkout sessions orders create. */
  async create(checkoutSessionId: string, body: CommerceOperationCommand): Promise<CommerceApiResult> {
    return this.client.post<CommerceApiResult>(appApiPath(`/checkout/sessions/${serializePathParameter(checkoutSessionId, { name: 'checkoutSessionId', style: 'simple', explode: false })}/orders`), body, undefined, undefined, 'application/json');
  }
}

export class CheckoutSessionsQuotesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Checkout sessions quotes create. */
  async create(checkoutSessionId: string, body: CommerceOperationCommand): Promise<CommerceApiResult> {
    return this.client.post<CommerceApiResult>(appApiPath(`/checkout/sessions/${serializePathParameter(checkoutSessionId, { name: 'checkoutSessionId', style: 'simple', explode: false })}/quotes`), body, undefined, undefined, 'application/json');
  }
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
  async create(body: CommerceOperationCommand): Promise<CommerceApiResult> {
    return this.client.post<CommerceApiResult>(appApiPath(`/checkout/sessions`), body, undefined, undefined, 'application/json');
  }

/** Checkout sessions retrieve. */
  async retrieve(checkoutSessionId: string): Promise<CommerceApiResult> {
    return this.client.get<CommerceApiResult>(appApiPath(`/checkout/sessions/${serializePathParameter(checkoutSessionId, { name: 'checkoutSessionId', style: 'simple', explode: false })}`));
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

import { backendApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { CreatePointsRechargeFulfillmentRequest, SdkWorkCommandData } from '../types';


export class FulfillmentsOrdersPointsRechargeFulfillmentsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Mark payment succeeded (optional) and fulfill a points recharge order */
  async create(orderId: string, body: CreatePointsRechargeFulfillmentRequest): Promise<SdkWorkCommandData> {
    return this.client.post<SdkWorkCommandData>(backendApiPath(`/orders/${serializePathParameter(orderId, { name: 'orderId', style: 'simple', explode: false })}/points_recharge/fulfillments`), body, undefined, undefined, 'application/json');
  }
}

export class FulfillmentsOrdersPointsRechargeApi {
  private client: HttpClient;
  public readonly fulfillments: FulfillmentsOrdersPointsRechargeFulfillmentsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.fulfillments = new FulfillmentsOrdersPointsRechargeFulfillmentsApi(client);
  }

}

export class FulfillmentsOrdersApi {
  private client: HttpClient;
  public readonly pointsRecharge: FulfillmentsOrdersPointsRechargeApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.pointsRecharge = new FulfillmentsOrdersPointsRechargeApi(client);
  }

}

export class FulfillmentsApi {
  private client: HttpClient;
  public readonly orders: FulfillmentsOrdersApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.orders = new FulfillmentsOrdersApi(client);
  }

}

export function createFulfillmentsApi(client: HttpClient): FulfillmentsApi {
  return new FulfillmentsApi(client);
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

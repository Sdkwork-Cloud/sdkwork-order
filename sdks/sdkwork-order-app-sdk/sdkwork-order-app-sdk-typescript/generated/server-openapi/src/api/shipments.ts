import { appApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { CommerceApiResult } from '../types';


export class ShipmentsTrackingEventsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Shipments tracking Events list. */
  async list(shipmentId: string): Promise<CommerceApiResult> {
    return this.client.get<CommerceApiResult>(appApiPath(`/shipments/${serializePathParameter(shipmentId, { name: 'shipmentId', style: 'simple', explode: false })}/tracking_events`));
  }
}

export class ShipmentsPackagesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Shipments packages list. */
  async list(shipmentId: string): Promise<CommerceApiResult> {
    return this.client.get<CommerceApiResult>(appApiPath(`/shipments/${serializePathParameter(shipmentId, { name: 'shipmentId', style: 'simple', explode: false })}/packages`));
  }
}

export class ShipmentsApi {
  private client: HttpClient;
  public readonly packages: ShipmentsPackagesApi;
  public readonly trackingEvents: ShipmentsTrackingEventsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.packages = new ShipmentsPackagesApi(client);
    this.trackingEvents = new ShipmentsTrackingEventsApi(client);
  }


/** Shipments retrieve. */
  async retrieve(shipmentId: string): Promise<CommerceApiResult> {
    return this.client.get<CommerceApiResult>(appApiPath(`/shipments/${serializePathParameter(shipmentId, { name: 'shipmentId', style: 'simple', explode: false })}`));
  }
}

export function createShipmentsApi(client: HttpClient): ShipmentsApi {
  return new ShipmentsApi(client);
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

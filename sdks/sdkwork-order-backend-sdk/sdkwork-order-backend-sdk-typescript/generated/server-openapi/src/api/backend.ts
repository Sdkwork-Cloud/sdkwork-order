import { backendApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { AccountValuePackageWriteCommand, AccountValueRequestReviewCommand, SdkWorkCommandData, SdkWorkPageData, TokenBankPlanWriteCommand } from '../types';


export interface BackendWithdrawalRequestsListParams {
  status?: string;
  page?: number;
  pageSize?: number;
}

export interface BackendWithdrawalRequestsApproveParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
  xIdempotencyFingerprint: string;
}

export interface BackendWithdrawalRequestsRejectParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
  xIdempotencyFingerprint: string;
}

export interface BackendWithdrawalRequestsRetryParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
  xIdempotencyFingerprint: string;
}

export class BackendWithdrawalRequestsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Withdrawal requests list. */
  async list(params?: BackendWithdrawalRequestsListParams): Promise<SdkWorkPageData> {
    const query = buildQueryString([
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData>(appendQueryString(backendApiPath(`/withdrawal_requests`), query));
  }

/** Withdrawal requests approve. */
  async approve(withdrawalRequestId: string, params: BackendWithdrawalRequestsApproveParams, body?: AccountValueRequestReviewCommand): Promise<SdkWorkCommandData> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
        'X-Idempotency-Fingerprint': { value: params.xIdempotencyFingerprint, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<SdkWorkCommandData>(backendApiPath(`/withdrawal_requests/${serializePathParameter(withdrawalRequestId, { name: 'withdrawalRequestId', style: 'simple', explode: false })}/approve`), body, undefined, requestHeaders, 'application/json');
  }

/** Withdrawal requests reject. */
  async reject(withdrawalRequestId: string, params: BackendWithdrawalRequestsRejectParams, body?: AccountValueRequestReviewCommand): Promise<SdkWorkCommandData> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
        'X-Idempotency-Fingerprint': { value: params.xIdempotencyFingerprint, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<SdkWorkCommandData>(backendApiPath(`/withdrawal_requests/${serializePathParameter(withdrawalRequestId, { name: 'withdrawalRequestId', style: 'simple', explode: false })}/reject`), body, undefined, requestHeaders, 'application/json');
  }

/** Withdrawal requests retry. */
  async retry(withdrawalRequestId: string, params: BackendWithdrawalRequestsRetryParams, body?: AccountValueRequestReviewCommand): Promise<SdkWorkCommandData> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
        'X-Idempotency-Fingerprint': { value: params.xIdempotencyFingerprint, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<SdkWorkCommandData>(backendApiPath(`/withdrawal_requests/${serializePathParameter(withdrawalRequestId, { name: 'withdrawalRequestId', style: 'simple', explode: false })}/retry`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface BackendRefundRequestsListParams {
  status?: string;
  page?: number;
  pageSize?: number;
}

export interface BackendRefundRequestsApproveParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
  xIdempotencyFingerprint: string;
}

export interface BackendRefundRequestsRejectParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
  xIdempotencyFingerprint: string;
}

export interface BackendRefundRequestsRetryParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
  xIdempotencyFingerprint: string;
}

export class BackendRefundRequestsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Refund requests list. */
  async list(params?: BackendRefundRequestsListParams): Promise<SdkWorkPageData> {
    const query = buildQueryString([
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData>(appendQueryString(backendApiPath(`/refund_requests`), query));
  }

/** Refund requests approve. */
  async approve(refundRequestId: string, params: BackendRefundRequestsApproveParams, body?: AccountValueRequestReviewCommand): Promise<SdkWorkCommandData> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
        'X-Idempotency-Fingerprint': { value: params.xIdempotencyFingerprint, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<SdkWorkCommandData>(backendApiPath(`/refund_requests/${serializePathParameter(refundRequestId, { name: 'refundRequestId', style: 'simple', explode: false })}/approve`), body, undefined, requestHeaders, 'application/json');
  }

/** Refund requests reject. */
  async reject(refundRequestId: string, params: BackendRefundRequestsRejectParams, body?: AccountValueRequestReviewCommand): Promise<SdkWorkCommandData> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
        'X-Idempotency-Fingerprint': { value: params.xIdempotencyFingerprint, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<SdkWorkCommandData>(backendApiPath(`/refund_requests/${serializePathParameter(refundRequestId, { name: 'refundRequestId', style: 'simple', explode: false })}/reject`), body, undefined, requestHeaders, 'application/json');
  }

/** Refund requests retry. */
  async retry(refundRequestId: string, params: BackendRefundRequestsRetryParams, body?: AccountValueRequestReviewCommand): Promise<SdkWorkCommandData> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
        'X-Idempotency-Fingerprint': { value: params.xIdempotencyFingerprint, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<SdkWorkCommandData>(backendApiPath(`/refund_requests/${serializePathParameter(refundRequestId, { name: 'refundRequestId', style: 'simple', explode: false })}/retry`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface BackendTokenBankPlansListParams {
  status?: string;
  page?: number;
  pageSize?: number;
}

export interface BackendTokenBankPlansCreateParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
  xIdempotencyFingerprint: string;
}

export interface BackendTokenBankPlansUpdateParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
  xIdempotencyFingerprint: string;
}

export interface BackendTokenBankPlansRetireParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
  xIdempotencyFingerprint: string;
}

export class BackendTokenBankPlansApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Token Bank plans list. */
  async list(params?: BackendTokenBankPlansListParams): Promise<SdkWorkPageData> {
    const query = buildQueryString([
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData>(appendQueryString(backendApiPath(`/token_bank_plans`), query));
  }

/** Token Bank plans create. */
  async create(body: TokenBankPlanWriteCommand, params: BackendTokenBankPlansCreateParams): Promise<Record<string, unknown>> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
        'X-Idempotency-Fingerprint': { value: params.xIdempotencyFingerprint, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<Record<string, unknown>>(backendApiPath(`/token_bank_plans`), body, undefined, requestHeaders, 'application/json');
  }

/** Token Bank plans update. */
  async update(planCode: string, body: TokenBankPlanWriteCommand, params: BackendTokenBankPlansUpdateParams): Promise<Record<string, unknown>> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
        'X-Idempotency-Fingerprint': { value: params.xIdempotencyFingerprint, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.patch<Record<string, unknown>>(backendApiPath(`/token_bank_plans/${serializePathParameter(planCode, { name: 'planCode', style: 'simple', explode: false })}`), body, undefined, requestHeaders, 'application/json');
  }

/** Token Bank plans retire. */
  async retire(planCode: string, params: BackendTokenBankPlansRetireParams, body?: unknown): Promise<SdkWorkCommandData> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
        'X-Idempotency-Fingerprint': { value: params.xIdempotencyFingerprint, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<SdkWorkCommandData>(backendApiPath(`/token_bank_plans/${serializePathParameter(planCode, { name: 'planCode', style: 'simple', explode: false })}/retire`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface BackendAccountValuePackagesListParams {
  targetAsset?: string;
  status?: string;
  page?: number;
  pageSize?: number;
}

export interface BackendAccountValuePackagesCreateParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
  xIdempotencyFingerprint: string;
}

export interface BackendAccountValuePackagesUpdateParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
  xIdempotencyFingerprint: string;
}

export interface BackendAccountValuePackagesRetireParams {
  idempotencyKey: string;
  sdkworkRequestHash: string;
  xIdempotencyFingerprint: string;
}

export class BackendAccountValuePackagesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Account value packages list. */
  async list(params?: BackendAccountValuePackagesListParams): Promise<SdkWorkPageData> {
    const query = buildQueryString([
      { name: 'target_asset', value: params?.targetAsset, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData>(appendQueryString(backendApiPath(`/account_value_packages`), query));
  }

/** Account value packages create. */
  async create(body: AccountValuePackageWriteCommand, params: BackendAccountValuePackagesCreateParams): Promise<Record<string, unknown>> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
        'X-Idempotency-Fingerprint': { value: params.xIdempotencyFingerprint, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<Record<string, unknown>>(backendApiPath(`/account_value_packages`), body, undefined, requestHeaders, 'application/json');
  }

/** Account value packages update. */
  async update(packageId: string, body: AccountValuePackageWriteCommand, params: BackendAccountValuePackagesUpdateParams): Promise<Record<string, unknown>> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
        'X-Idempotency-Fingerprint': { value: params.xIdempotencyFingerprint, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.patch<Record<string, unknown>>(backendApiPath(`/account_value_packages/${serializePathParameter(packageId, { name: 'packageId', style: 'simple', explode: false })}`), body, undefined, requestHeaders, 'application/json');
  }

/** Account value packages retire. */
  async retire(packageId: string, params: BackendAccountValuePackagesRetireParams, body?: unknown): Promise<SdkWorkCommandData> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
        'Sdkwork-Request-Hash': { value: params.sdkworkRequestHash, style: 'simple', explode: false },
        'X-Idempotency-Fingerprint': { value: params.xIdempotencyFingerprint, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<SdkWorkCommandData>(backendApiPath(`/account_value_packages/${serializePathParameter(packageId, { name: 'packageId', style: 'simple', explode: false })}/retire`), body, undefined, requestHeaders, 'application/json');
  }
}

export class BackendApi {
  private client: HttpClient;
  public readonly accountValuePackages: BackendAccountValuePackagesApi;
  public readonly tokenBankPlans: BackendTokenBankPlansApi;
  public readonly refundRequests: BackendRefundRequestsApi;
  public readonly withdrawalRequests: BackendWithdrawalRequestsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.accountValuePackages = new BackendAccountValuePackagesApi(client);
    this.tokenBankPlans = new BackendTokenBankPlansApi(client);
    this.refundRequests = new BackendRefundRequestsApi(client);
    this.withdrawalRequests = new BackendWithdrawalRequestsApi(client);
  }

}

export function createBackendApi(client: HttpClient): BackendApi {
  return new BackendApi(client);
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
interface QueryParameterSpec {
  name: string;
  value: unknown;
  style: string;
  explode: boolean;
  allowReserved: boolean;
  contentType?: string;
}

function buildQueryString(parameters: QueryParameterSpec[]): string {
  const pairs: string[] = [];
  for (const parameter of parameters) {
    appendSerializedParameter(pairs, parameter);
  }
  return pairs.join('&');
}

function appendSerializedParameter(pairs: string[], parameter: QueryParameterSpec): void {
  if (parameter.value === undefined || parameter.value === null) {
    return;
  }

  if (parameter.contentType) {
    pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(JSON.stringify(parameter.value), parameter.allowReserved)}`);
    return;
  }

  const style = parameter.style || 'form';
  if (style === 'deepObject') {
    appendDeepObjectParameter(pairs, parameter.name, parameter.value, parameter.allowReserved);
    return;
  }

  if (Array.isArray(parameter.value)) {
    appendArrayParameter(pairs, parameter.name, parameter.value, style, parameter.explode, parameter.allowReserved);
    return;
  }

  if (typeof parameter.value === 'object') {
    appendObjectParameter(pairs, parameter.name, parameter.value as Record<string, unknown>, style, parameter.explode, parameter.allowReserved);
    return;
  }

  pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(serializePrimitive(parameter.value), parameter.allowReserved)}`);
}

function appendArrayParameter(
  pairs: string[],
  name: string,
  value: unknown[],
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const values = value
    .filter((item) => item !== undefined && item !== null)
    .map((item) => serializePrimitive(item));
  if (values.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const item of values) {
      pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(item, allowReserved)}`);
    }
    return;
  }

  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(values.join(','), allowReserved)}`);
}

function appendObjectParameter(
  pairs: string[],
  name: string,
  value: Record<string, unknown>,
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const [key, entryValue] of entries) {
      pairs.push(`${encodeQueryComponent(key)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
    }
    return;
  }

  const serialized = entries.flatMap(([key, entryValue]) => [key, serializePrimitive(entryValue)]).join(',');
  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serialized, allowReserved)}`);
}

function appendDeepObjectParameter(
  pairs: string[],
  name: string,
  value: unknown,
  allowReserved: boolean,
): void {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serializePrimitive(value), allowReserved)}`);
    return;
  }

  for (const [key, entryValue] of Object.entries(value as Record<string, unknown>)) {
    if (entryValue === undefined || entryValue === null) {
      continue;
    }
    pairs.push(`${encodeQueryComponent(`${name}[${key}]`)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
  }
}

function serializePrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}

function encodeQueryComponent(value: string): string {
  return encodeURIComponent(value);
}

function encodeQueryValue(value: string, allowReserved: boolean): string {
  const encoded = encodeURIComponent(value);
  if (!allowReserved) {
    return encoded;
  }
  return encoded.replace(/%3A/gi, ':')
    .replace(/%2F/gi, '/')
    .replace(/%3F/gi, '?')
    .replace(/%23/gi, '#')
    .replace(/%5B/gi, '[')
    .replace(/%5D/gi, ']')
    .replace(/%40/gi, '@')
    .replace(/%21/gi, '!')
    .replace(/%24/gi, '$')
    .replace(/%26/gi, '&')
    .replace(/%27/gi, "'")
    .replace(/%28/gi, '(')
    .replace(/%29/gi, ')')
    .replace(/%2A/gi, '*')
    .replace(/%2B/gi, '+')
    .replace(/%2C/gi, ',')
    .replace(/%3B/gi, ';')
    .replace(/%3D/gi, '=');
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

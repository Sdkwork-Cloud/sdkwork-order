import { createHash, webcrypto } from 'node:crypto';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { createClient, SdkworkAppClient } from '@sdkwork/order-app-sdk';

const originalFetch = globalThis.fetch;
const originalCrypto = globalThis.crypto;

beforeEach(() => {
  Object.defineProperty(globalThis, 'crypto', {
    configurable: true,
    value: webcrypto,
  });
});

afterEach(() => {
  globalThis.fetch = originalFetch;
  Object.defineProperty(globalThis, 'crypto', {
    configurable: true,
    value: originalCrypto,
  });
  vi.restoreAllMocks();
});

describe('@sdkwork/order-app-sdk idempotent request fingerprints', () => {
  it('hashes the exact membership order request body sent over the wire', async () => {
    const requests: Array<{ input: RequestInfo | URL; init?: RequestInit }> = [];
    globalThis.fetch = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      requests.push({ input, init });
      return membershipOrderResponse();
    });

    const client = new SdkworkAppClient({
      accessToken: 'access-token',
      authToken: 'auth-token',
      baseUrl: 'https://orders.example.test',
    });
    const body = {
      action: 'purchase',
      packageId: '58',
      paymentMethod: 'wechat_pay',
      paymentProduct: 'mobile_cashier_h5',
    } as const;

    await client.memberships.orders.create(body, { idempotencyKey: 'membership-purchase-58' });

    expect(requests).toHaveLength(1);
    const request = requests[0];
    const serializedBody = JSON.stringify(body);
    const headers = new Headers(request.init?.headers);
    expect(request.input).toBe('https://orders.example.test/app/v3/api/memberships/orders');
    expect(request.init?.body).toBe(serializedBody);
    expect(headers.get('Idempotency-Key')).toBe('membership-purchase-58');
    expect(headers.get('X-Content-SHA256')).toBe(sha256Hex(serializedBody));
    expect(headers.get('X-Idempotency-Fingerprint')).toBeNull();
  });

  it('preserves an explicitly supplied idempotency fingerprint', async () => {
    let capturedHeaders: Headers | undefined;
    globalThis.fetch = vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) => {
      capturedHeaders = new Headers(init?.headers);
      return membershipOrderResponse();
    });

    const client = createClient({
      accessToken: 'access-token',
      authToken: 'auth-token',
      baseUrl: 'https://orders.example.test',
    });
    await client.http.post(
      '/app/v3/api/memberships/orders',
      { packageId: '58' },
      undefined,
      {
        'Idempotency-Key': 'membership-purchase-58',
        'x-idempotency-fingerprint': 'caller-supplied-fingerprint',
      },
      'application/json',
    );

    expect(capturedHeaders?.get('X-Idempotency-Fingerprint')).toBe('caller-supplied-fingerprint');
    expect(capturedHeaders?.get('X-Content-SHA256')).toBeNull();
  });
});

function membershipOrderResponse(): Response {
  return new Response(
    JSON.stringify({
      code: 0,
      data: {
        item: {
          amount: '58',
          orderId: 'membership-order-58',
          packageId: '58',
          status: 'pending',
        },
      },
      traceId: '00000000-0000-4000-8000-000000000001',
    }),
    {
      status: 201,
      headers: { 'content-type': 'application/json' },
    },
  );
}

function sha256Hex(value: string): string {
  return createHash('sha256').update(value).digest('hex');
}

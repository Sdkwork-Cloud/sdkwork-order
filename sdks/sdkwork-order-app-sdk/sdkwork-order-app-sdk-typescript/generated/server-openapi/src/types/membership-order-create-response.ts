import type { MembershipOrderCreateResult } from './membership-order-create-result';

export interface MembershipOrderCreateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}

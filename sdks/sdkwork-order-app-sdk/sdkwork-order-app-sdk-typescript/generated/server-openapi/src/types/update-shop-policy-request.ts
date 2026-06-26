export interface UpdateShopPolicyRequest {
  policyStatus?: string;
  policyVersion?: number;
  policy?: Record<string, unknown>;
  publishedAt?: string;
  reviewComment?: string;
}

export interface UpsertShopQualificationRequest {
  qualificationType: string;
  qualificationStatus: string;
  subjectType: string;
  subjectId: string;
  credentialName?: string;
  credentialHash?: string;
  credentialMediaResourceId?: string;
  qualificationSnapshot: Record<string, unknown>;
  issuedAt?: string;
  expiresAt?: string;
}

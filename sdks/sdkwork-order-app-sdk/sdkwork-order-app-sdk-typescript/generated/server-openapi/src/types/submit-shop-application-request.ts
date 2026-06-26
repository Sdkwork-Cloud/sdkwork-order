export interface SubmitShopApplicationRequest {
  applicationType: string;
  legalEntitySnapshot: Record<string, unknown>;
  contactSnapshot: Record<string, unknown>;
  qualificationSnapshot: Record<string, unknown>;
}

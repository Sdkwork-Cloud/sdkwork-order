export interface CreatePointsRechargeFulfillmentRequest {
  requestNo: string;
  idempotencyKey?: string;
  paidAt?: string;
  ownerUserId?: string;
}

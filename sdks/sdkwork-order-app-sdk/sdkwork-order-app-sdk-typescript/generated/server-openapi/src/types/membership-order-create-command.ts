export interface MembershipOrderCreateCommand {
  packageId: string;
  paymentMethod: string;
  clientRequestNo?: string;
  source?: string;
}

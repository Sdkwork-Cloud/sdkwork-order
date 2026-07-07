export interface ShipmentPackageSummary {
  packageId: string;
  shipmentId: string;
  packageNo: string;
  packageType: string;
  trackingNo?: string;
  status: string;
}

export interface ShippingAddressRequest {
  receiverName: string;
  receiverPhone: string;
  countryCode: string;
  province: string;
  city: string;
  district?: string;
  detailAddress: string;
  postalCode?: string;
}

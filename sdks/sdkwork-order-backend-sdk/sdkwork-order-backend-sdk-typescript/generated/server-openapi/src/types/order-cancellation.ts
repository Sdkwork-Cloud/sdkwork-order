export interface OrderCancellation {
  id: string;
  orderId: string;
  status: string;
  reasonCode: string;
  reasonMessage?: string;
  createdAt: string;
}

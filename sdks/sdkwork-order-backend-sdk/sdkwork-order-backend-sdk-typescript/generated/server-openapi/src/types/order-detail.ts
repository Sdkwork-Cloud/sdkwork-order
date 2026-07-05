import type { OrderItem } from './order-item';

export interface OrderDetail {
  orderId: string;
  orderSn: string;
  status: string;
  statusName: string;
  subject: string;
  totalAmount: string;
  paidAmount?: string;
  discountAmount?: string;
  quantity: string;
  createdAt: string;
  payTime?: string;
  expireTime?: string;
  paymentMethod?: string;
  items: OrderItem[];
  outTradeNo?: string;
  transactionId?: string;
}

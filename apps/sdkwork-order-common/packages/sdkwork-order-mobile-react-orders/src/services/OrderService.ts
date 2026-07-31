export interface Order {
  id: string;
  shopName: string;
  isVirtual?: boolean;
  status:
    | "pending_payment"
    | "to_ship"
    | "to_receive"
    | "to_review"
    | "completed"
    | "cancelled"
    | "refunded";
  statusText: string;
  items: Array<{
    id: string;
    image: string;
    title: string;
    specs: string;
    price: number;
    originalPrice?: number;
    quantity: number;
    virtualType?: "coupon" | "service" | "game_currency" | "group_chat";
    voucherCodes?: Array<{ code: string; status: "unused" | "used" }>;
  }>;
  totalAmount: number;
  shippingFee: number;
  address?: {
    name: string;
    phone: string;
    detail: string;
  };
  createTime: string;
  payTime?: string;
  shipTime?: string;
}

export class OrderCapabilityUnavailableError extends Error {
  constructor() {
    super("Orders are unavailable because the Order owner SDK and payment flow are not composed.");
    this.name = "OrderCapabilityUnavailableError";
  }
}

export class OrderService {
  static async getOrders(): Promise<Order[]> {
    throw new OrderCapabilityUnavailableError();
  }

  static async getOrderTabs(): Promise<Array<{ id: string; label: string }>> {
    throw new OrderCapabilityUnavailableError();
  }

  static async getOrderById(_id: string): Promise<Order | undefined> {
    throw new OrderCapabilityUnavailableError();
  }

  static async payOrder(_id: string): Promise<void> {
    throw new OrderCapabilityUnavailableError();
  }

  static async redeemVoucher(
    _code: string,
  ): Promise<{ success: boolean; message: string; order?: Order }> {
    throw new OrderCapabilityUnavailableError();
  }

  static async cancelOrder(_id: string): Promise<void> {
    throw new OrderCapabilityUnavailableError();
  }

  static async confirmReceipt(_id: string): Promise<void> {
    throw new OrderCapabilityUnavailableError();
  }

  static async reviewOrder(_id: string): Promise<void> {
    throw new OrderCapabilityUnavailableError();
  }

  static async remindShipping(_id: string): Promise<void> {
    throw new OrderCapabilityUnavailableError();
  }

  static async applyRefund(_id: string): Promise<void> {
    throw new OrderCapabilityUnavailableError();
  }

  static async modifyAddress(_id: string): Promise<void> {
    throw new OrderCapabilityUnavailableError();
  }

  static async deleteOrder(_id: string): Promise<void> {
    throw new OrderCapabilityUnavailableError();
  }

  static async createOrder(
    _order: Omit<Order, "id" | "createTime" | "status" | "statusText">,
  ): Promise<Order> {
    throw new OrderCapabilityUnavailableError();
  }
}

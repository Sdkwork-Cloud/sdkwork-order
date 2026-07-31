import assert from "node:assert/strict";
import test from "node:test";

import { OrderCapabilityUnavailableError, OrderService } from "./OrderService";

test("order and payment operations fail closed", async () => {
  const order = {
    items: [],
    shippingFee: 0,
    shopName: "Shop",
    totalAmount: 0,
  };
  for (const operation of [
    () => OrderService.getOrders(),
    () => OrderService.getOrderTabs(),
    () => OrderService.getOrderById("order-id"),
    () => OrderService.payOrder("order-id"),
    () => OrderService.redeemVoucher("voucher-code"),
    () => OrderService.cancelOrder("order-id"),
    () => OrderService.confirmReceipt("order-id"),
    () => OrderService.reviewOrder("order-id"),
    () => OrderService.remindShipping("order-id"),
    () => OrderService.applyRefund("order-id"),
    () => OrderService.modifyAddress("order-id"),
    () => OrderService.deleteOrder("order-id"),
    () => OrderService.createOrder(order),
  ]) {
    await assert.rejects(operation, OrderCapabilityUnavailableError);
  }
});

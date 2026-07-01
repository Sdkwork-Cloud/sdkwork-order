# Order Recharge & Unified Order Spec

Status: active  
Owner: SDKWork maintainers  
Capability: `commerce.order`  
Updated: 2026-06-29

Authority: `sdkwork-specs/RPC_SPEC.md` (`OrderService`, `RechargeService`), `sdkwork-specs/API_SPEC.md`

## 1. Purpose

**sdkwork-order** owns all `commerce_order` headers, including recharge (`subject=points_recharge`), for a single order center: list, detail, cancel, refund eligibility, and pay orchestration.

## 2. Owned domain

| Asset | Owner |
| --- | --- |
| `commerce_order` | Order (all subjects) |
| `commerce_order_item` | Order |
| `commerce_order_amount_breakdown` | Order |
| `commerce_recharge_package` | Order (target); migrate from Payment |
| Checkout session (product) | Order |
| Fulfillment / shipment (physical) | Order |

### Order subjects (extensible)

| `subject` | Scenario | Fulfillment |
| --- | --- | --- |
| `product` | Physical / general goods | Shipment |
| `virtual_goods` | Virtual SKU | Account / entitlement |
| `points_recharge` | Wallet top-up | Account `adjustments/points` |
| `membership` | Member purchase | Membership service |

## 3. Recharge API (app-api)

OperationIds per `RPC_SPEC.md` — routes owned by **order** repository:

| Operation | operationId | Notes |
| --- | --- | --- |
| List packages | `recharges.packages.list` | `GET .../recharges/packages` |
| Settings | `recharges.settings.retrieve` | Exchange preview |
| Create recharge order | `recharges.orders.create` | Writes `commerce_order`, `subject=points_recharge` |
| Retrieve recharge order | `recharges.orders.retrieve` | Same as order detail |
| List recharge orders | `recharges.orders.list` | Filter on unified `orders.list` |
| Cancel | `recharges.orders.cancel` / `orders.cancel` | |
| Pay | `orders.pay` | Orchestrates Payment intent for `orderId` |

Backend admin: `RechargeAdminService` (package publish) on order-backend-api.

## 4. Pay & fulfill flow

```text
1. recharges.orders.create  → commerce_order (pending_payment)
2. orders.pay(orderId)      → Payment creates intent (Payment repo)
3. Payment webhook success  → Order marks payment_status succeeded
4. Order saga               → POST account backend adjustments (idempotent)
5. Order                    → fulfillment_status = fulfilled (virtual)
```

Saga entrypoints (order-service):

- `mark_points_recharge_payment_succeeded` — payment webhook boundary (Payment → Order)
- `fulfill_points_recharge_order` — credits account via `AccountPointsCreditPort`, then commits order fulfillment

Idempotency keys:

- Payment success: `points-recharge:payment-success:{orderId}`
- Fulfillment credit: `points-recharge:fulfill:{orderId}`

Account ledger `business_type`: `points_recharge`

Order **must not** call payment provider SDK directly; use Payment service/repository boundary.

## 5. Dependencies

| Direction | Target | Allowed |
| --- | --- | --- |
| Order → Payment | `orders.pay`, payment store port | Yes |
| Order → Account | backend-api adjustments | Yes (saga) |
| Payment → Order | read/update order state | Yes |
| Order → Account tables | direct SQL | **No** |

## 6. SDK

| Artifact | Content |
| --- | --- |
| `@sdkwork/order-sdk-ports` | Extend `APP_ORDER_METHOD_TREE` with `recharges` |
| `@sdkwork/order-app-sdk` | Generated `recharges.*` + existing `orders.*` |
| `@sdkwork/order-service` | Facade; no account methods |

## 7. Create & pay boundary (O5 complete)

`recharges.orders.create` writes **order domain only** (`commerce_order`, items, amount breakdown). Payment intent/attempt is created by **`orders.pay`** via `PayOwnerOrderCommand` (payment repository). Account `commerce_billing_history` is **not** written at create; ledger credit runs in the fulfillment saga (`AccountPointsCreditPort`).

Points grant metadata is stored on `commerce_order_item.sku_snapshot_json` and copied into payment attempt `callback_payload` when pay orchestrates.

Legacy payment-local recharge SQL and routes were removed (P3); deprecated clients use `recharge_proxy_router` → sdkwork-order.

## 10. Gateway closure (O6 complete)

| Component | Route / env | Role |
| --- | --- | --- |
| Order backend saga | `POST .../orders/{orderId}/points-recharge/fulfillments` | Marks payment success (optional `paidAt`) + fulfills via `AccountPointsCreditPort` |
| Payment confirmation | `POST .../payments/owner-orders/{orderId}/confirmations` | Marks payment attempt succeeded, then calls order saga (Payment → Order only) |
| Account credit (HTTP) | `SDKWORK_ACCOUNT_BACKEND_API_ORIGIN`, `SDKWORK_ORDER_ACCOUNT_SERVICE_AUTH_TOKEN` | Default adapter: `POST .../wallet/adjustments/points` |
| Account credit (store) | `SDKWORK_ORDER_ACCOUNT_LEDGER_ADAPTER=store` | In-process ledger via ACCOUNT database pool (monolith / shared DB) |

Permissions: `commerce.orders.fulfill` (order backend), `commerce.payments.confirm` (payment backend).

Recharge **app-api** responses use `SdkWorkApiResponse` (`code: 0`, `data.item` / `data.items` + `pageInfo`, `traceId`). Errors use `ProblemDetail` with numeric `code`.

Track phases in [commerce-recharge.spec.json](./commerce-recharge.spec.json).

## 8. PC

| Package | Role |
| --- | --- |
| `@sdkwork/order-pc-order` | Unified order list, order detail, admin |
| `@sdkwork/account-pc-wallet` | May consume order SDK for recharge UI (composition) |

## 9. Verification

- Order integration tests: create recharge order → pay → mock account adjustment
- OpenAPI parity for `recharges` tag paths
- `pnpm sdk:generate` for order-app-sdk

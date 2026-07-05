# Order Recharge & Unified Order Spec

Status: active  
Owner: SDKWork maintainers  
Capability: `commerce.order`  
Updated: 2026-07-05

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
2. orders.pay(orderId)      → Payment creates intent + attempt (cashierUrl in paymentParams)
3. PSP webhook              → Payment ingests event, marks attempt succeeded
4. Payment settlement       → settle_owner_order_after_payment_success (webhook or backend confirm)
5. Order saga               → POST .../points_recharge/fulfillments → account ledger credit
6. Order                    → fulfillment_status = fulfilled (virtual)
```

Production path: **webhook ingestion** (`POST /app/v3/api/payments/webhooks/{provider}`) triggers settlement automatically when payment status maps to `succeeded`. **Manual replay** uses `POST /backend/v3/api/payments/owner-orders/{orderId}/confirmations` (same settlement saga).

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

Legacy payment-local recharge SQL and routes were removed (P3). Deprecated `/app/v3/api/recharges/*` proxy in **sdkwork-payment** is **opt-in only** (`SDKWORK_PAYMENT_ENABLE_RECHARGE_PROXY=1`); new clients must call order app-api directly.

## 10. Gateway closure (O6 complete)

| Component | Route / env | Role |
| --- | --- | --- |
| Order backend saga | `POST .../orders/{orderId}/points_recharge/fulfillments` | Marks payment success (optional `paidAt`) + fulfills via `AccountPointsCreditPort` |
| Payment confirmation | `POST .../payments/owner-orders/{orderId}/confirmations` | Marks payment attempt succeeded, then calls order saga (Payment → Order only) |
| Account credit (HTTP) | `SDKWORK_ACCOUNT_BACKEND_API_ORIGIN`, `SDKWORK_ORDER_ACCOUNT_SERVICE_AUTH_TOKEN` (required unless `SDKWORK_ORDER_ACCOUNT_CREDIT_ALLOW_INSECURE=1` for local dev) | Default adapter: `POST .../wallet/adjustments/points` |
| Account credit (store) | `SDKWORK_ORDER_ACCOUNT_LEDGER_ADAPTER=store` | In-process ledger via shared ACCOUNT database pool |

Permissions: `commerce.orders.fulfill` (order backend), `commerce.payments.confirm` (payment backend).

Recharge **app-api** responses use `SdkWorkApiResponse` (`code: 0`, `data.item` / `data.items` + `pageInfo`, `traceId`). Errors use `ProblemDetail` with numeric `code`.

All order **app-api** routers (`orders`, `recharges`, `checkout`, `fulfillments`, `shipments`, `after_sales`) and order **backend** admin routes use the same v3 envelope via `sdkwork-utils-rust` (`SdkWorkApiResponse`, `SdkWorkProblemDetail`). Legacy `CommerceApiResult` / string wire codes are removed from handlers.

OpenAPI authority and generated `@sdkwork/order-app-sdk` use `SdkWorkApiResponse` (`pnpm align:openapi` + `pnpm sdk:generate`). Legacy `CommerceApiResult` and `requestId` are removed from the contract.

Track phases in [commerce-recharge.spec.json](./commerce-recharge.spec.json).

## 8. PC

| Package | Role |
| --- | --- |
| `@sdkwork/order-pc-order` | Unified order list, order detail, admin |
| `@sdkwork/account-pc-wallet` | May consume order SDK for recharge UI (composition) |

## 9. Verification

- **Store E2E** (`cargo test -p sdkwork-order-integration-account points_recharge_store_e2e`): seeded recharge checkout → `mark_points_recharge_payment_succeeded` → `fulfill_points_recharge_order` with `StoreAccountPointsCreditAdapter` → wallet points balance + idempotent replay
- **Cancel audit** (`cargo test -p sdkwork-order-repository-sqlx sqlite_cancel_owner_order`): SQLite parity; optional Postgres via `ORDER_TEST_POSTGRES_URL`
- Order service unit tests: fulfillment saga idempotency keys and mock port orchestration (`points_recharge_fulfillment_standard.rs`)
- Account saga contract: ledger adjustment command shape (`points_recharge_saga_contract.rs`)
- OpenAPI parity for `recharges` tag paths; `pnpm align:openapi` + `pnpm sdk:generate` for order-app-sdk
- `pnpm verify` and `cargo test --workspace` in order and account repositories

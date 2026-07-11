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
| Create payment | `orders.payments.create` | Orchestrates Payment intent for `orderId` |

Backend admin: `RechargeAdminService` (package publish) on order-backend-api.

## 4. Pay & fulfill flow

```text
1. recharges.orders.create  → commerce_order (pending_payment)
2. orders.payments.create(orderId)      → Payment creates intent + attempt (cashierUrl in paymentParams)
3. PSP webhook              → Order app-api POST .../orders/payments/webhooks/{provider}
4. Order → Payment port     → ingest webhook, mark attempt succeeded (in-process)
5. Order settlement         → settle_owner_order_after_payment_success (in-process saga)
6. Order                    → fulfillment_status = fulfilled; account ledger credit
```

Production path: **webhook ingestion** on the **order gateway** (`POST /app/v3/api/orders/payments/webhooks/{providerCode}`) verifies the PSP signature, calls the payment repository port, then runs settlement in-process when status maps to `succeeded`. **Manual replay** uses `POST /backend/v3/api/orders/{orderId}/payment_confirmations` (permission `commerce.orders.fulfill`).

Legacy `POST /app/v3/api/payments/webhooks/{providerCode}` on the payment gateway returns **410 Gone** with a migration hint.

Saga entrypoints (order-service):

- `settle_owner_order_after_payment_success` — orchestrates confirm + subject fulfillment
- `mark_points_recharge_payment_succeeded` — order-side payment success markers
- `fulfill_points_recharge_order` — credits account via `AccountPointsCreditPort`, then commits order fulfillment

Idempotency keys:

- Payment success: `points-recharge:payment-success:{orderId}`
- Fulfillment credit: `points-recharge:fulfill:{orderId}`

Account ledger `business_type`: `points_recharge`

Order **must not** call payment provider SDK directly; use Payment service/repository boundary.

## 5. Dependencies

| Direction | Target | Allowed |
| --- | --- | --- |
| Order → Payment | `orders.payments.create`, webhook ingest, confirm payment ports | Yes |
| Order → Account | backend-api adjustments | Yes (saga) |
| Payment → Order | HTTP or service dependency | **No** |
| Order → Account tables | direct SQL | **No** |

## 6. SDK

| Artifact | Content |
| --- | --- |
| `@sdkwork/order-sdk-ports` | Extend `APP_ORDER_METHOD_TREE` with `recharges` |
| `@sdkwork/order-app-sdk` | Generated `recharges.*` + existing `orders.*` |
| `@sdkwork/order-service` | Facade; no account methods |

## 7. Create & pay boundary (O5 complete)

`recharges.orders.create` writes **order domain only** (`commerce_order`, items, amount breakdown). Payment intent/attempt is created by **`orders.payments.create`** via `PayOwnerOrderCommand` (payment repository). Account `commerce_billing_history` is **not** written at create; ledger credit runs in the fulfillment saga (`AccountPointsCreditPort`).

Points grant metadata is stored on `commerce_order_item.sku_snapshot_json` and copied into payment attempt `callback_payload` when pay orchestrates.

Legacy payment-local recharge SQL and routes were removed (P3). Deprecated `/app/v3/api/recharges/*` proxy in **sdkwork-payment** is **opt-in only** (`SDKWORK_PAYMENT_ENABLE_RECHARGE_PROXY=1`); new clients must call order app-api directly.

## 10. Gateway closure (O6 complete)

| Component | Route / env | Role |
| --- | --- | --- |
| PSP webhook | `POST .../orders/payments/webhooks/{providerCode}` | Order-owned public route; verify + ingest + settle |
| Manual confirm | `POST .../orders/{orderId}/payment_confirmations` | Operator replay of settlement saga |
| Webhook base URL | `ORDER_PAYMENT_WEBHOOK_BASE_URL` | `{base}/app/v3/api/orders/payments/webhooks/{providerCode}` |
| Account credit (HTTP) | `SDKWORK_ACCOUNT_BACKEND_API_ORIGIN`, `SDKWORK_ACCESS_TOKEN` | Default adapter: `POST .../wallet/adjustments/points` |
| Account credit (store) | `SDKWORK_ORDER_ACCOUNT_LEDGER_ADAPTER=store` | In-process ledger via shared ACCOUNT database pool |

Permissions: `commerce.orders.fulfill` (order backend payment_confirmations).

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

# Order Technical Architecture

Status: active  
Owner: SDKWork maintainers  
Updated: 2026-07-08
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md

## 1. Architecture Overview

`sdkwork-order` is the standalone commerce order capability: domain services, SQL repositories, HTTP routers, standalone gateway, TypeScript SDKs, and the PC client surface.

For account value movement, order is the orchestration layer. It owns commercial evidence and lifecycle state for recharge, Token Bank plans, package recharge, coupon redemption, refund requests, and withdrawal requests. It does not own payment provider execution or account ledger truth.

## 2. Capability Stack

| Layer | Path |
| --- | --- |
| Domain (Rust) | `crates/sdkwork-order-service/` |
| SQL repositories | `crates/sdkwork-order-repository-sqlx/` |
| HTTP routers (app) | `crates/sdkwork-routes-order-app-api/` |
| HTTP routers (backend) | `crates/sdkwork-routes-order-backend-api/` |
| API server | `crates/sdkwork-order-standalone-gateway/` |
| PC client | `apps/sdkwork-order-pc/` |
| Composed service facade | `apps/sdkwork-order-common/packages/sdkwork-order-service/` |
| App SDK | `sdks/sdkwork-order-app-sdk/` |
| Backend SDK | `sdks/sdkwork-order-backend-sdk/` |

## 3. API Surfaces

| Surface | Prefix | Contract |
| --- | --- | --- |
| App API | `/app/v3/api/orders`, `/app/v3/api/recharges`, `/app/v3/api/checkout`, `/app/v3/api/memberships` | `apis/app-api/order/order-app-api.openapi.json` |
| Backend API | `/backend/v3/api/orders` | `apis/backend-api/order/order-backend-api.openapi.json` |
| Backend after-sales | `/backend/v3/api/after_sales/requests` | same authority |
| Backend shipments | `/backend/v3/api/shipments` | same authority |

OpenAPI discovery is served at `/app/v3/api/openapi.json` and `/backend/v3/api/openapi.json`.

All success responses use `SdkWorkApiResponse` (`code: 0`, `data`, `traceId`). List endpoints return `data.items` plus `data.pageInfo` with SQL-level `LIMIT`/`OFFSET` and `COUNT(*) OVER()` totals. Errors use `ProblemDetail` with numeric SDKWork error codes and `traceId`.

Write commands require `Idempotency-Key`, `Sdkwork-Request-Hash`, and `X-Idempotency-Fingerprint` when the OpenAPI operation declares `x-sdkwork-idempotent`. Request hashes are generated from stable operationId-aligned scopes and canonical request bodies.

### Order Creation Entry Points

| Route | Operation | Use case |
| --- | --- | --- |
| `POST /app/v3/api/checkout/sessions/{checkoutSessionId}/orders` | `checkout.sessions.orders.create` | Canonical checkout-bound product order creation after quote |
| `POST /app/v3/api/orders` | `orders.create` | Deprecated transitional order creation |
| `POST /app/v3/api/recharges/orders` | `recharges.orders.create` | Account value order creation, starting with points recharge and extending to Token Bank/package/coupon subjects |
| `POST /app/v3/api/memberships/orders` | `memberships.orders.create` | Membership purchase checkout (`subject=membership`) |

New PC and integrator surfaces must use checkout sessions for product checkout and order app-api resources for account value orders. They must not call payment or account mutation APIs directly for recharge, refund, or withdrawal workflows.

## 4. Account Value Order Architecture

Account value order subjects:

| Subject | Target | Status |
| --- | --- | --- |
| `points_recharge` | Account `points` credit | complete |
| `token_bank_recharge` | Account `token_bank` credit | implemented settlement path |
| `token_bank_plan_purchase` | Token Bank first-cycle grant | implemented settlement path |
| `token_bank_plan_renewal` | Token Bank renewal grant | implemented settlement path |
| `account_recharge_package` | Package target account asset credit | implemented settlement path |
| `coupon_recharge` | Coupon-backed target account asset credit | implemented settlement path |
| `refund_request` | Account reversal hold plus provider refund | implemented review execution |
| `cash_withdrawal` | Account cash hold plus future provider payout | account hold lifecycle implemented; provider payout is fail-closed until payment exposes a concrete payout executor |

Dependency direction:

```text
sdkwork-order -> sdkwork-payment
sdkwork-order -> sdkwork-account
sdkwork-payment -X-> sdkwork-account
sdkwork-payment -X-> sdkwork-order service crates
sdkwork-account -X-> sdkwork-order
sdkwork-account -X-> sdkwork-payment
```

`sdkwork-payment` executes provider payment and refund channels today. Provider payout remains an explicit executor contract boundary; order keeps it fail-closed until payment publishes a concrete payout implementation. Payment must not call account ledger APIs, import account crates, or write account tables.

`sdkwork-account` executes idempotent ledger commands for credit, debit, hold, settlement, release, and reversal. It must not create orders, own packages or plans, execute provider channels, or approve refund/withdrawal business state.

### Account Value Ports

Order service ports:

| Port | Direction | Purpose |
| --- | --- | --- |
| `AccountValueLedgerPort` | order -> account | Credit Token Bank/points, hold cash, settle or release holds, reverse granted value |
| `PaymentRefundExecutorPort` | order -> payment | Execute provider refund for an approved refund request |
| `PaymentPayoutExecutorPort` | order -> payment | Reserved provider payout boundary for approved withdrawal requests; default runtime is fail-closed because payment has no concrete payout executor yet |
| `CouponRedemptionPort` | order -> coupon/promotion | Validate and consume coupon value for `coupon_recharge` |
| `TokenBankPlanOrderStore` | order-owned | Persist plan purchase and renewal commercial snapshots |

### Flow Summary

Paid account value orders:

```text
recharges.orders.create
  -> commerce_order pending_payment
  -> orders.payments.create through payment executor
  -> order-owned PSP webhook settlement
  -> account ledger command
  -> order fulfillment complete
```

Coupon recharge:

```text
coupon validation
  -> commerce_order subject=coupon_recharge
  -> optional mixed-payment orders.payments.create
  -> account target asset credit
```

Refund request:

```text
refund request
  -> account reversal hold
  -> provider refund through payment
  -> account reversal commit or hold release
```

Cash withdrawal:

```text
withdrawal request
  -> account cash hold
  -> default runtime fails closed through NoopPaymentPayoutExecutorPort
  -> current failure path releases the account hold
  -> future provider payout executor success settles the account hold
```

## 5. Database

- Engines: PostgreSQL (production), SQLite (local/tests)
- Table prefix: `commerce_`
- DDL authority: `database/contract/table-registry.json`
- Repository implementations stay behaviorally aligned across engines.
- List/search paths must use SQL-level pagination.

Existing order-owned or order-managed tables include `commerce_order`, `commerce_order_item`, `commerce_order_amount_breakdown`, `commerce_order_event`, `commerce_order_cancellation`, fulfillment, shipment, after-sales, and idempotency tables.

Account value extension tables:

| Table | Purpose |
| --- | --- |
| `commerce_account_value_package` | Recharge package catalog for points, Token Bank, or other account assets |
| `commerce_token_bank_plan` | Token Bank one-time and continuous plan catalog |
| `commerce_order_refund_request` | Refund request workflow and provider refund execution reference |
| `commerce_order_withdrawal_request` | Cash withdrawal workflow, account hold reference, and future provider payout execution reference |

Immutable package and plan facts are copied into `commerce_order_item.sku_snapshot_json` so account ledger rows never become the commercial catalog source of truth.

## 6. Payment Integration

`sdkwork-order` depends on `sdkwork-payment` for owner-order payment execution (`OwnerOrderPaymentStore`), payment webhook persistence, provider abstractions, and provider refund execution. The order host wires a concrete refund executor through `sdkwork-order-integration-payment`; payout remains behind `PaymentPayoutExecutorPort` and fails closed until payment provides a concrete payout executor. The standalone gateway may wire repositories in-process; split deployments use HTTP backend APIs.

Settlement orchestration is owned by order, not payment:

| Step | Owner | Route / function |
| --- | --- | --- |
| PSP webhook | Order app-api | `POST /app/v3/api/orders/payments/webhooks/{providerCode}` |
| In-process settlement | Order service | `settle_owner_order_after_payment_success` |
| Manual replay | Order backend-api | `POST /backend/v3/api/orders/{orderId}/payment_confirmations` |
| Legacy payment webhook | Payment app-api | `410 Gone` migration shim |

Configure PSP notify URL as `{ORDER_PAYMENT_WEBHOOK_BASE_URL}/app/v3/api/orders/payments/webhooks/{providerCode}`.

Duplicate webhook deliveries remain correlated to the exact persisted payment attempt and may re-enter settlement. Payment confirmation, Order state updates, fulfillment, and late-payment audit writes are idempotent, so retries do not duplicate effects. Operators use `payment_confirmations` for recovery; because its public request does not select an attempt, it proceeds only when the order has one unambiguous matching payment attempt.

A successful payment that arrives after an Order is terminal does not reopen or advance the Order lifecycle. Order preserves the terminal status, records `payment_status=success` and the first `paid_at`, and writes one idempotent `payment_succeeded_after_terminal` event.

## 7. Existing Fulfillment

Points recharge fulfillment currently uses a three-phase saga:

```text
fulfillment_status = processing
  -> idempotent Account wallet credit
  -> local fulfilled commit
```

Commit failure triggers compensation debit and reservation release. Membership-subject orders call `MembershipPurchaseFulfillmentPort` after payment confirmation.

Order detail projections cap line items at 500 rows per request (`MAX_ORDER_LINE_ITEMS`) to avoid unbounded memory use. Missing `commerce_*` read-model tables surface as storage errors in production. Local scaffolding may set `ORDER_READ_MODEL_LENIENT=1` to return empty pages when tables are absent; this is not allowed for production.

List/search endpoints reject invalid `page` or `page_size` with HTTP 400 (`ProblemDetail`) instead of silently clamping. Validation is centralized in `sdkwork-utils-rust::validated_offset_list_params` and `sdkwork-order-service::validation::offset_list_params`.

## 8. PC Surface

| Path | Package | SDK |
| --- | --- | --- |
| `/app/order` | `@sdkwork/order-pc-order` | `@sdkwork/order-app-sdk` |
| `/admin/orders` | `@sdkwork/order-pc-admin-orders` | `@sdkwork/order-backend-sdk` |

```text
apps/sdkwork-order-pc/
  packages/sdkwork-order-pc-core/
  packages/sdkwork-order-pc-shell/
  packages/sdkwork-order-pc-order/
  packages/sdkwork-order-pc-admin-orders/
```

Wallet recharge, refund, and withdrawal UI surfaces must delegate to order SDK resources or host navigation ports. They must not call payment or account mutation APIs directly.

## 9. Runtime Configuration

| Variable | Purpose | Default |
| --- | --- | --- |
| `ORDER_API_BIND` | Gateway listen address | `0.0.0.0:18093` |
| `ORDER_CORS_ALLOW_ORIGINS` | Comma-separated browser origins | empty (same-origin only) |
| `SDKWORK_ORDER_PLATFORM_CATALOG_TENANT_ID` | Tenant id for public recharge package catalog fallback | `100001` |
| `SDKWORK_ACCESS_TOKEN` | Bearer token for service-to-service wallet credit and membership fulfillment during order settlement | required in production |
| `ORDER_PAYMENT_WEBHOOK_BASE_URL` | Public base URL registered with PSP for order-owned webhooks | required in production |
| `ORDER_TEST_POSTGRES_URL` | PostgreSQL URL for repository parity tests | unset |
| `RUST_LOG` | Tracing filter (`order.bootstrap`, `order.runtime`, `order.readiness`, `order.security`) | `info` |

## 10. Observability

The standalone gateway mounts `/healthz`, `/livez`, `/readyz`, and `/metrics` via `sdkwork-web-bootstrap::service_router`. Contract fallback merges app-api and backend-api `HttpRouteManifest` entries through `sdkwork-order-gateway-assembly::order_contract_fallback_config`.

Structured tracing uses targets `order.bootstrap`, `order.runtime`, `order.readiness`, and `order.security`. API handlers propagate `traceId` through `SdkWorkApiResponse` and `ProblemDetail`. Readiness probes database connectivity via `SELECT 1`.

## 11. Verification

```powershell
cd E:\sdkwork-space\sdkwork-order
cargo test --workspace
pnpm install
pnpm verify
pnpm test:postgres
pnpm test:postgres:required
```

Before completing API, SDK, pagination, or frontend integration work, run the SDKWork validators from `../sdkwork-specs/tools`.

## 12. Related Docs

- Account value order spec: `specs/ACCOUNT_VALUE_ORDER_SPEC.md`
- Checkout and payment topology: `docs/architecture/commerce/COMMERCE_CHECKOUT_ARCHITECTURE.md`
- Commerce repository dissolution: `../../sdkwork-specs/MIGRATION_SPEC.md` section 8
- Recharge machine contract: `specs/commerce-recharge.spec.json`
- Checkout topology contract: `specs/commerce-checkout-topology.spec.json`
- Product: `docs/product/prd/PRD.md`
- Production operations: `docs/guides/operations/PRODUCTION.md`

# Order Technical Architecture

Status: active  
Owner: SDKWork maintainers  
Updated: 2026-07-06  
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md

## 1. Architecture Overview

`sdkwork-order` is the standalone **commerce order** capability: domain services, SQL repositories, HTTP routers, standalone gateway, TypeScript SDKs, and the PC client surface.

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

All success responses use `SdkWorkApiResponse` (`code: 0`, `data`, `traceId`). List endpoints return `data.items` + `data.pageInfo` with SQL-level `LIMIT`/`OFFSET` and `COUNT(*) OVER()` totals.

Order cancel/close commands write `commerce_order_event` (and `commerce_order_cancellation` for cancels) in the same transaction as the status update. **Payment intents are closed before order state mutation** on app and backend surfaces so terminal orders cannot retain open PSP attempts. Idempotent replays return success when the order is already in the target status.

Write commands (app cancel, backend cancel/close, payment confirmation replay, after-sales review, shipment package create/update, checkout create/quote/order, recharge create/cancel) require `Idempotency-Key` and `Sdkwork-Request-Hash` headers. Most writes use `stable_json_request_hash(operationId, canonicalJsonBody)` including route parameters where applicable. Checkout and checkout-bound order creation use **command digests** keyed by the same `operationId` (`checkout.sessions.create`, `checkout.sessions.quotes.create`, `checkout.sessions.orders.create`) over tenant, organization, owner, session, line, and request-no fields — mirrored in TypeScript via `checkoutSessionRequestHash` and related helpers in `@sdkwork/order-service`. OpenAPI declares both headers on every `x-sdkwork-idempotent` operation.

### Order creation entry points

| Route | Operation | Use case |
| --- | --- | --- |
| `POST /app/v3/api/checkout/sessions/{checkoutSessionId}/orders` | `checkout.sessions.orders.create` | **Canonical** checkout-bound order creation after quote; binds order to an active checkout session |
| `POST /app/v3/api/orders` | `orders.create` | **Deprecated** — use `checkout.sessions.orders.create` instead |
| `POST /app/v3/api/recharges/orders` | `recharges.orders.create` | Points-recharge checkout (separate fulfillment saga) |
| `POST /app/v3/api/memberships/orders` | `memberships.orders.create` | Membership purchase checkout (`subject=membership`; settlement via MembershipPurchaseFulfillmentPort) |

New PC and integrator surfaces **must** use the checkout-session route for product checkout. `orders.create` remains in the contract as deprecated for transitional integrators only.

Points-recharge fulfillment uses a three-phase saga: `fulfillment_status = processing` reservation, idempotent Account wallet credit, then local `fulfilled` commit. Commit failure triggers compensation debit and reservation release.

Membership-subject orders call `MembershipPurchaseFulfillmentPort` after payment confirmation (HTTP adapter to membership backend, or `noop` for external-only fulfillment).

Order detail projections cap line items at **500** rows per request (`MAX_ORDER_LINE_ITEMS`) to avoid unbounded memory use.

Missing `commerce_*` read-model tables surface as storage errors in production. Local scaffolding may set `ORDER_READ_MODEL_LENIENT=1` to return empty pages when tables are absent (not for production).

List/search endpoints reject invalid `page` or `page_size` with HTTP 400 (`ProblemDetail`) instead of silently clamping; validation is centralized in `sdkwork-utils-rust::validated_offset_list_params` and `sdkwork-order-service::validation::offset_list_params`.

## 4. Database

- Engines: PostgreSQL (production), SQLite (local/tests)
- Table prefix: `commerce_`
- DDL authority: `database/contract/table-registry.json` registers order-owned tables; platform-owned `commerce_order*` core tables remain `referenceOnly` and are bootstrapped by the commerce platform
- Order-owned tables (`commerce_after_sales_*`, `commerce_shipment*`, `commerce_idempotency_key`) ship in repository test migrations and platform bootstrap
- Repository implementations stay behaviorally aligned across engines (organization id normalization, status columns, SQL-level pagination)

## 5. PC Surface

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

## Production Operations

See `docs/guides/operations/PRODUCTION.md` for HA topology, health/metrics endpoints, and release verification.

## 6. Verification

```powershell
cd E:\sdkwork-space\sdkwork-order
cargo test --workspace
pnpm install
pnpm verify
pnpm test:postgres          # optional when ORDER_TEST_POSTGRES_URL is set
pnpm test:postgres:required # CI: fails when postgres URL is missing
```

## 7. Related Docs

- **Checkout and payment topology:** `docs/architecture/commerce/COMMERCE_CHECKOUT_ARCHITECTURE.md`
- Commerce repository dissolution: `../../sdkwork-specs/MIGRATION_SPEC.md` section 8
- Recharge boundary: `specs/commerce-recharge.spec.json`
- Checkout topology contract: `specs/commerce-checkout-topology.spec.json`
- Product: `docs/product/prd/PRD.md`

## 8. Payment Integration

`sdkwork-order` depends on `sdkwork-payment` for owner-order payment execution (`OwnerOrderPaymentStore`, `sdkwork-payment-providers`). The standalone gateway wires payment repositories in-process; split deployments use HTTP backend APIs.

**Settlement orchestration is owned by order**, not payment:

| Step | Owner | Route / function |
| --- | --- | --- |
| PSP webhook | Order app-api | `POST /app/v3/api/orders/payments/webhooks/{providerCode}` |
| In-process settlement | Order service | `settle_owner_order_after_payment_success` |
| Manual replay | Order backend-api | `POST /backend/v3/api/orders/{orderId}/payment_confirmations` |
| Legacy payment webhook | Payment app-api | **410 Gone** (migration shim) |

Configure PSP notify URL: `{ORDER_PAYMENT_WEBHOOK_BASE_URL}/app/v3/api/orders/payments/webhooks/{providerCode}`.

Duplicate webhook deliveries return `replayed: true` without re-running settlement — use `payment_confirmations` for operator recovery.

See `docs/architecture/commerce/COMMERCE_CHECKOUT_ARCHITECTURE.md` and `specs/commerce-payment-webhook.spec.json`.

## 9. Runtime Configuration

| Variable | Purpose | Default |
| --- | --- | --- |
| `ORDER_API_BIND` | Gateway listen address | `0.0.0.0:18093` |
| `ORDER_CORS_ALLOW_ORIGINS` | Comma-separated browser origins | empty (same-origin only) |
| `SDKWORK_ORDER_PLATFORM_CATALOG_TENANT_ID` | Tenant id for public recharge package catalog fallback | `100001` |
| `SDKWORK_ACCESS_TOKEN` | Bearer token for service-to-service wallet credit and membership fulfillment during order settlement | required in production |
| `ORDER_PAYMENT_WEBHOOK_BASE_URL` | Public base URL registered with PSP for order-owned webhooks | required in production |
| `ORDER_TEST_POSTGRES_URL` | PostgreSQL URL for repository parity tests | unset (SQLite-only CI) |
| `RUST_LOG` | Tracing filter (`order.bootstrap`, `order.runtime`, `order.readiness`, `order.security`) | `info` |

## 10. Observability

The standalone gateway mounts `/healthz`, `/livez`, `/readyz`, and `/metrics` via `sdkwork-web-bootstrap::service_router`. Contract fallback (501 for manifest-declared but unmounted routes, 404 otherwise) merges app-api and backend-api `HttpRouteManifest` entries through `sdkwork-order-gateway-assembly::order_contract_fallback_config`.

Structured tracing uses targets `order.bootstrap`, `order.runtime`, `order.readiness`, and `order.security`. API handlers propagate `traceId` through `SdkWorkApiResponse` and `ProblemDetail`. Readiness probes database connectivity via `SELECT 1`.

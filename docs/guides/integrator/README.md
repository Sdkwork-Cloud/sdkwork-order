# Integrator Guide

Status: active  
Updated: 2026-07-06

## SDK Packages

| Surface | Consumer package | OpenAPI discovery |
| --- | --- | --- |
| App API | `@sdkwork/order-app-sdk` | `GET /app/v3/api/openapi.json` |
| Backend API | `@sdkwork/order-backend-sdk` | `GET /backend/v3/api/openapi.json` |

Generate or refresh SDKs from the repository root:

```bash
pnpm sdk:generate
pnpm sdk:generate:backend
```

## Authentication

- App routes require IAM session context (`AuthToken` / `Access-Token` per OpenAPI).
- Backend routes require organization-scoped IAM with permissions such as `commerce.orders.read` and `commerce.orders.manage`.
- Points-recharge fulfillment credits the account wallet through `SDKWORK_ORDER_ACCOUNT_SERVICE_AUTH_TOKEN` (Bearer) unless `SDKWORK_ORDER_ACCOUNT_CREDIT_ALLOW_INSECURE=1` is set for local development only.

## Response Envelope

All business responses use `SdkWorkApiResponse` (`code: 0`, `data`, `traceId`). Errors use HTTP 4xx/5xx with `application/problem+json` (`ProblemDetail`).

List endpoints accept `page` and `page_size` (default 20, max 200). Invalid pagination returns HTTP 400; values are not silently clamped. Nested lists (`orders/{orderId}/events`, `orders/{orderId}/payments`, `shipments/{shipmentId}/packages`, `shipments/{shipmentId}/tracking_events`, `recharges/packages`) use the same pagination contract. Order-scoped payment history is available at `GET /app/v3/api/orders/{orderId}/payments`. After-sales return shipments list at `GET /app/v3/api/after_sales/requests/{afterSalesRequestId}/return_shipments`.

Cancel commands (`orders.cancel`, `orders.cancellations.create`, `recharges.orders.cancel`) return `SdkWorkCommandResponse` with `data.accepted: true`.

## Platform Recharge Catalog

When a tenant has no scoped recharge packages, public catalog queries fall back to the platform tenant configured by `SDKWORK_ORDER_PLATFORM_CATALOG_TENANT_ID` (default `100001`). Set this before production if the platform catalog tenant differs.

## Checkout and Payment

Full topology: `docs/architecture/commerce/COMMERCE_CHECKOUT_ARCHITECTURE.md`.

| Flow | Order operations | Payment |
| --- | --- | --- |
| Product checkout | `checkout.sessions.create` → `checkout.orders.create` → `orders.pay` | `@sdkwork/payment-app-sdk`; open `paymentParams.cashierUrl` |
| Points recharge | `recharges.orders.create` → `orders.pay` | Same; fulfillment via payment backend confirmation |

Cashier URL is returned in `orders.pay` outcome `paymentParams.cashierUrl`. Configure base URL with `SDKWORK_COMMERCE_CASHIER_BASE_URL` (default `https://im.sdkwork.com/cashier`).

**PC:** `sdkwork-order-pc` buyer surface at `/app/order`; operator admin at `/admin/orders` (`@sdkwork/order-backend-sdk`, permissions `commerce.orders.read` / `commerce.orders.manage`).  
**H5 / Flutter:** consume `@sdkwork/order-app-sdk` and `@sdkwork/payment-app-sdk` from host shells; navigate to `cashierUrl` after pay.

## Local Standalone Gateway

```bash
pnpm start
# listens on ORDER_API_BIND (default 0.0.0.0:18093)
```

Set `ORDER_CORS_ALLOW_ORIGINS` for browser clients. Readiness: platform health routes from `sdkwork-web-bootstrap` plus database `SELECT 1` probe.

## Verification

```bash
cargo test --workspace
pnpm verify
pnpm test:postgres          # optional when ORDER_TEST_POSTGRES_URL is set
pnpm test:postgres:required # CI: fails when postgres URL is missing
```

`pnpm verify` now includes `check:governance` (API envelope, pagination, SDK consumer imports).

Authority: `docs/architecture/tech/TECH_ARCHITECTURE.md`, `docs/architecture/commerce/COMMERCE_CHECKOUT_ARCHITECTURE.md`, `specs/RECHARGE_ORDER_SPEC.md`.

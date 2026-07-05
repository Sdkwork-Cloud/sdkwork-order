# Order PRD

Status: active  
Owner: SDKWork maintainers  
Application: order  
Updated: 2026-07-06  
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- Commerce repository dissolution: `../sdkwork-specs/MIGRATION_SPEC.md` section 8
- Technical architecture: `docs/architecture/tech/TECH_ARCHITECTURE.md`
- Recharge boundary: `specs/commerce-recharge.spec.json`

## 1. Background And Problem

Order, checkout, fulfillment, shipment, after-sales, and points-recharge lifecycles must scale independently from payment and catalog capabilities.

`sdkwork-order` is a **T1 commerce capability building block**: domain services, SQL repositories, HTTP routers, standalone gateway, TypeScript SDKs (`@sdkwork/order-app-sdk`, `@sdkwork/order-backend-sdk`), and the PC client surface.

## 2. Target Users

Buyers, merchant operators, fulfillment staff, and commerce integrators.

## 3. Goals And Non-Goals

### Goals

- Own order lifecycle domain, SQL repositories, and app/backend HTTP routers.
- Expose checkout, fulfillment, shipment, after-sales, and recharge routes with SdkWorkApiResponse envelopes.
- Publish OpenAPI authorities at `/app/v3/api/openapi.json` and `/backend/v3/api/openapi.json`.

### Non-Goals

- Payment intent/refund SQL ownership (payment capability).
- IAM middleware implementation inside capability routers (owned by gateway/IAM integration).

## 4. Scope

- Order create/list/detail/cancel flows with lifecycle audit (`commerce_order_event`, `commerce_order_cancellation`).
- Checkout session lifecycle.
- Fulfillment, shipment tracking, after-sales requests.
- Points recharge checkout and backend fulfillment saga.
- SQL repositories for `commerce_*` order tables (PostgreSQL + SQLite parity).

Primary API prefixes:

- App: `/app/v3/api/orders`, `/app/v3/api/recharges`, `/app/v3/api/checkout`, …
- Backend: `/backend/v3/api/orders`

## 5. User Scenarios

- A buyer creates a checkout session, places an order, pays via the cashier, and receives fulfillment after the order gateway processes the PSP webhook (points recharge credits the wallet in-process).
- An operator lists orders, inspects lifecycle events, and cancels or closes orders through the backend API.
- An operator replays stuck settlements via `POST /backend/v3/api/orders/{orderId}/payment_confirmations` (permission `commerce.orders.fulfill`).

## 6. Success Metrics

- `cargo test --workspace` and `pnpm verify` pass.
- Governance checks pass: API envelope, pagination, SDK consumer imports, repo composition.
- SQLite and optional PostgreSQL (`ORDER_TEST_POSTGRES_URL`) parity tests pass for critical lifecycles.

## 7. Phases

- Phase 1 (complete): SQL + app routers owned in sdkwork-order.
- Phase 2 (complete): Payment side-effects owned by payment repository; order validates via one-way dependency.
- Phase 3 (complete): Backend admin API, backend SDK, OpenAPI discovery, cancel/close audit, strict SQL list pagination, v3 SDK envelope on PC order center.
- Phase 4 (complete): Operator UI (`/admin/orders`), `HttpRouteManifest`, gateway contract fallback, CI verify with Postgres parity, production operations guide, OpenAPI static contract tests.
- Phase 5 (complete): Order-owned PSP webhooks, in-process payment settlement, deprecated payment webhook shim (410), `payment_confirmations` backend replay.
- Post-GA platform work (outside this repo): cluster Prometheus dashboards, Redis-backed rate limiting at the ingress/gateway mesh layer.

## 8. Linked Requirements

- Component contract: `specs/component.spec.json`
- Machine contracts: `apis/`, `sdks/`, `specs/`

## 9. Open Questions

None blocking pre-launch. Platform ingress rate-limit store (Redis) and Grafana dashboards are owned by deployment topology, documented in `docs/guides/operations/PRODUCTION.md`.

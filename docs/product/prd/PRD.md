# Order PRD

Status: active
Owner: SDKWork maintainers
Application: order
Updated: 2026-06-24
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- Commerce repository dissolution: `../sdkwork-specs/MIGRATION_SPEC.md` §8

## 1. Background And Problem

Order, checkout, fulfillment, shipment, and after-sales lifecycles must scale independently from payment and catalog capabilities.

This repository is a **T1 commerce capability building block**. The `sdkwork-commerce (deleted)` monolith has been dissolved; this repository is self-contained with its own domain logic, persistence, HTTP route builders, API server, and IAM middleware for the **order** capability.

## 2. Target Users

Buyers, merchant operators, fulfillment staff, and commerce integrators.

## 3. Goals And Non-Goals

### Goals

- Own order lifecycle domain, SQL repositories, and app HTTP routers.
- Expose checkout, fulfillment, shipment, and after-sales routes via `build_*_router` exports.

### Non-Goals

- Payment intent/refund SQL ownership (owned by payment capability).
- IAM middleware in capability routers.

## 4. Scope

- Order create/list/detail/cancel flows.
- Checkout session lifecycle.
- Fulfillment, shipment tracking, after-sales requests.
- Order repository SQL for order lifecycle tables only.

Primary API prefixes:

- App: `/app/v3/api/orders`
- Backend: `/backend/v3/api/orders`

Migration status: **complete**.

## 5. User Scenarios

- A buyer creates a checkout session, places an order, and tracks fulfillment status.
- The T1 `sdkwork-order-standalone-gateway` applies IAM identity middleware to order routers; handlers remain capability-owned.

## 6. Success Metrics

- Order integration tests pass in the T1 standalone-gateway.
- `sdkwork-commerce (deleted)-order-repository-sqlx` is the sole order SQL owner (domain=commerce, capability=order per `NAMING_SPEC.md`).

## 7. Phases

- Phase 1 (complete): SQL + five app routers owned in sdkwork-order.
- Phase 2 (complete): payment_intent/refund SQL owned by payment repository; order validates via one-way dependency.
- Phase 3 (complete): pay_owner_order and cancel payment side-effects owned by payment repository; order repo is order-table only.

## 8. Linked Requirements

- Commerce repository dissolution: `../sdkwork-specs/MIGRATION_SPEC.md` §8
- Component contract: `specs/component.spec.json` (when present)
- Machine contracts: local `specs/`, future `apis/`, and generated `sdks/`

## 9. Open Questions


